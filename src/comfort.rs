//! UTCI: the comfort index, and the range outside which it is an extrapolation.
//!
//! The polynomial itself is not written here. It is the sixth-order fit of
//! Bröde et al., vendored in `vendor/solweig/src/utci.rs` and reused verbatim,
//! for the reason the [choice of index](https://github.com/maeurong/CLIMESH/issues/3)
//! gives: UTCI has **one** authoritative polynomial, which is what makes a
//! number produced by CLIMESH comparable with a number produced by anything
//! else. Two hundred and ten coefficients retyped would be two hundred and ten
//! chances to produce an index that is nobody's.
//!
//! What this module adds is the part the polynomial cannot state about itself.

use crate::derivazione::Raster;

/// UTCI, in degrees Celsius, from the air temperature, the relative humidity,
/// the mean radiant temperature and the wind at ten metres.
///
/// The value comes back for any physical input: it is a polynomial and it
/// evaluates everywhere. Whether it *means* anything is
/// [`fuori_intervallo`]'s answer, and the two are kept apart on purpose —
/// a function that returned a sentinel for an input outside the fit would put
/// a magic number into a raster, which is the mistake the weather reader
/// refuses to make with 9999 W/m2.
///
/// One sentinel does survive from upstream: an argument at or below -999
/// returns -999, which is how the reference implementation marks an input it
/// will not evaluate. Nothing in CLIMESH can produce such an argument — a
/// missing weather value is `None` and never a number — and
/// [`fuori_intervallo`] reports it as out of range, but the behaviour is
/// pinned by a test rather than assumed away.
pub fn utci(
    temperatura_c: f64,
    umidita_relativa: f64,
    temperatura_media_radiante_c: f64,
    vento_ms: f64,
) -> f64 {
    f64::from(solweig_vendorato::utci::utci_single(
        temperatura_c as f32,
        umidita_relativa as f32,
        temperatura_media_radiante_c as f32,
        vento_ms as f32,
    ))
}

/// The bounds the polynomial was fitted inside, from Bröde et al. (2012).
///
/// Air temperature, the difference between mean radiant temperature and air
/// temperature, and the wind at ten metres. The humidity enters as a vapour
/// pressure, capped at 5 kPa, which at these temperatures is only reachable
/// above about 33 degrees.
const TEMPERATURA_C: (f64, f64) = (-50.0, 50.0);
const SCARTO_RADIANTE_K: (f64, f64) = (-30.0, 70.0);
const VENTO_MS: (f64, f64) = (0.5, 17.0);

/// Which bound of the fit the inputs fall outside, or `None` when they are
/// inside it.
///
/// **Nothing is clamped.** The common remedy for a wind below half a metre per
/// second is to raise it to half a metre per second, and it is the remedy that
/// makes a still summer courtyard — the case a mitigation study is *about* —
/// come back cooler than it is, without leaving a trace. Here the value is
/// computed on the wind that was measured and the Giornale says how many hours
/// stood outside the fit.
pub fn fuori_intervallo(
    temperatura_c: f64,
    temperatura_media_radiante_c: f64,
    vento_ms: f64,
) -> Option<&'static str> {
    if !(TEMPERATURA_C.0..=TEMPERATURA_C.1).contains(&temperatura_c) {
        return Some("temperatura dell'aria fuori da -50..50 gradi");
    }
    let scarto = temperatura_media_radiante_c - temperatura_c;
    if !(SCARTO_RADIANTE_K.0..=SCARTO_RADIANTE_K.1).contains(&scarto) {
        return Some("temperatura media radiante fuori da -30..70 K dall'aria");
    }
    if !(VENTO_MS.0..=VENTO_MS.1).contains(&vento_ms) {
        return Some("vento fuori da 0,5..17 m/s");
    }
    None
}

/// The ten stress bands UTCI is read in, from Bröde et al. (2012).
///
/// A number without its band is a number a reader has to look up, and the
/// bands are what a mitigation study actually argues about: moving a courtyard
/// out of *forte stress da caldo* is the claim, not moving it by 2,3 gradi.
pub fn banda(utci_c: f64) -> &'static str {
    match utci_c {
        v if v > 46.0 => "stress da caldo estremo",
        v if v > 38.0 => "stress da caldo molto forte",
        v if v > 32.0 => "stress da caldo forte",
        v if v > 26.0 => "stress da caldo moderato",
        v if v > 9.0 => "nessuno stress termico",
        v if v > 0.0 => "leggero stress da freddo",
        v if v > -13.0 => "stress da freddo moderato",
        v if v > -27.0 => "stress da freddo forte",
        v if v > -40.0 => "stress da freddo molto forte",
        _ => "stress da freddo estremo",
    }
}

/// UTCI over a whole field of mean radiant temperature, with the air
/// temperature, humidity and wind of one hour.
///
/// The three meteorological values are uniform over the Griglia, as the
/// [seam between the providers](https://github.com/maeurong/CLIMESH/issues/10)
/// prescribes: today they come from one weather file for the whole domain, and
/// the day one of them becomes a field per cell, this signature is the one that
/// changes.
pub fn utci_sul_campo(
    temperatura_media_radiante: &Raster,
    temperatura_c: f64,
    umidita_relativa: f64,
    vento_ms: f64,
) -> Raster {
    temperatura_media_radiante
        .mapv(|tmrt| utci(temperatura_c, umidita_relativa, f64::from(tmrt), vento_ms) as f32)
}
