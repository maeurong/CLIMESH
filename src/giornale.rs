//! The Giornale: the register of a Corsa, and the only source there is.
//!
//! It is a TOML file inside the folder of the Corsa. The view in the page and
//! the printed appendix will be **renderings** of that file, never parallel
//! artefacts: two surfaces that say the same thing drift apart, and the one
//! nobody looks at drifts first.
//!
//! Two rules the file shape depends on, both learned the hard way:
//!
//! - **It is opened at the start and written as the Corsa goes.** A Corsa that
//!   dies leaves a Giornale saying how far it got. There is **no `esito` at the
//!   top level**: a file that is appended to cannot rewrite a line above, so two
//!   outcomes in one file would be a permanent contradiction. As long as
//!   `[conclusione]` is missing the Corsa is not finished, and the absence *is*
//!   the state. `concludi` takes the Giornale by value, so a second one cannot
//!   be written.
//! - **Every text that goes in passes through `toml::to_string`.** Printing the
//!   `Debug` of a struct holding a `String` injects unprotected quotes, and a
//!   Giornale that does not re-read does not exist.

use crate::dominio::{FonteAltezza, Scenario};
use crate::motore::VersioneMotore;
use serde::Serialize;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum GiornaleError {
    Io {
        percorso: PathBuf,
        causa: std::io::Error,
    },
    /// A value that cannot become TOML. It never reaches the file: half a
    /// section written is a Giornale that does not re-read.
    Sezione { sezione: String, causa: String },
    /// A symbolic link where the Giornale expects a file of its own.
    Collegamento(PathBuf),
}

impl fmt::Display for GiornaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { percorso, causa } => write!(f, "{}: {causa}", percorso.display()),
            Self::Sezione { sezione, causa } => {
                write!(f, "la sezione [{sezione}] non diventa TOML: {causa}")
            }
            Self::Collegamento(percorso) => write!(
                f,
                "{}: un Giornale non scrive attraverso un collegamento simbolico",
                percorso.display()
            ),
        }
    }
}

impl std::error::Error for GiornaleError {}

/// What an input file whose bytes could not be read contributes to the Impronta.
///
/// A word and not an empty checksum, so that the file says which of the two it
/// is; and no file's checksum can collide with it, because a checksum is sixty-
/// four hexadecimal characters.
pub const ASSENTE: &str = "assente";

/// An input of the Corsa, with the checksum that says whether it is still the
/// file it was.
///
/// SHA-256 and not a cheaper hash: the checksums already published in
/// `casi/bastia/valori-di-riferimento.toml` come from `sha256sum`, and a reader
/// has to be able to check ours with the same tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ingresso {
    /// Relative to the folder of the Progetto when the file lives inside it,
    /// with `/` as the separator on every platform. See [`Ingresso::leggi`]:
    /// this string goes into the Impronta, so an absolute path would make the
    /// same Progetto a different Corsa in a different folder, and a backslash
    /// would make it a different Corsa on Windows.
    pub percorso: String,
    /// What the file is to the Corsa: `meteo`, `scenario`, `periodo`, `manifesto`.
    /// The weather values are an input and never a result, and this is where the
    /// file says so.
    pub ruolo: String,
    pub sha256: String,
    /// Whether this Corsa opened the file. The list is the same for every Corsa
    /// of a Progetto, because the Impronta is taken over the Progetto as a
    /// whole; a Corsa of one Scenario reads one Scenario file, and the other
    /// Scenari are there without being ingredients of *this* answer.
    pub usato: bool,
}

impl Ingresso {
    /// Reads a file and takes its checksum. A file that cannot be read
    /// contributes as [`ASSENTE`]: unreadable and missing are the same thing to
    /// a Corsa, and neither is ever mistaken for a file that is there.
    ///
    /// `radice` is the folder the recorded path is relative to — the folder of
    /// the Progetto. A file outside it keeps the path it was named with, which
    /// is what a weather file shared between Progetti has.
    ///
    /// Comes back `usato: true`; the caller marks the ones its Corsa never
    /// opened.
    pub fn leggi(percorso: impl AsRef<Path>, radice: impl AsRef<Path>, ruolo: &str) -> Self {
        let percorso = percorso.as_ref();
        let sha256 = match std::fs::read(percorso) {
            Ok(byte) => somma_di_controllo(&byte),
            Err(_) => ASSENTE.to_owned(),
        };
        Self {
            percorso: percorso_relativo(percorso, radice.as_ref()),
            ruolo: ruolo.to_owned(),
            sha256,
            usato: true,
        }
    }
}

/// A path as the Impronta takes it: relative to `radice` when it lies under it,
/// and `/` between the components whatever the platform writes.
fn percorso_relativo(percorso: &Path, radice: &Path) -> String {
    let relativo = percorso.strip_prefix(radice).unwrap_or(percorso);
    let pezzi: Vec<String> = relativo
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    pezzi.join("/")
}

/// The name a Corsa gets from its own content: two Corse with the same Impronta
/// *are* the same Corsa.
///
/// The other name is the etichetta, chosen by whoever launches it, because
/// nobody calls a run `a3f9c1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Impronta(String);

impl fmt::Display for Impronta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The ingredients, laid out as TOML so that no two different sets of them can
/// concatenate into the same bytes. Scalars before the array, which is the order
/// TOML needs anyway.
#[derive(Serialize)]
struct Ingredienti<'a> {
    versione_binario: &'a str,
    motore_commit: &'a str,
    motore_data_presa: &'a str,
    parametri: &'a str,
    ingressi: &'a [Ingresso],
}

impl Impronta {
    pub fn calcola(
        ingressi: &[Ingresso],
        versione_binario: &str,
        motore: &VersioneMotore,
        parametri: &str,
    ) -> Self {
        let ingredienti = Ingredienti {
            versione_binario,
            motore_commit: &motore.commit,
            motore_data_presa: &motore.data_presa,
            parametri,
            ingressi,
        };
        let testo = toml::to_string(&ingredienti)
            .expect("gli ingredienti dell'Impronta sono stringhe e numeri");
        Self(somma_di_controllo(testo.as_bytes()))
    }

    pub fn testo(&self) -> &str {
        &self.0
    }
}

/// How many decimals the Giornale writes. The reproducibility of a Corsa lives
/// in its Impronta, taken over the inputs; the fifteenth decimal of a mean is
/// noise a reader has to step over.
const DECIMALI: f64 = 1e4;

/// A number as the Giornale writes it.
pub(crate) fn arrotonda(valore: f64) -> f64 {
    (valore * DECIMALI).round() / DECIMALI
}

/// The range, the mean and the no-value fraction of one field.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Inviluppo {
    pub campo: String,
    pub unita: String,
    /// Absent when no cell of the field carries a value: a field with nothing in
    /// it gets no invented minimum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimo: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub massimo: Option<f64>,
    /// Over the cells that carry a value, and over no others: a field that says
    /// nothing about a cell does not average a zero in for it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<f64>,
    /// The share of cells the field carries no value for. Not always a defect:
    /// on the canopies it is the share of the domain with no tree over it, and
    /// [`Inviluppo::nota`] is where each field says which of the two it is.
    pub frazione_senza_dato: f64,
    pub intervallo_plausibile: (f64, f64),
    /// Raised when a value falls outside the plausible range. The value is
    /// reported all the same: a flag that hides the number it is about is worse
    /// than no flag.
    pub fuori_intervallo: bool,
    /// What the field is and what it is not, in the words of whoever computed
    /// it. A reader has this file and nothing else.
    pub nota: String,
}

/// The envelope of a field. `NaN` is no data, as it is everywhere else in the
/// project — it is the value the Derivazione writes where there is no canopy.
///
/// The flag is decided on the raw values and the figures are rounded on the way
/// out, so that a value just past the bound is not rounded back inside it.
pub fn inviluppo(
    campo: &str,
    unita: &str,
    valori: impl IntoIterator<Item = f32>,
    intervallo_plausibile: (f64, f64),
    nota: &str,
) -> Inviluppo {
    let (mut minimo, mut massimo) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut somma, mut con_dato, mut celle) = (0.0f64, 0usize, 0usize);
    for valore in valori {
        celle += 1;
        if valore.is_nan() {
            continue;
        }
        let valore = f64::from(valore);
        con_dato += 1;
        somma += valore;
        minimo = minimo.min(valore);
        massimo = massimo.max(valore);
    }
    let frazione_senza_dato = if celle == 0 {
        1.0
    } else {
        (celle - con_dato) as f64 / celle as f64
    };
    let (minimo, massimo, media) = if con_dato == 0 {
        (None, None, None)
    } else {
        (Some(minimo), Some(massimo), Some(somma / con_dato as f64))
    };
    let fuori_intervallo = minimo.is_some_and(|m| m < intervallo_plausibile.0)
        || massimo.is_some_and(|m| m > intervallo_plausibile.1);
    Inviluppo {
        campo: campo.to_owned(),
        unita: unita.to_owned(),
        minimo: minimo.map(arrotonda),
        massimo: massimo.map(arrotonda),
        media: media.map(arrotonda),
        frazione_senza_dato: arrotonda(frazione_senza_dato),
        intervallo_plausibile,
        fuori_intervallo,
        nota: nota.to_owned(),
    }
}

/// How many objects of a Scenario got their height from each link of the
/// fallback chain.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ConteggioProvenienza {
    pub rilievo: usize,
    pub modello_di_superficie: usize,
    pub numero_di_piani: usize,
    pub predefinito: usize,
}

/// Counts Edifici and Alberi by the link that supplied their height. An object
/// with no Provenienza of its own inherits the Scenario's, which is what the
/// Scenario field is for.
pub fn conta_provenienza(scenario: &Scenario) -> ConteggioProvenienza {
    let predefinita = scenario.provenienza.altezza;
    let anelli = (scenario.edifici.iter().map(|e| e.provenienza.as_ref()))
        .chain(scenario.alberi.iter().map(|a| a.provenienza.as_ref()))
        .map(|p| p.map_or(predefinita, |p| p.altezza));
    let mut conteggio = ConteggioProvenienza::default();
    for anello in anelli {
        let quale = match anello {
            FonteAltezza::Rilievo => &mut conteggio.rilievo,
            FonteAltezza::ModelloDiSuperficie => &mut conteggio.modello_di_superficie,
            FonteAltezza::NumeroDiPiani => &mut conteggio.numero_di_piani,
            FonteAltezza::Predefinito => &mut conteggio.predefinito,
        };
        *quale += 1;
    }
    conteggio
}

#[derive(Serialize)]
struct Conclusione<'a> {
    esito: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    errore: Option<&'a str>,
}

/// A Giornale open on disk, appended to as the Corsa proceeds.
#[derive(Debug)]
pub struct Giornale {
    file: std::fs::File,
    percorso: PathBuf,
    tempo_scrittura: Duration,
    vuoto: bool,
}

impl Giornale {
    /// Opens `radice/relativo`, refusing a symbolic link **at any level** of the
    /// path under `radice`.
    ///
    /// The two arguments are not decoration. Checking only the final file left
    /// the half that matters open: a Progetto is an archive people exchange, and
    /// one whose `corse` is a link to somewhere else makes `create_dir_all`
    /// follow it and the write land where the manifest never names. Walking the
    /// chain is the only way to see that.
    pub fn apri(
        radice: impl AsRef<Path>,
        relativo: impl AsRef<Path>,
    ) -> Result<Self, GiornaleError> {
        let mut percorso = radice.as_ref().to_path_buf();
        for parte in relativo.as_ref().components() {
            percorso.push(parte);
            match std::fs::symlink_metadata(&percorso) {
                Ok(m) if m.file_type().is_symlink() => {
                    return Err(GiornaleError::Collegamento(percorso))
                }
                _ => {}
            }
        }
        if let Some(genitore) = percorso.parent() {
            std::fs::create_dir_all(genitore).map_err(|causa| GiornaleError::Io {
                percorso: genitore.to_path_buf(),
                causa,
            })?;
        }
        let file = std::fs::File::create(&percorso).map_err(|causa| GiornaleError::Io {
            percorso: percorso.clone(),
            causa,
        })?;
        Ok(Self {
            file,
            percorso,
            tempo_scrittura: Duration::ZERO,
            vuoto: true,
        })
    }

    /// Appends one named section. Every section is named, including the first:
    /// a top-level scalar written after a table header would belong to that
    /// table instead, and the file would re-read as something else.
    pub fn annota<T: Serialize>(&mut self, sezione: &str, valore: &T) -> Result<(), GiornaleError> {
        let inizio = Instant::now();
        let mut tabella = toml::Table::new();
        let valore = toml::Value::try_from(valore).map_err(|e| GiornaleError::Sezione {
            sezione: sezione.to_owned(),
            causa: e.to_string(),
        })?;
        tabella.insert(sezione.to_owned(), valore);
        let testo = toml::to_string(&tabella).map_err(|e| GiornaleError::Sezione {
            sezione: sezione.to_owned(),
            causa: e.to_string(),
        })?;
        // A blank line between sections: the file is what a reader reads, and
        // the page and the print sheet are renderings of it.
        let testo = if self.vuoto {
            self.vuoto = false;
            testo
        } else {
            format!("\n{testo}")
        };
        let esito = self.file.write_all(testo.as_bytes()).and_then(|()| {
            // Flushed at every section, because the point of writing as it goes
            // is that a Corsa killed halfway leaves what it had already done.
            self.file.flush()
        });
        self.tempo_scrittura += inizio.elapsed();
        esito.map_err(|causa| GiornaleError::Io {
            percorso: self.percorso.clone(),
            causa,
        })
    }

    /// Closes the Giornale. Takes it by value, so there is exactly one
    /// `[conclusione]` and no way to write a second outcome.
    pub fn concludi(mut self, errore: Option<&str>) -> Result<Duration, GiornaleError> {
        let conclusione = Conclusione {
            esito: if errore.is_some() {
                "fallita"
            } else {
                "riuscita"
            },
            errore,
        };
        self.annota("conclusione", &conclusione)?;
        Ok(self.tempo_scrittura)
    }

    pub fn tempo_scrittura(&self) -> Duration {
        self.tempo_scrittura
    }

    pub fn percorso(&self) -> &Path {
        &self.percorso
    }
}

/// The SHA-256 of a byte string, in lower-case hexadecimal.
///
/// Written here rather than taken from a crate: it is seventy lines, the project
/// ships a single binary with no native dependency, and the known-answer vectors
/// of FIPS 180-4 are in `tests/corsa.rs`.
pub fn somma_di_controllo(dati: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    // ponytail: the padded message is a copy of the input. Inputs here are
    // project files of a few megabytes; a streaming version pays only when one
    // of them stops fitting in memory.
    let bit = (dati.len() as u64).wrapping_mul(8);
    let mut messaggio = Vec::with_capacity(dati.len() + 72);
    messaggio.extend_from_slice(dati);
    messaggio.push(0x80);
    while messaggio.len() % 64 != 56 {
        messaggio.push(0);
    }
    messaggio.extend_from_slice(&bit.to_be_bytes());

    for blocco in messaggio.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (parola, byte) in w.iter_mut().zip(blocco.as_chunks::<4>().0) {
            *parola = u32::from_be_bytes(*byte);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for (i, k) in K.iter().enumerate() {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(*k)
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (somma, parziale) in h.iter_mut().zip(v) {
            *somma = somma.wrapping_add(parziale);
        }
    }

    h.iter().map(|parola| format!("{parola:08x}")).collect()
}
