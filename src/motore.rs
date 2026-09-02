//! The only module of the program that names the vendored kernel.
//!
//! Everything else asks for shadows and gets a Raster back. If the copy in
//! `vendor/solweig/` were ever swapped for another kernel, this file is the
//! whole of the change.

use crate::derivazione::{Raster, StratoDiChioma};
use crate::sole::PosizioneSolare;
use ndarray::Array2;
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
/// shadow of a building or of the terrain, and the transmissivity of a canopy
/// under a tree.
///
/// Buildings and terrain are opaque, so their shadow is 0 and nothing a canopy
/// does can lighten it. A canopy is not: it lets [`StratoDiChioma::trasmissivita`]
/// of the beam through, and two canopies over the same cell let through the
/// product of the two. Which is why the layers are marched one at a time — the
/// kernel says whether a cell is shaded by vegetation, never by which
/// vegetation.
///
/// At or below [`ALTEZZA_MINIMA_GRADI`] everything comes back in shadow. That is
/// ours and not the kernel's: with the sun that low the kernel's ray meets no
/// obstruction worth the name and it reports every cell lit, which is
/// geometrically true and says nothing about a place at night. Where the
/// shortwave goes to zero the lit fraction must go with it.
pub fn ombre(
    modello_di_superficie: &Raster,
    passo_m: f64,
    sole: PosizioneSolare,
    strati: &[StratoDiChioma],
) -> Raster {
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
    //
    // Every obstacle includes the canopies, which stand above the surface model
    // and are not in it. Leaving them out is not a march cut short by a little:
    // over flat ground the relief is zero, the ray stops at the first step, and
    // no canopy casts anything at all. `una_chioma_lascia_passare_la_sua_
    // trasmissivita_e_non_spegne_la_cella` in `tests/motore.rs` is what notices.
    let cima_delle_chiome = strati
        .iter()
        .flat_map(|strato| strato.chiome.iter())
        .copied()
        .filter(|quota| quota.is_finite())
        .fold(f32::NEG_INFINITY, f32::max);
    let rilievo = massimo.max(cima_delle_chiome) - minimo;

    // The kernel treats vegetation as present only when canopy, trunk zone *and*
    // bushes are all there: with one of the three missing it computes a canopy
    // shadow and then skips the pass that normalises it, and the array that
    // comes back means something else. CLIMESH has no Cespuglio in its
    // vocabulary, so the third raster is zeros — the kernel calls a bush a bush
    // above one metre, and zero is below it everywhere.
    let cespugli: Raster = Array2::zeros(modello_di_superficie.dim());

    let marcia = |strato: Option<&StratoDiChioma>| {
        pool_a_un_thread().install(|| {
            solweig_vendorato::shadowing::calculate_shadows_rust(
                sole.azimut_gradi as f32,
                sole.altezza_gradi as f32,
                passo_m as f32,
                rilievo,
                modello_di_superficie.view(),
                strato.map(|s| s.chiome.view()),
                strato.map(|s| s.zona_tronco.view()),
                strato.map(|_| cespugli.view()),
                // Muri, esposizioni e i due schemi: servono alla temperatura
                // delle pareti, che questa versione non calcola.
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
        })
    };

    let Some((primo, resto)) = strati.split_first() else {
        return marcia(None).bldg_sh;
    };
    let esito = marcia(Some(primo));
    let mut illuminata = esito.bldg_sh;
    illuminata *= &attenuazione(&esito.veg_sh, primo.trasmissivita);
    for strato in resto {
        illuminata *= &attenuazione(&marcia(Some(strato)).veg_sh, strato.trasmissivita);
    }
    illuminata
}

/// How much of the sky hemisphere a cell can see, with and without the canopies.
///
/// Two fields and not one because they answer two questions, and a mitigation
/// study needs both: masonry is permanent and a tree is a decision.
#[derive(Debug, Clone, PartialEq)]
pub struct SkyViewFactor {
    /// What buildings and terrain leave. 1 on open flat ground. `svf` upstream.
    pub senza_chiome: Raster,
    /// The same, less what every canopy layer takes away, each weighted by its
    /// own transmissivity. `svfbuveg` upstream.
    pub con_le_chiome: Raster,
}

/// The sky is divided into 153 patches: option 2 of the kernel, and the number
/// SOLWEIG's own callers pass.
///
/// A finer division (option 3 or 4) costs a march per patch and buys a
/// resolution the rest of the chain does not use.
const PATCH_DEL_CIELO: u8 = 2;

/// The sky view factor of every cell of the surface model.
///
/// Unlike a shadow this does not depend on the sun: it is a property of the
/// Scenario, computed once and good for every hour of every Periodo. Which is
/// the reason it is worth caching, and the reason the reference case computes
/// it twice and not four times.
///
/// Every canopy layer costs one pass over the whole sky, because the kernel
/// reports whether a cell's view of a patch is blocked by vegetation, never by
/// which vegetation — the same reason [`ombre`] marches the layers one at a
/// time. The building field that comes back is the same on every pass, so the
/// first one's is kept and the rest discarded.
pub fn sky_view_factor(
    modello_di_superficie: &Raster,
    passo_m: f64,
    strati: &[StratoDiChioma],
) -> Result<SkyViewFactor, String> {
    let (minimo, massimo) = modello_di_superficie.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(minimo, massimo), &quota| (minimo.min(quota), massimo.max(quota)),
    );
    let cima_delle_chiome = strati
        .iter()
        .flat_map(|strato| strato.chiome.iter())
        .copied()
        .filter(|quota| quota.is_finite())
        .fold(f32::NEG_INFINITY, f32::max);
    let rilievo = massimo.max(cima_delle_chiome) - minimo;

    let passata = |strato: Option<&StratoDiChioma>| {
        pool_a_un_thread().install(|| {
            solweig_vendorato::skyview::calculate_svf_inner(
                modello_di_superficie.to_owned(),
                strato.map_or_else(
                    || Array2::zeros(modello_di_superficie.dim()),
                    |s| s.chiome.to_owned(),
                ),
                strato.map_or_else(
                    || Array2::zeros(modello_di_superficie.dim()),
                    |s| s.zona_tronco.to_owned(),
                ),
                passo_m as f32,
                strato.is_some(),
                rilievo,
                PATCH_DEL_CIELO,
                // No floor on the patch elevation and no cap on the march: the
                // kernel's own default of 3 degrees sits below the lowest patch
                // of option 2, which is at 6, so it never truncates a patch that
                // counts. Zero means uncapped, and the domains this program
                // works on are two hundred metres across.
                None,
                None,
                None,
                0.0,
            )
        })
    };

    let Some((primo, resto)) = strati.split_first() else {
        // With no vegetation the kernel never runs the pass that fills
        // `svf_veg`: it comes back as the zeros it was allocated with, and the
        // published combination would subtract a hemisphere no tree is
        // blocking. Without a canopy the two fields are the same field.
        let cielo = passata(None)?.svf;
        return Ok(SkyViewFactor {
            con_le_chiome: cielo.clone(),
            senza_chiome: cielo,
        });
    };

    let esito = passata(Some(primo))?;
    let senza_chiome = esito.svf;
    let mut con_le_chiome = senza_chiome.clone();
    con_le_chiome -= &detrazione(&esito.svf_veg, primo.trasmissivita);
    for strato in resto {
        con_le_chiome -= &detrazione(&passata(Some(strato))?.svf_veg, strato.trasmissivita);
    }
    // Two layers can block the same patch of the same cell, and their two
    // deductions then count that patch twice. Clamping is the honest floor: a
    // negative sky view factor is not a number, and the alternative — tracking
    // which patch each layer took — is a joint occlusion the kernel does not
    // report.
    con_le_chiome.mapv_inplace(|v| v.max(0.0));
    Ok(SkyViewFactor {
        senza_chiome,
        con_le_chiome,
    })
}

/// The sky one canopy layer takes away: nothing where it blocks nothing, and
/// what it blocks weighted by how little it lets through.
///
/// `svf - (1 - svf_veg) * (1 - psi)` is upstream's own formula, in
/// `pysrc/solweig/components/svf_resolution.py`; this is the term it subtracts.
/// `svf_veg` comes back already cleared of the sky the buildings were blocking,
/// so the two deductions do not overlap.
fn detrazione(svf_veg: &Raster, trasmissivita: f32) -> Raster {
    svf_veg.mapv(|visibile| (1.0 - visibile) * (1.0 - trasmissivita))
}

/// What one canopy layer leaves of the beam: 1 where it casts nothing, its
/// transmissivity where it does.
///
/// `veg_sh` comes back from the kernel with 1 meaning lit, and already cleared
/// of the building shadow — the kernel zeroes its canopy shadow wherever a
/// building is already shading, so a cell in a building's shadow gets a factor
/// of 1 here and stays at the zero the building put there.
fn attenuazione(veg_sh: &Raster, trasmissivita: f32) -> Raster {
    veg_sh.mapv(|illuminata| 1.0 - (1.0 - illuminata) * (1.0 - trasmissivita))
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
