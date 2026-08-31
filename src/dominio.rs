//! The domain types. This module reads nothing and computes nothing.
//!
//! Names come from CONTEXT.md and are binding: the domain speaks Italian, the
//! rest of the code speaks English.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A terrain written as runs of equal values: `[[0.0, 2500]]` instead of 2500
/// zeros. Reading expands the runs, so the field stays one value per cell.
mod terreno {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// The longest terrain a run list may expand to. A run count is a number in a
    /// file a user may edit, and eleven characters of it would otherwise ask for
    /// any amount of memory at all; `progetto::valida` only gets to compare the
    /// length once it exists.
    const CELLE_MASSIME: usize = 10_000_000;

    pub fn serialize<S: Serializer>(valori: &[f32], s: S) -> Result<S::Ok, S::Error> {
        let mut tratti: Vec<(f32, usize)> = Vec::new();
        for &valore in valori {
            match tratti.last_mut() {
                Some((corrente, quante)) if corrente.to_bits() == valore.to_bits() => *quante += 1,
                _ => tratti.push((valore, 1)),
            }
        }
        tratti.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f32>, D::Error> {
        let tratti = Vec::<(f32, usize)>::deserialize(d)?;
        let mut valori: Vec<f32> = Vec::new();
        for (valore, quante) in tratti {
            if quante > CELLE_MASSIME - valori.len().min(CELLE_MASSIME) {
                return Err(serde::de::Error::custom(format!(
                    "il terreno supera {CELLE_MASSIME} celle"
                )));
            }
            valori.resize(valori.len() + quante, valore);
        }
        Ok(valori)
    }
}

/// Extent, step and coordinate system, shared by every raster of the Progetto.
///
/// It lives on the Progetto and not on the Scenario: two Scenari with different
/// grids could not be compared, and comparing them is the whole point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Griglia {
    pub nx: usize,
    pub ny: usize,
    pub passo_m: f64,
    /// The coordinate system `origine` is expressed in.
    ///
    /// Known gap: `passo_m` is metres while a geographic `crs` puts `origine` in
    /// degrees, so the two do not yet belong to the same system. Nothing in this
    /// plan reprojects, so nothing breaks here; the first georeferenced export
    /// will hit it, and it is recorded as an open question in the spec.
    pub crs: String,
    /// Lower-left corner in the coordinate system named by `crs`.
    pub origine: (f64, f64),
    pub rotazione_gradi: f64,
}

impl Griglia {
    /// Cell count, or `None` when `nx * ny` does not fit in a `usize`.
    ///
    /// The two sides come from a file a user may edit, so their product is not
    /// guaranteed to be a number this machine can hold.
    pub fn celle(&self) -> Option<usize> {
        self.nx.checked_mul(self.ny)
    }
}

/// Where an object comes from, and which of its attributes were surveyed rather
/// than estimated. It rides on the object because it outlives any single Corsa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenienza {
    pub origine: String,
    pub altezza: FonteAltezza,
}

/// Which link of the fallback chain supplied a height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FonteAltezza {
    Rilievo,
    ModelloDiSuperficie,
    NumeroDiPiani,
    Predefinito,
}

/// A point on the Progetto, in metres from the Griglia origin: `x` runs east,
/// `y` runs north.
///
/// Metres and not cell indices, because the Griglia can be resampled and a
/// position expressed in cells would silently mean somewhere else afterwards.
pub type Posizione = (f64, f64);

/// An axis-aligned rectangle in the same metric frame as `Posizione`.
///
/// Footprints are unions of these and nothing else. A cell mask converts
/// exactly, one square per cell; polygons wait for the OpenStreetMap import
/// that will actually need them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rettangolo {
    pub x_min_m: f64,
    pub y_min_m: f64,
    pub x_max_m: f64,
    pub y_max_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edificio {
    pub altezza_m: f32,
    /// `None` when the object has nothing to add to `Scenario::provenienza`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenienza: Option<Provenienza>,
    pub impronta: Vec<Rettangolo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Albero {
    pub posizione_m: Posizione,
    /// ENVI-met plant id: the key of the table in `specie.rs`, and the same six
    /// characters the `.INX` writes. A plant id the table does not know takes the
    /// default estimates; it never becomes a catalogue name.
    pub specie: String,
    pub altezza_m: f32,
    /// Trunk-zone top as a fraction of canopy height.
    ///
    /// `f64` and not `f32`: the fractions are short decimal literals, and a `f32`
    /// widened on the way to a file writes `0.44999998807907104` for `0.45`.
    pub frazione_tronco: f64,
    /// `None` when the object has nothing to add to `Scenario::provenienza`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenienza: Option<Provenienza>,
}

/// `Ord` so that a caller may key a map by surface type and get one fixed order
/// out of it: the order the Superfici land in a file is a discrete outcome, and
/// the reproducibility contract asks two machines to write the same bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TipoSuperficie {
    Pavimentato,
    Erba,
    Acqua,
    TerrenoNudo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Superficie {
    pub tipo: TipoSuperficie,
    pub impronta: Vec<Rettangolo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PuntoDiOsservazione {
    pub id: u32,
    pub posizione_m: Posizione,
    pub etichetta: String,
}

/// A calendar date. A dependency for three fields would not pay for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub anno: i32,
    pub mese: u32,
    pub giorno: u32,
}

impl Data {
    /// Day of the year, 1 to 366, or `None` when the date is not a date.
    ///
    /// A Data is deserialised from a file a user may edit by hand, so neither
    /// the month nor the day is trustworthy. Both are checked, and a day the
    /// month does not have gets no plausible answer: 31 April would otherwise
    /// come back as 1 May, which no reader would question.
    pub fn giorno_dell_anno(&self) -> Option<u32> {
        const CUMULATI: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        const LUNGHEZZE: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let indice = self.mese.checked_sub(1)? as usize;
        let cumulato = *CUMULATI.get(indice)?;
        let bisestile = (self.anno % 4 == 0 && self.anno % 100 != 0) || self.anno % 400 == 0;
        let lunghezza = LUNGHEZZE[indice] + u32::from(bisestile && self.mese == 2);
        if self.giorno == 0 || self.giorno > lunghezza {
            return None;
        }
        Some(cumulato + self.giorno + u32::from(bisestile && self.mese > 2))
    }
}

/// The place in one arrangement: everything that does not change with time.
///
/// A Scenario is self-contained. `derivato_da` records which other Scenario it
/// was created from, and stays an annotation: no live inheritance, because a
/// change to a parent must never silently change an already published result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub nome: String,
    pub derivato_da: Option<String>,
    /// Ground elevation per cell, in written order, row 0 northernmost.
    ///
    /// The one thing in a Scenario still indexed by cell, and deliberately so:
    /// the terrain is a sampled field, not an object, and it has no object
    /// shape. Everything with a shape carries metres instead.
    ///
    /// Written as runs of equal values, because a flat domain of 2500 cells is
    /// 2500 zeros on one line and the file carries no `nx` to fold them back.
    #[serde(with = "terreno")]
    pub terreno_m: Vec<f32>,
    /// Where the objects of this Scenario come from, unless one of them says
    /// otherwise.
    ///
    /// On the Scenario and not only on the objects because a Scenario
    /// reconstructed from a description rather than surveyed has to say so once,
    /// at the head of its file, instead of hiding it in the eight hundredth tree.
    pub provenienza: Provenienza,
    pub edifici: Vec<Edificio>,
    pub alberi: Vec<Albero>,
    pub superfici: Vec<Superficie>,
}

/// The weather file plus the date range and forcing parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Periodo {
    pub nome: String,
    pub meteo: PathBuf,
    pub ore: u32,
    pub direzione_vento_gradi: Option<f64>,
    pub inizio: Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progetto {
    pub nome: String,
    pub griglia: Griglia,
    pub punti: Vec<PuntoDiOsservazione>,
    pub scenari: Vec<Scenario>,
    pub periodi: Vec<Periodo>,
}
