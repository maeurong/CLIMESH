//! Times what CLIMESH already calls, and exports the Bastia rasters so the
//! upstream Python path can be timed on the same geometry.
//!
//! Wayfinder ticket #24, "Quanto costa la catena piena". Run from the repository
//! root:
//!
//! ```text
//! cargo run --release --manifest-path research/costo-catena-piena/Cargo.toml -- casi/bastia/progetto
//! ```
//!
//! Two things come out. On stdout, per Corsa: the sky view factor pass, the
//! forty-eight hours of shadow, and how many of those hours the Motore answers
//! without marching because the sun is under `ALTEZZA_MINIMA_GRADI`. On disk,
//! under `research/costo-catena-piena/raster/`, one CSV per raster per Corsa,
//! which `passo_fuso.py` reads.
//!
//! The sun is computed the way `corsa::coordinate` computes it when the weather
//! file will not read: whole hours of longitude. `materiale università/` is in
//! `.gitignore`, so on a fresh clone that is the branch taken anyway, and the
//! shadow cost depends on the sun's height, not on which of two neighbouring
//! zones named it.

use climesh::derivazione::{self, Raster};
use climesh::motore;
use climesh::progetto;
use climesh::sole;
use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

/// The sky patches of option 2, the divison CLIMESH asks the kernel for.
/// `vendor/solweig/src/skyview.rs`, `create_patches`: 31+30+28+24+19+13+7+1.
const PATCH: usize = 153;

fn csv(raster: &Raster) -> String {
    let mut s = String::new();
    for riga in raster.rows() {
        let mut prima = true;
        for &v in riga {
            if !prima {
                s.push(',');
            }
            prima = false;
            // NaN is CLIMESH's "no canopy here"; numpy would read the token, so
            // it is written as the literal numpy parses.
            let _ = write!(s, "{}", if v.is_nan() { f32::NAN } else { v });
        }
        s.push('\n');
    }
    s
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "casi/bastia/progetto".to_string());
    let uscita = Path::new("research/costo-catena-piena/raster");
    std::fs::create_dir_all(uscita).expect("cartella dei raster");

    let p = progetto::leggi(&dir).expect("il Progetto si legge");
    let (longitudine, latitudine) = p.griglia.origine;
    let fuso_ore = (longitudine / 15.0).round();
    let celle = p.griglia.nx * p.griglia.ny;

    println!("progetto: {} — {} x {} celle a {} m, fuso {:+} h",
        p.nome, p.griglia.nx, p.griglia.ny, p.griglia.passo_m, fuso_ore);
    println!(
        "matrici d'ombra: {} celle x ceil({}/8) = {} byte impacchettati per matrice, \
         {} byte per le tre",
        celle,
        PATCH,
        celle * PATCH.div_ceil(8),
        3 * celle * PATCH.div_ceil(8)
    );
    println!();

    let mut totale = Duration::ZERO;
    for scenario in &p.scenari {
        for periodo in &p.periodi {
            let inizio = Instant::now();
            let r = derivazione::deriva(&p.griglia, scenario, periodo).expect("la Derivazione");
            let t_deriva = inizio.elapsed();

            let inizio = Instant::now();
            let _svf = motore::sky_view_factor(
                &r.modello_di_superficie,
                p.griglia.passo_m,
                &r.strati_di_chioma,
            )
            .expect("lo sky view factor");
            let t_svf = inizio.elapsed();

            let mut t_ombre = Duration::ZERO;
            let mut t_ombre_globale = Duration::ZERO;
            let (mut di_giorno, mut di_notte) = (0u32, 0u32);
            for ora in 0..periodo.ore {
                let s = sole::posizione(
                    periodo.inizio,
                    f64::from(ora),
                    fuso_ore,
                    latitudine,
                    longitudine,
                )
                .expect("la data del Periodo è una data");
                if s.altezza_gradi > motore::ALTEZZA_MINIMA_GRADI {
                    di_giorno += 1;
                } else {
                    di_notte += 1;
                }
                let inizio = Instant::now();
                let _ = motore::ombre(
                    &r.modello_di_superficie,
                    p.griglia.passo_m,
                    s,
                    &r.strati_di_chioma,
                );
                t_ombre += inizio.elapsed();

                // Lo stesso nucleo, senza `install`: la stessa cosa che fa il
                // percorso Python di monte, che chiama `par_azip!` dal thread
                // chiamante e lascia che rayon consegni il lavoro al pool.
                if s.altezza_gradi > motore::ALTEZZA_MINIMA_GRADI {
                    let inizio = Instant::now();
                    let _ = ombre_senza_install(&r, p.griglia.passo_m, s);
                    t_ombre_globale += inizio.elapsed();
                }
            }
            totale += t_deriva + t_svf + t_ombre;

            println!(
                "{} — {}: {} strati di chioma | derivazione {:.1} ms | svf {:.1} ms | \
                 ombre {:.1} ms su {} ore ({} di giorno, {} sotto {} gradi) = {:.3} ms/ora di giorno",
                scenario.nome,
                periodo.nome,
                r.strati_di_chioma.len(),
                t_deriva.as_secs_f64() * 1e3,
                t_svf.as_secs_f64() * 1e3,
                t_ombre.as_secs_f64() * 1e3,
                periodo.ore,
                di_giorno,
                di_notte,
                motore::ALTEZZA_MINIMA_GRADI,
                t_ombre.as_secs_f64() * 1e3 / f64::from(di_giorno.max(1)),
            );
            println!(
                "    lo stesso nucleo senza il pool installato: {:.1} ms su {} ore di giorno \
                 = {:.3} ms/ora, {:.1}x",
                t_ombre_globale.as_secs_f64() * 1e3,
                di_giorno,
                t_ombre_globale.as_secs_f64() * 1e3 / f64::from(di_giorno.max(1)),
                t_ombre_globale.as_secs_f64() / t_ombre.as_secs_f64(),
            );

            let base = format!("{}-{}", scenario.nome, periodo.nome);
            std::fs::write(
                uscita.join(format!("{base}-dsm.csv")),
                csv(&r.modello_di_superficie),
            )
            .expect("scrittura del modello di superficie");
            for (i, strato) in r.strati_di_chioma.iter().enumerate() {
                std::fs::write(
                    uscita.join(format!("{base}-cdsm-{i}.csv")),
                    csv(&strato.chiome),
                )
                .expect("scrittura delle chiome");
                std::fs::write(
                    uscita.join(format!("{base}-tdsm-{i}.csv")),
                    csv(&strato.zona_tronco),
                )
                .expect("scrittura della zona di tronco");
                println!(
                    "    strato {i}: {} trasmissività {}",
                    strato.nome, strato.trasmissivita
                );
            }
        }
    }
    println!();
    println!(
        "totale delle quattro Corse, senza Giornale né lettura dei file: {:.3} s",
        totale.as_secs_f64()
    );
}

/// One shadow march, calling the kernel from the calling thread instead of from
/// inside `motore`'s one-thread pool.
///
/// `motore::ombre` wraps every kernel call in `pool_a_un_thread().install(…)`.
/// The kernel splits its inner loops with `par_azip!`, and from a thread that is
/// not a rayon worker every one of those splits is a hand-off to the pool and
/// back. Inside `install` the closure already runs on the pool's own thread, so
/// the splits stay put. This function exists to price that difference, because
/// the upstream Python path does not install anything and its numbers carry the
/// hand-off.
fn ombre_senza_install(
    r: &derivazione::RasterDiScenario,
    passo_m: f64,
    sole: climesh::sole::PosizioneSolare,
) -> ndarray::Array2<f32> {
    let dsm = &r.modello_di_superficie;
    let (minimo, massimo) = dsm.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY),
        |(a, b), &q| (a.min(q), b.max(q)),
    );
    let cima = r
        .strati_di_chioma
        .iter()
        .flat_map(|s| s.chiome.iter())
        .copied()
        .filter(|q| q.is_finite())
        .fold(f32::NEG_INFINITY, f32::max);
    let rilievo = massimo.max(cima) - minimo;
    let cespugli: ndarray::Array2<f32> = ndarray::Array2::zeros(dsm.dim());
    let strato = r.strati_di_chioma.first();
    let esito = solweig_vendorato::shadowing::calculate_shadows_rust(
        sole.azimut_gradi as f32,
        sole.altezza_gradi as f32,
        passo_m as f32,
        rilievo,
        dsm.view(),
        strato.map(|s| s.chiome.view()),
        strato.map(|s| s.zona_tronco.view()),
        strato.map(|_| cespugli.view()),
        None,
        None,
        None,
        None,
        false,
        0.0,
        f32::INFINITY,
    );
    esito.bldg_sh
}
