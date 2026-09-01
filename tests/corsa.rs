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

/// The Impronta of the inputs `l_impronta_di_ingressi_fissati_vale_questa`
/// builds. A golden value: it moves only when what goes into an Impronta is
/// meant to move.
const IMPRONTA_FISSATA: &str = "7631d0b8bdebc89ba216960546891b38da17c9a6ddaeaecf8ce0b7e11b01203b";

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

/// The lengths the padding turns on. At 55 bytes the length field is the last
/// eight of one block; at 56 it no longer fits and a whole second block appears;
/// 63, 64, 119 and 120 are the same two steps one block further on. A padding
/// loop that is off by one block is right on every other length, so these are
/// the only lengths that notice.
///
/// Reference values from `hashlib.sha256` on `b"a" * n`.
#[test]
fn la_somma_di_controllo_riproduce_i_bordi_del_riempimento() {
    for (quanti, atteso) in [
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            119,
            "31eba51c313a5c08226adf18d4a359cfdfd8d2e816b13f4af952f7ea6584dcfb",
        ),
        (
            120,
            "2f3d335432c70b580af0e8e1b3674a7c020d683aa5f73aaaedfdc55af904c21c",
        ),
    ] {
        assert_eq!(
            somma_di_controllo(&vec![b'a'; quanti]),
            atteso,
            "{quanti} byte"
        );
    }
}

#[test]
fn il_giornale_resta_toml_valido_con_virgolette_barre_e_ritorni_a_capo() {
    let dir = tempdir_di_prova("giornale-ostile");
    let mut giornale = Giornale::apri(&dir, "giornale.toml").unwrap();
    let percorso = giornale.percorso().to_path_buf();
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
    let dir = tempdir_di_prova("giornale-interrotto");
    let mut giornale = Giornale::apri(&dir, "giornale.toml").unwrap();
    let percorso = giornale.percorso().to_path_buf();
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
    let dir = tempdir_di_prova("giornale-fallito");
    let mut giornale = Giornale::apri(&dir, "giornale.toml").unwrap();
    let percorso = giornale.percorso().to_path_buf();
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
    let dir = tempdir_di_prova("giornale-riuscito");
    let mut giornale = Giornale::apri(&dir, "giornale.toml").unwrap();
    let percorso = giornale.percorso().to_path_buf();
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
    let inv = inviluppo("chiome", "m", [f32::NAN; 4], (0.0, 100.0), "");
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
    let inv = inviluppo(
        "chiome",
        "m",
        [f32::NAN, 2.0, 4.0, f32::NAN],
        (0.0, 100.0),
        "",
    );
    assert_eq!(inv.minimo, Some(2.0));
    assert_eq!(inv.massimo, Some(4.0));
    assert_eq!(inv.media, Some(3.0));
    assert_eq!(inv.frazione_senza_dato, 0.5);
}

#[test]
fn un_valore_fuori_dall_intervallo_plausibile_alza_la_bandiera_e_viene_riportato() {
    let inv = inviluppo("ore di sole", "h", [0.5, 3.0], (0.0, 1.0), "");
    assert!(inv.fuori_intervallo);
    assert_eq!(inv.massimo, Some(3.0), "il valore si riporta comunque");

    let dentro = inviluppo("ore di sole", "h", [0.0, 1.0], (0.0, 1.0), "");
    assert!(!dentro.fuori_intervallo, "gli estremi sono dentro");
}

#[test]
fn l_impronta_cambia_se_cambia_un_solo_byte_di_un_ingresso() {
    let dir = tempdir_di_prova("impronta-byte");
    let file = dir.join("meteo.epw");
    fs::write(&file, "abc").unwrap();
    let motore = motore::versione();
    let calcola = || {
        Impronta::calcola(
            &[Ingresso::leggi(&file, &dir, "meteo")],
            "0.1.0",
            &motore,
            "p",
        )
    };

    let prima = calcola();
    fs::write(&file, "abd").unwrap();
    assert_ne!(prima, calcola(), "un byte diverso, Impronta diversa");
}

#[test]
fn l_impronta_cambia_se_cambiano_i_parametri_o_il_binario() {
    let dir = tempdir_di_prova("impronta-parametri");
    let file = dir.join("meteo.epw");
    fs::write(&file, "abc").unwrap();
    let motore = motore::versione();
    let ingressi = [Ingresso::leggi(&file, &dir, "meteo")];

    let base = Impronta::calcola(&ingressi, "0.1.0", &motore, "p");
    assert_ne!(base, Impronta::calcola(&ingressi, "0.2.0", &motore, "p"));
    assert_ne!(base, Impronta::calcola(&ingressi, "0.1.0", &motore, "q"));
}

#[test]
fn un_ingresso_assente_non_e_mai_identico_a_uno_presente() {
    let dir = tempdir_di_prova("impronta-assente");
    let file = dir.join("mai-scritto.epw");
    let motore = motore::versione();

    let mancante = Ingresso::leggi(&file, &dir, "meteo");
    assert_eq!(mancante.sha256, ASSENTE);
    let senza = Impronta::calcola(&[mancante], "0.1.0", &motore, "p");

    // Including the text an absent input writes: a file whose contents are the
    // word "assente" is still a file that is there.
    for contenuto in ["", "assente", ASSENTE] {
        fs::write(&file, contenuto).unwrap();
        let presente = Impronta::calcola(
            &[Ingresso::leggi(&file, &dir, "meteo")],
            "0.1.0",
            &motore,
            "p",
        );
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

/// The branch a value *below* the minimum takes. Nothing in this project ever
/// produced one, so the branch was never walked: a plausible range with only its
/// upper half enforced is half a check.
#[test]
fn un_valore_sotto_il_minimo_plausibile_alza_la_bandiera_e_viene_riportato() {
    let inv = inviluppo("quota", "m", [-3.0, 5.0], (0.0, 10.0), "");
    assert!(inv.fuori_intervallo);
    assert_eq!(inv.minimo, Some(-3.0), "il valore si riporta comunque");
}

/// The Giornale is read by a person. The reproducibility lives in the Impronta,
/// not in the fifteenth decimal of a mean.
#[test]
fn l_inviluppo_arrotonda_le_cifre_che_scrive() {
    let inv = inviluppo("quota", "m", [10.0, 18.0, 15.0, f32::NAN], (0.0, 100.0), "");
    assert_eq!(inv.media, Some(14.3333));
    assert_eq!(inv.frazione_senza_dato, 0.25);

    let terzi = inviluppo("quota", "m", [f32::NAN, f32::NAN, 1.0], (0.0, 100.0), "");
    assert_eq!(terzi.frazione_senza_dato, 0.6667);
}

/// A field of a Giornale that a reader can misread has to carry the sentence
/// that stops them: `frazione_senza_dato` on the canopies is not missing data.
#[test]
fn ogni_campo_porta_la_sua_nota() {
    let inv = inviluppo("chiome", "m", [1.0], (0.0, 100.0), "una nota qualunque");
    assert_eq!(inv.nota, "una nota qualunque");
}

/// Two files with the same bytes under two different names are two different
/// inputs, and a Corsa that read one did not read the other.
#[test]
fn l_impronta_cambia_se_cambia_il_percorso_di_un_ingresso() {
    let dir = tempdir_di_prova("impronta-percorso");
    fs::write(dir.join("uno.epw"), "abc").unwrap();
    fs::write(dir.join("due.epw"), "abc").unwrap();
    let motore = motore::versione();
    let impronta = |nome: &str| {
        Impronta::calcola(
            &[Ingresso::leggi(dir.join(nome), &dir, "meteo")],
            "0.1.0",
            &motore,
            "p",
        )
    };
    assert_ne!(impronta("uno.epw"), impronta("due.epw"));
}

/// The pinned kernel is half of what makes the answer what it is, and the
/// Giornale cites it. An Impronta blind to it would call two Corse the same
/// Corsa across a change of engine.
#[test]
fn l_impronta_cambia_se_cambia_il_motore() {
    let motore = |commit: &str, data: &str| climesh::motore::VersioneMotore {
        commit: commit.to_owned(),
        data_presa: data.to_owned(),
    };
    let base = Impronta::calcola(&[], "0.1.0", &motore("aaa", "2026-01-01"), "p");
    assert_ne!(
        base,
        Impronta::calcola(&[], "0.1.0", &motore("bbb", "2026-01-01"), "p"),
        "commit diverso"
    );
    assert_ne!(
        base,
        Impronta::calcola(&[], "0.1.0", &motore("aaa", "2026-01-02"), "p"),
        "data di presa diversa"
    );
}

/// A fixed value, so that a change to what goes into the Impronta or to the
/// order it goes in has somewhere to fail. Regenerate it on purpose, never to
/// make a red test green.
#[test]
fn l_impronta_di_ingressi_fissati_vale_questa() {
    let motore = climesh::motore::VersioneMotore {
        commit: "0000000000000000000000000000000000000000".into(),
        data_presa: "2026-01-01".into(),
    };
    let ingresso = |percorso: &str| Ingresso {
        percorso: percorso.to_owned(),
        ruolo: "scenario".to_owned(),
        sha256: somma_di_controllo(percorso.as_bytes()),
        usato: true,
    };
    let ingressi = [ingresso("scenari/a.toml"), ingresso("scenari/b.toml")];
    assert_eq!(
        Impronta::calcola(&ingressi, "0.1.0", &motore, "parametri").testo(),
        IMPRONTA_FISSATA
    );

    let invertiti = [ingressi[1].clone(), ingressi[0].clone()];
    assert_ne!(
        Impronta::calcola(&invertiti, "0.1.0", &motore, "parametri").testo(),
        IMPRONTA_FISSATA,
        "l'ordine degli ingressi è parte dell'Impronta"
    );
}

/// A Progetto is a folder that gets copied, moved and unzipped somewhere else.
/// Its Corse are the same Corse afterwards, or the reproducibility this project
/// promises at the head of `src/corsa.rs` is not true.
#[test]
fn lo_stesso_progetto_in_due_cartelle_ha_la_stessa_impronta() {
    let motore = motore::versione();
    let impronta_di = |nome: &str| {
        let dir = tempdir_di_prova(nome);
        fs::create_dir_all(dir.join("scenari")).unwrap();
        fs::write(dir.join("scenari/stato-di-fatto.toml"), "abc").unwrap();
        Impronta::calcola(
            &[Ingresso::leggi(
                dir.join("scenari/stato-di-fatto.toml"),
                &dir,
                "scenario",
            )],
            "0.1.0",
            &motore,
            "p",
        )
    };
    assert_eq!(impronta_di("progetto-qui"), impronta_di("progetto-altrove"));
}

/// The Giornale refuses to write through a link. `giornale.toml` was covered and
/// its folders were not, which is the half that a `corse -> /etc` uses.
#[cfg(unix)]
#[test]
fn un_giornale_sotto_una_cartella_collegata_e_rifiutato() {
    let dir = tempdir_di_prova("corse-collegate");
    fs::create_dir_all(dir.join("altrove")).unwrap();
    std::os::unix::fs::symlink(dir.join("altrove"), dir.join("corse")).unwrap();
    let err = Giornale::apri(&dir, "corse/abc/giornale.toml").expect_err("deve fallire");
    assert!(
        matches!(err, climesh::giornale::GiornaleError::Collegamento(_)),
        "variante: {err:?}"
    );
    assert!(
        !dir.join("altrove/abc").exists(),
        "nessuna scrittura attraverso il collegamento"
    );
}

#[cfg(unix)]
#[test]
fn un_giornale_che_e_un_collegamento_e_rifiutato() {
    let dir = tempdir_di_prova("giornale-collegato");
    fs::write(dir.join("fuori.toml"), "").unwrap();
    std::os::unix::fs::symlink(dir.join("fuori.toml"), dir.join("giornale.toml")).unwrap();
    let err = Giornale::apri(&dir, "giornale.toml").expect_err("deve fallire");
    assert!(
        matches!(err, climesh::giornale::GiornaleError::Collegamento(_)),
        "variante: {err:?}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("fuori.toml")).unwrap(),
        "",
        "nessuna scrittura attraverso il collegamento"
    );
}

// ---------------------------------------------------------------------------
// La Corsa
// ---------------------------------------------------------------------------

use climesh::corsa;
use climesh::dominio::{
    Albero, Data, Edificio, FonteAltezza, Griglia, Periodo, Progetto, Provenienza,
    PuntoDiOsservazione, Rettangolo, Scenario,
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

fn albero_di_prova(specie: &str, posizione_m: (f64, f64)) -> Albero {
    Albero {
        posizione_m,
        specie: specie.into(),
        altezza_m: 12.0,
        frazione_tronco: 0.35,
        provenienza: Some(Provenienza {
            origine: "prova".into(),
            altezza: FonteAltezza::Predefinito,
        }),
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
    assert!(
        !nomi.contains(&"chiome"),
        "questo Scenario non ha alberi: un campo delle chiome tutto senza dato \
         direbbe che qualcosa è stato misurato e non trovato"
    );
}

#[test]
fn il_giornale_dichiara_ogni_strato_di_chioma_con_la_sua_trasmissivita() {
    let mut progetto = progetto_di_prova(6, vec![periodo("inverno", 1, 15, 2)]);
    // A pine and a plane tree: in January the first is still opaque and the
    // second is bare, so the Corsa carries two layers and the Giornale has to
    // name both.
    progetto.scenari[0].alberi = vec![
        albero_di_prova("020027", (1.5, 1.5)),
        albero_di_prova("020060", (4.5, 4.5)),
    ];
    let (_, rapporto) = scrivi_ed_esegui("giornale-strati", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);

    assert_eq!(
        tabella["derivazione"]["chiome_spogliate"].as_integer(),
        Some(1)
    );
    let campi = tabella["campo"].as_array().unwrap();
    let nomi: Vec<&str> = campi.iter().map(|c| c["campo"].as_str().unwrap()).collect();
    assert!(nomi.contains(&"chiome"), "{nomi:?}");
    assert!(nomi.contains(&"chiome spoglie"), "{nomi:?}");
    for (campo, per_cento) in [("chiome", "3"), ("chiome spoglie", "50")] {
        let nota = campi
            .iter()
            .find(|c| c["campo"].as_str() == Some(campo))
            .unwrap()["nota"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            nota.contains(&format!("{per_cento} per cento")),
            "la nota di «{campo}» non dice quanto lascia passare: {nota}"
        );
    }
}

/// A weather file with `fuso` as its time zone and one record per hour of two
/// days, written into the Progetto so the test does not depend on a file that
/// is not in the repository.
fn scrivi_epw(dir: &std::path::Path, fuso: f64) -> PathBuf {
    let mut testo = format!("LOCATION,Prova,-,ITA,X,000000,43.07,12.56,{fuso},213.0");
    for _ in 0..7 {
        testo.push_str("\nRIGA DI INTESTAZIONE,ignorata");
    }
    for giorno in [15, 16] {
        for ora in 1..=24 {
            testo.push_str(&format!(
                "\n2005,7,{giorno},{ora},0,?9?9,24.0,7.2,55,98792,9999,9999,290,120,300,80,\
                 999900,999900,999900,99990,45,4.0,1,1,999.0,999,9,999999999,0,0.0000,0,88,\
                 999.000,999.0,99.0"
            ));
        }
    }
    testo.push('\n');
    let percorso = dir.join("prova.epw");
    fs::write(&percorso, testo).unwrap();
    percorso
}

#[test]
fn il_fuso_orario_della_corsa_viene_dal_file_meteo() {
    // Twelve degrees and a half of longitude round to a zone of +1, so a file
    // that declares +3 is the only way to tell a read zone from a guessed one.
    // Three hours are forty-five degrees of angolo orario: the sun of a Corsa
    // that guessed would be somewhere else entirely.
    let dir = tempdir_di_prova("fuso-dal-meteo");
    let epw = scrivi_epw(&dir, 3.0);
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.periodi[0].meteo = epw;
    climesh::progetto::scrivi(&dir, &progetto).unwrap();
    let rapporto = corsa::esegui_progetto(&dir).unwrap();
    let tabella = rileggi(&rapporto.corse[0].giornale);

    assert_eq!(tabella["sole"]["fuso_ore"].as_float(), Some(3.0));
    assert_eq!(tabella["sole"]["fuso_da"].as_str(), Some("file meteo"));
}

#[test]
fn un_fuso_a_mezz_ora_arriva_intero_fino_al_giornale() {
    // Kathmandu is at +5.75 and Adelaide at +9.5. The guess from the longitude
    // could only ever produce whole hours, so the half-hour zones were not
    // approximated: they were unrepresentable.
    let dir = tempdir_di_prova("fuso-a-mezz-ora");
    let epw = scrivi_epw(&dir, 5.75);
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.periodi[0].meteo = epw;
    climesh::progetto::scrivi(&dir, &progetto).unwrap();
    let rapporto = corsa::esegui_progetto(&dir).unwrap();
    assert_eq!(
        rileggi(&rapporto.corse[0].giornale)["sole"]["fuso_ore"].as_float(),
        Some(5.75)
    );
}

#[test]
fn senza_file_meteo_il_fuso_torna_alla_longitudine_e_il_giornale_lo_dice() {
    // The Corsa still runs — the weather file is an Ingresso and its absence is
    // already recorded — but the number it used is a guess, and a Giornale that
    // showed only the number would not let anyone tell the two apart.
    let dir = tempdir_di_prova("fuso-senza-meteo");
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.periodi[0].meteo = dir.join("questo-file-non-esiste.epw");
    climesh::progetto::scrivi(&dir, &progetto).unwrap();
    let rapporto = corsa::esegui_progetto(&dir).unwrap();
    let tabella = rileggi(&rapporto.corse[0].giornale);

    assert_eq!(tabella["sole"]["fuso_ore"].as_float(), Some(1.0));
    let fonte = tabella["sole"]["fuso_da"].as_str().unwrap();
    assert!(fonte.contains("longitudine"), "{fonte}");
    assert!(fonte.contains("non si è potuto leggere"), "{fonte}");
}

fn punto(id: u32, posizione_m: (f64, f64)) -> PuntoDiOsservazione {
    PuntoDiOsservazione {
        id,
        posizione_m,
        etichetta: format!("punto {id}"),
    }
}

#[test]
fn ogni_punto_di_osservazione_porta_la_sua_serie_ora_per_ora() {
    // The map answers "where" and the series answers "when". A courtyard that
    // gains two hours of sun in the morning and loses two in the afternoon has
    // the same daily mean as one where nothing changed, and only the series
    // tells them apart.
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 24)]);
    progetto.punti = vec![punto(1, (0.5, 0.5)), punto(2, (5.5, 5.5))];
    let (_, rapporto) = scrivi_ed_esegui("serie-dei-punti", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);

    assert_eq!(tabella["punti"]["dichiarati"].as_integer(), Some(2));
    assert_eq!(tabella["punti"]["dentro_la_griglia"].as_integer(), Some(2));

    let serie = tabella["serie"].as_array().unwrap();
    assert_eq!(serie.len(), 2);
    for (indice, una) in serie.iter().enumerate() {
        assert_eq!(una["punto"].as_integer(), Some(indice as i64 + 1));
        let valori = una["frazione_illuminata"].as_array().unwrap();
        assert_eq!(valori.len(), 24, "una serie per ogni ora del Periodo");
        assert!(
            valori
                .iter()
                .all(|v| (0.0..=1.0).contains(&v.as_float().unwrap())),
            "una frazione illuminata fuori da 0..1"
        );
    }

    // A day has a night in it: some hour of the twenty-four is dark, in every
    // cell. A series of all ones would mean the sun never set on that Punto.
    let primo = serie[0]["frazione_illuminata"].as_array().unwrap();
    assert!(primo.iter().any(|v| v.as_float() == Some(0.0)));
    assert!(primo.iter().any(|v| v.as_float().unwrap() > 0.0));
}

#[test]
fn la_serie_di_un_punto_somma_alle_ore_di_sole_della_sua_cella() {
    // The two outputs are the same numbers seen twice, and this is what says
    // so: if the series were read from another cell — a swapped row and column
    // is the mistake this project is built to fear — the sums would part.
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 12)]);
    // Off the diagonal on purpose: on a cell where row and column are equal, a
    // swap of the two would leave the answer right.
    progetto.punti = vec![punto(1, (1.5, 4.5))];
    let (_, rapporto) = scrivi_ed_esegui("serie-somma", &progetto);

    let campi = rapporto.corse[0].campi.as_ref().unwrap();
    let serie = &campi.serie[0];
    let somma: f64 = serie.frazione_illuminata.iter().sum();
    let dalla_mappa = f64::from(campi.ore_di_sole[[serie.riga, serie.colonna]]);
    assert!(
        (somma - dalla_mappa).abs() < 0.01,
        "la serie somma {somma} e la mappa dice {dalla_mappa}"
    );
}

#[test]
fn un_punto_fuori_dalla_griglia_non_arriva_mai_a_una_corsa() {
    // It is refused where a Progetto is written, so no Corsa ever has to decide
    // what to do with it. The alternative — accept it and serve the nearest
    // cell — would attribute a number to a place nobody asked about.
    let dir = tempdir_di_prova("punto-fuori");
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.punti = vec![punto(1, (2.5, 2.5)), punto(2, (99.0, 99.0))];
    let errore = climesh::progetto::scrivi(&dir, &progetto)
        .unwrap_err()
        .to_string();
    assert!(errore.contains("punto di osservazione 2"), "{errore}");
}

#[test]
fn un_punto_sul_bordo_lontano_e_fuori_come_ovunque_nel_progetto() {
    // Half-open intervals: minimum included, maximum excluded, which is the
    // coverage rule of the Derivazione. With the maximum included, a Punto at
    // x = 6 on a Griglia of six cells passed the check and then had no cell —
    // accepted by one rule, invisible to the other.
    let dir = tempdir_di_prova("punto-sul-bordo");
    let mut progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    progetto.punti = vec![punto(1, (6.0, 3.0))];
    assert!(climesh::progetto::scrivi(&dir, &progetto).is_err());

    // Just inside, it is accepted and it gets its series.
    progetto.punti = vec![punto(1, (5.999, 3.0))];
    let (_, rapporto) = scrivi_ed_esegui("punto-quasi-sul-bordo", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);
    assert_eq!(tabella["punti"]["dentro_la_griglia"].as_integer(), Some(1));
    assert_eq!(
        tabella["serie"].as_array().unwrap()[0]["colonna"].as_integer(),
        Some(5)
    );
}

#[test]
fn un_progetto_senza_punti_dichiara_zero_e_non_tace() {
    let progetto = progetto_di_prova(6, vec![periodo("estate", 7, 15, 2)]);
    let (_, rapporto) = scrivi_ed_esegui("senza-punti", &progetto);
    let tabella = rileggi(&rapporto.corse[0].giornale);
    assert_eq!(tabella["punti"]["dichiarati"].as_integer(), Some(0));
    assert!(tabella.get("serie").is_none(), "nessuna serie da scrivere");
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

/// Una verifica che non ha potuto girare non deve somigliare a una passata.
/// `scarto_massimo` resta a zero quando nessun'ora è verificabile, e zero si
/// legge come accordo perfetto: la bandiera deve alzarsi lo stesso.
#[test]
fn una_verifica_mai_eseguita_alza_la_bandiera() {
    // Uno Scenario senza Edifici: nessun volume di cui confrontare l'ombra,
    // quindi nessun'ora verificabile.
    let mut progetto = progetto_di_prova(6, vec![periodo("luglio", 7, 15, 6)]);
    progetto.scenari[0].edifici.clear();
    let (dir, rapporto) = scrivi_ed_esegui("verifica-mai-eseguita", &progetto);

    let giornale = rileggi(
        &dir.join("corse")
            .join(rapporto.corse[0].impronta.testo())
            .join("giornale.toml"),
    );
    let v = giornale["verifica_ombra"].as_table().unwrap();
    assert_eq!(
        v["ore_verificate"].as_integer(),
        Some(0),
        "senza Edifici non c'è niente da verificare"
    );
    assert_eq!(
        v["bandiera"].as_bool(),
        Some(true),
        "zero ore verificate deve alzare la bandiera, non lasciarla bassa"
    );
}
