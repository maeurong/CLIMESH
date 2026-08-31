//! The species table: canopy height, trunk fraction, and whether the species
//! drops its leaves.
//!
//! ENVI-met keeps these numbers in a plant database that ships with the program
//! and is not part of an `.INX` file, so CLIMESH cannot read them from the case.
//! The catalogue names below come from `casi/bastia/valori-di-riferimento.toml`,
//! which read them from the file itself; the heights and trunk fractions are
//! estimates for a mature specimen of each species, which is why every Albero
//! built from an `.INX` records `FonteAltezza::Predefinito`.

struct Specie {
    plant_id: &'static str,
    nome: &'static str,
    altezza_di_chioma_m: f32,
    frazione_tronco: f64,
    decidua: bool,
}

/// The five species of the reference case.
const TABELLA: [Specie; 5] = [
    Specie {
        plant_id: "020027",
        nome: ".Pine Tree (middle)",
        altezza_di_chioma_m: 15.0,
        frazione_tronco: 0.45,
        decidua: false,
    },
    Specie {
        plant_id: "020060",
        nome: ".London Plane Tree (middle)",
        altezza_di_chioma_m: 15.0,
        frazione_tronco: 0.35,
        decidua: true,
    },
    Specie {
        plant_id: "0000PR",
        nome: ".Tilia",
        altezza_di_chioma_m: 12.0,
        frazione_tronco: 0.35,
        decidua: true,
    },
    Specie {
        plant_id: "0000PA",
        nome: ".Populus Alba",
        altezza_di_chioma_m: 18.0,
        frazione_tronco: 0.30,
        decidua: true,
    },
    Specie {
        plant_id: "020111",
        nome: ".Hanging Birch (middle)",
        altezza_di_chioma_m: 10.0,
        frazione_tronco: 0.35,
        decidua: true,
    },
];

/// A modest tree, deliberately: a species this table does not know is a species
/// nobody has checked, and overstating a canopy overstates the shade it casts.
pub const ALTEZZA_PREDEFINITA_M: f32 = 5.0;
pub const FRAZIONE_TRONCO_PREDEFINITA: f64 = 0.3;

fn cerca(plant_id: &str) -> Option<&'static Specie> {
    TABELLA.iter().find(|s| s.plant_id == plant_id)
}

/// The catalogue name of a known species, `None` for one the table does not have.
pub fn nome(plant_id: &str) -> Option<&'static str> {
    cerca(plant_id).map(|s| s.nome)
}

pub fn altezza_di_chioma_m(plant_id: &str) -> f32 {
    cerca(plant_id).map_or(ALTEZZA_PREDEFINITA_M, |s| s.altezza_di_chioma_m)
}

pub fn frazione_tronco(plant_id: &str) -> f64 {
    cerca(plant_id).map_or(FRAZIONE_TRONCO_PREDEFINITA, |s| s.frazione_tronco)
}

/// Whether the species drops its leaves in winter.
///
/// An unknown species is not deciduous. The Derivazione, when it is written,
/// will keep a canopy it cannot justify dropping: shade removed by mistake is the
/// worse of the two errors, because it changes a result and leaves nothing behind
/// to notice.
pub fn e_decidua(plant_id: &str) -> bool {
    cerca(plant_id).is_some_and(|s| s.decidua)
}
