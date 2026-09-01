//! Where the sun is, computed here and not asked of the Motore.
//!
//! The Giornale checks the Motore's shadow against a sun position derived
//! independently: a check that shares its source with what it checks verifies
//! nothing. So the position comes from the NOAA general solar position
//! calculations, and the vendored kernel only ever receives the two angles.
//!
//! Azimuth is degrees from north, clockwise, as everywhere else in the project.

use crate::dominio::Data;

/// The two angles the Motore needs, and the only two anyone here asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosizioneSolare {
    /// Degrees above the horizon. Negative at night.
    pub altezza_gradi: f64,
    /// Degrees from north, clockwise: 90 is east, 180 south, 270 west.
    pub azimut_gradi: f64,
}

/// The sun over a place at a moment of the local clock.
///
/// `ora_locale_h` is the hour of the local clock, `fuso_ore` the offset of that
/// clock from UTC, and longitude is positive east. `None` when the Data is not
/// a date: it is deserialised from a file a user may edit by hand.
pub fn posizione(
    data: Data,
    ora_locale_h: f64,
    fuso_ore: f64,
    latitudine_gradi: f64,
    longitudine_gradi: f64,
) -> Option<PosizioneSolare> {
    let giorno = f64::from(data.giorno_dell_anno()?);

    let gamma = std::f64::consts::TAU / 365.0 * (giorno - 1.0 + (ora_locale_h - 12.0) / 24.0);
    let equazione_del_tempo_min = 229.18
        * (0.000075 + 0.001868 * gamma.cos()
            - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin());
    let declinazione = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin()
        - 0.006758 * (2.0 * gamma).cos()
        + 0.000907 * (2.0 * gamma).sin()
        - 0.002697 * (3.0 * gamma).cos()
        + 0.00148 * (3.0 * gamma).sin();

    let scarto_min = equazione_del_tempo_min + 4.0 * longitudine_gradi - 60.0 * fuso_ore;
    let vero_solare_min = ora_locale_h * 60.0 + scarto_min;
    let angolo_orario = (vero_solare_min / 4.0 - 180.0).to_radians();

    let latitudine = latitudine_gradi.to_radians();
    let altezza = (latitudine.sin() * declinazione.sin()
        + latitudine.cos() * declinazione.cos() * angolo_orario.cos())
    .asin();

    // From south, positive westward, then turned into degrees from north
    // clockwise. `atan2` rather than the `acos` of the NOAA note, which loses
    // the morning-afternoon half of the sky and needs the sign patched back on.
    let azimut_da_sud = angolo_orario
        .sin()
        .atan2(angolo_orario.cos() * latitudine.sin() - declinazione.tan() * latitudine.cos());

    Some(PosizioneSolare {
        altezza_gradi: altezza.to_degrees(),
        azimut_gradi: (azimut_da_sud.to_degrees() + 180.0).rem_euclid(360.0),
    })
}
