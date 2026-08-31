//! From an ENVI-met `.INX` to a Progetto: the one place that knows both.
//!
//! The `.INX` speaks in 1-based cells with `j = 1` to the south; the domain
//! speaks in metres from the south-west corner of the Griglia. Cell `(i, j)`
//! covers the rectangle from `((i-1)·passo, (j-1)·passo)` to `(i·passo, j·passo)`,
//! and a plant rooted in `(i, j)` stands at its centre.

use crate::dominio::*;
use crate::inx::{Inx, Matrix};
use crate::specie;
use std::collections::{BTreeMap, HashSet};
use std::fmt;

const ORIGINE_INX: &str = "file .INX di ENVI-met";
const ORIGINE_EDIFICIO: &str = "file .INX di ENVI-met, matrice zTop";
const ORIGINE_ALBERO: &str = "file .INX di ENVI-met, istanza <3Dplants>";

#[derive(Debug)]
pub enum DaInxError {
    /// A grid whose cells are not square. A Griglia has one `passo_m`, so the
    /// conversion would have to pick one of the two and stay silent about it.
    PassoNonUniforme { dx: f64, dy: f64 },
    AlberoFuoriGriglia {
        indice: usize,
        i: usize,
        j: usize,
        nx: usize,
        ny: usize,
    },
}

impl fmt::Display for DaInxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PassoNonUniforme { dx, dy } => write!(
                f,
                "il file ha dx = {dx} e dy = {dy}: la Griglia di CLIMESH ha un passo solo, \
                 e assumere l'uno per l'altro sposterebbe ogni oggetto del modello. \
                 riesporta il modello da ENVI-met con dx uguale a dy: quale dei due \
                 passi tenere è una scelta del modello, non di CLIMESH"
            ),
            Self::AlberoFuoriGriglia { indice, i, j, nx, ny } => write!(
                f,
                "l'albero #{indice} è radicato nella cella ({i};{j}), fuori dalla griglia di {nx} × {ny} celle"
            ),
        }
    }
}

impl std::error::Error for DaInxError {}

/// The centre of ENVI-met cell `(i, j)`, in metres from the Griglia origin.
///
/// Public because the observation points of a case are given in the same cell
/// indices and must land on the same metres as the objects around them.
pub fn centro_cella_m(i: usize, j: usize, passo_m: f64) -> Posizione {
    ((i as f64 - 0.5) * passo_m, (j as f64 - 0.5) * passo_m)
}

/// Every cell of the grid in written order: northernmost row first, west to east.
///
/// Public because the generator of the reference case walks the same grid, and a
/// second scan written by hand is a second chance to reverse the north-south flip.
pub fn celle(nx: usize, ny: usize) -> impl Iterator<Item = (usize, usize)> {
    (1..=ny)
        .rev()
        .flat_map(move |j| (1..=nx).map(move |i| (i, j)))
}

/// Maximal rectangles covering a mask of cells: runs along `i`, then coalescence
/// of identical runs along `j`.
///
/// A footprint is a union of rectangles, and one rectangle per cell is the union
/// written the longest way: the reference case needs 2780 of them where 4 say the
/// same thing. Duplicate cells in the mask are collapsed, so a caller may hand
/// over whatever it has scanned.
///
/// The order is fixed — northernmost band first, then west to east — because two
/// machines must write the same file.
pub fn unisci_rettangoli(maschera: &[(usize, usize)], passo_m: f64) -> Vec<Rettangolo> {
    let mut per_riga: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &(i, j) in maschera {
        per_riga.entry(j).or_default().push(i);
    }
    // A band still growing northwards: the run of columns it spans, and the rows
    // it covers so far.
    let mut aperte: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    let mut bande: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (j, mut colonne) in per_riga {
        colonne.sort_unstable();
        colonne.dedup();
        let mut tratti: Vec<(usize, usize)> = Vec::new();
        for i in colonne {
            match tratti.last_mut() {
                Some(ultimo) if ultimo.1 + 1 == i => ultimo.1 = i,
                _ => tratti.push((i, i)),
            }
        }
        let mut prosegue = BTreeMap::new();
        for tratto in tratti {
            let banda = match aperte.remove(&tratto) {
                Some((j_min, j_max)) if j_max + 1 == j => (j_min, j),
                Some((j_min, j_max)) => {
                    bande.push((tratto.0, tratto.1, j_min, j_max));
                    (j, j)
                }
                None => (j, j),
            };
            prosegue.insert(tratto, banda);
        }
        // Whatever is left open did not continue on this row, so it is finished.
        for ((i_min, i_max), (j_min, j_max)) in std::mem::take(&mut aperte) {
            bande.push((i_min, i_max, j_min, j_max));
        }
        aperte = prosegue;
    }
    for ((i_min, i_max), (j_min, j_max)) in aperte {
        bande.push((i_min, i_max, j_min, j_max));
    }
    bande.sort_unstable_by_key(|&(i_min, _, _, j_max)| (std::cmp::Reverse(j_max), i_min));
    bande
        .into_iter()
        .map(|(i_min, i_max, j_min, j_max)| Rettangolo {
            x_min_m: (i_min - 1) as f64 * passo_m,
            y_min_m: (j_min - 1) as f64 * passo_m,
            x_max_m: i_max as f64 * passo_m,
            y_max_m: j_max as f64 * passo_m,
        })
        .collect()
}

/// The connected blocks of built cells: cells that touch on a side and stand at
/// the same height.
///
/// Contiguity and not height alone, because two blocks of the same height are two
/// Edifici; and not `buildingNr`, which splits the three blocks of the reference
/// case into seven numbers, some of them one cell wide. Heights are compared for
/// equality on purpose: they come from the file as written and are not computed.
fn blocchi_costruiti(z: &Matrix<f64>, nx: usize, ny: usize) -> Vec<(f64, Vec<(usize, usize)>)> {
    let altezza = |i: usize, j: usize| z.at(i, j).copied().unwrap_or(0.0);
    let mut viste: HashSet<(usize, usize)> = HashSet::new();
    let mut blocchi = Vec::new();
    for (i, j) in celle(nx, ny) {
        let quota = altezza(i, j);
        if quota <= 0.0 || !viste.insert((i, j)) {
            continue;
        }
        let mut pila = vec![(i, j)];
        let mut blocco = Vec::new();
        while let Some((x, y)) = pila.pop() {
            blocco.push((x, y));
            let vicine = [
                (x + 1, y),
                (x, y + 1),
                (x.saturating_sub(1), y),
                (x, y.saturating_sub(1)),
            ];
            for (a, b) in vicine {
                let dentro = a >= 1 && a <= nx && b >= 1 && b <= ny;
                if dentro && altezza(a, b) == quota && viste.insert((a, b)) {
                    pila.push((a, b));
                }
            }
        }
        blocchi.push((quota, blocco));
    }
    blocchi
}

/// The cell masks of the Superfici the file describes, one per type.
///
/// A cell carrying a plant code is Erba: the low-vegetation matrix is where a
/// model puts its lawn, and reading only the soil profiles declares bare ground
/// over 1602 of the 2500 cells of the reference case. A cell with a soil profile
/// and no plant code is TerrenoNudo. A cell neither section mentions gets no
/// Superficie at all, which is not the same claim as bare ground.
///
/// Masks and not rectangles, because a caller that removes cells — the pool of
/// the reconstructed Scenario does — has to remove them before the union, not
/// after.
pub fn maschere_superfici(letto: &Inx) -> Vec<(TipoSuperficie, Vec<(usize, usize)>)> {
    let (nx, ny) = (letto.geometry.grids_i, letto.geometry.grids_j);
    let occupata = |matrice: &Option<Matrix<Option<String>>>, i, j| {
        matrice
            .as_ref()
            .is_some_and(|m| matches!(m.at(i, j), Some(Some(_))))
    };
    let mut erba = Vec::new();
    let mut nudo = Vec::new();
    for (i, j) in celle(nx, ny) {
        if occupata(&letto.plants_2d, i, j) {
            erba.push((i, j));
        } else if occupata(&letto.soil_profiles, i, j) {
            nudo.push((i, j));
        }
    }
    [
        (TipoSuperficie::Erba, erba),
        (TipoSuperficie::TerrenoNudo, nudo),
    ]
    .into_iter()
    .filter(|(_, maschera)| !maschera.is_empty())
    .collect()
}

/// Builds a Progetto holding one Scenario, the one the `.INX` describes.
///
/// `nome_progetto` is a parameter and not `location.name`: in the reference case
/// that field says `bergamo` for a model of Bastia Umbra, a typo left behind by
/// an earlier model.
pub fn progetto_da_inx(
    letto: &Inx,
    nome_progetto: &str,
    nome_scenario: &str,
) -> Result<Progetto, DaInxError> {
    let g = &letto.geometry;
    if g.dx != g.dy {
        return Err(DaInxError::PassoNonUniforme { dx: g.dx, dy: g.dy });
    }
    let passo_m = g.dx;
    let (nx, ny) = (g.grids_i, g.grids_j);

    let terreno_m = celle(nx, ny)
        .map(|(i, j)| {
            letto
                .terrain_height
                .as_ref()
                .and_then(|m| m.at(i, j))
                .copied()
                .unwrap_or(0.0) as f32
        })
        .collect();

    let edifici = letto
        .z_top
        .as_ref()
        .map(|z| blocchi_costruiti(z, nx, ny))
        .unwrap_or_default()
        .into_iter()
        .map(|(quota, blocco)| Edificio {
            altezza_m: quota as f32,
            provenienza: Some(Provenienza {
                origine: ORIGINE_EDIFICIO.to_owned(),
                altezza: FonteAltezza::Rilievo,
            }),
            impronta: unisci_rettangoli(&blocco, passo_m),
        })
        .collect();

    let superfici = maschere_superfici(letto)
        .into_iter()
        .map(|(tipo, maschera)| Superficie {
            tipo,
            impronta: unisci_rettangoli(&maschera, passo_m),
        })
        .collect();

    let mut alberi = Vec::with_capacity(letto.plants.len());
    for (indice, pianta) in letto.plants.iter().enumerate() {
        // `read_inx` refuses such a file, but an `Inx` is a plain struct a
        // caller can also build, and a position outside the extent would be
        // caught by `progetto::valida` far from its cause.
        if pianta.i < 1 || pianta.i > nx || pianta.j < 1 || pianta.j > ny {
            return Err(DaInxError::AlberoFuoriGriglia {
                indice: indice + 1,
                i: pianta.i,
                j: pianta.j,
                nx,
                ny,
            });
        }
        let id = &pianta.plant_id;
        alberi.push(Albero {
            posizione_m: centro_cella_m(pianta.i, pianta.j, passo_m),
            specie: id.clone(),
            altezza_m: specie::altezza_di_chioma_m(id),
            frazione_tronco: specie::frazione_tronco(id),
            // A known species adds nothing to what the Scenario already says; an
            // unknown one does, because its height and trunk are the defaults of
            // a species nobody has checked.
            provenienza: match specie::nome(id) {
                Some(_) => None,
                None => Some(Provenienza {
                    origine: format!("{ORIGINE_ALBERO}, specie sconosciuta: valori predefiniti"),
                    altezza: FonteAltezza::Predefinito,
                }),
            },
        });
    }

    // The catalogue names, once per Scenario: `specie = "020027"` is otherwise
    // decodable only from the ENVI-met plant database, which is not in the file.
    let mut viste: Vec<&str> = Vec::new();
    for pianta in &letto.plants {
        let id = pianta.plant_id.as_str();
        if !viste.contains(&id) {
            viste.push(id);
        }
    }
    let legenda: Vec<String> = viste
        .iter()
        .filter_map(|id| specie::nome(id).map(|nome| format!("{id} {nome}")))
        .collect();
    let origine_scenario = if legenda.is_empty() {
        ORIGINE_INX.to_owned()
    } else {
        format!(
            "{ORIGINE_INX}; specie delle istanze <3Dplants>: {}",
            legenda.join(", ")
        )
    };

    Ok(Progetto {
        nome: nome_progetto.to_owned(),
        griglia: Griglia {
            nx,
            ny,
            passo_m,
            crs: "EPSG:4326".to_owned(),
            origine: (letto.location.longitude, letto.location.latitude),
            rotazione_gradi: letto.location.model_rotation,
        },
        punti: Vec::new(),
        scenari: vec![Scenario {
            nome: nome_scenario.to_owned(),
            derivato_da: None,
            terreno_m,
            // The height of a plant lives in the ENVI-met plant database, which is
            // not in the file: every canopy here is an estimate.
            provenienza: Provenienza {
                origine: origine_scenario,
                altezza: FonteAltezza::Predefinito,
            },
            edifici,
            alberi,
            superfici,
        }],
        periodi: Vec::new(),
    })
}
