//! The Motore and the sun that drives it. Degenerate inputs first: each test
//! below fails if the defence it names is removed from `src/motore.rs` or
//! `src/sole.rs`.
//!
//! Row 0 of every raster is the northernmost, and the azimuth is degrees from
//! north clockwise, as everywhere else in the project.

use climesh::derivazione::Raster;
use climesh::dominio::Data;
use climesh::motore::{ombre, versione, ALTEZZA_MINIMA_GRADI};
use climesh::sole::{posizione, PosizioneSolare};

/// Bastia Umbra, the reference case: the coordinates read from the model, not
/// the ones of the weather station.
const LATITUDINE: f64 = 43.07;
const LONGITUDINE: f64 = 12.56;
/// +1 all year, daylight saving included, and that is not an oversight: an EPW
/// file carries its hours in local standard time and never shifts them, so the
/// clock these tests read is the clock the weather data is written against. A
/// July hour here is one hour behind the wall clock an Italian would read, and
/// that is the correct reading of the measurement.
const FUSO: f64 = 1.0;

fn luglio() -> Data {
    Data {
        anno: 2026,
        mese: 7,
        giorno: 15,
    }
}

fn sole_a(altezza_gradi: f64, azimut_gradi: f64) -> PosizioneSolare {
    PosizioneSolare {
        altezza_gradi,
        azimut_gradi,
    }
}

/// The highest the sun gets in the day, and the local hour it gets there.
fn culminazione(data: Data) -> (f64, PosizioneSolare) {
    (0..1440)
        .map(|minuto| {
            let ora = minuto as f64 / 60.0;
            (
                ora,
                posizione(data, ora, FUSO, LATITUDINE, LONGITUDINE).unwrap(),
            )
        })
        .fold(
            (0.0, sole_a(f64::NEG_INFINITY, 0.0)),
            |migliore, corrente| {
                if corrente.1.altezza_gradi > migliore.1.altezza_gradi {
                    corrente
                } else {
                    migliore
                }
            },
        )
}

#[test]
fn a_mezzogiorno_solare_di_luglio_il_sole_e_alto_e_a_sud() {
    let (_, culmine) = culminazione(luglio());
    // 90 - 43.07 + 21.4 degrees of declination in mid-July.
    assert!(
        (culmine.altezza_gradi - 68.3).abs() < 1.0,
        "altezza al culmine: {}",
        culmine.altezza_gradi
    );
    assert!(
        (culmine.azimut_gradi - 180.0).abs() < 1.0,
        "azimut al culmine: {}",
        culmine.azimut_gradi
    );
}

#[test]
fn a_un_ora_dell_orologio_fissa_il_sole_sta_dove_deve() {
    // The one assertion in this file tied to the clock rather than to the shape
    // of the day. Everything that scans for a culmination is blind to a time
    // error by construction: shift the clock and the culmination moves with it.
    // 170.17 degrees is where the sun is over Bastia Umbra at 12:00 of the local
    // solar clock on 15 July. Drop the equation of time and it reads 173.80,
    // drop the longitude and it reads 142.19, get the offset wrong by an hour
    // and it reads 137.72: half a degree of tolerance refuses all three.
    let mezzogiorno = posizione(luglio(), 12.0, FUSO, LATITUDINE, LONGITUDINE).unwrap();
    assert!(
        (mezzogiorno.azimut_gradi - 170.17).abs() < 0.5,
        "azimut alle 12:00: {}",
        mezzogiorno.azimut_gradi
    );
    assert!(
        (mezzogiorno.altezza_gradi - 68.35).abs() < 0.5,
        "altezza alle 12:00: {}",
        mezzogiorno.altezza_gradi
    );
}

#[test]
fn l_alba_e_il_tramonto_cadono_all_ora_giusta_dell_orologio() {
    // Sunrise at 04:48 and sunset at 19:42 of the local solar clock, twelve
    // minutes either side of each crossing. The sun climbs about ten degrees an
    // hour here, so the box below is two degrees tall: any time error worth the
    // name — a quarter of an hour of equation of time, fifty minutes of
    // longitude, a whole hour of offset — pushes the sun out of it.
    for (ora, atteso) in [(4.7, -2.0..0.0), (4.9, 0.0..2.0)] {
        let sole = posizione(luglio(), ora, FUSO, LATITUDINE, LONGITUDINE).unwrap();
        assert!(
            atteso.contains(&sole.altezza_gradi),
            "all'alba, alle {ora}, altezza {} fuori da {atteso:?}",
            sole.altezza_gradi
        );
    }
    for (ora, atteso) in [(19.6, 0.0..2.0), (19.8, -2.0..0.0)] {
        let sole = posizione(luglio(), ora, FUSO, LATITUDINE, LONGITUDINE).unwrap();
        assert!(
            atteso.contains(&sole.altezza_gradi),
            "al tramonto, alle {ora}, altezza {} fuori da {atteso:?}",
            sole.altezza_gradi
        );
    }
}

#[test]
fn il_sole_gira_da_est_a_ovest_passando_per_il_sud() {
    let (ora_del_culmine, _) = culminazione(luglio());
    let mattina = posizione(
        luglio(),
        ora_del_culmine - 3.0,
        FUSO,
        LATITUDINE,
        LONGITUDINE,
    )
    .unwrap();
    let sera = posizione(
        luglio(),
        ora_del_culmine + 3.0,
        FUSO,
        LATITUDINE,
        LONGITUDINE,
    )
    .unwrap();
    assert!(
        (60.0..180.0).contains(&mattina.azimut_gradi),
        "azimut del mattino: {}",
        mattina.azimut_gradi
    );
    assert!(
        (180.0..300.0).contains(&sera.azimut_gradi),
        "azimut della sera: {}",
        sera.azimut_gradi
    );
}

#[test]
fn una_data_che_non_e_una_data_non_ha_posizione_solare() {
    let inesistente = Data {
        anno: 2026,
        mese: 2,
        giorno: 30,
    };
    assert!(posizione(inesistente, 12.0, FUSO, LATITUDINE, LONGITUDINE).is_none());
}

/// A flat domain at `quota`, with a 10 m tower at the centre.
fn torre(lato: usize, quota: f32) -> Raster {
    let mut dsm = Raster::from_elem((lato, lato), quota);
    dsm[[lato / 2, lato / 2]] = quota + 10.0;
    dsm
}

#[test]
fn il_sole_sotto_l_orizzonte_lascia_tutto_in_ombra() {
    // Ours, not the kernel's: below the horizon the kernel finds no obstruction
    // and reports every cell lit. That a place at night is not in the sun is a
    // modelling decision, and it is taken here.
    let illuminazione = ombre(&torre(11, 0.0), 1.0, sole_a(-10.0, 180.0));
    assert!(
        illuminazione.iter().all(|&v| v == 0.0),
        "una cella al sole di notte: {illuminazione}"
    );
}

#[test]
fn un_dominio_piatto_col_sole_alto_e_tutto_al_sole() {
    let piatto = Raster::zeros((21, 21));
    let illuminazione = ombre(&piatto, 1.0, sole_a(60.0, 180.0));
    assert!(
        illuminazione.iter().all(|&v| v > 0.99),
        "ombra dal nulla su un dominio piatto: {illuminazione}"
    );
}

#[test]
fn la_torre_getta_la_sua_ombra_a_nord_col_sole_a_sud_a_quarantacinque_gradi() {
    let lato = 25;
    let c = lato / 2;
    let illuminazione = ombre(&torre(lato, 0.0), 1.0, sole_a(45.0, 180.0));

    // A 10 m tower with the sun at 45 degrees casts 10 m of shadow, and it falls
    // to the north because the sun is to the south.
    for distanza in 1..=9 {
        assert!(
            illuminazione[[c - distanza, c]] < 0.5,
            "a {distanza} m a nord la torre non fa ombra: {illuminazione}"
        );
    }
    assert!(
        illuminazione[[c - 11, c]] > 0.5,
        "l'ombra arriva a 11 m a nord, piu' lunga della torre: {illuminazione}"
    );
    assert!(
        illuminazione[[c + 1, c]] > 0.5,
        "ombra a sud, dalla parte del sole: {illuminazione}"
    );
    assert!(
        illuminazione[[c, c - 1]] > 0.5,
        "ombra a ovest, di traverso al sole: {illuminazione}"
    );
}

#[test]
fn il_sole_all_alba_non_fa_panico_ne_ombre_infinite() {
    let lato = 21;
    let c = lato / 2;
    // Just above the threshold, where the shadow of a 10 m tower is some 285 m
    // and runs off a 21 m domain: the longest march the Motore ever takes.
    let illuminazione = ombre(
        &torre(lato, 0.0),
        1.0,
        sole_a(ALTEZZA_MINIMA_GRADI + 0.01, 90.0),
    );
    assert!(
        illuminazione
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
        "illuminazione fuori scala all'alba: {illuminazione}"
    );
    // A domain entirely in shadow would satisfy the range too, and it is the
    // wrong answer: with the sun in the east the cells east of the tower see it.
    assert!(
        illuminazione[[c, lato - 1]] > 0.5,
        "all'alba il bordo est, lontano dalla torre, non e' al sole: {illuminazione}"
    );
    assert!(
        illuminazione[[0, 0]] > 0.5,
        "all'alba l'angolo nord-ovest, che nessuna ombra raggiunge, e' spento: {illuminazione}"
    );
}

#[test]
fn il_terreno_a_quota_costante_da_le_stesse_ombre_del_terreno_a_zero() {
    // What the kernel is owed is the relief, max minus min, not the maximum:
    // the height the ray must climb to clear every obstacle. Feed it the
    // maximum and a domain sitting below the datum stops casting shadows at all.
    let riferimento = ombre(&torre(21, 0.0), 1.0, sole_a(45.0, 180.0));
    for quota in [100.0, -100.0] {
        let sollevato = ombre(&torre(21, quota), 1.0, sole_a(45.0, 180.0));
        assert_eq!(
            sollevato, riferimento,
            "a quota {quota} le ombre cambiano: {sollevato}"
        );
    }
}

#[test]
fn una_griglia_di_una_cella_funziona() {
    let illuminazione = ombre(&Raster::zeros((1, 1)), 1.0, sole_a(45.0, 180.0));
    assert_eq!(illuminazione.dim(), (1, 1));
    assert!(illuminazione[[0, 0]] > 0.5);
}

#[test]
fn la_versione_riporta_il_commit_vendorato() {
    let v = versione();
    assert_eq!(v.commit, "02246ab71a3a8b127d740dde9640449ee9d558ff");
    assert_eq!(v.data_presa, "2026-09-01");
}

#[test]
fn alla_soglia_di_altezza_e_tutto_in_ombra_e_appena_sopra_no() {
    // The threshold is a modelling decision, so it is the threshold and not the
    // horizon that has to be exact: at the declared elevation the domain is
    // shaded, and the smallest step above it starts computing again.
    let alla_soglia = ombre(&torre(11, 0.0), 1.0, sole_a(ALTEZZA_MINIMA_GRADI, 180.0));
    assert!(
        alla_soglia.iter().all(|&v| v == 0.0),
        "alla soglia di {ALTEZZA_MINIMA_GRADI} gradi una cella e' al sole: {alla_soglia}"
    );

    let appena_sopra = ombre(
        &torre(11, 0.0),
        1.0,
        sole_a(ALTEZZA_MINIMA_GRADI + 0.001, 180.0),
    );
    assert!(
        appena_sopra.iter().any(|&v| v > 0.5),
        "appena sopra la soglia il Motore non calcola piu' niente: {appena_sopra}"
    );
}

#[test]
fn col_passo_di_due_metri_l_ombra_della_torre_e_lunga_cinque_celle() {
    // The same 10 m tower and the same 45 degrees as above, on a coarser grid:
    // 10 m of shadow is five cells here and ten there. Every other call in this
    // file passes a step of 1.0, where a step ignored and a step of one metre
    // give the same picture.
    let lato = 25;
    let c = lato / 2;
    let illuminazione = ombre(&torre(lato, 0.0), 2.0, sole_a(45.0, 180.0));

    for distanza in 1..=4 {
        assert!(
            illuminazione[[c - distanza, c]] < 0.5,
            "a {} m a nord la torre non fa ombra: {illuminazione}",
            distanza * 2
        );
    }
    assert!(
        illuminazione[[c - 6, c]] > 0.5,
        "l'ombra arriva a 12 m a nord, piu' lunga della torre: {illuminazione}"
    );
}
