//! A Corsa: one Scenario computed for one Periodo, with its Giornale.
//!
//! The Giornale is opened before anything is computed and written as the Corsa
//! goes, so a Corsa that dies leaves a file saying how far it got. See
//! `crate::giornale` for why there is no outcome at the top level of that file.
//!
//! **No checkpoint of the computation.** At sixty seconds the remedy is to press
//! again. Should the budget ever be missed, that decision has to be reopened
//! together with the budget: they are the same decision.

use crate::derivazione::{self, DerivazioneError, Raster, RasterDiScenario, Stagione};
use crate::dominio::{FonteAltezza, Griglia, Periodo, Progetto, Scenario};
use crate::giornale::{
    arrotonda, conta_provenienza, inviluppo, Giornale, GiornaleError, Impronta, Ingresso, Inviluppo,
};
use crate::motore;
use crate::progetto::{self, ProgettoError};
use crate::sole::{self, PosizioneSolare};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The reference case, relative to the working directory.
pub const CASO_DI_RIFERIMENTO: &str = "casi/bastia/progetto";

/// The tolerance the reproducibility contract promises on continuous
/// quantities. Discrete outcomes — which cells are in shadow, the counts, the
/// order of anything — are identical on any machine instead, and a discrete
/// outcome that depends on the platform is a defect to be fixed.
const TOLLERANZA_RELATIVA: f64 = 1e-6;

/// Below this elevation the shadow of a building leaves the domain, and the
/// centroid of what is left says more about where the edge is than about where
/// the sun is. The check skips those hours instead of reporting a number it
/// cannot mean.
const ALTEZZA_MINIMA_PER_LA_VERIFICA_GRADI: f64 = 10.0;

/// A quadrant. The check answers "does the shadow fall away from the sun", not
/// "how long is it": a caster wider than it is tall pulls the centroid sideways,
/// and the test that matters is the sign of the direction.
const SCARTO_AMMESSO_GRADI: f64 = 45.0;

#[derive(Debug)]
pub enum CorsaError {
    /// A Periodo whose first date is not a date. Without a day of the year there
    /// is no sun, and a Corsa without a sun computes nothing: it fails and says
    /// so, rather than picking a day nobody asked for.
    DataImpossibile {
        periodo: String,
        giorno: String,
    },
    /// A Griglia whose origin is not a latitude and a longitude. The sun needs
    /// the place; nothing here reprojects, so the alternative to failing is
    /// guessing.
    SenzaLatitudine {
        crs: String,
        origine: (f64, f64),
    },
    Derivazione(DerivazioneError),
    Progetto(ProgettoError),
    Giornale(GiornaleError),
}

impl fmt::Display for CorsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataImpossibile { periodo, giorno } => write!(
                f,
                "il periodo «{periodo}» comincia il {giorno}, che non è una data: \
                 senza giorno dell'anno non c'è posizione solare"
            ),
            Self::SenzaLatitudine { crs, origine } => write!(
                f,
                "la griglia è in {crs} con origine ({}, {}): senza latitudine non c'è \
                 posizione solare, e questo programma non riproietta",
                origine.0, origine.1
            ),
            Self::Derivazione(e) => write!(f, "derivazione: {e}"),
            Self::Progetto(e) => write!(f, "progetto: {e}"),
            Self::Giornale(e) => write!(f, "giornale: {e}"),
        }
    }
}

impl std::error::Error for CorsaError {}

impl From<GiornaleError> for CorsaError {
    fn from(e: GiornaleError) -> Self {
        Self::Giornale(e)
    }
}

/// What a Corsa computed.
pub struct Campi {
    /// Hours in the sun, per cell, over the whole Periodo.
    pub ore_di_sole: Raster,
    /// The same divided by the hours of the Periodo.
    pub frazione_illuminata_media: Raster,
}

/// One Corsa, done or failed.
pub struct Esito {
    pub etichetta: String,
    pub impronta: Impronta,
    pub giornale: PathBuf,
    /// `None` when the Corsa succeeded. The same text the Giornale carries.
    pub errore: Option<String>,
    /// `None` when the Corsa failed before computing anything.
    pub campi: Option<Campi>,
    pub tempo_motore: Duration,
    pub tempo_scrittura: Duration,
}

/// Every Corsa of a Progetto, and where the time went.
///
/// The times are here and not in the Giornale on purpose: a clock reading is
/// different on every machine, and a Giornale that carried one would stop being
/// byte-identical between two runs of the same Corsa.
pub struct Rapporto {
    pub corse: Vec<Esito>,
    pub derivazioni: usize,
    pub tempo_derivazione: Duration,
    pub tempo_motore: Duration,
    pub tempo_scrittura: Duration,
    pub tempo_totale: Duration,
}

// --- the sections of the Giornale ------------------------------------------

#[derive(Serialize)]
struct Intestazione<'a> {
    etichetta: &'a str,
    impronta: &'a str,
    progetto: &'a str,
    scenario: &'a str,
    periodo: &'a str,
    citazione: String,
}

#[derive(Serialize)]
struct Binario<'a> {
    nome: &'a str,
    versione: &'a str,
}

#[derive(Serialize)]
struct MotoreCitato {
    commit: String,
    data_presa: String,
    nota: &'static str,
}

#[derive(Serialize)]
struct Riproducibilita {
    esiti_discreti: &'static str,
    grandezze_continue: &'static str,
    tolleranza_relativa: f64,
}

#[derive(Serialize)]
struct VerifichePerRilascio {
    nota: &'static str,
    parita_con_l_implementazione_di_riferimento: &'static str,
    confronto_con_le_misure_di_campo: &'static str,
}

#[derive(Serialize)]
struct ScenarioCitato<'a> {
    nome: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    derivato_da: Option<&'a str>,
    provenienza_origine: &'a str,
    edifici: usize,
    alberi: usize,
    superfici: usize,
}

#[derive(Serialize)]
struct PeriodoCitato<'a> {
    nome: &'a str,
    meteo: String,
    ore: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    direzione_vento_gradi: Option<f64>,
    stagione: &'static str,
    inizio: crate::dominio::Data,
}

#[derive(Serialize)]
struct SoleCitato {
    latitudine_gradi: f64,
    longitudine_gradi: f64,
    fuso_ore: f64,
    nota: &'static str,
}

/// `ScelteDiDerivazione` written out. A modelling choice the program makes on
/// its own is not swallowed in silence.
#[derive(Serialize)]
struct ScelteCitate {
    chiome_escluse: usize,
    oggetti_fuori_griglia: usize,
    celle_costruite: usize,
    celle_con_chioma: usize,
    rettangoli_degeneri: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    terreno_sostituito: Option<usize>,
}

#[derive(Serialize)]
struct VerificaOmbra {
    ore_verificate: usize,
    ore_notturne: usize,
    notte_tutta_in_ombra: bool,
    scarto_massimo_gradi: f64,
    scarto_medio_gradi: f64,
    altezza_minima_gradi: f64,
    scarto_ammesso_gradi: f64,
    bandiera: bool,
    nota: &'static str,
}

/// The parameters the Impronta is taken over: everything that decides the
/// answer and is not already an input file.
#[derive(Serialize)]
struct Parametri<'a> {
    griglia: &'a Griglia,
    scenario: &'a Scenario,
    periodo: &'a Periodo,
}

// ---------------------------------------------------------------------------

/// Latitude, longitude and clock offset of a Griglia.
///
/// The offset is the nearest whole hour of longitude, and the Giornale writes
/// it down: no field of the domain carries the offset of the weather file, and
/// a stand-in a reader can see is better than one they cannot.
fn coordinate(griglia: &Griglia) -> Result<(f64, f64, f64), CorsaError> {
    let (longitudine, latitudine) = griglia.origine;
    if !griglia.crs.eq_ignore_ascii_case("EPSG:4326")
        || !(-90.0..=90.0).contains(&latitudine)
        || !(-180.0..=180.0).contains(&longitudine)
    {
        return Err(CorsaError::SenzaLatitudine {
            crs: griglia.crs.clone(),
            origine: griglia.origine,
        });
    }
    Ok((latitudine, longitudine, (longitudine / 15.0).round()))
}

/// The centre of mass of the cells `dentro` accepts, in metric axes: `x` east,
/// `y` north, with row 0 the northernmost.
fn centroide(raster: &Raster, dentro: impl Fn(usize, usize, f32) -> bool) -> Option<(f64, f64)> {
    let ny = raster.nrows();
    let (mut x, mut y, mut quante) = (0.0f64, 0.0f64, 0usize);
    for ((riga, colonna), &valore) in raster.indexed_iter() {
        if dentro(riga, colonna, valore) {
            x += colonna as f64;
            y += (ny - 1 - riga) as f64;
            quante += 1;
        }
    }
    (quante > 0).then(|| (x / quante as f64, y / quante as f64))
}

/// The angle, in degrees, between the direction the shadow of the buildings runs
/// in and the direction opposite the sun.
///
/// The sun position comes from `crate::sole`, computed here and never asked of
/// the Motore: a check that shares its source with what it checks verifies
/// nothing.
fn scarto_dell_ombra(
    illuminata: &Raster,
    costruite: &Raster,
    sole: PosizioneSolare,
) -> Option<f64> {
    let (x_caster, y_caster) = centroide(costruite, |_, _, v| v > 0.5)?;
    let (x_ombra, y_ombra) = centroide(illuminata, |riga, colonna, v| {
        v < 0.5 && costruite[[riga, colonna]] <= 0.5
    })?;
    let (dx, dy) = (x_ombra - x_caster, y_ombra - y_caster);
    if dx == 0.0 && dy == 0.0 {
        return None;
    }
    let azimut_dell_ombra = dx.atan2(dy).to_degrees().rem_euclid(360.0);
    let atteso = (sole.azimut_gradi + 180.0).rem_euclid(360.0);
    Some(((azimut_dell_ombra - atteso + 180.0).rem_euclid(360.0) - 180.0).abs())
}

struct Marcia {
    campi: Campi,
    verifica: VerificaOmbra,
    tempo_motore: Duration,
}

/// The hours of the Periodo, one call to the Motore each.
///
/// `ora_locale_h` runs past 24 on purpose: `sole::posizione` advances the day
/// through the same argument, and twenty-four hours are three hundred and sixty
/// degrees of hour angle, so hour 30 of the first day *is* hour 6 of the second.
fn marcia(
    modello_di_superficie: &Raster,
    costruite: &Raster,
    griglia: &Griglia,
    periodo: &Periodo,
) -> Result<Marcia, CorsaError> {
    let (latitudine, longitudine, fuso) = coordinate(griglia)?;
    let mut ore_di_sole = Raster::zeros(modello_di_superficie.dim());
    let mut tempo_motore = Duration::ZERO;
    let (mut ore_verificate, mut ore_notturne) = (0usize, 0usize);
    let (mut scarto_massimo, mut scarto_somma) = (0.0f64, 0.0f64);
    let mut notte_tutta_in_ombra = true;

    for ora in 0..periodo.ore {
        let sole = sole::posizione(
            periodo.inizio,
            f64::from(ora),
            fuso,
            latitudine,
            longitudine,
        )
        .ok_or_else(|| CorsaError::DataImpossibile {
            periodo: periodo.nome.clone(),
            giorno: format!(
                "{}/{}/{}",
                periodo.inizio.giorno, periodo.inizio.mese, periodo.inizio.anno
            ),
        })?;
        let inizio = Instant::now();
        let illuminata = motore::ombre(modello_di_superficie, griglia.passo_m, sole);
        tempo_motore += inizio.elapsed();

        if sole.altezza_gradi <= 0.0 {
            ore_notturne += 1;
            notte_tutta_in_ombra &= illuminata.iter().all(|&v| v == 0.0);
        } else if sole.altezza_gradi >= ALTEZZA_MINIMA_PER_LA_VERIFICA_GRADI {
            if let Some(scarto) = scarto_dell_ombra(&illuminata, costruite, sole) {
                ore_verificate += 1;
                scarto_somma += scarto;
                scarto_massimo = scarto_massimo.max(scarto);
            }
        }
        ore_di_sole += &illuminata;
    }

    let frazione_illuminata_media = ore_di_sole.mapv(|v| v / periodo.ore as f32);
    Ok(Marcia {
        campi: Campi {
            ore_di_sole,
            frazione_illuminata_media,
        },
        verifica: VerificaOmbra {
            ore_verificate,
            ore_notturne,
            notte_tutta_in_ombra,
            scarto_massimo_gradi: arrotonda(scarto_massimo),
            scarto_medio_gradi: if ore_verificate == 0 {
                0.0
            } else {
                arrotonda(scarto_somma / ore_verificate as f64)
            },
            altezza_minima_gradi: ALTEZZA_MINIMA_PER_LA_VERIFICA_GRADI,
            scarto_ammesso_gradi: SCARTO_AMMESSO_GRADI,
            // A check that never ran must not look like a check that passed.
            // `scarto_massimo` stays at zero when no hour could be verified -
            // no buildings, or the sun never high enough - and zero reads as
            // agreement. The night is in here too: a Motore lighting the domain
            // at midnight used to raise nothing at all.
            bandiera: scarto_massimo > SCARTO_AMMESSO_GRADI
                || !notte_tutta_in_ombra
                || ore_verificate == 0,
            nota: "l'ombra degli edifici, confrontata con una posizione solare \
                   calcolata a parte dal Motore: scarto fra la direzione che va dal \
                   centro dei volumi al centro dell'ombra al suolo e la direzione \
                   opposta al sole. La bandiera si alza anche quando nessun'ora ha \
                   potuto essere verificata: una verifica non eseguita non è una \
                   verifica passata",
        },
        tempo_motore,
    })
}

/// A plausible range per field, and a note saying what a reader of that number
/// needs to know before trusting it.
///
/// Each field gets its own range. Giving canopy heights the range of terrain
/// elevations would leave a flag that can never fire, which is worse than no
/// flag: it looks like a check and is not one.
fn inviluppi(campi: &Campi, raster: &RasterDiScenario, ore: u32) -> Vec<Inviluppo> {
    const QUOTE_M: (f64, f64) = (-500.0, 9000.0);
    const CHIOME_M: (f64, f64) = (-500.0, 9150.0);
    vec![
        inviluppo(
            "modello di superficie",
            "m",
            raster.modello_di_superficie.iter().copied(),
            QUOTE_M,
            "Terreno più Edifici. Le chiome non ne fanno parte: vedi la nota del campo \"chiome\".",
        ),
        inviluppo(
            "chiome",
            "m",
            raster.chiome.iter().copied(),
            CHIOME_M,
            "Quota assoluta della cima della chioma. Le celle senza chioma valgono NaN e \
             contano nella frazione senza dato: qui \"senza dato\" vuol dire \"senza albero\", \
             non \"dato mancante\". ATTENZIONE: questo campo è calcolato ma NON entra ancora \
             nel calcolo dell'ombra, che oggi viene da terreno più Edifici soltanto. Due \
             Scenari che differiscono solo per gli alberi danno perciò ombre identiche, e \
             non è un risultato: è un limite di questa versione.",
        ),
        inviluppo(
            "ore di sole",
            "h",
            campi.ore_di_sole.iter().copied(),
            (0.0, f64::from(ore)),
            "Ombra da terreno ed Edifici soltanto: la vegetazione non scherma ancora.",
        ),
        inviluppo(
            "frazione illuminata media",
            "adimensionale",
            campi.frazione_illuminata_media.iter().copied(),
            (0.0, 1.0),
            "Media sulle ore del Periodo, con la stessa riserva sulla vegetazione.",
        ),
    ]
}

/// Runs one Corsa and writes its Giornale.
///
/// `raster` comes from the caller so that a Derivazione can be shared between
/// two Corse of the same Scenario in the same Stagione.
///
/// `Err` means the Giornale itself could not be written, and there is nowhere
/// left to record anything. A Corsa that fails while computing comes back as
/// `Ok` with `errore` set, and its Giornale says the same.
#[allow(clippy::too_many_arguments)]
pub fn esegui(
    dir_corse: &Path,
    progetto: &Progetto,
    scenario: &Scenario,
    periodo: &Periodo,
    raster: &RasterDiScenario,
    ingressi: &[Ingresso],
    etichetta: &str,
) -> Result<Esito, CorsaError> {
    let versione_binario = env!("CARGO_PKG_VERSION");
    let motore = motore::versione();
    let parametri = toml::to_string(&Parametri {
        griglia: &progetto.griglia,
        scenario,
        periodo,
    })
    .expect("Griglia, Scenario e Periodo si serializzano già in un Progetto");
    let impronta = Impronta::calcola(ingressi, versione_binario, &motore, &parametri);

    // The folder is named by the Impronta and by nothing else: two Corse with
    // the same Impronta are the same Corsa, and sixty-four hexadecimal
    // characters cannot carry a path separator out of an etichetta.
    let relativo = Path::new(impronta.testo()).join("giornale.toml");
    let percorso = dir_corse.join(&relativo);
    let mut giornale = Giornale::apri(dir_corse, &relativo)?;

    giornale.annota(
        "corsa",
        &Intestazione {
            etichetta,
            impronta: impronta.testo(),
            progetto: &progetto.nome,
            scenario: &scenario.nome,
            periodo: &periodo.nome,
            // The caveat travels inside the citation, not forty lines below it:
            // whoever copies this line is exactly the person who will never
            // scroll down to the Scenario's provenance.
            citazione: format!(
                "CLIMESH {versione_binario}, nucleo solweig {} preso il {}; \
                 Progetto «{}», Scenario «{}»{}, Periodo «{}»; Corsa {}. \
                 https://github.com/maeurong/CLIMESH",
                motore.commit,
                motore.data_presa,
                progetto.nome,
                scenario.nome,
                if scenario.provenienza.altezza == FonteAltezza::Rilievo {
                    ""
                } else {
                    " (ricostruito, non rilevato)"
                },
                periodo.nome,
                &impronta.testo()[..12],
            ),
        },
    )?;
    giornale.annota(
        "binario",
        &Binario {
            nome: env!("CARGO_PKG_NAME"),
            versione: versione_binario,
        },
    )?;
    giornale.annota(
        "motore",
        &MotoreCitato {
            commit: motore.commit.clone(),
            data_presa: motore.data_presa.clone(),
            nota: "nucleo di calcolo riusato da UMEP-dev/solweig, copia vendorata in \
                   vendor/solweig",
        },
    )?;
    giornale.annota(
        "riproducibilita",
        &Riproducibilita {
            esiti_discreti: "identici su qualunque macchina: quali celle sono in ombra, i \
                             conteggi, l'ordine di qualunque cosa. Un esito discreto che \
                             dipende dalla piattaforma è un difetto da correggere",
            grandezze_continue: "entro la tolleranza relativa dichiarata qui sotto. \
                                 L'identità bit a bit non è promessa perché sarebbe falsa",
            tolleranza_relativa: TOLLERANZA_RELATIVA,
        },
    )?;
    giornale.annota(
        "verifiche_per_rilascio",
        &VerifichePerRilascio {
            nota: "citate e non rieseguite: sfonderebbero da sole il budget di una Corsa. \
                   Questo binario usa il nucleo alla versione dichiarata in [motore], ed è \
                   la ragione per cui quel commit è pinnato",
            parita_con_l_implementazione_di_riferimento: "per rilascio, non per Corsa",
            confronto_con_le_misure_di_campo: "per rilascio, non per Corsa",
        },
    )?;
    giornale.annota("griglia", &progetto.griglia)?;
    giornale.annota(
        "scenario",
        &ScenarioCitato {
            nome: &scenario.nome,
            derivato_da: scenario.derivato_da.as_deref(),
            provenienza_origine: &scenario.provenienza.origine,
            edifici: scenario.edifici.len(),
            alberi: scenario.alberi.len(),
            superfici: scenario.superfici.len(),
        },
    )?;
    let stagione = Stagione::da_periodo(periodo);
    giornale.annota(
        "periodo",
        &PeriodoCitato {
            nome: &periodo.nome,
            meteo: periodo.meteo.to_string_lossy().into_owned(),
            ore: periodo.ore,
            direzione_vento_gradi: periodo.direzione_vento_gradi,
            stagione: match stagione {
                Stagione::ConFoglie => "con foglie",
                Stagione::SenzaFoglie => "senza foglie",
            },
            inizio: periodo.inizio,
        },
    )?;
    giornale.annota("ingresso", &ingressi)?;
    giornale.annota(
        "derivazione",
        &ScelteCitate {
            chiome_escluse: raster.scelte.chiome_escluse,
            oggetti_fuori_griglia: raster.scelte.oggetti_fuori_griglia,
            celle_costruite: raster.scelte.celle_costruite,
            celle_con_chioma: raster.scelte.celle_con_chioma,
            rettangoli_degeneri: raster.scelte.rettangoli_degeneri,
            terreno_sostituito: raster.scelte.terreno_sostituito,
        },
    )?;
    giornale.annota("provenienza", &conta_provenienza(scenario))?;

    let costruite = &raster.modello_di_superficie - &raster.modello_di_terreno;
    let esito = match coordinate(&progetto.griglia) {
        Err(e) => Err(e),
        Ok((latitudine, longitudine, fuso)) => {
            giornale.annota(
                "sole",
                &SoleCitato {
                    latitudine_gradi: latitudine,
                    longitudine_gradi: longitudine,
                    fuso_ore: fuso,
                    nota: "posizione solare calcolata da CLIMESH e non chiesta al Motore. \
                           Il fuso è l'ora intera più vicina alla longitudine: nessun campo \
                           del Progetto porta ancora quello del file meteo",
                },
            )?;
            marcia(
                &raster.modello_di_superficie,
                &costruite,
                &progetto.griglia,
                periodo,
            )
        }
    };

    let (campi, errore, tempo_motore) = match esito {
        Ok(fatto) => {
            giornale.annota("campo", &inviluppi(&fatto.campi, raster, periodo.ore))?;
            giornale.annota("verifica_ombra", &fatto.verifica)?;
            (Some(fatto.campi), None, fatto.tempo_motore)
        }
        Err(e) => (None, Some(e.to_string()), Duration::ZERO),
    };
    let tempo_scrittura = giornale.concludi(errore.as_deref())?;

    Ok(Esito {
        etichetta: etichetta.to_owned(),
        impronta,
        giornale: percorso,
        errore,
        campi,
        tempo_motore,
        tempo_scrittura,
    })
}

/// Every Scenario of a Progetto for every one of its Periodi.
///
/// The Derivazione is cached by `(Scenario, Stagione)` and not by Scenario
/// alone: the leafless Periodo drops the deciduous canopies, so the raster
/// really does change with the season. On the reference case the two Periodi
/// fall on either side of the leaf window, so **the cache saves nothing there**;
/// it pays when one Scenario takes a second Periodo in the same season, which is
/// what a parametric study is.
pub fn esegui_progetto(dir: impl AsRef<Path>) -> Result<Rapporto, CorsaError> {
    let inizio = Instant::now();
    let dir = dir.as_ref();
    let progetto = progetto::leggi(dir).map_err(CorsaError::Progetto)?;

    let mut ingressi = vec![Ingresso::leggi(dir.join("progetto.toml"), dir, "manifesto")];
    for scenario in &progetto.scenari {
        let file = dir.join("scenari").join(format!("{}.toml", scenario.nome));
        ingressi.push(Ingresso::leggi(file, dir, "scenario"));
    }
    for periodo in &progetto.periodi {
        let file = dir.join("periodi").join(format!("{}.toml", periodo.nome));
        ingressi.push(Ingresso::leggi(file, dir, "periodo"));
    }
    let meteo: BTreeSet<&Path> = progetto.periodi.iter().map(|p| p.meteo.as_path()).collect();
    for file in meteo {
        ingressi.push(Ingresso::leggi(file, dir, "meteo"));
    }

    let dir_corse = dir.join("corse");
    // ponytail: linear scan. The cache holds one entry per (Scenario, Stagione),
    // and a Progetto has a handful of each.
    let mut cache: Vec<(&str, Stagione, RasterDiScenario)> = Vec::new();
    let mut tempo_derivazione = Duration::ZERO;
    let mut corse = Vec::new();

    for scenario in &progetto.scenari {
        for periodo in &progetto.periodi {
            let stagione = Stagione::da_periodo(periodo);
            let gia_derivato =
                |c: &(&str, Stagione, RasterDiScenario)| c.0 == scenario.nome && c.1 == stagione;
            if !cache.iter().any(gia_derivato) {
                let inizio = Instant::now();
                let raster = derivazione::deriva(&progetto.griglia, scenario, periodo)
                    .map_err(CorsaError::Derivazione)?;
                tempo_derivazione += inizio.elapsed();
                cache.push((&scenario.nome, stagione, raster));
            }
            let raster = &cache
                .iter()
                .find(|c| gia_derivato(c))
                .expect("appena messo")
                .2;
            let etichetta = format!("{} — {}", scenario.nome, periodo.nome);
            corse.push(esegui(
                &dir_corse, &progetto, scenario, periodo, raster, &ingressi, &etichetta,
            )?);
        }
    }

    Ok(Rapporto {
        derivazioni: cache.len(),
        tempo_derivazione,
        tempo_motore: corse.iter().map(|c| c.tempo_motore).sum(),
        tempo_scrittura: corse.iter().map(|c| c.tempo_scrittura).sum(),
        tempo_totale: inizio.elapsed(),
        corse,
    })
}

/// The reference case: Casa Evolutiva, Bastia Umbra, two Scenari for two
/// Periodi. The path is relative to the working directory.
pub fn esegui_caso_di_riferimento() -> Result<Rapporto, CorsaError> {
    esegui_progetto(CASO_DI_RIFERIMENTO)
}
