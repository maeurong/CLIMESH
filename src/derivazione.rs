//! From the objects of a Scenario to the co-registered rasters the Motore eats.
//!
//! One way only, as [ADR 0001](../docs/adr/0001-oggetti-e-raster.md) prescribes:
//! the objects are the truth, the rasters are derived and regenerable, and
//! nothing here reads a raster back into an object.
//!
//! **The coverage rule, written once and obeyed everywhere.** A cell belongs to a
//! rectangle when its *centre* falls inside it, on *half-open* intervals: minimum
//! included, maximum excluded. The centre, because it is the convention that does
//! not double the cells along a shared border; the half-open interval, because two
//! rectangles that share a side must cover every cell exactly once. The same rule
//! places a tree: a position exactly on the side between two cells belongs to the
//! cell whose minimum it is, that is the eastern or the northern of the two.
//!
//! Row 0 of every raster is the northernmost, as everywhere else in the project.

use crate::dominio::{Griglia, Periodo, Posizione, Rettangolo, Scenario, TipoSuperficie};
use crate::specie;
use ndarray::Array2;
use std::fmt;
use std::ops::Range;

/// The raster type of the Motore, which takes `ArrayView2<f32>` and defines the
/// shape of everything else from the surface model.
pub type Raster = Array2<f32>;

/// A cell no Superficie of the Scenario mentions. Not the same claim as bare
/// ground: the Scenario simply says nothing about it.
pub const CLASSE_NESSUNA: u8 = 0;
pub const CLASSE_PAVIMENTATO: u8 = 1;
pub const CLASSE_ERBA: u8 = 2;
pub const CLASSE_ACQUA: u8 = 3;
pub const CLASSE_TERRENO_NUDO: u8 = 4;

/// The first and last day of the year this Derivazione treats as leafy.
///
/// Day 100 to day 300, with conifers leafy all year, is the default of the
/// SOLWEIG line the Motore comes from, recorded in `research/solweig-riuso.md`
/// § 4. The vendored core carries no leaf logic of its own — it takes one canopy
/// raster at a time — so which canopies stand in January, and what each of them
/// lets through, is decided here and nowhere else. Day 300 falling *inside* the window is our
/// own choice: the source names the bound, not which side it belongs to.
const FINESTRA_CON_FOGLIE: (u32, u32) = (100, 300);

/// The fraction of the direct beam a canopy in leaf lets through: three per
/// cent, measured on single urban trees by Konarska et al. (2013) and the
/// default of the SOLWEIG line the Motore comes from.
///
/// One number for every species. No per-species measurement exists for the five
/// species of the reference case, and five numbers made up to look specific
/// would read as five measurements.
pub const TRASMISSIVITA_CON_FOGLIE: f32 = 0.03;

/// The same for a deciduous canopy in the season without leaves: bare branches
/// are not air, and they still stop about half the beam. SOLWEIG's leaf-off
/// default, and the reason a deciduous canopy stays in the raster all winter
/// instead of being dropped from it.
pub const TRASMISSIVITA_SENZA_FOGLIE: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stagione {
    ConFoglie,
    SenzaFoglie,
}

impl Stagione {
    /// The season of a Periodo, taken from the day its first date falls on.
    ///
    /// A date `Data::giorno_dell_anno` refuses — 30 February, written by hand in
    /// a Periodo file — keeps the leaves. Of the two mistakes available, removing
    /// shade the model cannot justify is the worse one: it changes a result and
    /// leaves nothing behind to notice.
    pub fn da_periodo(periodo: &Periodo) -> Self {
        let (primo, ultimo) = FINESTRA_CON_FOGLIE;
        match periodo.inizio.giorno_dell_anno() {
            Some(giorno) if giorno < primo || giorno > ultimo => Self::SenzaFoglie,
            _ => Self::ConFoglie,
        }
    }
}

/// What the Derivazione decided, for the Giornale to publish.
///
/// A modelling choice the program makes on its own is not swallowed in silence:
/// these are the lines a reader needs to know that a canopy was dropped or a
/// terrain replaced.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScelteDiDerivazione {
    /// Deciduous canopies moved to the bare layer, because the Periodo falls
    /// outside the leafy window. They still cast shade, at
    /// [`TRASMISSIVITA_SENZA_FOGLIE`]; the count says how many.
    pub chiome_spogliate: usize,
    /// Alberi rooted outside the extent of the Griglia.
    pub oggetti_fuori_griglia: usize,
    pub celle_costruite: usize,
    pub celle_con_chioma: usize,
    /// Rectangles with no area, or with minimum and maximum swapped: they cover
    /// nothing rather than covering it backwards.
    pub rettangoli_degeneri: usize,
    /// The length the terrain had, when it was not one value per cell and a flat
    /// terrain took its place.
    pub terreno_sostituito: Option<usize>,
}

/// One canopy layer: where the canopies are, and how much light they let
/// through.
///
/// A layer per transmissivity, and not a transmissivity per cell, because of
/// what the Motore answers: whether a cell stands in the shade of *some*
/// canopy, never of *which* one. The two classes can therefore only be told
/// apart on the way in, by handing the Motore one layer at a time and
/// multiplying what comes back.
///
/// In the season with leaves there is one layer and every tree is in it. In the
/// season without there are two — the evergreens, still opaque, and the bare
/// ones — and a cell shaded by both gets the product of the two
/// transmissivities, which is what a beam crossing both canopies does.
#[derive(Debug, Clone, PartialEq)]
pub struct StratoDiChioma {
    /// What this layer is, for the Giornale to name it by.
    pub nome: &'static str,
    /// The fraction of the direct beam that gets through these canopies.
    pub trasmissivita: f32,
    /// Canopy top, absolute: `f32::NAN` where this layer has no tree.
    ///
    /// NAN and not `0.0`, because these are elevations and not local heights: a
    /// zero over a terrain below the datum would read as a canopy metres above
    /// the ground. It is also the value the vendored core already treats as no
    /// vegetation — `vendor/solweig/src/shadowing.rs` guards its canopy reads
    /// with `is_finite` — so the two conventions agree instead of diverging.
    pub chiome: Raster,
    /// Top of the trunk zone, absolute, with the same `f32::NAN` convention as
    /// `chiome`.
    pub zona_tronco: Raster,
}

impl StratoDiChioma {
    fn vuoto(nome: &'static str, trasmissivita: f32, forma: (usize, usize)) -> Self {
        Self {
            nome,
            trasmissivita,
            chiome: Array2::from_elem(forma, f32::NAN),
            zona_tronco: Array2::from_elem(forma, f32::NAN),
        }
    }

    /// Puts one tree on one cell of this layer.
    fn posa(&mut self, riga: usize, colonna: usize, chioma: f32, tronco: f32) {
        let corrente = self.chiome[[riga, colonna]];
        if corrente.is_nan() || chioma > corrente {
            // Two trees on one cell leave the taller canopy, and the trunk zone
            // that goes with it rather than the other tree's. An empty cell holds
            // NAN, which compares false against everything, so the first tree
            // always writes.
            self.chiome[[riga, colonna]] = chioma;
            self.zona_tronco[[riga, colonna]] = tronco;
        } else if chioma == corrente {
            // Two canopies of the same height: the lower trunk zone, so the
            // answer does not depend on the order the Alberi are listed in, and
            // of the two the deeper canopy is the one that keeps its shade.
            self.zona_tronco[[riga, colonna]] = self.zona_tronco[[riga, colonna]].min(tronco);
        }
    }

    fn ha_chiome(&self) -> bool {
        self.chiome.iter().any(|v| !v.is_nan())
    }
}

/// The co-registered rasters, all of the shape of the Griglia.
#[derive(Debug, Clone)]
pub struct RasterDiScenario {
    /// Terrain plus buildings, in metres above the datum of the terrain.
    pub modello_di_superficie: Raster,
    pub modello_di_terreno: Raster,
    /// The canopy layers that have at least one canopy in them, in the order
    /// the Motore is to apply them. Empty when the Scenario has no tree inside
    /// the Griglia, and then the Motore is never asked about vegetation at all.
    pub strati_di_chioma: Vec<StratoDiChioma>,
    pub classi_di_superficie: Array2<u8>,
    pub scelte: ScelteDiDerivazione,
}

#[derive(Debug)]
pub enum DerivazioneError {
    /// A Griglia whose two sides multiply to more than a `usize`. Refused before
    /// allocating anything: the sides come from a file a user may edit.
    GrigliaSmisurata { nx: usize, ny: usize },
}

impl fmt::Display for DerivazioneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrigliaSmisurata { nx, ny } => write!(
                f,
                "la griglia di {nx} × {ny} celle non sta in memoria su questa macchina: \
                 il prodotto dei due lati non è nemmeno un numero rappresentabile"
            ),
        }
    }
}

impl std::error::Error for DerivazioneError {}

/// The half-open range of cell indices along one axis whose centres fall in
/// `[min_m, max_m)`, clipped to the `n` cells of the Griglia.
///
/// The cast of a float to an integer saturates in Rust, and a NaN becomes zero,
/// so a rectangle far outside the Griglia clips instead of wrapping.
fn intervallo(min_m: f64, max_m: f64, passo_m: f64, n: usize) -> Range<usize> {
    let indice = |m: f64| ((agganciato(m / passo_m - 0.5)).ceil().max(0.0) as usize).min(n);
    indice(min_m)..indice(max_m)
}

/// A value nudged onto the whole number it is a few ulp away from.
///
/// Dividing metres by a step is not exact: `1.05 / 0.3` lands just above `3.5`
/// and `0.3 / 0.1` just below `3`. Without this, a coordinate sitting exactly on
/// a cell centre or on a cell side falls on the wrong side of the coverage rule
/// — the minimum the rule includes drops out, the maximum it excludes creeps in
/// — and the resulting model is plausible and wrong.
fn agganciato(x: f64) -> f64 {
    let intero = x.round();
    if (x - intero).abs() <= 4.0 * f64::EPSILON * x.abs().max(1.0) {
        intero
    } else {
        x
    }
}

/// The rows and columns a rectangle covers, or `None` when the rectangle has no
/// area to cover with.
fn celle_coperte(griglia: &Griglia, r: &Rettangolo) -> Option<(Range<usize>, Range<usize>)> {
    // `partial_cmp` and not `<`: a side that is not a number is not a side, and
    // it has to fall on this branch rather than on an empty coverage nobody counts.
    let senza_area = |min: f64, max: f64| min.partial_cmp(&max) != Some(std::cmp::Ordering::Less);
    if senza_area(r.x_min_m, r.x_max_m) || senza_area(r.y_min_m, r.y_max_m) {
        return None;
    }
    let colonne = intervallo(r.x_min_m, r.x_max_m, griglia.passo_m, griglia.nx);
    let da_sud = intervallo(r.y_min_m, r.y_max_m, griglia.passo_m, griglia.ny);
    // Row 0 is the northernmost, so the range flips end for end.
    let righe = (griglia.ny - da_sud.end)..(griglia.ny - da_sud.start);
    Some((righe, colonne))
}

/// The cell a position falls in, `None` when it falls outside the Griglia.
fn cella_della_posizione(griglia: &Griglia, (x_m, y_m): Posizione) -> Option<(usize, usize)> {
    let indice = |m: f64, n: usize| {
        let i = agganciato(m / griglia.passo_m).floor();
        (i >= 0.0 && i < n as f64).then_some(i as usize)
    };
    let colonna = indice(x_m, griglia.nx)?;
    let da_sud = indice(y_m, griglia.ny)?;
    Some((griglia.ny - 1 - da_sud, colonna))
}

fn classe_di(tipo: TipoSuperficie) -> u8 {
    match tipo {
        TipoSuperficie::Pavimentato => CLASSE_PAVIMENTATO,
        TipoSuperficie::Erba => CLASSE_ERBA,
        TipoSuperficie::Acqua => CLASSE_ACQUA,
        TipoSuperficie::TerrenoNudo => CLASSE_TERRENO_NUDO,
    }
}

/// Turns the objects of a Scenario into the rasters of a Corsa on `periodo`.
///
/// The season is read from the Periodo and not taken from the caller: which
/// canopies stand in January is a property of the dates, not of whoever asks.
pub fn deriva(
    griglia: &Griglia,
    scenario: &Scenario,
    periodo: &Periodo,
) -> Result<RasterDiScenario, DerivazioneError> {
    let (nx, ny) = (griglia.nx, griglia.ny);
    let celle = griglia
        .celle()
        .ok_or(DerivazioneError::GrigliaSmisurata { nx, ny })?;
    let stagione = Stagione::da_periodo(periodo);
    let mut scelte = ScelteDiDerivazione::default();

    let terreno = if scenario.terreno_m.len() == celle {
        scenario.terreno_m.clone()
    } else {
        scelte.terreno_sostituito = Some(scenario.terreno_m.len());
        vec![0.0; celle]
    };
    let modello_di_terreno = Array2::from_shape_vec((ny, nx), terreno)
        .expect("the terrain has just been given one value per cell");

    let mut modello_di_superficie = modello_di_terreno.clone();
    let mut costruite = Array2::from_elem((ny, nx), false);
    for edificio in &scenario.edifici {
        for rettangolo in &edificio.impronta {
            let Some((righe, colonne)) = celle_coperte(griglia, rettangolo) else {
                scelte.rettangoli_degeneri += 1;
                continue;
            };
            for riga in righe {
                for colonna in colonne.clone() {
                    let quota = modello_di_terreno[[riga, colonna]] + edificio.altezza_m;
                    // Two Edifici on one cell leave the taller of the two. The
                    // surface starts at the terrain of this very cell, so the
                    // comparison also keeps a building from digging into it.
                    if quota > modello_di_superficie[[riga, colonna]] {
                        modello_di_superficie[[riga, colonna]] = quota;
                    }
                    costruite[[riga, colonna]] = true;
                }
            }
        }
    }
    scelte.celle_costruite = costruite.iter().filter(|&&c| c).count();

    let mut in_foglia = StratoDiChioma::vuoto("chiome", TRASMISSIVITA_CON_FOGLIE, (ny, nx));
    let mut spoglie = StratoDiChioma::vuoto("chiome spoglie", TRASMISSIVITA_SENZA_FOGLIE, (ny, nx));
    for albero in &scenario.alberi {
        // Standing outside the Griglia is a fact about the geometry and not about
        // the season, so it is counted before the leaves are considered: the same
        // Scenario reports the same number of stray trees in either Stagione.
        let Some((riga, colonna)) = cella_della_posizione(griglia, albero.posizione_m) else {
            scelte.oggetti_fuori_griglia += 1;
            continue;
        };
        // Absolute heights: the Motore reasons on elevations, so a canopy of 12 m
        // over a terrain at 2 m has its top at 14 m.
        let suolo = modello_di_terreno[[riga, colonna]];
        let chioma = suolo + albero.altezza_m;
        let tronco = suolo + albero.altezza_m * albero.frazione_tronco as f32;
        // A deciduous tree out of season keeps its geometry and loses its
        // opacity: the branches are still there, and a canopy dropped from the
        // raster would put full sun where half of it belongs.
        if stagione == Stagione::SenzaFoglie && specie::e_decidua(&albero.specie) {
            scelte.chiome_spogliate += 1;
            spoglie.posa(riga, colonna, chioma, tronco);
        } else {
            in_foglia.posa(riga, colonna, chioma, tronco);
        }
    }
    // A layer with no canopy in it would cost the Motore a whole march to learn
    // that nothing casts anything.
    let strati_di_chioma: Vec<StratoDiChioma> = [in_foglia, spoglie]
        .into_iter()
        .filter(StratoDiChioma::ha_chiome)
        .collect();
    // Cells with a canopy in *some* layer: an evergreen and a bare tree on the
    // same cell are one shaded cell, not two.
    scelte.celle_con_chioma = (0..ny)
        .flat_map(|riga| (0..nx).map(move |colonna| (riga, colonna)))
        .filter(|&(riga, colonna)| {
            strati_di_chioma
                .iter()
                .any(|strato| !strato.chiome[[riga, colonna]].is_nan())
        })
        .count();

    let mut classi_di_superficie = Array2::from_elem((ny, nx), CLASSE_NESSUNA);
    for superficie in &scenario.superfici {
        let classe = classe_di(superficie.tipo);
        for rettangolo in &superficie.impronta {
            let Some((righe, colonne)) = celle_coperte(griglia, rettangolo) else {
                scelte.rettangoli_degeneri += 1;
                continue;
            };
            for riga in righe {
                for colonna in colonne.clone() {
                    classi_di_superficie[[riga, colonna]] = classe;
                }
            }
        }
    }

    Ok(RasterDiScenario {
        modello_di_superficie,
        modello_di_terreno,
        strati_di_chioma,
        classi_di_superficie,
        scelte,
    })
}
