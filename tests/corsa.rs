//! The Giornale and the Corsa. Degenerate inputs first: every test below fails
//! if the defence it names is removed from `src/giornale.rs` or `src/corsa.rs`.
//!
//! The Giornale is read back **with the TOML parser**, never by looking for a
//! substring: a Giornale that does not re-read does not exist, because the page
//! and the print sheet are renderings of that one file.

use climesh::giornale::{
    conta_provenienza, inviluppo, somma_di_controllo, Giornale, Impronta, Ingresso, ASSENTE,
};
use climesh::motore;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn tempdir_di_prova(nome: &str) -> PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn rileggi(percorso: &std::path::Path) -> toml::Table {
    let testo = fs::read_to_string(percorso).unwrap();
    toml::from_str(&testo).unwrap_or_else(|e| panic!("il Giornale non si rilegge: {e}\n{testo}"))
}

/// Quotes, a backslash and a line break, which is what a user types into an
/// etichetta and what breaks a file written by hand.
const TESTO_OSTILE: &str = "corsa \"buona\" \\ del\nlunedì";

fn riga(chiave: &str, valore: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(chiave.to_owned(), valore.to_owned())])
}

#[test]
fn la_somma_di_controllo_riproduce_i_vettori_noti() {
    // The three vectors of FIPS 180-4: empty, one block, and the message that
    // needs a second block for its padding.
    assert_eq!(
        somma_di_controllo(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        somma_di_controllo(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        somma_di_controllo(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn il_giornale_resta_toml_valido_con_virgolette_barre_e_ritorni_a_capo() {
    let percorso = tempdir_di_prova("giornale-ostile").join("giornale.toml");
    let mut giornale = Giornale::apri(&percorso).unwrap();
    giornale
        .annota("corsa", &riga("etichetta", TESTO_OSTILE))
        .unwrap();
    giornale.concludi(None).unwrap();

    let tabella = rileggi(&percorso);
    assert_eq!(
        tabella["corsa"]["etichetta"].as_str(),
        Some(TESTO_OSTILE),
        "il testo deve tornare identico, non solo essere presente"
    );
}

#[test]
fn un_giornale_interrotto_non_ha_conclusione_ne_un_esito_al_livello_superiore() {
    let percorso = tempdir_di_prova("giornale-interrotto").join("giornale.toml");
    let mut giornale = Giornale::apri(&percorso).unwrap();
    giornale
        .annota("corsa", &riga("etichetta", "interrotta"))
        .unwrap();
    giornale
        .annota("derivazione", &riga("celle_costruite", "12"))
        .unwrap();
    drop(giornale);

    let tabella = rileggi(&percorso);
    assert!(tabella.contains_key("corsa"), "i passi già fatti restano");
    assert!(
        tabella.contains_key("derivazione"),
        "i passi già fatti restano"
    );
    assert!(
        !tabella.contains_key("conclusione"),
        "finché [conclusione] non c'è la Corsa non è finita: l'assenza è lo stato"
    );
    assert!(
        !tabella.contains_key("esito"),
        "nessun esito al livello superiore: un file appeso non può riscrivere una riga di sopra"
    );
}

#[test]
fn una_corsa_fallita_conclude_con_esito_fallita_e_l_errore_leggibile() {
    let percorso = tempdir_di_prova("giornale-fallito").join("giornale.toml");
    let mut giornale = Giornale::apri(&percorso).unwrap();
    giornale
        .annota("corsa", &riga("etichetta", "che fallisce"))
        .unwrap();
    giornale
        .concludi(Some(
            "il periodo «inverno» comincia il 30 febbraio 2021, che non è una data",
        ))
        .unwrap();

    let tabella = rileggi(&percorso);
    let conclusione = tabella["conclusione"].as_table().unwrap();
    assert_eq!(conclusione["esito"].as_str(), Some("fallita"));
    assert!(conclusione["errore"]
        .as_str()
        .unwrap()
        .contains("30 febbraio"));
    assert!(!tabella.contains_key("esito"));
}

#[test]
fn una_corsa_riuscita_conclude_con_esito_riuscita_e_nessun_errore() {
    let percorso = tempdir_di_prova("giornale-riuscito").join("giornale.toml");
    let mut giornale = Giornale::apri(&percorso).unwrap();
    giornale
        .annota("corsa", &riga("etichetta", "che riesce"))
        .unwrap();
    giornale.concludi(None).unwrap();

    let tabella = rileggi(&percorso);
    let conclusione = tabella["conclusione"].as_table().unwrap();
    assert_eq!(conclusione["esito"].as_str(), Some("riuscita"));
    assert!(!conclusione.contains_key("errore"));
}

#[test]
fn un_campo_tutto_nan_non_si_inventa_un_minimo() {
    let inv = inviluppo("chiome", "m", [f32::NAN; 4], (0.0, 100.0));
    assert_eq!(inv.minimo, None);
    assert_eq!(inv.massimo, None);
    assert_eq!(inv.media, None);
    assert_eq!(inv.frazione_senza_dato, 1.0);
    assert!(
        !inv.fuori_intervallo,
        "senza dato non c'è niente da segnalare"
    );
}

#[test]
fn l_inviluppo_conta_solo_le_celle_con_dato() {
    let inv = inviluppo("chiome", "m", [f32::NAN, 2.0, 4.0, f32::NAN], (0.0, 100.0));
    assert_eq!(inv.minimo, Some(2.0));
    assert_eq!(inv.massimo, Some(4.0));
    assert_eq!(inv.media, Some(3.0));
    assert_eq!(inv.frazione_senza_dato, 0.5);
}

#[test]
fn un_valore_fuori_dall_intervallo_plausibile_alza_la_bandiera_e_viene_riportato() {
    let inv = inviluppo("frazione illuminata", "1", [0.5, 3.0], (0.0, 1.0));
    assert!(inv.fuori_intervallo);
    assert_eq!(inv.massimo, Some(3.0), "il valore si riporta comunque");

    let dentro = inviluppo("frazione illuminata", "1", [0.0, 1.0], (0.0, 1.0));
    assert!(!dentro.fuori_intervallo, "gli estremi sono dentro");
}

#[test]
fn l_impronta_cambia_se_cambia_un_solo_byte_di_un_ingresso() {
    let dir = tempdir_di_prova("impronta-byte");
    let file = dir.join("meteo.epw");
    fs::write(&file, "abc").unwrap();
    let motore = motore::versione();
    let calcola = || Impronta::calcola(&[Ingresso::leggi(&file, "meteo")], "0.1.0", &motore, "p");

    let prima = calcola();
    assert_eq!(prima, calcola(), "stessi ingressi, stessa Impronta");

    fs::write(&file, "abd").unwrap();
    assert_ne!(prima, calcola(), "un byte diverso, Impronta diversa");
}

#[test]
fn l_impronta_cambia_se_cambiano_i_parametri_o_il_binario() {
    let dir = tempdir_di_prova("impronta-parametri");
    let file = dir.join("meteo.epw");
    fs::write(&file, "abc").unwrap();
    let motore = motore::versione();
    let ingressi = [Ingresso::leggi(&file, "meteo")];

    let base = Impronta::calcola(&ingressi, "0.1.0", &motore, "p");
    assert_ne!(base, Impronta::calcola(&ingressi, "0.2.0", &motore, "p"));
    assert_ne!(base, Impronta::calcola(&ingressi, "0.1.0", &motore, "q"));
}

#[test]
fn un_ingresso_assente_non_e_mai_identico_a_uno_presente() {
    let dir = tempdir_di_prova("impronta-assente");
    let file = dir.join("mai-scritto.epw");
    let motore = motore::versione();

    let mancante = Ingresso::leggi(&file, "meteo");
    assert_eq!(mancante.sha256, ASSENTE);
    let senza = Impronta::calcola(&[mancante], "0.1.0", &motore, "p");

    // Including the text an absent input writes: a file whose contents are the
    // word "assente" is still a file that is there.
    for contenuto in ["", "assente", ASSENTE] {
        fs::write(&file, contenuto).unwrap();
        let presente = Impronta::calcola(&[Ingresso::leggi(&file, "meteo")], "0.1.0", &motore, "p");
        assert_ne!(senza, presente, "contenuto «{contenuto}»");
    }
}

#[test]
fn la_provenienza_si_conta_per_anello_della_catena() {
    use climesh::dominio::*;
    let provenienza = |altezza| Provenienza {
        origine: "prova".into(),
        altezza,
    };
    let scenario = Scenario {
        nome: "prova".into(),
        derivato_da: None,
        terreno_m: vec![0.0; 4],
        provenienza: provenienza(FonteAltezza::Predefinito),
        edifici: vec![
            Edificio {
                altezza_m: 6.0,
                provenienza: Some(provenienza(FonteAltezza::Rilievo)),
                impronta: vec![],
            },
            Edificio {
                altezza_m: 3.0,
                provenienza: Some(provenienza(FonteAltezza::NumeroDiPiani)),
                impronta: vec![],
            },
            Edificio {
                altezza_m: 3.0,
                provenienza: Some(provenienza(FonteAltezza::ModelloDiSuperficie)),
                impronta: vec![],
            },
        ],
        // No provenienza of its own: it inherits the Scenario's link.
        alberi: vec![Albero {
            posizione_m: (0.5, 0.5),
            specie: "020027".into(),
            altezza_m: 12.0,
            frazione_tronco: 0.45,
            provenienza: None,
        }],
        superfici: vec![],
    };

    let conteggio = conta_provenienza(&scenario);
    assert_eq!(conteggio.rilievo, 1);
    assert_eq!(conteggio.numero_di_piani, 1);
    assert_eq!(conteggio.modello_di_superficie, 1);
    assert_eq!(
        conteggio.predefinito, 1,
        "l'albero eredita l'anello dello Scenario"
    );
}

// ---------------------------------------------------------------------------
// La Corsa
// ---------------------------------------------------------------------------

use climesh::corsa;
use climesh::dominio::{
    Data, Edificio, FonteAltezza, Griglia, Periodo, Progetto, Provenienza, Rettangolo, Scenario,
};

/// A Progetto with one flat Scenario and a tower in the middle of it.
fn progetto_di_prova(lato: usize, periodi: Vec<Periodo>) -> Progetto {
    let mezzo = (lato / 2) as f64;
    Progetto {
        nome: "prova".into(),
        griglia: Griglia {
            nx: lato,
            ny: lato,
            passo_m: 1.0,
            crs: "EPSG:4326".into(),
            origine: (12.56, 43.07),
            rotazione_gradi: 0.0,
        },
        punti: vec![],
        scenari: vec![Scenario {
            nome: "stato-di-fatto".into(),
            derivato_da: None,
            terreno_m: vec![0.0; lato * lato],
            provenienza: Provenienza {
                origine: "prova".into(),
                altezza: FonteAltezza::Predefinito,
            },
            edifici: vec![Edificio {
                altezza_m: 10.0,
                provenienza: Some(Provenienza {
                    origine: "prova".into(),
                    altezza: FonteAltezza::Rilievo,
                }),
                impronta: vec![Rettangolo {
                    x_min_m: mezzo - 1.0,
                    y_min_m: mezzo - 1.0,
                    x_max_m: mezzo + 1.0,
                    y_max_m: mezzo + 1.0,
                }],
            }],
            alberi: vec![],
            superfici: vec![],
        }],
        periodi,
    }
}

fn periodo(nome: &str, mese: u32, giorno: u32, ore: u32) -> Periodo {
    Periodo {
        nome: nome.into(),
        meteo: "materiale università/ITA_Perugia.161810_IGDG.epw".into(),
        ore,
        direzione_vento_gradi: Some(45.0),
        inizio: Data {
            anno: 2021,
            mese,
            giorno,
        },
    }
}

fn scrivi_ed_esegui(nome: &str, progetto: &Progetto) -> (PathBuf, corsa::Rapporto) {
    let dir = tempdir_di_prova(nome);
    climesh::progetto::scrivi(&dir, progetto).unwrap();
    let rapporto = corsa::esegui_progetto(&dir).unwrap();
    (dir, rapporto)
}

#[test]
fn due_periodi_nella_stessa_stagione_condividono_una_sola_derivazione() {
    let progetto = progetto_di_prova(
        6,
        vec![periodo("luglio", 7, 15, 1), periodo("agosto", 8, 15, 1)],
    );
    let (_, rapporto) = scrivi_ed_esegui("una-derivazione", &progetto);
    assert_eq!(rapporto.corse.len(), 2);
    assert_eq!(
        rapporto.derivazioni, 1,
        "stesso Scenario e stessa Stagione: la Derivazione si fa una volta sola"
    );
}

#[test]
fn due_periodi_di_stagioni_diverse_derivano_due_volte() {
    let progetto = progetto_di_prova(
        6,
        vec![periodo("estate", 7, 15, 1), periodo("inverno", 1, 15, 1)],
    );
    let (_, rapporto) = scrivi_ed_esegui("due-derivazioni", &progetto);
    assert_eq!(rapporto.corse.len(), 2);
    assert_eq!(
        rapporto.derivazioni, 2,
        "il Periodo senza foglie toglie le chiome decidue: il raster cambia davvero"
    );
}

#[test]
fn un_periodo_con_una_data_impossibile_fa_fallire_la_corsa_invece_di_andare_in_panico() {
    let progetto = progetto_di_prova(6, vec![periodo("impossibile", 2, 30, 1)]);
    let (_, rapporto) = scrivi_ed_esegui("data-impossibile", &progetto);

    let esito = &rapporto.corse[0];
    let errore = esito.errore.as_deref().expect("la Corsa deve fallire");
    assert!(errore.contains("30"), "l'errore nomina la data: {errore}");

    let tabella = rileggi(&esito.giornale);
    assert_eq!(tabella["conclusione"]["esito"].as_str(), Some("fallita"));
    assert!(
        tabella.contains_key("derivazione"),
        "i passi già fatti restano nel Giornale"
    );
    assert!(
        !tabella.contains_key("campo"),
        "nessun campo è stato calcolato"
    );
    assert!(!tabella.contains_key("esito"));
}

#[test]
fn una_griglia_senza_latitudine_fa_fallire_la_corsa_invece_di_indovinarla() {
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 1)]);
    progetto.griglia.crs = "EPSG:32633".into();
    let (_, rapporto) = scrivi_ed_esegui("crs-proiettato", &progetto);

    let errore = rapporto.corse[0]
        .errore
        .as_deref()
        .expect("senza latitudine non c'è sole");
    assert!(errore.contains("EPSG:32633"), "{errore}");
}

#[test]
fn il_giornale_di_una_corsa_si_rilegge_col_parser_anche_con_testo_ostile() {
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.nome = TESTO_OSTILE.into();
    progetto.scenari[0].provenienza.origine = TESTO_OSTILE.into();
    let (_, rapporto) = scrivi_ed_esegui("corsa-ostile", &progetto);

    let esito = &rapporto.corse[0];
    assert_eq!(esito.errore, None);
    let tabella = rileggi(&esito.giornale);
    assert_eq!(tabella["corsa"]["progetto"].as_str(), Some(TESTO_OSTILE));
    assert_eq!(
        tabella["scenario"]["provenienza_origine"].as_str(),
        Some(TESTO_OSTILE)
    );
}

#[test]
fn il_giornale_registra_ingressi_versioni_scelte_e_inviluppi() {
    let progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    let (_, rapporto) = scrivi_ed_esegui("giornale-completo", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);

    assert_eq!(
        tabella["corsa"]["impronta"].as_str(),
        Some(rapporto.corse[0].impronta.testo())
    );
    assert!(!tabella["corsa"]["citazione"].as_str().unwrap().is_empty());
    assert_eq!(
        tabella["motore"]["commit"].as_str(),
        Some(motore::versione().commit.as_str())
    );
    assert_eq!(tabella["binario"]["versione"].as_str(), Some("0.1.0"));
    assert_eq!(tabella["griglia"]["nx"].as_integer(), Some(6));
    assert_eq!(
        tabella["derivazione"]["celle_costruite"].as_integer(),
        Some(4)
    );
    assert_eq!(tabella["provenienza"]["rilievo"].as_integer(), Some(1));
    assert!(tabella.contains_key("riproducibilita"));
    assert!(tabella.contains_key("verifiche_per_rilascio"));

    let ingressi = tabella["ingresso"].as_array().unwrap();
    let ruoli: Vec<&str> = ingressi
        .iter()
        .map(|i| i["ruolo"].as_str().unwrap())
        .collect();
    assert!(ruoli.contains(&"manifesto"));
    assert!(ruoli.contains(&"scenario"));
    assert!(ruoli.contains(&"periodo"));
    assert!(
        ruoli.contains(&"meteo"),
        "i valori meteorologici sono un ingresso, non un risultato"
    );

    let campi = tabella["campo"].as_array().unwrap();
    let nomi: Vec<&str> = campi.iter().map(|c| c["campo"].as_str().unwrap()).collect();
    assert!(nomi.contains(&"frazione illuminata media"));
    assert!(nomi.contains(&"ore di sole"));
    assert!(nomi.contains(&"chiome"));
    let chiome = campi
        .iter()
        .find(|c| c["campo"].as_str() == Some("chiome"))
        .unwrap();
    assert_eq!(
        chiome["frazione_senza_dato"].as_float(),
        Some(1.0),
        "nessun albero: il campo è tutto senza dato"
    );
    assert!(chiome.get("minimo").is_none(), "nessun minimo inventato");
}

#[test]
fn l_ombra_cade_dalla_parte_opposta_al_sole_calcolato_a_parte() {
    let progetto = progetto_di_prova(21, vec![periodo("estate", 7, 15, 24)]);
    let (_, rapporto) = scrivi_ed_esegui("verifica-ombra", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);

    let verifica = tabella["verifica_ombra"].as_table().unwrap();
    let ore = verifica["ore_verificate"].as_integer().unwrap();
    assert!(
        ore >= 6,
        "il controllo deve avere ore da controllare, non {ore}"
    );
    let scarto = verifica["scarto_massimo_gradi"].as_float().unwrap();
    assert!(
        scarto < 45.0,
        "l'ombra deve cadere dalla parte opposta al sole, scarto {scarto}°"
    );
    assert_eq!(verifica["bandiera"].as_bool(), Some(false));
    assert_eq!(
        verifica["notte_tutta_in_ombra"].as_bool(),
        Some(true),
        "col sole sotto l'orizzonte non c'è nessuna cella illuminata"
    );
}

#[test]
#[ignore = "il cancello dei 60 secondi: legge il caso di riferimento versionato e va misurato in release"]
fn il_caso_di_riferimento_sta_sotto_i_sessanta_secondi() {
    let inizio = std::time::Instant::now();
    let rapporto = corsa::esegui_caso_di_riferimento().unwrap();
    let totale = inizio.elapsed();

    println!("--- cancello del caso di riferimento ---");
    println!("Corse: {}", rapporto.corse.len());
    println!("Derivazioni: {}", rapporto.derivazioni);
    println!("totale ............ {:?}", totale);
    println!("  Derivazione ..... {:?}", rapporto.tempo_derivazione);
    println!("  Motore .......... {:?}", rapporto.tempo_motore);
    println!("  scrittura ....... {:?}", rapporto.tempo_scrittura);
    for esito in &rapporto.corse {
        println!(
            "  {} → {}",
            esito.etichetta,
            esito.errore.as_deref().unwrap_or("riuscita")
        );
    }

    assert_eq!(rapporto.corse.len(), 4, "due Scenari per due Periodi");
    for esito in &rapporto.corse {
        assert_eq!(esito.errore, None, "{}", esito.etichetta);
    }
    assert!(
        totale < std::time::Duration::from_secs(60),
        "il caso di riferimento ha impiegato {totale:?}"
    );
}
