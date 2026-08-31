use climesh::dominio::*;
use climesh::progetto;

fn progetto_di_prova() -> Progetto {
    Progetto {
        nome: "bastia".into(),
        griglia: Griglia {
            nx: 50,
            ny: 50,
            passo_m: 1.0,
            crs: "EPSG:4326".into(),
            origine: (12.56, 43.07),
            rotazione_gradi: 21.0,
        },
        punti: vec![PuntoDiOsservazione {
            id: 1,
            posizione_m: (14.5, 14.5),
            etichetta: "corte".into(),
        }],
        scenari: vec![Scenario {
            nome: "stato-di-fatto".into(),
            derivato_da: None,
            terreno_m: vec![0.0; 2500],
            edifici: vec![Edificio {
                altezza_m: 6.0,
                provenienza: Provenienza {
                    origine: "rilievo di laboratorio".into(),
                    altezza: FonteAltezza::Rilievo,
                },
                impronta: vec![
                    Rettangolo {
                        x_min_m: 10.0,
                        y_min_m: 39.0,
                        x_max_m: 11.0,
                        y_max_m: 40.0,
                    },
                    Rettangolo {
                        x_min_m: 11.0,
                        y_min_m: 39.0,
                        x_max_m: 12.0,
                        y_max_m: 40.0,
                    },
                ],
            }],
            alberi: vec![Albero {
                posizione_m: (4.5, 44.5),
                specie: "020027".into(),
                altezza_m: 12.0,
                frazione_tronco: 0.25,
                provenienza: Provenienza {
                    origine: "LAB1.INX".into(),
                    altezza: FonteAltezza::Predefinito,
                },
            }],
            superfici: vec![Superficie {
                tipo: TipoSuperficie::Erba,
                impronta: vec![Rettangolo {
                    x_min_m: 0.0,
                    y_min_m: 49.0,
                    x_max_m: 1.0,
                    y_max_m: 50.0,
                }],
            }],
        }],
        periodi: vec![Periodo {
            nome: "luglio-2021".into(),
            meteo: "ITA_Perugia.161810_IGDG.epw".into(),
            inizio: Data {
                anno: 2021,
                mese: 7,
                giorno: 15,
            },
            ore: 48,
            direzione_vento_gradi: Some(45.0),
        }],
    }
}

#[test]
fn a_project_survives_a_write_and_read_round_trip() {
    let dir = tempdir_di_prova("round-trip");
    let atteso = progetto_di_prova();
    progetto::scrivi(&dir, &atteso).expect("scrittura");
    let letto = progetto::leggi(&dir).expect("rilettura");
    assert_eq!(letto, atteso);
}

/// Dove finiscono i file su disco, che il round-trip non fissa perché `leggi`
/// costruisce i percorsi con lo stesso join di `scrivi`: se cambiassero
/// entrambi, il round-trip resterebbe verde e la cartella non sarebbe più
/// quella documentata. Questo test la guarda dall'esterno.
#[test]
fn each_scenario_and_period_lands_in_its_own_file_under_the_project_directory() {
    let dir = tempdir_di_prova("posizione-dei-file");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    for percorso in [
        dir.join("progetto.toml"),
        dir.join("scenari/stato-di-fatto.toml"),
        dir.join("periodi/luglio-2021.toml"),
    ] {
        assert!(percorso.is_file(), "manca {}", percorso.display());
    }
    for (sotto, attesi) in [("scenari", 1), ("periodi", 1)] {
        let quanti = std::fs::read_dir(dir.join(sotto)).unwrap().count();
        assert_eq!(quanti, attesi, "file di troppo in {sotto}/");
    }
}

/// Il modulo dice che questi file si leggono a mano, e un file di Scenario porta
/// un valore di terreno per cella.
#[test]
fn a_scenario_file_does_not_spend_a_line_on_every_terrain_value() {
    let dir = tempdir_di_prova("terreno-compatto");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let testo = std::fs::read_to_string(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    let righe = testo.lines().count();
    assert!(righe < 100, "{righe} righe per 2500 valori di terreno");
}

/// Il terreno è scritto bene e accorciato a mano dopo: la rilettura è il solo
/// punto in cui il Progetto può accorgersene.
#[test]
fn a_terrain_of_the_wrong_length_is_caught_when_the_scenario_is_read_back() {
    let dir = tempdir_di_prova("terreno-corto-in-lettura");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let mut scenario = progetto_di_prova().scenari.remove(0);
    scenario.terreno_m.truncate(3);
    std::fs::write(
        dir.join("scenari/stato-di-fatto.toml"),
        toml::to_string(&scenario).unwrap(),
    )
    .unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    let messaggio = err.to_string();
    assert!(
        messaggio.contains("2500") && messaggio.contains('3'),
        "il messaggio deve riportare attese e trovate: {messaggio}"
    );
}

#[test]
fn an_io_error_halfway_through_a_write_leaves_no_manifest_behind() {
    let dir = tempdir_di_prova("scrittura-interrotta");
    std::fs::create_dir_all(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    let err = progetto::scrivi(&dir, &progetto_di_prova()).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Io { .. }),
        "variante: {err:?}"
    );
    assert!(
        !dir.join("progetto.toml").exists(),
        "un manifesto che nomina file assenti è peggio di nessun manifesto"
    );
}

/// Un Progetto è un archivio che gli utenti si scambiano: un collegamento al
/// suo interno fa scrivere o leggere altrove, e nessuno lo vede nel manifesto.
#[cfg(unix)]
#[test]
fn a_scenario_directory_that_is_a_symlink_is_refused_on_write() {
    let dir = tempdir_di_prova("scenari-collegati");
    std::fs::create_dir_all(dir.join("altrove")).unwrap();
    std::os::unix::fs::symlink(dir.join("altrove"), dir.join("scenari")).unwrap();
    let err = progetto::scrivi(&dir, &progetto_di_prova()).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Collegamento(_)),
        "variante: {err:?}"
    );
    assert!(
        !dir.join("altrove/stato-di-fatto.toml").exists(),
        "nessuna scrittura attraverso il collegamento"
    );
}

#[cfg(unix)]
#[test]
fn a_scenario_file_that_is_a_symlink_is_refused_on_write() {
    let dir = tempdir_di_prova("scenario-collegato");
    std::fs::create_dir_all(dir.join("scenari")).unwrap();
    std::fs::write(dir.join("fuori.toml"), "").unwrap();
    std::os::unix::fs::symlink(
        dir.join("fuori.toml"),
        dir.join("scenari/stato-di-fatto.toml"),
    )
    .unwrap();
    let err = progetto::scrivi(&dir, &progetto_di_prova()).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Collegamento(_)),
        "variante: {err:?}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("fuori.toml")).unwrap(),
        "",
        "nessuna scrittura attraverso il collegamento"
    );
}

#[cfg(unix)]
#[test]
fn a_scenario_file_that_is_a_symlink_is_refused_on_read() {
    let dir = tempdir_di_prova("scenario-collegato-in-lettura");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let vero = dir.join("scenari/stato-di-fatto.toml");
    std::fs::rename(&vero, dir.join("fuori.toml")).unwrap();
    std::os::unix::fs::symlink(dir.join("fuori.toml"), &vero).unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Collegamento(_)),
        "variante: {err:?}"
    );
}

#[test]
fn writing_over_an_existing_project_stays_possible() {
    let dir = tempdir_di_prova("sovrascrittura");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let mut p = progetto_di_prova();
    p.nome = "bastia-riletta".into();
    progetto::scrivi(&dir, &p).expect("un Progetto esistente si sovrascrive");
    assert_eq!(progetto::leggi(&dir).unwrap(), p);
}

#[test]
fn a_project_directory_without_its_manifest_names_the_missing_file() {
    let dir = tempdir_di_prova("senza-manifesto");
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        err.to_string().contains("progetto.toml"),
        "messaggio: {err}"
    );
}

#[test]
fn a_scenario_referenced_by_the_manifest_but_absent_on_disk_is_named() {
    let dir = tempdir_di_prova("scenario-mancante");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    std::fs::remove_file(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        err.to_string().contains("stato-di-fatto"),
        "messaggio: {err}"
    );
}

#[test]
fn a_grid_with_no_cells_is_rejected_rather_than_written() {
    let dir = tempdir_di_prova("griglia-vuota");
    let mut p = progetto_di_prova();
    p.griglia.nx = 0;
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(err.to_string().contains("griglia"), "messaggio: {err}");
    assert!(
        !dir.join("progetto.toml").exists(),
        "una griglia rifiutata non deve lasciare un manifesto su disco"
    );
}

#[test]
fn a_grid_with_more_cells_than_usize_can_count_is_rejected() {
    let dir = tempdir_di_prova("griglia-traboccante");
    let mut p = progetto_di_prova();
    p.griglia.nx = 1 << 32;
    p.griglia.ny = 1 << 32;
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Griglia(_)),
        "variante: {err:?}"
    );
}

#[test]
fn a_terrain_that_does_not_match_the_grid_is_rejected() {
    let dir = tempdir_di_prova("terreno-corto");
    let mut p = progetto_di_prova();
    p.scenari[0].terreno_m.truncate(10);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        err.to_string().contains("2500") && err.to_string().contains("10"),
        "il messaggio deve riportare attese e trovate: {err}"
    );
}

#[test]
fn reading_a_directory_that_does_not_exist_names_the_path() {
    let dir = tempdir_di_prova("cartella-assente").join("mai-creata");
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        err.to_string().contains("mai-creata") && err.to_string().contains("progetto.toml"),
        "messaggio: {err}"
    );
}

#[test]
fn a_truncated_manifest_names_the_file_and_the_cause() {
    let dir = tempdir_di_prova("manifesto-troncato");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let testo = std::fs::read_to_string(dir.join("progetto.toml")).unwrap();
    std::fs::write(dir.join("progetto.toml"), &testo[..testo.len() / 3]).unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Sintassi { .. }),
        "variante: {err:?}"
    );
    assert!(
        err.to_string().contains("progetto.toml"),
        "messaggio: {err}"
    );
}

#[test]
fn a_building_whose_footprint_leaves_the_grid_is_rejected() {
    let dir = tempdir_di_prova("impronta-fuori");
    let mut p = progetto_di_prova();
    p.scenari[0].edifici[0].impronta[0].x_max_m = 60.0;
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::FuoriGriglia(_)),
        "variante: {err:?}"
    );
}

#[test]
fn a_tree_planted_outside_the_grid_is_rejected() {
    let dir = tempdir_di_prova("albero-fuori");
    let mut p = progetto_di_prova();
    p.scenari[0].alberi[0].posizione_m = (-1.0, 44.5);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::FuoriGriglia(_)),
        "variante: {err:?}"
    );
}

#[test]
fn an_observation_point_outside_the_grid_is_rejected() {
    let dir = tempdir_di_prova("punto-fuori");
    let mut p = progetto_di_prova();
    p.punti[0].posizione_m = (14.5, 50.5);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::FuoriGriglia(_)),
        "variante: {err:?}"
    );
}

#[test]
fn a_scenario_name_that_climbs_out_of_the_project_directory_is_rejected() {
    let dir = tempdir_di_prova("nome-in-fuga");
    let mut p = progetto_di_prova();
    p.scenari[0].nome = "../../fuga".into();
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(err.to_string().contains("../../fuga"), "messaggio: {err}");
    assert!(
        !dir.parent().unwrap().join("fuga.toml").exists(),
        "nessuna scrittura fuori dalla cartella del Progetto"
    );
}

#[test]
fn a_scenario_name_that_is_not_a_plain_file_name_is_rejected() {
    for nome in [
        "C:evil",
        r"\\server\share",
        "CON",
        "nul",
        "com1",
        "LPT9",
        "aux.toml",
        "prn.txt",
    ] {
        let dir = tempdir_di_prova("nome-non-semplice");
        let mut p = progetto_di_prova();
        p.scenari[0].nome = nome.into();
        let err = progetto::scrivi(&dir, &p)
            .err()
            .unwrap_or_else(|| panic!("«{nome}» deve fallire"));
        assert!(
            matches!(err, progetto::ProgettoError::Nome(_)),
            "«{nome}»: {err:?}"
        );
        assert!(
            !dir.join("scenari").exists(),
            "«{nome}» non deve lasciare nulla su disco"
        );
    }
}

#[test]
fn a_scenario_name_that_is_empty_or_padded_or_all_dots_is_rejected() {
    for nome in [
        "",
        ".",
        "..",
        "...",
        "con spazio",
        "spazio finale ",
        "finale.",
    ] {
        let dir = tempdir_di_prova("nome-degenere");
        let mut p = progetto_di_prova();
        p.scenari[0].nome = nome.into();
        let err = progetto::scrivi(&dir, &p)
            .err()
            .unwrap_or_else(|| panic!("«{nome}» deve fallire"));
        assert!(
            matches!(err, progetto::ProgettoError::Nome(_)),
            "«{nome}»: {err:?}"
        );
    }
}

#[test]
fn two_scenario_names_differing_only_in_case_are_rejected_rather_than_overwriting() {
    let dir = tempdir_di_prova("scenari-omonimi");
    let mut p = progetto_di_prova();
    let mut copia = p.scenari[0].clone();
    copia.nome = "Stato-Di-Fatto".into();
    p.scenari.push(copia);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::NomeDuplicato(_)),
        "variante: {err:?}"
    );
    assert!(
        !dir.join("scenari").exists(),
        "nessuna sovrascrittura silenziosa"
    );
}

#[test]
fn two_periods_with_the_same_name_are_rejected() {
    let dir = tempdir_di_prova("periodi-omonimi");
    let mut p = progetto_di_prova();
    let copia = p.periodi[0].clone();
    p.periodi.push(copia);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::NomeDuplicato(_)),
        "variante: {err:?}"
    );
}

#[test]
fn a_period_name_with_a_path_separator_is_rejected() {
    let dir = tempdir_di_prova("periodo-con-separatore");
    let mut p = progetto_di_prova();
    p.periodi[0].nome = "luglio/2021".into();
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(err.to_string().contains("luglio/2021"), "messaggio: {err}");
}

#[test]
fn a_manifest_naming_a_scenario_outside_the_project_is_rejected_before_opening_it() {
    let dir = tempdir_di_prova("manifesto-in-fuga");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let testo = std::fs::read_to_string(dir.join("progetto.toml")).unwrap();
    std::fs::write(
        dir.join("progetto.toml"),
        testo.replace("stato-di-fatto", "../../fuga"),
    )
    .unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(
        matches!(err, progetto::ProgettoError::Nome(_)),
        "variante: {err:?}"
    );
}

/// Un Progetto appena creato non ha ancora Scenari né Periodi, e non è un errore.
#[test]
fn a_project_with_no_scenarios_and_no_periods_round_trips() {
    let dir = tempdir_di_prova("progetto-vuoto");
    let atteso = Progetto {
        scenari: vec![],
        periodi: vec![],
        ..progetto_di_prova()
    };
    progetto::scrivi(&dir, &atteso).expect("scrittura");
    assert_eq!(progetto::leggi(&dir).expect("rilettura"), atteso);
}

#[test]
fn day_of_the_year_counts_the_leap_day() {
    let ordinario = Data {
        anno: 2021,
        mese: 7,
        giorno: 15,
    };
    assert_eq!(ordinario.giorno_dell_anno(), Some(196));
    let bisestile = Data {
        anno: 2020,
        mese: 3,
        giorno: 1,
    };
    assert_eq!(bisestile.giorno_dell_anno(), Some(61));
}

#[test]
fn a_month_outside_the_calendar_yields_no_day_of_the_year() {
    for mese in [0, 13] {
        let data = Data {
            anno: 2021,
            mese,
            giorno: 1,
        };
        assert_eq!(data.giorno_dell_anno(), None, "mese {mese}");
    }
}

#[test]
fn a_day_the_month_does_not_have_yields_no_day_of_the_year() {
    for (mese, giorno) in [(4, 31), (2, 29), (1, 0), (12, u32::MAX)] {
        let data = Data {
            anno: 2021,
            mese,
            giorno,
        };
        assert_eq!(data.giorno_dell_anno(), None, "{mese}/{giorno}");
    }
}

/// La regola secolare: divisibile per 100 non basta, serve anche per 400.
#[test]
fn the_twenty_ninth_of_february_follows_the_century_rule() {
    let feb29 = |anno| {
        Data {
            anno,
            mese: 2,
            giorno: 29,
        }
        .giorno_dell_anno()
    };
    assert_eq!(feb29(2000), Some(60));
    assert_eq!(feb29(2024), Some(60));
    assert_eq!(feb29(1900), None);
    assert_eq!(feb29(2100), None);
}

/// Una cartella pulita sotto `target/`, così i test non lasciano nulla in giro
/// e non serve una dipendenza per i file temporanei.
fn tempdir_di_prova(nome: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
