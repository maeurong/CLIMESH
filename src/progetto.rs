//! The Progetto on disk: a directory whose manifest is the truth.
//!
//! Layout:
//! ```text
//! progetto/
//! ├── progetto.toml     grid, observation points, scenario and period names
//! ├── scenari/<nome>.toml
//! └── periodi/<nome>.toml
//! ```
//! Scenarios and periods live in their own files because a scenario carries a
//! cell-sized terrain array and hundreds of objects: keeping them in the manifest
//! would make the one file a reader is meant to skim unreadable.

use crate::dominio::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ProgettoError {
    Io {
        percorso: PathBuf,
        causa: std::io::Error,
    },
    Sintassi {
        percorso: PathBuf,
        causa: String,
    },
    Griglia(String),
    Terreno {
        scenario: String,
        attese: usize,
        trovate: usize,
    },
    /// A Scenario or Periodo name that cannot become a file name inside the
    /// Progetto directory.
    Nome(String),
    /// A symbolic link where the Progetto expects a real file or directory.
    Collegamento(PathBuf),
    /// Two Scenari or two Periodi that would land on the same file.
    NomeDuplicato(String),
    /// An object placed where the Griglia does not reach.
    FuoriGriglia(String),
}

impl fmt::Display for ProgettoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { percorso, causa } => write!(f, "{}: {causa}", percorso.display()),
            Self::Sintassi { percorso, causa } => write!(f, "{}: {causa}", percorso.display()),
            Self::Griglia(msg) => write!(f, "griglia non valida: {msg}"),
            Self::Terreno {
                scenario,
                attese,
                trovate,
            } => write!(
                f,
                "scenario {scenario}: il terreno ha {trovate} celle, la griglia ne vuole {attese}"
            ),
            Self::Nome(nome) => write!(
                f,
                "il nome «{nome}» non può diventare un file dentro la cartella del Progetto"
            ),
            Self::Collegamento(percorso) => write!(
                f,
                "{}: un Progetto non segue collegamenti simbolici",
                percorso.display()
            ),
            Self::NomeDuplicato(nome) => write!(
                f,
                "il nome «{nome}» compare due volte: due file omonimi si sovrascrivono"
            ),
            Self::FuoriGriglia(cosa) => {
                write!(f, "{cosa} cade fuori dall'estensione della Griglia")
            }
        }
    }
}

impl std::error::Error for ProgettoError {}

/// The manifest: everything but the scenarios and periods themselves.
#[derive(Serialize, Deserialize)]
struct Manifesto {
    nome: String,
    scenari: Vec<String>,
    periodi: Vec<String>,
    griglia: Griglia,
    punti: Vec<PuntoDiOsservazione>,
}

/// Device names Windows resolves whatever the extension, so `CON.toml` is not a
/// file there. A Progetto travels between machines, so the check travels too.
const RISERVATI: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Whether `nome` can become the stem of a file inside the Progetto directory.
///
/// An allow-list, not a list of forbidden characters: `C:evil` carries no
/// separator and no `..`, yet on Windows a drive prefix without a root makes
/// `PathBuf::push` drop everything before it, and the file lands wherever that
/// drive happens to be. Only what is listed here gets through.
fn nome_valido(nome: &str) -> bool {
    if nome.is_empty() || nome.ends_with('.') {
        return false;
    }
    if !nome
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return false;
    }
    let radice = nome.split('.').next().unwrap_or(nome);
    !RISERVATI.iter().any(|r| radice.eq_ignore_ascii_case(r))
}

/// A Progetto is a directory users exchange, so the names inside its manifest
/// are external input: they decide which files get opened and written.
fn percorso_di(dir: &Path, sotto: &str, nome: &str) -> Result<PathBuf, ProgettoError> {
    if !nome_valido(nome) {
        return Err(ProgettoError::Nome(nome.to_owned()));
    }
    Ok(dir.join(sotto).join(format!("{nome}.toml")))
}

/// Names that are each usable and all distinct. Distinct case-insensitively,
/// because NTFS would let two of them share one file.
fn valida_nomi<'a>(nomi: impl Iterator<Item = &'a str>) -> Result<(), ProgettoError> {
    let mut visti = HashSet::new();
    for nome in nomi {
        if !nome_valido(nome) {
            return Err(ProgettoError::Nome(nome.to_owned()));
        }
        if !visti.insert(nome.to_ascii_lowercase()) {
            return Err(ProgettoError::NomeDuplicato(nome.to_owned()));
        }
    }
    Ok(())
}

fn valida(p: &Progetto) -> Result<(), ProgettoError> {
    if p.griglia.nx == 0 || p.griglia.ny == 0 {
        return Err(ProgettoError::Griglia(
            "nx e ny devono essere maggiori di zero".into(),
        ));
    }
    if !(p.griglia.passo_m.is_finite() && p.griglia.passo_m > 0.0) {
        return Err(ProgettoError::Griglia(format!(
            "il passo di {} m non è una lunghezza: dev'essere finito e maggiore di zero",
            p.griglia.passo_m
        )));
    }
    let celle = p.griglia.celle().ok_or_else(|| {
        ProgettoError::Griglia(format!(
            "{} × {} celle non entrano in questa macchina",
            p.griglia.nx, p.griglia.ny
        ))
    })?;
    valida_nomi(p.scenari.iter().map(|s| s.nome.as_str()))?;
    valida_nomi(p.periodi.iter().map(|x| x.nome.as_str()))?;
    let est_x = p.griglia.nx as f64 * p.griglia.passo_m;
    let est_y = p.griglia.ny as f64 * p.griglia.passo_m;
    let dentro = |(x, y): Posizione| x >= 0.0 && y >= 0.0 && x <= est_x && y <= est_y;
    for punto in &p.punti {
        if !dentro(punto.posizione_m) {
            return Err(ProgettoError::FuoriGriglia(format!(
                "il punto di osservazione {}",
                punto.id
            )));
        }
    }
    for s in &p.scenari {
        if s.terreno_m.len() != celle {
            return Err(ProgettoError::Terreno {
                scenario: s.nome.clone(),
                attese: celle,
                trovate: s.terreno_m.len(),
            });
        }
        let impronte = (s.edifici.iter().map(|e| ("un edificio", &e.impronta)))
            .chain(s.superfici.iter().map(|x| ("una superficie", &x.impronta)));
        for (cosa, impronta) in impronte {
            for r in impronta {
                if !dentro((r.x_min_m, r.y_min_m)) || !dentro((r.x_max_m, r.y_max_m)) {
                    return Err(ProgettoError::FuoriGriglia(format!(
                        "scenario {}: l'impronta di {cosa}",
                        s.nome
                    )));
                }
            }
        }
        for a in &s.alberi {
            if !dentro(a.posizione_m) {
                return Err(ProgettoError::FuoriGriglia(format!(
                    "scenario {}: un albero",
                    s.nome
                )));
            }
        }
    }
    Ok(())
}

/// A Progetto is an archive that gets exchanged, and a link inside one makes a
/// write land somewhere the manifest never names. The directory the caller
/// passes in is the caller's own and is not questioned; everything the Progetto
/// puts inside it is.
fn rifiuta_collegamento(percorso: &Path) -> Result<(), ProgettoError> {
    match std::fs::symlink_metadata(percorso) {
        Ok(m) if m.file_type().is_symlink() => Err(ProgettoError::Collegamento(percorso.into())),
        _ => Ok(()),
    }
}

fn scrivi_toml<T: Serialize>(percorso: &Path, valore: &T) -> Result<(), ProgettoError> {
    rifiuta_collegamento(percorso)?;
    let testo = toml::to_string(valore).map_err(|e| ProgettoError::Sintassi {
        percorso: percorso.into(),
        causa: e.to_string(),
    })?;
    if let Some(genitore) = percorso.parent() {
        std::fs::create_dir_all(genitore).map_err(|e| ProgettoError::Io {
            percorso: genitore.into(),
            causa: e,
        })?;
    }
    std::fs::write(percorso, testo).map_err(|e| ProgettoError::Io {
        percorso: percorso.into(),
        causa: e,
    })
}

fn leggi_toml<T: for<'a> Deserialize<'a>>(percorso: &Path) -> Result<T, ProgettoError> {
    rifiuta_collegamento(percorso)?;
    let testo = std::fs::read_to_string(percorso).map_err(|e| ProgettoError::Io {
        percorso: percorso.into(),
        causa: e,
    })?;
    toml::from_str(&testo).map_err(|e| ProgettoError::Sintassi {
        percorso: percorso.into(),
        causa: e.to_string(),
    })
}

/// Writes a Progetto into `dir`, overwriting whatever it finds.
///
/// Not atomic, and it does not pretend to be: an I/O error partway through
/// leaves the files written so far on disk. It is ordered, though, and the
/// manifest goes last, so a half-written Progetto has no manifest rather than a
/// manifest naming files that are not there. `leggi` refuses the first case
/// loudly and would believe the second.
pub fn scrivi(dir: impl AsRef<Path>, p: &Progetto) -> Result<(), ProgettoError> {
    valida(p)?;
    let dir = dir.as_ref();
    rifiuta_collegamento(&dir.join("scenari"))?;
    rifiuta_collegamento(&dir.join("periodi"))?;
    for s in &p.scenari {
        scrivi_toml(&dir.join("scenari").join(format!("{}.toml", s.nome)), s)?;
    }
    for x in &p.periodi {
        scrivi_toml(&dir.join("periodi").join(format!("{}.toml", x.nome)), x)?;
    }
    let manifesto = Manifesto {
        nome: p.nome.clone(),
        scenari: p.scenari.iter().map(|s| s.nome.clone()).collect(),
        periodi: p.periodi.iter().map(|x| x.nome.clone()).collect(),
        griglia: p.griglia.clone(),
        punti: p.punti.clone(),
    };
    scrivi_toml(&dir.join("progetto.toml"), &manifesto)
}

pub fn leggi(dir: impl AsRef<Path>) -> Result<Progetto, ProgettoError> {
    let dir = dir.as_ref();
    let manifesto: Manifesto = leggi_toml(&dir.join("progetto.toml"))?;
    rifiuta_collegamento(&dir.join("scenari"))?;
    rifiuta_collegamento(&dir.join("periodi"))?;
    let mut scenari = Vec::with_capacity(manifesto.scenari.len());
    for nome in &manifesto.scenari {
        scenari.push(leggi_toml(&percorso_di(dir, "scenari", nome)?)?);
    }
    let mut periodi = Vec::with_capacity(manifesto.periodi.len());
    for nome in &manifesto.periodi {
        periodi.push(leggi_toml(&percorso_di(dir, "periodi", nome)?)?);
    }
    let p = Progetto {
        nome: manifesto.nome,
        griglia: manifesto.griglia,
        punti: manifesto.punti,
        scenari,
        periodi,
    };
    valida(&p)?;
    Ok(p)
}
