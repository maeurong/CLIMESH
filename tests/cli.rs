//! The command line, driven the way a user drives it: the built binary, its
//! output and its exit code.
//!
//! The exit code carries a distinction a script needs and prose cannot make:
//! **2 means the command was called wrongly**, 1 means it was called rightly
//! and could not be carried out. A parametric study that cannot tell the two
//! apart retries the wrong ones.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Wrong usage. `1` is a command that ran and failed.
const USO_SBAGLIATO: i32 = 2;

fn climesh(argomenti: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_climesh"))
        .args(argomenti)
        // The tests state the language rather than inheriting it: a suite whose
        // assertions depend on the machine's locale passes at home and fails in
        // continuous integration.
        .env_remove("CLIMESH_LINGUA")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("LANG")
        .output()
        .expect("il binario di prova si esegue")
}

fn uscita(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn errori(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn codice(output: &Output) -> i32 {
    output.status.code().expect("terminato, non ucciso")
}

fn tempdir(nome: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A weather file with two days of July, so a Progetto has something to run.
fn scrivi_epw(dir: &Path) -> PathBuf {
    let mut testo = String::from("LOCATION,Prova,-,ITA,X,000000,43.07,12.56,1.0,213.0");
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

/// The smallest `.INX` that is one: a header, a grid, and nothing on it.
fn scrivi_inx(dir: &Path) -> PathBuf {
    let percorso = dir.join("modellino.INX");
    fs::write(
        &percorso,
        "<ENVI-MET_Datafile>\n\
         <modelGeometry>\n\
         <grids-I> 4 </grids-I>\n\
         <grids-J> 4 </grids-J>\n\
         <grids-Z> 4 </grids-Z>\n\
         <dx> 1.00000 </dx>\n\
         <dy> 1.00000 </dy>\n\
         <dz-base> 2.00000 </dz-base>\n\
         </modelGeometry>\n\
         <locationData>\n\
         <modelRotation> 0.00000 </modelRotation>\n\
         <locationName> prova </locationName>\n\
         <location_Longitude> 12.56000 </location_Longitude>\n\
         <location_Latitude> 43.07000 </location_Latitude>\n\
         <locationTimeZone_Name> CET/ UTC+1 </locationTimeZone_Name>\n\
         <locationTimeZone_Longitude> 15.00000 </locationTimeZone_Longitude>\n\
         </locationData>\n\
         </ENVI-MET_Datafile>\n",
    )
    .unwrap();
    percorso
}

/// A Progetto on disk with one tower, one Scenario and one Periodo of two
/// hours, written through the library so the command line has something real
/// to run.
fn scrivi_progetto(dir: &Path) {
    use climesh::dominio::*;
    let epw = scrivi_epw(dir);
    let progetto = Progetto {
        nome: "prova".into(),
        griglia: Griglia {
            nx: 6,
            ny: 6,
            passo_m: 1.0,
            crs: "EPSG:4326".into(),
            origine: (12.56, 43.07),
            rotazione_gradi: 0.0,
        },
        punti: vec![PuntoDiOsservazione {
            id: 1,
            posizione_m: (0.5, 0.5),
            etichetta: "angolo".into(),
        }],
        scenari: vec![Scenario {
            nome: "stato-di-fatto".into(),
            derivato_da: None,
            terreno_m: vec![0.0; 36],
            provenienza: Provenienza {
                origine: "prova".into(),
                altezza: FonteAltezza::Predefinito,
            },
            edifici: vec![Edificio {
                altezza_m: 10.0,
                provenienza: None,
                impronta: vec![Rettangolo {
                    x_min_m: 2.0,
                    y_min_m: 2.0,
                    x_max_m: 4.0,
                    y_max_m: 4.0,
                }],
            }],
            alberi: vec![],
            superfici: vec![],
        }],
        periodi: vec![Periodo {
            nome: "estate".into(),
            meteo: epw,
            // Long enough to contain a midday: a Periodo of two hours from
            // midnight is all night, and every number it produces is zero —
            // which a test can pass without ever touching the sun.
            ore: 14,
            direzione_vento_gradi: Some(45.0),
            inizio: Data {
                anno: 2021,
                mese: 7,
                giorno: 15,
            },
        }],
    };
    climesh::progetto::scrivi(dir, &progetto).unwrap();
}

#[test]
fn senza_argomenti_stampa_l_aiuto_e_riesce() {
    // Not an error: a user who types the name of a program is asking what it
    // does, and answering with exit code 2 would make a shell script think the
    // invocation was malformed.
    let esito = climesh(&[]);
    assert_eq!(codice(&esito), 0);
    assert!(uscita(&esito).contains("costruisci"), "{}", uscita(&esito));
}

#[test]
fn l_aiuto_esce_nella_lingua_chiesta() {
    let italiano = climesh(&["--lingua", "it", "--aiuto"]);
    let inglese = climesh(&["--lingua", "en", "--aiuto"]);
    assert!(
        uscita(&italiano).starts_with("uso:"),
        "{}",
        uscita(&italiano)
    );
    assert!(
        uscita(&inglese).starts_with("usage:"),
        "{}",
        uscita(&inglese)
    );
    // The command names do not move with the language: a shell script written
    // on one machine has to run on another.
    for esito in [&italiano, &inglese] {
        assert!(uscita(esito).contains("costruisci"));
        assert!(uscita(esito).contains("esegui"));
        assert!(uscita(esito).contains("interroga"));
    }
}

#[test]
fn la_lingua_si_scrive_anche_con_l_uguale() {
    assert!(uscita(&climesh(&["--lingua=it", "--aiuto"])).starts_with("uso:"));
}

#[test]
fn senza_lingua_dichiarata_la_decide_l_ambiente() {
    let esito = Command::new(env!("CARGO_BIN_EXE_climesh"))
        .arg("--aiuto")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("CLIMESH_LINGUA", "it_IT.UTF-8")
        .output()
        .unwrap();
    assert!(uscita(&esito).starts_with("uso:"), "{}", uscita(&esito));
}

#[test]
fn una_lingua_che_non_c_e_e_un_errore_d_uso() {
    let esito = climesh(&["--lingua", "de", "--aiuto"]);
    assert_eq!(codice(&esito), USO_SBAGLIATO);
    assert!(errori(&esito).contains("de"), "{}", errori(&esito));
}

#[test]
fn la_versione_e_quella_del_pacchetto() {
    let esito = climesh(&["--versione"]);
    assert_eq!(codice(&esito), 0);
    assert_eq!(
        uscita(&esito).trim(),
        format!("climesh {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn un_comando_che_non_esiste_esce_con_due_e_mostra_l_aiuto() {
    let esito = climesh(&["--lingua", "it", "pippo"]);
    assert_eq!(codice(&esito), USO_SBAGLIATO);
    assert!(errori(&esito).contains("pippo"), "{}", errori(&esito));
    assert!(errori(&esito).contains("uso:"), "{}", errori(&esito));
}

#[test]
fn un_opzione_che_non_esiste_non_viene_ignorata() {
    // The third possible answer — accept it and do nothing — is the worst: a
    // misspelt flag would silently not apply.
    let esito = climesh(&["--lingua", "it", "esegui", "cartella", "--tutto"]);
    assert_eq!(codice(&esito), USO_SBAGLIATO);
    assert!(errori(&esito).contains("--tutto"), "{}", errori(&esito));
}

#[test]
fn un_argomento_di_troppo_e_un_errore_e_non_un_argomento_ignorato() {
    let esito = climesh(&["--lingua", "it", "esegui", "una", "due"]);
    assert_eq!(codice(&esito), USO_SBAGLIATO);
    assert!(errori(&esito).contains("due"), "{}", errori(&esito));
}

#[test]
fn un_argomento_che_manca_viene_nominato() {
    let esito = climesh(&["--lingua", "it", "costruisci"]);
    assert_eq!(codice(&esito), USO_SBAGLIATO);
    assert!(
        errori(&esito).contains("<modello.inx>"),
        "{}",
        errori(&esito)
    );
    let esito = climesh(&["--lingua", "it", "costruisci", "modello.inx"]);
    assert!(errori(&esito).contains("<cartella>"), "{}", errori(&esito));
}

#[test]
fn una_cartella_che_non_e_un_progetto_fallisce_con_uno_e_non_con_due() {
    // Called correctly, and the answer is no. A script that retried this as a
    // usage error would retry it forever.
    let dir = tempdir("cli-cartella-vuota");
    let esito = climesh(&["--lingua", "it", "esegui", dir.to_str().unwrap()]);
    assert_eq!(codice(&esito), 1, "{}", errori(&esito));
    assert!(errori(&esito).starts_with("errore:"), "{}", errori(&esito));
}

#[test]
fn costruisci_scrive_un_progetto_e_dice_che_non_ha_periodi() {
    // An `.INX` holds geometry and no weather, so what comes out has nothing to
    // run yet. Saying it here beats letting `esegui` report zero Corse and
    // leaving the user to work out why.
    let dir = tempdir("cli-costruisci");
    let modello = scrivi_inx(&dir);
    let uscita_dir = dir.join("progetto");
    let esito = climesh(&[
        "--lingua",
        "it",
        "costruisci",
        modello.to_str().unwrap(),
        uscita_dir.to_str().unwrap(),
    ]);
    assert_eq!(codice(&esito), 0, "{}", errori(&esito));
    assert!(
        uscita(&esito).contains("1 Scenario, 0 Periodi"),
        "{}",
        uscita(&esito)
    );
    assert!(
        uscita(&esito).contains("nessun Periodo"),
        "{}",
        uscita(&esito)
    );
    assert!(uscita_dir.join("progetto.toml").is_file());
    // The Scenario is named after the model file and not after `locationName`,
    // which in the reference case says `bergamo` for a model of Bastia Umbra.
    assert!(uscita_dir.join("scenari").join("modellino.toml").is_file());
}

#[test]
fn esegui_e_interroga_lavorano_sullo_stesso_progetto() {
    let dir = tempdir("cli-esegui");
    scrivi_progetto(&dir);

    let esito = climesh(&["--lingua", "it", "esegui", dir.to_str().unwrap()]);
    assert_eq!(codice(&esito), 0, "{}", errori(&esito));
    let detto = uscita(&esito);
    assert!(detto.contains("1 Corsa"), "{detto}");
    assert!(detto.contains("riuscita"), "{detto}");

    // The path of the Giornale is on the page precisely so the next command can
    // be typed from it.
    let giornale = detto
        .lines()
        .find_map(|r| r.trim().strip_prefix("giornale: "))
        .expect("il rapporto porta il percorso del Giornale");
    assert!(Path::new(giornale).is_file(), "{giornale}");

    let esito = climesh(&["--lingua", "it", "interroga", giornale]);
    assert_eq!(codice(&esito), 0, "{}", errori(&esito));
    let detto = uscita(&esito);
    assert!(detto.contains("citazione:"), "{detto}");
    assert!(detto.contains("esito: riuscita"), "{detto}");
    assert!(detto.contains("ore di sole"), "{detto}");
    // The observation point is the other half of the answer: the map says
    // where, the series says when, and the summary shows both.
    assert!(detto.contains("punti di osservazione:"), "{detto}");
    assert!(detto.contains("angolo:"), "{detto}");
    assert!(detto.contains("h di sole su 14"), "{detto}");
    assert!(
        !detto.contains("0.0 h di sole"),
        "un Periodo che contiene mezzogiorno deve avere del sole: {detto}"
    );

    let inglese = climesh(&["--lingua", "en", "interroga", giornale]);
    let detto = uscita(&inglese);
    // The Giornale's own words — «riuscita» — are not translated: they are what
    // the file records, and a summary that reworded them would show something
    // other than what is written.
    assert!(detto.contains("outcome: riuscita"), "{detto}");
    assert!(detto.contains("observation points:"), "{detto}");
    assert!(detto.contains("h of sun out of 14"), "{detto}");
}

#[test]
fn interroga_su_un_file_che_non_e_un_giornale_fallisce_con_uno() {
    let dir = tempdir("cli-interroga-storto");
    let file = dir.join("non-un-giornale.toml");
    fs::write(&file, "questo non e' TOML = = =\n").unwrap();
    let esito = climesh(&["--lingua", "it", "interroga", file.to_str().unwrap()]);
    assert_eq!(codice(&esito), 1, "{}", errori(&esito));
}
