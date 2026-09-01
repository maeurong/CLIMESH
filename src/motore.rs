//! The only module of the program that names the vendored kernel.
//!
//! Everything else asks for shadows and gets a Raster back. If the copy in
//! `vendor/solweig/` were ever swapped for another kernel, this file is the
//! whole of the change.

use crate::derivazione::Raster;
use crate::sole::PosizioneSolare;
use serde::Deserialize;
use std::sync::OnceLock;

/// Which copy of the kernel produced a result. The Giornale cites it.
///
/// Deserialised straight from `PROVENIENZA.toml`: the keys the file carries and
/// this struct does not are ignored, so a second struct to map across would be
/// two names for one thing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct VersioneMotore {
    pub commit: String,
    pub data_presa: String,
}

/// The provenance of the vendored copy, read at compile time from the file the
/// vendoring wrote. Reading it at run time would make the answer depend on the
/// working directory, and the answer is a property of the binary.
const PROVENIENZA: &str = include_str!("../vendor/solweig/PROVENIENZA.toml");

pub fn versione() -> VersioneMotore {
    toml::from_str(PROVENIENZA).expect("PROVENIENZA.toml della copia vendorata illeggibile")
}

/// Below this solar elevation the Motore calls the whole domain shaded.
///
/// Not the horizon: at 2 degrees the beam crosses about twenty-five air masses,
/// the direct normal irradiance left is a few W/m2, and the shortwave a comfort
/// model cares about is already carried by the diffuse. The ray march, meanwhile,
/// gets more expensive the lower the sun: at a grazing elevation it walks the
/// whole domain from every cell, and it does it to decide the shading of a beam
/// that is not there.
pub const ALTEZZA_MINIMA_GRADI: f64 = 2.0;

/// The lit fraction of every cell of the surface model: 1 in the sun, 0 in the
/// shadow of a building or of the terrain.
///
/// At or below [`ALTEZZA_MINIMA_GRADI`] everything comes back in shadow. That is
/// ours and not the kernel's: with the sun that low the kernel's ray meets no
/// obstruction worth the name and it reports every cell lit, which is
/// geometrically true and says nothing about a place at night. Where the
/// shortwave goes to zero the lit fraction must go with it.
pub fn ombre(modello_di_superficie: &Raster, passo_m: f64, sole: PosizioneSolare) -> Raster {
    if sole.altezza_gradi <= ALTEZZA_MINIMA_GRADI {
        return Raster::zeros(modello_di_superficie.dim());
    }

    let (minimo, massimo) = modello_di_superficie.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(minimo, massimo), &quota| (minimo.min(quota), massimo.max(quota)),
    );

    // `max_local_dsm_ht` is the relief, not the maximum: the height the ray has
    // to climb before it is above every obstacle. Handing over the maximum
    // marches further than needed on any terrain that does not sit at zero, and
    // on terrain below the datum it is negative and no shadow is cast at all.
    let rilievo = massimo - minimo;

    let esito = pool_a_un_thread().install(|| {
        solweig_ombre::shadowing::calculate_shadows_rust(
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
            // No floor under the sun elevation: the guard above has already
            // ended the day. No cap on the shadow either, because the march
            // stops when it walks off the domain and the domain is what we were
            // asked about.
            //
            // Which leaves the kernel with `rilievo / tan(0)` for its index
            // bound: infinite when there is relief, and 0/0 when the domain is
            // flat. The march still ends, on `rilievo >= dz` and on walking off
            // the domain — but the flat case ends only because `f32::min` hands
            // back the operand that is not NaN, so the infinite cap survives the
            // NaN and the cast to a pixel count saturates instead of trapping.
            // That is a property of `f32::min`, not a decision anybody took: if
            // a future release of the kernel or of the standard library
            // propagated the NaN instead, the bound would silently become zero
            // and every shadow would disappear. `un_dominio_piatto_col_sole_alto`
            // in `tests/motore.rs` is what notices.
            0.0,
            f32::INFINITY,
        )
    });
    esito.bldg_sh
}

/// The kernel parallelises its own inner loops with `rayon`, and on a domain
/// this size the fork-join costs far more than the arithmetic it splits.
///
/// Measured on the reference case, 50x50 cells, one call to [`ombre`]: 35.94 ms
/// with rayon's default pool against 0.39 ms on a single thread with the sun at
/// 45 degrees, and 144.78 ms against 1.49 ms with the sun at 5. A reviewer
/// measured 500x500 at 48.2 ms against 3.8 ms, so the break-even sits well above
/// the 100x100 ceiling this project works to: there is no size we run at where
/// the threads pay for themselves.
///
/// Built once and kept, because building it per call would put back exactly the
/// cost this removes.
fn pool_a_un_thread() -> &'static rayon::ThreadPool {
    static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .expect("un pool rayon da un thread")
    })
}
