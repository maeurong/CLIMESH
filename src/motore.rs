//! The only module of the program that names the vendored kernel.
//!
//! Everything else asks for shadows and gets a Raster back. If the copy in
//! `vendor/solweig/` were ever swapped for another kernel, this file is the
//! whole of the change.

use crate::derivazione::Raster;
use crate::sole::PosizioneSolare;
use serde::Deserialize;

/// Which copy of the kernel produced a result. The Giornale cites it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersioneMotore {
    pub commit: String,
    pub data_presa: String,
}

#[derive(Deserialize)]
struct Provenienza {
    commit: String,
    data_presa: String,
}

/// The provenance of the vendored copy, read at compile time from the file the
/// vendoring wrote. Reading it at run time would make the answer depend on the
/// working directory, and the answer is a property of the binary.
const PROVENIENZA: &str = include_str!("../vendor/solweig/PROVENIENZA.toml");

pub fn versione() -> VersioneMotore {
    let letta: Provenienza =
        toml::from_str(PROVENIENZA).expect("PROVENIENZA.toml della copia vendorata illeggibile");
    VersioneMotore {
        commit: letta.commit,
        data_presa: letta.data_presa,
    }
}

/// The lit fraction of every cell of the surface model: 1 in the sun, 0 in the
/// shadow of a building or of the terrain.
///
/// Below the horizon everything comes back in shadow. That is ours and not the
/// kernel's: with the sun under the horizon the kernel's ray never meets an
/// obstruction and it reports every cell lit, which is geometrically true and
/// says nothing about a place at night. Where the shortwave goes to zero the
/// lit fraction must go with it.
pub fn ombre(modello_di_superficie: &Raster, passo_m: f64, sole: PosizioneSolare) -> Raster {
    if sole.altezza_gradi <= 0.0 {
        return Raster::zeros(modello_di_superficie.dim());
    }

    let massimo = modello_di_superficie
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let minimo = modello_di_superficie
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);

    // `max_local_dsm_ht` is the relief, not the maximum: the height the ray has
    // to climb before it is above every obstacle. Handing over the maximum
    // marches further than needed on any terrain that does not sit at zero, and
    // on terrain below the datum it is negative and no shadow is cast at all.
    let rilievo = massimo - minimo;

    let esito = solweig_ombre::shadowing::calculate_shadows_rust(
        sole.azimut_gradi as f32,
        sole.altezza_gradi as f32,
        passo_m as f32,
        rilievo,
        modello_di_superficie.view(),
        // Chiome, tronchi, cespugli, muri, esposizioni e i due schemi: il
        // Motore vede per ora il solo modello di superficie.
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false, // need_full_wall_outputs
        // No floor under the sun elevation: the guard above has already ended
        // the day. No cap on the shadow either, because the march stops when it
        // walks off the domain and the domain is what we were asked about.
        0.0,
        f32::INFINITY,
    );
    esito.bldg_sh
}
