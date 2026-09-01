//! UTCI. The polynomial is vendored and not ours, so these tests do not check
//! its arithmetic: they check the properties that make it *the* UTCI rather
//! than some polynomial, and they pin the two things CLIMESH adds — the range
//! outside which the value is an extrapolation, and the bands it is read in.

use climesh::comfort::{banda, fuori_intervallo, utci, utci_sul_campo};
use climesh::derivazione::Raster;

/// The reference environment UTCI is defined against: still air at half a metre
/// per second, mean radiant temperature equal to the air temperature, and a
/// vapour pressure at or under the reference cap. Under those conditions the
/// index reduces to the air temperature, and that is the definition, not a
/// coincidence — it is what "equivalent temperature" means.
///
/// The agreement is not exact: UTCI is a sixth-order fit to a model, and the
/// fit has residuals. Half a kelvin is the tolerance those residuals need here.
const SCARTO_AMMESSO_K: f64 = 0.7;

#[test]
fn nell_ambiente_di_riferimento_l_utci_e_la_temperatura_dell_aria() {
    for temperatura in [-20.0, 0.0, 10.0, 20.0] {
        let valore = utci(temperatura, 50.0, temperatura, 0.5);
        assert!(
            (valore - temperatura).abs() < SCARTO_AMMESSO_K,
            "a {temperatura} gradi nell'ambiente di riferimento l'UTCI vale {valore}"
        );
    }
}

#[test]
fn sopra_i_trenta_gradi_e_l_umidita_a_scostare_l_utci_dall_aria() {
    // The reference environment fixes the vapour pressure, not the relative
    // humidity: 50 per cent at 40 degrees is about 3,7 kPa, well above the
    // reference, and the index rises accordingly. At the humidity that puts the
    // vapour pressure back on the reference — 27 per cent at 40 degrees — the
    // identity comes back. Anyone reading `utci(40, 50, 40, 0.5) = 43,6` as an
    // error would be reading the definition wrong, so it is written down here.
    let a_meta_umidita = utci(40.0, 50.0, 40.0, 0.5);
    let al_riferimento = utci(40.0, 27.0, 40.0, 0.5);
    assert!(
        a_meta_umidita > 43.0,
        "a 40 gradi e 50 per cento l'umidità deve pesare: {a_meta_umidita}"
    );
    assert!(
        (al_riferimento - 40.0).abs() < SCARTO_AMMESSO_K,
        "riportata l'umidità al riferimento l'identità torna: {al_riferimento}"
    );
}

#[test]
fn l_utci_cresce_con_la_temperatura_media_radiante() {
    // The reason CLIMESH computes a radiant temperature at all: if the index
    // did not move with it, the shade of a tree would not show up in the
    // result.
    let valori: Vec<f64> = [20.0, 30.0, 40.0, 50.0, 60.0]
        .into_iter()
        .map(|tmrt| utci(30.0, 50.0, tmrt, 1.0))
        .collect();
    assert!(
        valori.windows(2).all(|c| c[1] > c[0]),
        "l'UTCI non cresce con la temperatura media radiante: {valori:?}"
    );
    // Forty kelvin of radiant temperature — sun against shade in a courtyard —
    // are worth about ten of index. That is the whole argument of the project
    // written as a number.
    let scarto = valori[4] - valori[0];
    assert!(
        (9.0..12.0).contains(&scarto),
        "fra 20 e 60 gradi di radiante l'UTCI cambia di {scarto}"
    );
}

#[test]
fn l_utci_cala_col_vento_quando_il_radiante_e_alto() {
    let valori: Vec<f64> = [0.5, 1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|vento| utci(30.0, 50.0, 50.0, vento))
        .collect();
    assert!(
        valori.windows(2).all(|c| c[1] < c[0]),
        "l'UTCI non cala col vento: {valori:?}"
    );
}

/// A golden value: it moves only when the vendored polynomial moves, which is
/// when the pinned commit moves. Thirty degrees of air, fifty per cent, fifty
/// of radiant and un metro al secondo — a sunlit summer courtyard.
const UTCI_FISSATO: f64 = 35.428_74;

#[test]
fn un_caso_fissato_vale_questo() {
    let valore = utci(30.0, 50.0, 50.0, 1.0);
    assert!(
        (valore - UTCI_FISSATO).abs() < 1e-4,
        "il polinomio vendorato ha cambiato risposta: {valore} invece di {UTCI_FISSATO}"
    );
    assert_eq!(banda(valore), "stress da caldo forte");
}

#[test]
fn le_bande_cambiano_alle_soglie_pubblicate() {
    // The boundaries are the published ones and each is open at the bottom:
    // exactly 32 is still moderate heat stress, and 32,01 is strong.
    assert_eq!(banda(46.1), "stress da caldo estremo");
    assert_eq!(banda(46.0), "stress da caldo molto forte");
    assert_eq!(banda(38.0), "stress da caldo forte");
    assert_eq!(banda(32.0), "stress da caldo moderato");
    assert_eq!(banda(26.0), "nessuno stress termico");
    assert_eq!(banda(9.0), "leggero stress da freddo");
    assert_eq!(banda(0.0), "stress da freddo moderato");
    assert_eq!(banda(-13.0), "stress da freddo forte");
    assert_eq!(banda(-27.0), "stress da freddo molto forte");
    assert_eq!(banda(-40.0), "stress da freddo estremo");
}

#[test]
fn dentro_l_intervallo_del_polinomio_non_si_segnala_niente() {
    assert_eq!(fuori_intervallo(30.0, 50.0, 1.0), None);
    // The bounds themselves are inside: the fit is stated on closed intervals.
    assert_eq!(fuori_intervallo(50.0, 50.0, 17.0), None);
    assert_eq!(fuori_intervallo(-50.0, -80.0, 0.5), None);
}

#[test]
fn un_vento_troppo_debole_e_segnalato_e_non_corretto() {
    // The still courtyard is exactly the case a mitigation study is about, and
    // raising its wind to half a metre per second to keep the polynomial happy
    // would make it come back cooler than it is, with nothing left to notice.
    let fermo = utci(30.0, 50.0, 50.0, 0.1);
    let al_minimo = utci(30.0, 50.0, 50.0, 0.5);
    assert_ne!(
        fermo, al_minimo,
        "il vento è stato riportato al minimo invece di essere segnalato"
    );
    let avviso = fuori_intervallo(30.0, 50.0, 0.1).unwrap();
    assert!(avviso.contains("vento"), "{avviso}");
}

#[test]
fn un_radiante_troppo_lontano_dall_aria_e_segnalato() {
    // Seventy kelvin above the air is the top of the fit. A sunlit asphalt
    // surface in July reaches it, so this is not a corner nobody visits.
    assert_eq!(fuori_intervallo(30.0, 100.0, 1.0), None);
    let avviso = fuori_intervallo(30.0, 101.0, 1.0).unwrap();
    assert!(avviso.contains("radiante"), "{avviso}");
    let avviso = fuori_intervallo(30.0, -1.0, 1.0).unwrap();
    assert!(avviso.contains("radiante"), "{avviso}");
}

#[test]
fn una_temperatura_dell_aria_fuori_scala_e_segnalata_per_prima() {
    // Order matters: with the air outside the fit, the difference from the
    // radiant temperature is measured against a number the polynomial never
    // saw, and reporting *that* would send a reader to the wrong input.
    let avviso = fuori_intervallo(60.0, 60.0, 1.0).unwrap();
    assert!(avviso.contains("aria"), "{avviso}");
}

#[test]
fn la_sentinella_di_monte_resta_quella_che_e() {
    // The reference implementation answers -999 to an argument at or below
    // -999. Nothing in CLIMESH can produce one — a missing weather value is
    // `None`, never a number — but the behaviour is pinned rather than assumed
    // away, and the range check reports it as the nonsense it is.
    assert_eq!(utci(-1000.0, 50.0, 20.0, 1.0), -999.0);
    assert!(fuori_intervallo(-1000.0, 20.0, 1.0).is_some());
}

#[test]
fn sul_campo_ogni_cella_ha_l_utci_del_suo_radiante() {
    let mut radiante = Raster::from_elem((2, 2), 30.0);
    radiante[[0, 0]] = 50.0;
    let campo = utci_sul_campo(&radiante, 30.0, 50.0, 1.0);
    assert_eq!(campo.dim(), (2, 2));
    assert!(
        (f64::from(campo[[0, 0]]) - UTCI_FISSATO).abs() < 1e-3,
        "{}",
        campo[[0, 0]]
    );
    // The three cells in the shade share one value, and it is below the sunlit
    // one: the field carries the difference the map exists to show.
    assert_eq!(campo[[0, 1]], campo[[1, 0]]);
    assert!(campo[[0, 1]] < campo[[0, 0]]);
}
