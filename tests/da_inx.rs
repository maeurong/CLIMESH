//! From an `.INX` to a Progetto: the species table, the conversion, and the
//! generator of the reference case. Degenerate inputs first.

use climesh::specie;

#[test]
fn la_tabella_delle_specie_copre_le_cinque_del_caso() {
    // Names as they are written in casi/bastia/valori-di-riferimento.toml, which
    // read them from the file itself.
    // The estimates are pinned and not merely checked for being a number: a
    // canopy silently moved from fifteen metres to forty changes every shadow the
    // Motore casts, and a test that only asks for "positive" would not notice.
    let attese = [
        ("020027", ".Pine Tree (middle)", 15.0, 0.45),
        ("020060", ".London Plane Tree (middle)", 15.0, 0.35),
        ("0000PR", ".Tilia", 12.0, 0.35),
        ("0000PA", ".Populus Alba", 18.0, 0.30),
        ("020111", ".Hanging Birch (middle)", 10.0, 0.35),
    ];
    for (plant_id, nome, altezza, tronco) in attese {
        assert_eq!(specie::nome(plant_id), Some(nome));
        assert_eq!(
            specie::altezza_di_chioma_m(plant_id),
            altezza,
            "{plant_id}: canopy height"
        );
        assert_eq!(
            specie::frazione_tronco(plant_id),
            tronco,
            "{plant_id}: trunk fraction"
        );
    }
}

#[test]
fn i_valori_predefiniti_sono_quelli_di_un_albero_modesto() {
    // A species the table does not know is a species nobody has checked, and
    // overstating a canopy overstates the shade it casts. The two numbers are
    // pinned for the same reason as the table's.
    assert_eq!(specie::ALTEZZA_PREDEFINITA_M, 5.0);
    assert_eq!(specie::FRAZIONE_TRONCO_PREDEFINITA, 0.3);
}

#[test]
fn il_platano_e_il_tiglio_sono_decidui_il_pino_no() {
    assert!(
        specie::e_decidua("020060"),
        "London Plane Tree is deciduous"
    );
    assert!(specie::e_decidua("0000PR"), "Tilia is deciduous");
    assert!(!specie::e_decidua("020027"), "a pine keeps its needles");
}

#[test]
fn una_specie_sconosciuta_prende_i_valori_predefiniti_e_non_e_decidua() {
    assert_eq!(specie::nome("ZZZZZZ"), None);
    assert_eq!(
        specie::altezza_di_chioma_m("ZZZZZZ"),
        specie::ALTEZZA_PREDEFINITA_M
    );
    assert_eq!(
        specie::frazione_tronco("ZZZZZZ"),
        specie::FRAZIONE_TRONCO_PREDEFINITA
    );
    // Dropping shade the model cannot justify is the worse of the two errors.
    assert!(!specie::e_decidua("ZZZZZZ"));
}

// --- Dal file al Progetto -------------------------------------------------

use climesh::da_inx::{self, progetto_da_inx};
use climesh::dominio::*;
use climesh::inx::parse_inx;

/// Wraps a body in the smallest header a valid `.INX` file has. The grid is
/// deliberately neither square nor 50x50, and the step is not one metre.
fn wrap(dx: f64, dy: f64, body: &str) -> String {
    wrap_griglia(3, 2, dx, dy, body)
}

/// The same header on a grid of any size, for the cases three by two cannot show.
fn wrap_griglia(nx: usize, ny: usize, dx: f64, dy: f64, body: &str) -> String {
    format!(
        "<ENVI-MET_Datafile>
<modelGeometry>
   <grids-I> {nx} </grids-I>
   <grids-J> {ny} </grids-J>
   <grids-Z> 4 </grids-Z>
   <dx> {dx} </dx>
   <dy> {dy} </dy>
   <dz-base> 2.00000 </dz-base>
</modelGeometry>
<locationData>
   <modelRotation> 21.00000 </modelRotation>
   <locationName> bergamo </locationName>
   <location_Longitude> 12.56000 </location_Longitude>
   <location_Latitude> 43.07000 </location_Latitude>
   <locationTimeZone_Name> CET/ UTC+1 </locationTimeZone_Name>
   <locationTimeZone_Longitude> 15.00000 </locationTimeZone_Longitude>
</locationData>
{body}
</ENVI-MET_Datafile>
"
    )
}

fn converti(body: &str) -> Progetto {
    let letto = parse_inx(&wrap(1.5, 1.5, body)).unwrap();
    progetto_da_inx(&letto, "bastia", "stato-di-fatto").unwrap()
}

/// The same conversion on a square grid of `lato` cells with a one-metre step.
fn converti_griglia(lato: usize, body: &str) -> Progetto {
    let letto = parse_inx(&wrap_griglia(lato, lato, 1.0, 1.0, body)).unwrap();
    progetto_da_inx(&letto, "bastia", "stato-di-fatto").unwrap()
}

/// Whether `r` covers the centre of ENVI-met cell `(i, j)` on a one-metre grid.
fn copre(r: &Rettangolo, i: usize, j: usize) -> bool {
    let (x, y) = da_inx::centro_cella_m(i, j, 1.0);
    r.x_min_m < x && x < r.x_max_m && r.y_min_m < y && y < r.y_max_m
}

/// How many of the cells of a `lato` by `lato` grid the rectangles cover, counting
/// a cell once per rectangle that covers it.
fn celle_coperte(impronta: &[Rettangolo], lato: usize) -> usize {
    (1..=lato)
        .flat_map(|j| (1..=lato).map(move |i| (i, j)))
        .map(|(i, j)| impronta.iter().filter(|r| copre(r, i, j)).count())
        .sum()
}

fn rettangolo(x_min_m: f64, y_min_m: f64, x_max_m: f64, y_max_m: f64) -> Rettangolo {
    Rettangolo {
        x_min_m,
        y_min_m,
        x_max_m,
        y_max_m,
    }
}

const EDIFICI: &str = "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   6,6,0
   0,3,0
   </zTop>
</buildings2D>";

#[test]
fn una_griglia_di_dimensione_qualsiasi_e_convertita_lo_stesso() {
    let p = converti(EDIFICI);
    assert_eq!((p.griglia.nx, p.griglia.ny), (3, 2));
    assert_eq!(p.griglia.passo_m, 1.5);
    assert_eq!(p.griglia.rotazione_gradi, 21.0);
    assert_eq!(p.griglia.origine, (12.56, 43.07));
    assert_eq!(p.scenari[0].terreno_m.len(), 6);
}

#[test]
fn le_celle_costruite_contigue_alla_stessa_altezza_formano_un_edificio() {
    let edifici = &converti(EDIFICI).scenari[0].edifici;
    assert_eq!(edifici.len(), 2, "two blocks, two Edifici: {edifici:#?}");
    let alto = edifici.iter().find(|e| e.altezza_m == 6.0).unwrap();
    // The first row written is the northernmost one, j = 2, and the two cells of
    // that block leave one rectangle and not two.
    assert_eq!(alto.impronta, vec![rettangolo(0.0, 1.5, 3.0, 3.0)]);
    let basso = edifici.iter().find(|e| e.altezza_m == 3.0).unwrap();
    assert_eq!(basso.impronta, vec![rettangolo(1.5, 0.0, 3.0, 1.5)]);
}

#[test]
fn due_blocchi_separati_alla_stessa_altezza_sono_due_edifici() {
    // Two blocks six metres tall with three free columns between them: grouping
    // by height alone would weld them into one Edificio.
    let p = converti_griglia(
        5,
        "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"5\" dataJ=\"5\">
   0,0,0,0,0
   6,0,0,6,6
   6,0,0,6,6
   0,0,0,0,0
   0,0,0,0,0
   </zTop>
</buildings2D>",
    );
    let edifici = &p.scenari[0].edifici;
    assert_eq!(edifici.len(), 2, "two blocks, two Edifici: {edifici:#?}");
    assert!(edifici.iter().all(|e| e.altezza_m == 6.0));
    let mut celle: Vec<usize> = edifici
        .iter()
        .map(|e| celle_coperte(&e.impronta, 5))
        .collect();
    celle.sort_unstable();
    assert_eq!(celle, vec![2, 4]);
}

#[test]
fn un_blocco_a_elle_e_un_edificio_solo_e_l_impronta_lo_copre() {
    let p = converti_griglia(
        3,
        "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"3\" dataJ=\"3\">
   3,0,0
   3,3,0
   0,0,0
   </zTop>
</buildings2D>",
    );
    let edifici = &p.scenari[0].edifici;
    assert_eq!(edifici.len(), 1, "an L is one block: {edifici:#?}");
    // Three cells, each covered once: no rectangle spills over the notch.
    assert_eq!(celle_coperte(&edifici[0].impronta, 3), 3);
    assert!(copre(&edifici[0].impronta[0], 1, 3) || copre(&edifici[0].impronta[1], 1, 3));
    for impronta in &edifici[0].impronta {
        assert!(
            !copre(impronta, 2, 3),
            "the notch stays empty: {impronta:?}"
        );
    }
}

#[test]
fn senza_la_matrice_degli_edifici_non_ci_sono_edifici() {
    let p = converti(
        "<soils2D>
   <ID_soilprofile type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   000000,000000,000000
   000000,000000,000000
   </ID_soilprofile>
</soils2D>",
    );
    assert!(
        p.scenari[0].edifici.is_empty(),
        "an absent section is no Edificio, not an Edificio at height zero"
    );
}

#[test]
fn le_celle_a_quota_zero_non_sono_un_edificio() {
    let p = converti(
        "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   0,0,0
   0,0,0
   </zTop>
</buildings2D>",
    );
    assert!(p.scenari[0].edifici.is_empty());
}

#[test]
fn senza_la_sezione_dei_suoli_non_ci_sono_superfici() {
    let p = converti(EDIFICI);
    assert!(
        p.scenari[0].superfici.is_empty(),
        "no soils section is no Superficie, which is not a domain of bare ground"
    );
}

#[test]
fn la_sezione_dei_suoli_senza_piante_diventa_una_sola_superficie_di_terreno_nudo() {
    let p = converti(
        "<soils2D>
   <ID_soilprofile type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   000000,000000,000000
   000000,000000,000000
   </ID_soilprofile>
</soils2D>",
    );
    let superfici = &p.scenari[0].superfici;
    assert_eq!(superfici.len(), 1);
    assert_eq!(superfici[0].tipo, TipoSuperficie::TerrenoNudo);
    // Six cells, one rectangle: the whole grid.
    assert_eq!(superfici[0].impronta, vec![rettangolo(0.0, 0.0, 4.5, 3.0)]);
}

#[test]
fn una_cella_con_codice_pianta_e_erba_anche_senza_profilo_di_suolo() {
    let p = converti_griglia(
        3,
        "<simpleplants2D>
   <ID_plants1D type=\"matrix-data\" dataI=\"3\" dataJ=\"3\">
   0100XX,0100XX,0100XX
   0100XX,0100XX,0100XX
   0100XX,0100XX,0100XX
   </ID_plants1D>
</simpleplants2D>",
    );
    let superfici = &p.scenari[0].superfici;
    assert_eq!(superfici.len(), 1, "grass and nothing else: {superfici:#?}");
    assert_eq!(superfici[0].tipo, TipoSuperficie::Erba);
    assert_eq!(celle_coperte(&superfici[0].impronta, 3), 9);
}

#[test]
fn il_codice_pianta_vince_sul_profilo_di_suolo_e_nessuna_cella_prende_due_classi() {
    // Every cell has a soil profile; the middle column also has a grass code.
    let p = converti_griglia(
        3,
        "<soils2D>
   <ID_soilprofile type=\"matrix-data\" dataI=\"3\" dataJ=\"3\">
   000000,000000,000000
   000000,000000,000000
   000000,000000,000000
   </ID_soilprofile>
</soils2D>
<simpleplants2D>
   <ID_plants1D type=\"matrix-data\" dataI=\"3\" dataJ=\"3\">
   ,0100XX,
   ,0100XX,
   ,0100XX,
   </ID_plants1D>
</simpleplants2D>",
    );
    let superfici = &p.scenari[0].superfici;
    let erba = superfici
        .iter()
        .find(|s| s.tipo == TipoSuperficie::Erba)
        .expect("the cells with a plant code are grass");
    let nudo = superfici
        .iter()
        .find(|s| s.tipo == TipoSuperficie::TerrenoNudo)
        .expect("the cells with only a soil profile are bare ground");
    assert_eq!(celle_coperte(&erba.impronta, 3), 3);
    assert_eq!(celle_coperte(&nudo.impronta, 3), 6);
    let tutte: Vec<Rettangolo> = superfici.iter().flat_map(|s| s.impronta.clone()).collect();
    assert_eq!(
        celle_coperte(&tutte, 3),
        9,
        "one class per cell, counted: {superfici:#?}"
    );
}

#[test]
fn un_buco_nella_maschera_non_e_coperto_da_nessun_rettangolo() {
    let p = converti_griglia(
        3,
        "<soils2D>
   <ID_soilprofile type=\"matrix-data\" dataI=\"3\" dataJ=\"3\">
   000000,000000,000000
   000000,,000000
   000000,000000,000000
   </ID_soilprofile>
</soils2D>",
    );
    let impronta = &p.scenari[0].superfici[0].impronta;
    assert_eq!(celle_coperte(impronta, 3), 8);
    for r in impronta {
        assert!(!copre(r, 2, 2), "the hole stays a hole: {r:?}");
    }
}

#[test]
fn il_terreno_e_scritto_riga_zero_a_nord() {
    let p = converti(
        "<dem>
   <terrainheight type=\"matrix-data\" dataI=\"3\" dataJ=\"2\">
   1,2,3
   4,5,6
   </terrainheight>
</dem>",
    );
    assert_eq!(p.scenari[0].terreno_m, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn senza_dem_il_terreno_e_piatto_e_lungo_quanto_la_griglia() {
    assert_eq!(converti(EDIFICI).scenari[0].terreno_m, vec![0.0; 6]);
}

const PINO: &str = "<3Dplants>
   <rootcell_i> 2 </rootcell_i>
   <rootcell_j> 1 </rootcell_j>
   <rootcell_k> 0 </rootcell_k>
   <plantID> 020027 </plantID>
   <name> .Pine Tree (middle) </name>
   <observe> 0 </observe>
</3Dplants>";

#[test]
fn un_albero_sta_al_centro_della_sua_cella() {
    let alberi = &converti(PINO).scenari[0].alberi;
    assert_eq!(alberi.len(), 1);
    assert_eq!(alberi[0].posizione_m, (2.25, 0.75));
    assert_eq!(alberi[0].specie, "020027");
    assert_eq!(alberi[0].altezza_m, specie::altezza_di_chioma_m("020027"));
    assert_eq!(alberi[0].frazione_tronco, specie::frazione_tronco("020027"));
}

#[test]
fn il_centro_di_cella_e_la_stessa_regola_per_i_punti_di_osservazione() {
    assert_eq!(da_inx::centro_cella_m(15, 18, 1.0), (14.5, 17.5));
    assert_eq!(da_inx::centro_cella_m(2, 1, 1.5), (2.25, 0.75));
}

#[test]
fn le_altezze_degli_edifici_sono_rilevate_quelle_degli_alberi_no() {
    let p = converti(&format!("{EDIFICI}\n{PINO}"));
    let s = &p.scenari[0];
    assert_eq!(
        s.edifici[0].provenienza.as_ref().unwrap().altezza,
        FonteAltezza::Rilievo
    );
    // The height of a plant lives in the ENVI-met plant database, not in the file.
    assert_eq!(s.provenienza.altezza, FonteAltezza::Predefinito);
    assert_eq!(s.alberi[0].provenienza, None);
}

#[test]
fn lo_scenario_porta_in_testa_la_provenienza_che_i_suoi_alberi_condividono() {
    let p = converti(&format!("{EDIFICI}\n{PINO}"));
    let s = &p.scenari[0];
    assert!(
        s.provenienza.origine.contains(".INX"),
        "the Scenario should say where it comes from, was: {}",
        s.provenienza.origine
    );
    // The catalogue name, once, so that `specie = "020027"` is decodable from the
    // same file instead of from the ENVI-met plant database.
    assert!(
        s.provenienza.origine.contains(".Pine Tree (middle)"),
        "the head of the file should name the species, was: {}",
        s.provenienza.origine
    );
    assert_eq!(
        s.alberi[0].provenienza, None,
        "an object that says nothing new says nothing"
    );
}

/// Provenance is worth its bytes only when it differs; the file is where that
/// shows.
#[test]
fn la_provenienza_ripetuta_non_finisce_in_ogni_albero_del_file() {
    let p = converti(&format!("{EDIFICI}\n{PINO}"));
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("provenienza-scenario");
    let _ = std::fs::remove_dir_all(&dir);
    climesh::progetto::scrivi(&dir, &p).unwrap();
    let testo = std::fs::read_to_string(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    assert_eq!(
        testo.matches("[alberi.provenienza]").count(),
        0,
        "the tree inherits the Scenario's provenance:\n{testo}"
    );
    assert_eq!(testo.matches("[provenienza]").count(), 1);
}

#[test]
fn la_frazione_di_tronco_si_scrive_come_e_stata_dichiarata() {
    let p = converti(PINO);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("frazione-tronco");
    let _ = std::fs::remove_dir_all(&dir);
    climesh::progetto::scrivi(&dir, &p).unwrap();
    let testo = std::fs::read_to_string(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    assert!(
        testo.contains("frazione_tronco = 0.45"),
        "0.45 declared, 0.45 written:\n{testo}"
    );
}

#[test]
fn un_terreno_uniforme_non_costa_un_numero_per_cella() {
    let p = converti(EDIFICI);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("terreno-uniforme");
    let _ = std::fs::remove_dir_all(&dir);
    climesh::progetto::scrivi(&dir, &p).unwrap();
    let testo = std::fs::read_to_string(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    let riga = testo
        .lines()
        .find(|r| r.starts_with("terreno_m"))
        .expect("the terrain is written");
    assert!(
        riga.len() < 30,
        "six equal values should not cost six numbers: {riga}"
    );
    assert_eq!(climesh::progetto::leggi(&dir).unwrap(), p);
}

#[test]
fn un_plant_id_sconosciuto_prende_i_predefiniti_e_la_provenienza_lo_dice() {
    let p = converti(
        "<3Dplants>
   <rootcell_i> 1 </rootcell_i>
   <rootcell_j> 1 </rootcell_j>
   <rootcell_k> 0 </rootcell_k>
   <plantID> ZZZZZZ </plantID>
   <name> .Nothing Known </name>
   <observe> 0 </observe>
</3Dplants>",
    );
    let albero = &p.scenari[0].alberi[0];
    assert_eq!(albero.altezza_m, specie::ALTEZZA_PREDEFINITA_M);
    assert_eq!(albero.frazione_tronco, specie::FRAZIONE_TRONCO_PREDEFINITA);
    assert!(!specie::e_decidua(&albero.specie));
    let propria = albero
        .provenienza
        .as_ref()
        .expect("an unknown species differs from the rest of the Scenario");
    assert!(
        propria.origine.contains("sconosciuta"),
        "provenance should say the species is unknown, was: {}",
        propria.origine
    );
}

#[test]
fn il_nome_del_progetto_e_un_parametro_non_quello_del_file() {
    let letto = parse_inx(&wrap(1.5, 1.5, EDIFICI)).unwrap();
    assert_eq!(letto.location.name, "bergamo", "a typo left in the file");
    let p = progetto_da_inx(&letto, "bastia", "stato-di-fatto").unwrap();
    assert_eq!(p.nome, "bastia");
    assert_eq!(p.scenari[0].nome, "stato-di-fatto");
    assert_eq!(p.scenari[0].derivato_da, None);
}

#[test]
fn un_passo_diverso_su_x_e_su_y_e_rifiutato_con_un_messaggio_che_lo_dice() {
    let letto = parse_inx(&wrap(1.5, 2.0, EDIFICI)).unwrap();
    let msg = progetto_da_inx(&letto, "bastia", "stato-di-fatto")
        .unwrap_err()
        .to_string();
    assert!(
        msg.contains("dx = 1.5") && msg.contains("dy = 2"),
        "error should tie each axis to its own value, was: {msg}"
    );
    assert!(
        msg.contains("riesporta"),
        "error should say what to do about it, was: {msg}"
    );
}

#[test]
fn un_albero_radicato_fuori_dalla_griglia_e_segnalato() {
    // parse_inx refuses such a file, but an Inx is a plain struct a caller can
    // also build, and a position outside the extent is one progetto::valida
    // would refuse later, further from the cause. All four sides of the grid are
    // checked: the cell indices are 1-based, so zero is outside just as much as
    // one past the last column.
    for (i, j) in [(9, 1), (2, 9), (0, 1), (2, 0)] {
        let mut letto = parse_inx(&wrap(1.5, 1.5, PINO)).unwrap();
        letto.plants[0].i = i;
        letto.plants[0].j = j;
        let msg = progetto_da_inx(&letto, "bastia", "stato-di-fatto")
            .unwrap_err()
            .to_string();
        assert!(
            msg.contains(&format!("({i};{j})")) && msg.contains("3 × 2"),
            "error should report the cell and the grid, was: {msg}"
        );
    }
}

#[test]
fn il_progetto_convertito_e_accettato_da_progetto_scrivi() {
    let p = converti(&format!("{EDIFICI}\n{PINO}"));
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("convertito");
    let _ = std::fs::remove_dir_all(&dir);
    climesh::progetto::scrivi(&dir, &p).unwrap();
    assert_eq!(climesh::progetto::leggi(&dir).unwrap(), p);
}

// --- Il generatore del caso di riferimento --------------------------------

use std::path::{Path, PathBuf};
use std::process::Command;

const MODELLO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/materiale università/LAB1.INX");

/// The species the reconstruction plants, pinned here so that a change of mind in
/// the generator has to be a change of mind in a test too. The report calls them
/// H2 and H4, codes whose `plantID` is not in the course material.
const SIEPE_DI_CONFINE: &str = "020060";
const VERDE_ADDOSSATO: &str = "0000PR";

/// A synthetic model with the shape of the reference one: fifty cells a side at
/// one metre, two blocks six metres tall either side of a free courtyard, a lower
/// block to the south, grass wherever nothing is built, and three plants.
///
/// Synthetic and not the course file, because `materiale università/` is not
/// redistributable: a check that quietly passes where the file is missing is a
/// check that proves nothing on any machine but this one.
fn modello_sintetico() -> String {
    const LATO: usize = 50;
    const PIANTE: [(usize, usize, &str); 3] =
        [(26, 30, "020027"), (27, 30, "020060"), (5, 5, "0000PA")];
    let quota = |i: usize, j: usize| -> f64 {
        let duplex = ((11..=23).contains(&i) || (32..=44).contains(&i)) && (26..=32).contains(&j);
        let simplex = (21..=34).contains(&i) && (18..=24).contains(&j);
        match (duplex, simplex) {
            (true, _) => 6.0,
            (_, true) => 3.0,
            _ => 0.0,
        }
    };
    let matrice = |cella: &dyn Fn(usize, usize) -> String| -> String {
        (1..=LATO)
            .rev()
            .map(|j| {
                (1..=LATO)
                    .map(|i| cella(i, j))
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("\n   ")
    };
    let z_top = matrice(&|i, j| format!("{:.1}", quota(i, j)));
    let suoli = matrice(&|_, _| "000000".to_owned());
    // Grass everywhere the model is neither built on nor planted, as in the file.
    let erba = matrice(&|i, j| {
        let piantata = PIANTE.iter().any(|&(pi, pj, _)| (pi, pj) == (i, j));
        if quota(i, j) > 0.0 || piantata {
            String::new()
        } else {
            "0100XX".to_owned()
        }
    });
    let piante: String = PIANTE
        .iter()
        .map(|(i, j, id)| {
            format!(
                "<3Dplants>
   <rootcell_i> {i} </rootcell_i>
   <rootcell_j> {j} </rootcell_j>
   <rootcell_k> 0 </rootcell_k>
   <plantID> {id} </plantID>
   <name> sintetica </name>
   <observe> 0 </observe>
</3Dplants>"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    wrap_griglia(
        LATO,
        LATO,
        1.0,
        1.0,
        &format!(
            "<buildings2D>
   <zTop type=\"matrix-data\" dataI=\"{LATO}\" dataJ=\"{LATO}\">
   {z_top}
   </zTop>
</buildings2D>
<soils2D>
   <ID_soilprofile type=\"matrix-data\" dataI=\"{LATO}\" dataJ=\"{LATO}\">
   {suoli}
   </ID_soilprofile>
</soils2D>
<simpleplants2D>
   <ID_plants1D type=\"matrix-data\" dataI=\"{LATO}\" dataJ=\"{LATO}\">
   {erba}
   </ID_plants1D>
</simpleplants2D>
{piante}"
        ),
    )
}

fn cartella_di_prova(nome: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Writes `testo` as an `.INX` under `dir` and returns its path.
fn modello_su_disco(dir: &Path, nome: &str, testo: &str) -> PathBuf {
    let percorso = dir.join(nome);
    std::fs::write(&percorso, testo).unwrap();
    percorso
}

/// Every file under `dir`, path and bytes, in a stable order.
fn albero_di_file(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut trovati = Vec::new();
    let mut da_visitare = vec![dir.to_path_buf()];
    while let Some(corrente) = da_visitare.pop() {
        for voce in std::fs::read_dir(&corrente).unwrap() {
            let percorso = voce.unwrap().path();
            if percorso.is_dir() {
                da_visitare.push(percorso);
            } else {
                let relativo = percorso.strip_prefix(dir).unwrap().to_path_buf();
                trovati.push((relativo, std::fs::read(&percorso).unwrap()));
            }
        }
    }
    trovati.sort();
    trovati
}

fn genera(uscita: &Path, modello: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_costruisci_caso"))
        .arg(uscita)
        .arg(modello)
        .output()
        .unwrap()
}

/// Builds the case from the synthetic model into a fresh directory and reads it
/// back, failing loudly with whatever the generator said.
fn caso_sintetico(nome: &str) -> (PathBuf, Progetto) {
    let dir = cartella_di_prova(nome);
    let modello = modello_su_disco(&dir, "sintetico.INX", &modello_sintetico());
    let uscita = dir.join("progetto");
    let esito = genera(&uscita, &modello);
    assert!(
        esito.status.success(),
        "{}",
        String::from_utf8_lossy(&esito.stderr)
    );
    let p = climesh::progetto::leggi(&uscita).unwrap();
    (uscita, p)
}

fn scenario_ricostruito(p: &Progetto) -> &Scenario {
    p.scenari
        .iter()
        .find(|s| s.derivato_da.is_some())
        .expect("the reconstructed Scenario")
}

#[test]
fn il_generatore_lascia_lo_stesso_albero_di_file_a_ogni_esecuzione() {
    let dir = cartella_di_prova("generatore-idempotente");
    let modello = modello_su_disco(&dir, "sintetico.INX", &modello_sintetico());
    let uscita = dir.join("progetto");
    assert!(genera(&uscita, &modello).status.success());
    let prima = albero_di_file(&uscita);
    assert!(genera(&uscita, &modello).status.success());
    assert_eq!(prima, albero_di_file(&uscita), "byte for byte, twice");
}

#[test]
fn il_progetto_generato_si_rilegge_e_ha_due_scenari_distinti() {
    let (_, p) = caso_sintetico("generatore-riletto");
    assert_eq!(p.scenari.len(), 2);
    assert_ne!(p.scenari[0].nome, p.scenari[1].nome);
    assert_eq!(p.periodi.len(), 2, "estate e inverno");
    assert_eq!((p.griglia.nx, p.griglia.ny), (50, 50));

    // The three points the report publishes time series for, in ENVI-met cell
    // indices, converted with the same cell-centre rule as the objects.
    assert_eq!(p.punti.len(), 3);
    assert_eq!(p.punti[0].posizione_m, da_inx::centro_cella_m(15, 18, 1.0));
    assert_eq!(p.punti[2].posizione_m, da_inx::centro_cella_m(37, 15, 1.0));

    let periodo = p.periodi.iter().find(|x| x.nome == "estate").unwrap();
    assert_eq!(periodo.ore, 48);
    assert_eq!(periodo.direzione_vento_gradi, Some(45.0));
    assert_eq!(periodo.inizio.giorno_dell_anno(), Some(196));

    let fatto = p.scenari.iter().find(|s| s.derivato_da.is_none()).unwrap();
    let interventi = scenario_ricostruito(&p);
    assert_eq!(interventi.derivato_da.as_deref(), Some(fatto.nome.as_str()));
    assert!(
        interventi.alberi.len() > fatto.alberi.len(),
        "the reconstructed Scenario adds vegetation"
    );
    assert!(
        interventi
            .superfici
            .iter()
            .any(|s| s.tipo == TipoSuperficie::Acqua),
        "the reconstructed Scenario lays down the pool"
    );
}

/// The Scenario says once, at the head of its file, what its objects would
/// otherwise repeat eight hundred times.
#[test]
fn lo_scenario_ricostruito_lo_dichiara_in_testa_al_file() {
    let (uscita, p) = caso_sintetico("generatore-provenienza");
    let interventi = scenario_ricostruito(&p);
    assert!(
        interventi.provenienza.origine.contains("ricostruit"),
        "the head of the file should say the Scenario is rebuilt, was: {}",
        interventi.provenienza.origine
    );
    // The species the report calls H2 and H4 stand in for two of the model, and
    // the file says which, by catalogue name.
    assert!(
        interventi
            .provenienza
            .origine
            .contains(".London Plane Tree (middle)")
            && interventi.provenienza.origine.contains(".Tilia"),
        "was: {}",
        interventi.provenienza.origine
    );
    let testo = std::fs::read_to_string(uscita.join("scenari/interventi.toml")).unwrap();
    let aggiunti = interventi
        .alberi
        .iter()
        .filter(|a| a.provenienza.is_none())
        .count();
    assert!(aggiunti > 100, "{aggiunti} reconstructed trees");
    assert_eq!(
        testo.matches("ricostruit").count(),
        1,
        "the sentence belongs at the head of the file, not in every tree"
    );
}

/// No object added by the reconstruction is left claiming to have been surveyed.
#[test]
fn gli_oggetti_ereditati_dicono_di_essere_rilevati() {
    let (_, p) = caso_sintetico("generatore-ereditati");
    let fatto = p.scenari.iter().find(|s| s.derivato_da.is_none()).unwrap();
    let interventi = scenario_ricostruito(&p);
    for albero in &interventi.alberi[..fatto.alberi.len()] {
        let propria = albero
            .provenienza
            .as_ref()
            .expect("a surveyed tree is not reconstructed and says so");
        assert!(
            propria.origine.contains("rilevato"),
            "was: {}",
            propria.origine
        );
    }
}

#[test]
fn le_siepi_stanno_al_bordo_e_il_verde_e_addossato_agli_edifici() {
    let (_, p) = caso_sintetico("generatore-siepi");
    let fatto = p.scenari.iter().find(|s| s.derivato_da.is_none()).unwrap();
    let interventi = scenario_ricostruito(&p);
    let aggiunti = &interventi.alberi[fatto.alberi.len()..];
    let in_cella = |i: usize, j: usize| {
        aggiunti
            .iter()
            .find(|a| a.posizione_m == da_inx::centro_cella_m(i, j, 1.0))
    };

    // All four sides of the grid, not just the first column.
    for (i, j) in [
        (1usize, 1usize),
        (50, 1),
        (1, 50),
        (50, 50),
        (25, 1),
        (1, 25),
    ] {
        let albero = in_cella(i, j).unwrap_or_else(|| panic!("no hedge at ({i};{j})"));
        assert_eq!(albero.specie, SIEPE_DI_CONFINE, "at ({i};{j})");
    }
    // West of the western duplex, which starts at i = 11 over j 26 to 32.
    assert_eq!(in_cella(10, 26).unwrap().specie, VERDE_ADDOSSATO);
    // South of the lower block, which spans j 18 to 24 over i 21 to 34.
    assert_eq!(in_cella(25, 17).unwrap().specie, VERDE_ADDOSSATO);
    // Free, inside, and against nothing: the reconstruction leaves it alone.
    assert!(in_cella(8, 8).is_none(), "open ground stays open");
    // Nothing is planted on a building or in the pool.
    for albero in aggiunti {
        assert!(
            !interventi.edifici.iter().any(|e| e
                .impronta
                .iter()
                .any(|r| copre_posizione(r, albero.posizione_m))),
            "a tree on a roof: {albero:?}"
        );
    }
}

fn copre_posizione(r: &Rettangolo, (x, y): Posizione) -> bool {
    r.x_min_m < x && x < r.x_max_m && r.y_min_m < y && y < r.y_max_m
}

#[test]
fn nessuna_cella_del_progetto_generato_prende_due_superfici() {
    let (_, p) = caso_sintetico("generatore-superfici");
    for scenario in &p.scenari {
        let tutte: Vec<Rettangolo> = scenario
            .superfici
            .iter()
            .flat_map(|s| s.impronta.clone())
            .collect();
        let coperte = celle_coperte(&tutte, 50);
        assert_eq!(
            coperte, 2500,
            "scenario {}: 2500 cells, one class each",
            scenario.nome
        );
    }
}

#[test]
fn lo_specchio_d_acqua_sta_nella_corte_e_toglie_il_terreno_sotto() {
    let (_, p) = caso_sintetico("generatore-vasca");
    let interventi = scenario_ricostruito(&p);
    let vasca = interventi
        .superfici
        .iter()
        .find(|s| s.tipo == TipoSuperficie::Acqua)
        .unwrap();
    // The courtyard between the two duplexes: i 24 to 31, j 26 to 28.
    assert_eq!(celle_coperte(&vasca.impronta, 50), 24);
    for j in 26..=28 {
        for i in 24..=31 {
            assert!(
                vasca.impronta.iter().any(|r| copre(r, i, j)),
                "the pool should cover ({i};{j})"
            );
            for altra in interventi
                .superfici
                .iter()
                .filter(|s| s.tipo != TipoSuperficie::Acqua)
            {
                assert!(
                    !altra.impronta.iter().any(|r| copre(r, i, j)),
                    "{:?} still covers ({i};{j}) under the pool",
                    altra.tipo
                );
            }
        }
    }
}

#[test]
fn uno_scenario_rinominato_non_lascia_orfani() {
    let dir = cartella_di_prova("generatore-orfani");
    let modello = modello_su_disco(&dir, "sintetico.INX", &modello_sintetico());
    let uscita = dir.join("progetto");
    assert!(genera(&uscita, &modello).status.success());
    // What a previous run under other names would have left behind.
    std::fs::copy(
        uscita.join("scenari/interventi.toml"),
        uscita.join("scenari/vecchio-nome.toml"),
    )
    .unwrap();
    std::fs::copy(
        uscita.join("periodi/estate.toml"),
        uscita.join("periodi/primavera.toml"),
    )
    .unwrap();
    // Not the generator's to remove: it owns two directories, not the Corse.
    std::fs::create_dir_all(uscita.join("corse")).unwrap();
    std::fs::write(uscita.join("corse/estate.toml"), "").unwrap();

    assert!(genera(&uscita, &modello).status.success());
    let rimasti: Vec<PathBuf> = albero_di_file(&uscita)
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    assert!(
        !rimasti
            .iter()
            .any(|p| p.ends_with("vecchio-nome.toml") || p.ends_with("primavera.toml")),
        "orphans left behind: {rimasti:?}"
    );
    assert!(
        rimasti.iter().any(|p| p.ends_with("corse/estate.toml")),
        "the Corse are not the generator's to remove: {rimasti:?}"
    );
    assert_eq!(climesh::progetto::leggi(&uscita).unwrap().scenari.len(), 2);
}

#[test]
fn un_generatore_che_fallisce_lascia_dov_era_il_progetto_precedente() {
    let dir = cartella_di_prova("generatore-fallito");
    let buono = modello_su_disco(&dir, "sintetico.INX", &modello_sintetico());
    let uscita = dir.join("progetto");
    assert!(genera(&uscita, &buono).status.success());
    let prima = albero_di_file(&uscita);

    // A grid of no cells at all: readable as a file, refused as a Progetto.
    let degenere = modello_su_disco(&dir, "degenere.INX", &wrap_griglia(0, 0, 1.0, 1.0, ""));
    let esito = genera(&uscita, &degenere);
    assert!(!esito.status.success(), "the exit code must not be zero");
    assert_eq!(
        prima,
        albero_di_file(&uscita),
        "a run that fails is a run that changed nothing"
    );
}

#[test]
fn senza_il_materiale_del_corso_il_generatore_si_ferma_nominando_il_file() {
    let dir = cartella_di_prova("generatore-senza-materiale");
    let assente = dir.join("non-c-e").join("LAB1.INX");
    let esito = genera(&dir.join("progetto"), &assente);
    assert!(!esito.status.success(), "the exit code must not be zero");
    let msg = String::from_utf8_lossy(&esito.stderr);
    assert!(
        msg.contains(&*assente.to_string_lossy()),
        "the error should name the file, was: {msg}"
    );
    assert!(
        msg.contains("ridistribui"),
        "the error should say the material is not redistributable, was: {msg}"
    );
    assert!(
        msg.contains("cargo run --bin costruisci_caso"),
        "the error should name the command that takes another model, was: {msg}"
    );
    assert!(
        albero_di_file(&dir).is_empty(),
        "nothing at all should be written: a half Progetto is worse than none"
    );
}

/// The one check the synthetic model cannot make: the counts of the real case.
/// Ignored where the course material is not on disk, and ignored is what a
/// clean clone reports — never passed.
#[test]
#[ignore = "richiede materiale università/"]
fn il_modello_del_corso_da_tre_edifici_seicentosedici_alberi_e_l_erba_dove_sta() {
    let dir = cartella_di_prova("generatore-modello-vero");
    let esito = genera(&dir, Path::new(MODELLO));
    assert!(
        esito.status.success(),
        "{}",
        String::from_utf8_lossy(&esito.stderr)
    );
    let p = climesh::progetto::leggi(&dir).unwrap();
    let fatto = p.scenari.iter().find(|s| s.derivato_da.is_none()).unwrap();

    // Two duplexes at six metres and one simplex at three, as the report has it:
    // grouping by height alone would weld the two duplexes into one Edificio.
    let mut altezze: Vec<f32> = fatto.edifici.iter().map(|e| e.altezza_m).collect();
    altezze.sort_by(f32::total_cmp);
    assert_eq!(altezze, vec![3.0, 6.0, 6.0]);
    let mut celle: Vec<usize> = fatto
        .edifici
        .iter()
        .map(|e| celle_coperte(&e.impronta, 50))
        .collect();
    celle.sort_unstable();
    assert_eq!(celle, vec![91, 91, 98]);

    assert_eq!(fatto.alberi.len(), 616);
    let erba = fatto
        .superfici
        .iter()
        .find(|s| s.tipo == TipoSuperficie::Erba)
        .expect("1602 cells of the model carry a grass code");
    assert_eq!(celle_coperte(&erba.impronta, 50), 1602);
    let nudo = fatto
        .superfici
        .iter()
        .find(|s| s.tipo == TipoSuperficie::TerrenoNudo)
        .expect("the rest has a soil profile and no plant code");
    assert_eq!(celle_coperte(&nudo.impronta, 50), 2500 - 1602);
}
