//! From the objects of a Scenario to the co-registered rasters. Degenerate
//! inputs first: every coverage rule below is the one written in
//! `src/derivazione.rs`, and each test fails if that rule is dropped.

use climesh::derivazione::{
    deriva, RasterDiScenario, Stagione, CLASSE_ACQUA, CLASSE_ERBA, CLASSE_NESSUNA,
    CLASSE_PAVIMENTATO, CLASSE_TERRENO_NUDO,
};
use climesh::dominio::*;
use std::path::PathBuf;

fn griglia(nx: usize, ny: usize) -> Griglia {
    Griglia {
        nx,
        ny,
        passo_m: 1.0,
        crs: "EPSG:4326".to_owned(),
        origine: (0.0, 0.0),
        rotazione_gradi: 0.0,
    }
}

fn scenario(terreno_m: Vec<f32>) -> Scenario {
    Scenario {
        nome: "prova".to_owned(),
        derivato_da: None,
        terreno_m,
        provenienza: Provenienza {
            origine: "test".to_owned(),
            altezza: FonteAltezza::Rilievo,
        },
        edifici: Vec::new(),
        alberi: Vec::new(),
        superfici: Vec::new(),
    }
}

fn periodo(mese: u32, giorno: u32) -> Periodo {
    Periodo {
        nome: "prova".to_owned(),
        meteo: PathBuf::from("meteo.epw"),
        ore: 24,
        direzione_vento_gradi: None,
        inizio: Data {
            anno: 2021,
            mese,
            giorno,
        },
    }
}

/// 15 July: inside the leaf window of the Motore.
fn estate() -> Periodo {
    periodo(7, 15)
}

/// 15 January: outside it.
fn inverno() -> Periodo {
    periodo(1, 15)
}

fn rettangolo(x_min_m: f64, y_min_m: f64, x_max_m: f64, y_max_m: f64) -> Rettangolo {
    Rettangolo {
        x_min_m,
        y_min_m,
        x_max_m,
        y_max_m,
    }
}

fn edificio(altezza_m: f32, impronta: Vec<Rettangolo>) -> Edificio {
    Edificio {
        altezza_m,
        provenienza: None,
        impronta,
    }
}

/// A tree of the reference case: the plant ids are the ones in `src/specie.rs`,
/// never invented. `020060` is the deciduous London Plane, `020027` the pine.
fn albero(specie: &str, posizione_m: Posizione, altezza_m: f32, frazione_tronco: f64) -> Albero {
    Albero {
        posizione_m,
        specie: specie.to_owned(),
        altezza_m,
        frazione_tronco,
        provenienza: None,
    }
}

fn non_nulle(r: &climesh::derivazione::Raster) -> usize {
    r.iter().filter(|&&v| v != 0.0).count()
}

#[test]
fn un_rettangolo_oltre_il_bordo_copre_solo_le_celle_dentro() {
    let mut s = scenario(vec![0.0; 9]);
    // Far past every side of a 3 × 3 metre grid, and starting outside it.
    s.edifici.push(edificio(
        4.0,
        vec![rettangolo(-100.0, -100.0, 100.0, 100.0)],
    ));
    let d = deriva(&griglia(3, 3), &s, &estate()).unwrap();
    assert_eq!(d.scelte.celle_costruite, 9);
    assert!(d.modello_di_superficie.iter().all(|&v| v == 4.0));
}

#[test]
fn un_rettangolo_tutto_fuori_non_copre_nulla() {
    let mut s = scenario(vec![0.0; 9]);
    s.edifici
        .push(edificio(4.0, vec![rettangolo(10.0, 10.0, 20.0, 20.0)]));
    let d = deriva(&griglia(3, 3), &s, &estate()).unwrap();
    assert_eq!(d.scelte.celle_costruite, 0);
    assert_eq!(d.scelte.rettangoli_degeneri, 0);
    assert!(d.modello_di_superficie.iter().all(|&v| v == 0.0));
}

#[test]
fn un_rettangolo_di_area_nulla_o_rovesciato_non_copre_e_viene_contato() {
    let mut s = scenario(vec![0.0; 9]);
    s.edifici.push(edificio(
        4.0,
        vec![
            rettangolo(1.0, 1.0, 1.0, 2.0), // zero width
            rettangolo(2.0, 2.0, 1.0, 1.0), // minimum and maximum swapped
            rettangolo(0.0, 2.0, 3.0, 0.0), // swapped on one axis only
        ],
    ));
    let d = deriva(&griglia(3, 3), &s, &estate()).unwrap();
    assert_eq!(d.scelte.rettangoli_degeneri, 3);
    assert_eq!(d.scelte.celle_costruite, 0);
    assert!(d.modello_di_superficie.iter().all(|&v| v == 0.0));
}

#[test]
fn due_rettangoli_che_condividono_un_lato_coprono_ogni_cella_una_volta() {
    // The shared side sits at x = 2.5, that is on the centre of the third cell:
    // a closed interval would hand that cell to both rectangles, a half-open one
    // hands it to the eastern rectangle alone. The western building is the taller
    // of the two, so a double coverage would leave 5.0 where 3.0 belongs.
    let mut s = scenario(vec![0.0; 4]);
    s.edifici
        .push(edificio(5.0, vec![rettangolo(0.5, 0.0, 2.5, 1.0)]));
    s.edifici
        .push(edificio(3.0, vec![rettangolo(2.5, 0.0, 4.0, 1.0)]));
    let d = deriva(&griglia(4, 1), &s, &estate()).unwrap();
    assert_eq!(
        d.modello_di_superficie
            .iter()
            .copied()
            .collect::<Vec<f32>>(),
        vec![5.0, 5.0, 3.0, 3.0]
    );
    assert_eq!(d.scelte.celle_costruite, 4);
}

#[test]
fn una_cella_appartiene_al_rettangolo_che_contiene_il_suo_centro() {
    // Corners instead of centres would cover cells 1 and 2 rather than 0 and 1.
    let mut s = scenario(vec![0.0; 4]);
    s.edifici
        .push(edificio(2.0, vec![rettangolo(0.5, 0.0, 2.5, 1.0)]));
    let d = deriva(&griglia(4, 1), &s, &estate()).unwrap();
    assert_eq!(
        d.modello_di_superficie
            .iter()
            .copied()
            .collect::<Vec<f32>>(),
        vec![2.0, 2.0, 0.0, 0.0]
    );
}

#[test]
fn un_albero_sul_confine_fra_due_celle_va_in_una_sola() {
    // x = 1.0 is the side shared by cell 0 and cell 1: the rule gives it to the
    // cell whose minimum it is, the eastern one.
    let mut s = scenario(vec![0.0; 3]);
    s.alberi.push(albero("020027", (1.0, 0.5), 10.0, 0.5));
    let d = deriva(&griglia(3, 1), &s, &estate()).unwrap();
    assert_eq!(
        d.chiome.iter().copied().collect::<Vec<f32>>(),
        vec![0.0, 10.0, 0.0]
    );
    assert_eq!(d.scelte.celle_con_chioma, 1);
    assert_eq!(d.scelte.oggetti_fuori_griglia, 0);
}

#[test]
fn un_albero_fuori_dalla_griglia_e_contato() {
    let mut s = scenario(vec![0.0; 4]);
    s.alberi.push(albero("020027", (-0.5, 0.5), 10.0, 0.5)); // west of the grid
    s.alberi.push(albero("020027", (0.5, 9.0), 10.0, 0.5)); // north of it
    s.alberi.push(albero("020027", (0.5, 0.5), 10.0, 0.5)); // inside
    let d = deriva(&griglia(2, 2), &s, &estate()).unwrap();
    assert_eq!(d.scelte.oggetti_fuori_griglia, 2);
    assert_eq!(d.scelte.celle_con_chioma, 1);
}

#[test]
fn un_terreno_di_lunghezza_sbagliata_diventa_piatto_e_viene_registrato() {
    let s = scenario(vec![7.0, 7.0]); // two values for nine cells
    let d = deriva(&griglia(3, 3), &s, &estate()).unwrap();
    assert_eq!(d.scelte.terreno_sostituito, Some(2));
    assert!(d.modello_di_terreno.iter().all(|&v| v == 0.0));
    assert_eq!(d.modello_di_terreno.dim(), (3, 3));
}

#[test]
fn un_terreno_della_lunghezza_giusta_non_viene_sostituito() {
    let s = scenario(vec![1.0, 2.0, 3.0, 4.0]);
    let d = deriva(&griglia(2, 2), &s, &estate()).unwrap();
    assert_eq!(d.scelte.terreno_sostituito, None);
    assert_eq!(
        d.modello_di_terreno.iter().copied().collect::<Vec<f32>>(),
        vec![1.0, 2.0, 3.0, 4.0]
    );
}

#[test]
fn uno_scenario_senza_alberi_in_un_periodo_senza_foglie_non_esclude_nulla() {
    let s = scenario(vec![0.0; 4]);
    let d = deriva(&griglia(2, 2), &s, &inverno()).unwrap();
    assert_eq!(d.scelte.chiome_escluse, 0);
    assert_eq!(d.scelte.celle_con_chioma, 0);
    assert!(d.chiome.iter().all(|&v| v == 0.0));
}

#[test]
fn i_decidui_escono_dal_raster_delle_chiome_nel_periodo_senza_foglie() {
    let mut s = scenario(vec![0.0; 4]);
    s.alberi.push(albero("020060", (0.5, 0.5), 15.0, 0.35)); // London Plane
    s.alberi.push(albero("0000PR", (1.5, 0.5), 12.0, 0.35)); // Tilia
    let d = deriva(&griglia(2, 2), &s, &inverno()).unwrap();
    assert_eq!(d.scelte.chiome_escluse, 2);
    assert_eq!(d.scelte.celle_con_chioma, 0);
    assert!(d.chiome.iter().all(|&v| v == 0.0));
    assert!(d.zona_tronco.iter().all(|&v| v == 0.0));

    let estiva = deriva(&griglia(2, 2), &s, &estate()).unwrap();
    assert_eq!(estiva.scelte.chiome_escluse, 0);
    assert_eq!(estiva.scelte.celle_con_chioma, 2);
}

#[test]
fn un_sempreverde_resta_nel_raster_delle_chiome_senza_foglie() {
    let mut s = scenario(vec![0.0; 4]);
    s.alberi.push(albero("020027", (0.5, 0.5), 15.0, 0.45)); // Pine
    let d = deriva(&griglia(2, 2), &s, &inverno()).unwrap();
    assert_eq!(d.scelte.chiome_escluse, 0);
    assert_eq!(d.scelte.celle_con_chioma, 1);
}

#[test]
fn le_altezze_sono_assolute_e_la_zona_tronco_e_una_frazione_della_chioma() {
    let mut s = scenario(vec![2.0]);
    s.edifici
        .push(edificio(6.0, vec![rettangolo(0.0, 0.0, 1.0, 1.0)]));
    s.alberi.push(albero("020027", (0.5, 0.5), 12.0, 0.25));
    let d = deriva(&griglia(1, 1), &s, &estate()).unwrap();
    assert_eq!(d.modello_di_terreno[[0, 0]], 2.0);
    assert_eq!(d.modello_di_superficie[[0, 0]], 8.0);
    assert_eq!(d.chiome[[0, 0]], 14.0);
    assert_eq!(d.zona_tronco[[0, 0]], 5.0);
}

#[test]
fn due_alberi_sulla_stessa_cella_lasciano_la_chioma_piu_alta_e_il_suo_tronco() {
    let mut s = scenario(vec![0.0]);
    s.alberi.push(albero("020027", (0.5, 0.5), 20.0, 0.8));
    s.alberi.push(albero("020027", (0.5, 0.5), 10.0, 0.2));
    let d = deriva(&griglia(1, 1), &s, &estate()).unwrap();
    assert_eq!(d.chiome[[0, 0]], 20.0);
    assert_eq!(d.zona_tronco[[0, 0]], 16.0);
    assert_eq!(d.scelte.celle_con_chioma, 1);

    // The same two trees in the other order must give the same raster.
    s.alberi.reverse();
    let girato = deriva(&griglia(1, 1), &s, &estate()).unwrap();
    assert_eq!(girato.chiome, d.chiome);
    assert_eq!(girato.zona_tronco, d.zona_tronco);
}

#[test]
fn la_riga_zero_e_a_nord() {
    // Terrain is written row 0 northernmost; the building covers the northern
    // metre and the tree stands in the southern one. A north-south flip swaps
    // both.
    let mut s = scenario(vec![9.0, 1.0]);
    s.edifici
        .push(edificio(1.0, vec![rettangolo(0.0, 1.0, 1.0, 2.0)]));
    s.alberi.push(albero("020027", (0.5, 0.5), 3.0, 0.5));
    let d = deriva(&griglia(1, 2), &s, &estate()).unwrap();
    assert_eq!(
        d.modello_di_terreno.iter().copied().collect::<Vec<f32>>(),
        vec![9.0, 1.0]
    );
    assert_eq!(
        d.modello_di_superficie
            .iter()
            .copied()
            .collect::<Vec<f32>>(),
        vec![10.0, 1.0]
    );
    assert_eq!(
        d.chiome.iter().copied().collect::<Vec<f32>>(),
        vec![0.0, 4.0]
    );
}

#[test]
fn le_classi_di_superficie_seguono_la_stessa_regola_di_copertura() {
    let mut s = scenario(vec![0.0; 4]);
    s.superfici.push(Superficie {
        tipo: TipoSuperficie::Erba,
        impronta: vec![rettangolo(0.0, 0.0, 1.0, 1.0)],
    });
    s.superfici.push(Superficie {
        tipo: TipoSuperficie::Acqua,
        impronta: vec![rettangolo(1.0, 0.0, 2.0, 1.0)],
    });
    s.superfici.push(Superficie {
        tipo: TipoSuperficie::Pavimentato,
        impronta: vec![rettangolo(0.0, 1.0, 1.0, 2.0)],
    });
    let d = deriva(&griglia(2, 2), &s, &estate()).unwrap();
    assert_eq!(
        d.classi_di_superficie.iter().copied().collect::<Vec<u8>>(),
        vec![
            CLASSE_PAVIMENTATO,
            CLASSE_NESSUNA,
            CLASSE_ERBA,
            CLASSE_ACQUA
        ]
    );
    assert_ne!(CLASSE_TERRENO_NUDO, CLASSE_NESSUNA);
}

#[test]
fn una_griglia_di_una_cella_funziona() {
    let s = scenario(vec![3.0]);
    let d = deriva(&griglia(1, 1), &s, &estate()).unwrap();
    assert_eq!(d.modello_di_superficie.dim(), (1, 1));
    assert_eq!(d.modello_di_superficie[[0, 0]], 3.0);
}

#[test]
fn una_griglia_che_non_sta_in_un_usize_e_rifiutata() {
    let mut g = griglia(usize::MAX, 2);
    g.ny = 2;
    let s = scenario(Vec::new());
    let errore = deriva(&g, &s, &estate()).unwrap_err();
    assert!(
        errore.to_string().contains(&usize::MAX.to_string()),
        "the message must name the grid it refuses: {errore}"
    );
}

#[test]
fn una_data_impossibile_tiene_le_foglie() {
    // 30 February has no day of the year. Removing shade the model cannot
    // justify is the worse of the two errors, so the canopy stays.
    let impossibile = periodo(2, 30);
    assert_eq!(impossibile.inizio.giorno_dell_anno(), None);
    assert_eq!(Stagione::da_periodo(&impossibile), Stagione::ConFoglie);

    let mut s = scenario(vec![0.0]);
    s.alberi.push(albero("020060", (0.5, 0.5), 15.0, 0.35));
    let d = deriva(&griglia(1, 1), &s, &impossibile).unwrap();
    assert_eq!(d.scelte.chiome_escluse, 0);
    assert_eq!(d.chiome[[0, 0]], 15.0);
}

#[test]
fn la_stagione_si_decide_dalla_finestra_con_foglie_del_motore() {
    // Day 100 to day 300 inclusive.
    assert_eq!(Stagione::da_periodo(&periodo(4, 10)), Stagione::ConFoglie); // day 100
    assert_eq!(Stagione::da_periodo(&periodo(4, 9)), Stagione::SenzaFoglie); // day 99
    assert_eq!(Stagione::da_periodo(&periodo(10, 27)), Stagione::ConFoglie); // day 300
    assert_eq!(
        Stagione::da_periodo(&periodo(10, 28)),
        Stagione::SenzaFoglie
    ); // day 301
}

#[test]
fn due_derivazioni_dello_stesso_scenario_danno_gli_stessi_byte() {
    let progetto = climesh::progetto::leggi("casi/bastia/progetto").unwrap();
    let s = &progetto.scenari[0];
    let prima = deriva(&progetto.griglia, s, &progetto.periodi[0]).unwrap();
    let seconda = deriva(&progetto.griglia, s, &progetto.periodi[0]).unwrap();
    for (a, b) in [
        (&prima.modello_di_superficie, &seconda.modello_di_superficie),
        (&prima.modello_di_terreno, &seconda.modello_di_terreno),
        (&prima.chiome, &seconda.chiome),
        (&prima.zona_tronco, &seconda.zona_tronco),
    ] {
        let bit =
            |r: &climesh::derivazione::Raster| r.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        assert_eq!(bit(a), bit(b));
    }
    assert_eq!(prima.classi_di_superficie, seconda.classi_di_superficie);
    assert_eq!(prima.scelte, seconda.scelte);
}

#[test]
fn il_caso_di_riferimento_deriva_in_entrambe_le_stagioni() {
    let progetto = climesh::progetto::leggi("casi/bastia/progetto").unwrap();
    let estivo: Vec<RasterDiScenario> = progetto
        .scenari
        .iter()
        .map(|s| deriva(&progetto.griglia, s, &periodo(7, 15)).unwrap())
        .collect();
    let invernale: Vec<RasterDiScenario> = progetto
        .scenari
        .iter()
        .map(|s| deriva(&progetto.griglia, s, &periodo(1, 15)).unwrap())
        .collect();
    let celle = progetto.griglia.celle().unwrap();
    for (estivo, invernale) in estivo.iter().zip(&invernale) {
        assert_eq!(estivo.modello_di_superficie.len(), celle);
        assert_eq!(estivo.scelte.terreno_sostituito, None);
        assert_eq!(estivo.scelte.oggetti_fuori_griglia, 0);
        assert_eq!(estivo.scelte.rettangoli_degeneri, 0);
        assert!(estivo.scelte.celle_costruite > 0);
        assert!(
            non_nulle(&invernale.chiome) < non_nulle(&estivo.chiome),
            "the winter canopy must be the sparser one"
        );
        assert!(invernale.scelte.chiome_escluse > 0);
    }
}
