//! The message catalogue. Completeness is the compiler's job — [`Messaggi`] is
//! a struct, and a message added to one language and forgotten in the other
//! does not build — so what is left to test is what the compiler cannot see:
//! that the two constants are not copies of each other, and that the choice of
//! language does what a user expects.

use climesh::lingua::{Lingua, INGLESE, ITALIANO};

#[test]
fn una_lingua_si_riconosce_dal_prefisso_del_codice() {
    // `it`, `it-IT` and `it_IT.UTF-8` are the same answer: a locale carries a
    // territory and an encoding, and neither changes which language it is.
    for codice in ["it", "IT", "it-IT", "it_IT.UTF-8"] {
        assert_eq!(
            Lingua::dal_codice(codice),
            Some(Lingua::Italiano),
            "{codice}"
        );
    }
    for codice in ["en", "en-GB", "en_US.UTF-8"] {
        assert_eq!(
            Lingua::dal_codice(codice),
            Some(Lingua::Inglese),
            "{codice}"
        );
    }
    for codice in ["de", "fr_FR", "", "C", "POSIX"] {
        assert_eq!(Lingua::dal_codice(codice), None, "{codice}");
    }
}

#[test]
fn l_ambiente_si_legge_nell_ordine_dichiarato() {
    // `CLIMESH_LINGUA` wins over the locale, and `LC_ALL` over `LANG`, which is
    // the order the shell itself uses. A user who set the specific variable
    // meant it.
    for variabile in ["CLIMESH_LINGUA", "LC_ALL", "LC_MESSAGES", "LANG"] {
        let solo_questa = |nome: &str| (nome == variabile).then(|| "it_IT.UTF-8".to_owned());
        assert_eq!(
            Lingua::dall_ambiente(solo_questa),
            Lingua::Italiano,
            "{variabile}"
        );
    }

    let contrastanti = |nome: &str| match nome {
        "LC_ALL" => Some("en_GB".to_owned()),
        "LANG" => Some("it_IT".to_owned()),
        _ => None,
    };
    assert_eq!(Lingua::dall_ambiente(contrastanti), Lingua::Inglese);
}

#[test]
fn un_ambiente_muto_o_ignoto_parla_inglese() {
    // English and not Italian: someone whose machine is set to a third language
    // is likelier to read English. The project's own documents are where
    // Italian is the default, not the interface.
    assert_eq!(Lingua::dall_ambiente(|_| None), Lingua::Inglese);
    assert_eq!(
        Lingua::dall_ambiente(|_| Some("ja_JP.UTF-8".to_owned())),
        Lingua::Inglese
    );
}

#[test]
fn le_due_lingue_non_sono_la_stessa() {
    // The failure this catches is the one the compiler cannot: a field filled
    // in by copying the other language's text across.
    let coppie = [
        (ITALIANO.uso, INGLESE.uso),
        (ITALIANO.opzioni, INGLESE.opzioni),
        (ITALIANO.comando_assente, INGLESE.comando_assente),
        (ITALIANO.senza_periodi, INGLESE.senza_periodi),
        (ITALIANO.citazione, INGLESE.citazione),
        (ITALIANO.esito, INGLESE.esito),
        (ITALIANO.nessuna_bandiera, INGLESE.nessuna_bandiera),
        (ITALIANO.campi, INGLESE.campi),
        (ITALIANO.campo_senza_dato, INGLESE.campo_senza_dato),
        (ITALIANO.errore, INGLESE.errore),
        (
            ITALIANO.descrizione_costruisci,
            INGLESE.descrizione_costruisci,
        ),
        (ITALIANO.descrizione_esegui, INGLESE.descrizione_esegui),
        (
            ITALIANO.descrizione_interroga,
            INGLESE.descrizione_interroga,
        ),
    ];
    for (italiano, inglese) in coppie {
        assert!(!italiano.trim().is_empty(), "un messaggio italiano è vuoto");
        assert!(!inglese.trim().is_empty(), "un messaggio inglese è vuoto");
        assert_ne!(italiano, inglese, "le due lingue dicono la stessa cosa");
    }
}

#[test]
fn i_messaggi_con_parametri_ci_mettono_dentro_il_parametro() {
    // A parametrised message is a function pointer and not a template with
    // holes: the arity is checked, and the two languages cannot disagree about
    // how many holes there are. What is left to check is that the value gets
    // in at all.
    assert!((ITALIANO.comando_ignoto)("pippo").contains("pippo"));
    assert!((INGLESE.comando_ignoto)("pippo").contains("pippo"));
    assert!((ITALIANO.opzione_ignota)("--boh").contains("--boh"));
    assert!((INGLESE.opzione_ignota)("--boh").contains("--boh"));
    assert!((ITALIANO.corse_eseguite)(4).contains('4'));
    assert!((INGLESE.corse_eseguite)(4).contains('4'));
    let fallita = (ITALIANO.corsa_fallita)("estate", "niente sole");
    assert!(fallita.contains("estate") && fallita.contains("niente sole"));
}

#[test]
fn il_singolare_e_il_plurale_si_accordano() {
    // "1 Corse" and "1 verifiche" are what the first run of the reference case
    // actually printed. A program that cannot count to one reads as one that
    // was not looked at.
    assert!((ITALIANO.corse_eseguite)(1).contains("1 Corsa"));
    assert!((ITALIANO.corse_eseguite)(4).contains("4 Corse"));
    assert!((ITALIANO.verifiche_con_bandiera)(1).contains("1 verifica con"));
    assert!((ITALIANO.verifiche_con_bandiera)(3).contains("3 verifiche con"));
    assert_eq!((ITALIANO.scenari_e_periodi)(1, 1), "1 Scenario, 1 Periodo");
    assert_eq!((ITALIANO.scenari_e_periodi)(2, 3), "2 Scenari, 3 Periodi");

    assert!((INGLESE.corse_eseguite)(1).contains("1 Corsa"));
    assert!((INGLESE.verifiche_con_bandiera)(1).contains("1 check "));
    assert!((INGLESE.verifiche_con_bandiera)(2).contains("2 checks "));
    assert_eq!((INGLESE.scenari_e_periodi)(1, 1), "1 Scenario, 1 Periodo");
}

#[test]
fn i_termini_del_dominio_restano_gli_stessi_in_tutte_e_due() {
    // Progetto, Corsa, Periodo, Giornale: the vocabulary of `CONTEXT.md` is
    // binding, and translating it in the English messages would give an English
    // reader a second set of names for the same things.
    assert!(INGLESE.senza_periodi.contains("Periodo"));
    assert!((INGLESE.corse_eseguite)(4).contains("Corse"));
    assert!((INGLESE.scenari_e_periodi)(2, 2).contains("Scenari"));
    assert!((INGLESE.progetto_scritto)("x").contains("Progetto"));
    assert!((INGLESE.giornale_in)("x").contains("giornale"));
}
