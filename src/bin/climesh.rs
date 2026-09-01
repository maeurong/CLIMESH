//! The command line: build a Progetto, run its Corse, ask a Giornale what it
//! says. Headless, so a parametric study or a continuous-integration job drives
//! it the same way a person does.
//!
//! **No argument-parsing crate.** The interface is bilingual from the first day,
//! and a parser that carries its own help text and its own error messages
//! carries them in one language: the strings that matter most — the ones a user
//! sees when they got the command wrong — would be the ones outside
//! `crate::lingua`. Three commands and half a dozen options do not need a
//! framework, and this way every sentence the program prints comes from the
//! same place.
//!
//! **The command names are not translated**, only the sentences around them.
//! A verb that changed spelling with the environment would break a shell script
//! on somebody else's machine.

use climesh::corsa;
use climesh::da_inx::progetto_da_inx;
use climesh::inx::read_inx;
use climesh::lingua::{Lingua, Messaggi, COSTRUISCI, ESEGUI, INTERROGA};
use climesh::progetto;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Wrong usage, told apart from a command that ran and failed. A script can
/// then tell "I called it wrong" from "it called it right and the answer was
/// no".
const USO_SBAGLIATO: u8 = 2;

fn main() -> ExitCode {
    let argomenti: Vec<String> = std::env::args().skip(1).collect();
    // The language is settled before anything can be said, including the
    // complaint about the language itself.
    let (lingua, argomenti) = match separa_lingua(argomenti) {
        Ok(esito) => esito,
        Err(codice) => {
            let m = Lingua::dall_ambiente(|n| std::env::var(n).ok()).messaggi();
            eprintln!("{} {}", m.errore, (m.lingua_ignota)(&codice));
            return ExitCode::from(USO_SBAGLIATO);
        }
    };
    let m = lingua.messaggi();

    match esegui(&argomenti, m) {
        Ok(testo) => {
            if !testo.is_empty() {
                println!("{testo}");
            }
            ExitCode::SUCCESS
        }
        Err(Problema::Uso(testo)) => {
            eprintln!("{} {testo}\n", m.errore);
            eprintln!("{}", aiuto(m));
            ExitCode::from(USO_SBAGLIATO)
        }
        Err(Problema::Fallito(testo)) => {
            eprintln!("{} {testo}", m.errore);
            ExitCode::FAILURE
        }
    }
}

enum Problema {
    /// The command was not called correctly. The help text follows.
    Uso(String),
    /// The command was called correctly and could not be carried out.
    Fallito(String),
}

/// Pulls `--lingua <codice>` out of the arguments, wherever it stands.
///
/// It is read before the command because everything else, the complaint about a
/// wrong command included, has to come out in it. `Err` carries the code that
/// was not understood.
fn separa_lingua(argomenti: Vec<String>) -> Result<(Lingua, Vec<String>), String> {
    let mut lingua = None;
    let mut restanti = Vec::new();
    let mut iteratore = argomenti.into_iter();
    while let Some(argomento) = iteratore.next() {
        match argomento.split_once('=') {
            Some(("--lingua", codice)) => {
                lingua = Some(Lingua::dal_codice(codice).ok_or_else(|| codice.to_owned())?);
            }
            _ if argomento == "--lingua" => {
                let codice = iteratore.next().unwrap_or_default();
                lingua = Some(Lingua::dal_codice(&codice).ok_or(codice)?);
            }
            _ => restanti.push(argomento),
        }
    }
    Ok((
        lingua.unwrap_or_else(|| Lingua::dall_ambiente(|n| std::env::var(n).ok())),
        restanti,
    ))
}

fn aiuto(m: &Messaggi) -> String {
    [
        m.uso,
        "",
        m.descrizione_costruisci,
        m.descrizione_esegui,
        m.descrizione_interroga,
        "",
        m.opzioni,
    ]
    .join("\n")
}

/// The whole program bar the printing, so a test can call it and read what it
/// would have said.
fn esegui(argomenti: &[String], m: &Messaggi) -> Result<String, Problema> {
    if argomenti.iter().any(|a| a == "--versione") {
        return Ok(format!("climesh {}", env!("CARGO_PKG_VERSION")));
    }
    if argomenti.is_empty() || argomenti.iter().any(|a| a == "--aiuto") {
        return Ok(aiuto(m));
    }

    let (comando, resto) = argomenti.split_first().expect("non è vuoto");
    if comando.starts_with('-') {
        return Err(Problema::Uso((m.opzione_ignota)(comando)));
    }
    let posizionali = solo_posizionali(resto, m)?;
    match comando.as_str() {
        COSTRUISCI => costruisci(&posizionali, m),
        ESEGUI => esegui_progetto(&posizionali, m),
        INTERROGA => interroga(&posizionali, m),
        altro => Err(Problema::Uso((m.comando_ignoto)(altro))),
    }
}

/// The arguments of a command, which today are all positional.
///
/// An option none of the commands takes is refused rather than ignored: a
/// misspelt flag that changed nothing and said nothing would be the worst of
/// the three possible answers.
fn solo_posizionali<'a>(resto: &'a [String], m: &Messaggi) -> Result<Vec<&'a str>, Problema> {
    let mut posizionali = Vec::new();
    for argomento in resto {
        if argomento.starts_with("--") {
            return Err(Problema::Uso((m.opzione_ignota)(argomento)));
        }
        posizionali.push(argomento.as_str());
    }
    Ok(posizionali)
}

fn quanti(
    posizionali: &[&str],
    attesi: usize,
    nomi: &[&str],
    m: &Messaggi,
) -> Result<(), Problema> {
    if posizionali.len() < attesi {
        return Err(Problema::Uso((m.manca_argomento)(nomi[posizionali.len()])));
    }
    if let Some(di_troppo) = posizionali.get(attesi) {
        return Err(Problema::Uso((m.argomento_di_troppo)(di_troppo)));
    }
    Ok(())
}

fn costruisci(posizionali: &[&str], m: &Messaggi) -> Result<String, Problema> {
    quanti(posizionali, 2, &["<modello.inx>", "<cartella>"], m)?;
    let (modello, cartella) = (Path::new(posizionali[0]), PathBuf::from(posizionali[1]));

    let letto = read_inx(modello).map_err(|e| Problema::Fallito(e.to_string()))?;
    // The name of the Progetto is the folder's, and the name of the Scenario is
    // the model file's: an `.INX` carries a `location.name` that in the
    // reference case says `bergamo` for a model of Bastia Umbra.
    let nome_progetto = nome_di(&cartella).unwrap_or("progetto");
    let nome_scenario = nome_di(modello).unwrap_or("scenario");
    let progetto = progetto_da_inx(&letto, nome_progetto, nome_scenario)
        .map_err(|e| Problema::Fallito(e.to_string()))?;
    progetto::scrivi(&cartella, &progetto).map_err(|e| Problema::Fallito(e.to_string()))?;

    let mut righe = vec![
        (m.progetto_scritto)(&cartella.display().to_string()),
        (m.scenari_e_periodi)(progetto.scenari.len(), progetto.periodi.len()),
    ];
    // An `.INX` holds geometry and no weather, so the Progetto it builds has no
    // Periodo and nothing to run yet. Saying so here is cheaper than letting
    // `esegui` report zero Corse and leaving the user to work out why.
    if progetto.periodi.is_empty() {
        righe.push(m.senza_periodi.to_owned());
    }
    Ok(righe.join("\n"))
}

/// The stem of a path, when it has one that is not empty.
fn nome_di(percorso: &Path) -> Option<&str> {
    percorso
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
}

fn esegui_progetto(posizionali: &[&str], m: &Messaggi) -> Result<String, Problema> {
    quanti(posizionali, 1, &["<cartella>"], m)?;
    let rapporto =
        corsa::esegui_progetto(posizionali[0]).map_err(|e| Problema::Fallito(e.to_string()))?;

    let mut righe = vec![format!(
        "{} {}",
        (m.corse_eseguite)(rapporto.corse.len()),
        (m.tempo_totale)(rapporto.tempo_totale.as_secs_f64())
    )];
    for corsa in &rapporto.corse {
        righe.push(match &corsa.errore {
            None => (m.corsa_riuscita)(&corsa.etichetta),
            Some(errore) => (m.corsa_fallita)(&corsa.etichetta, errore),
        });
        righe.push((m.giornale_in)(&corsa.giornale.display().to_string()));
    }
    Ok(righe.join("\n"))
}

fn interroga(posizionali: &[&str], m: &Messaggi) -> Result<String, Problema> {
    quanti(posizionali, 1, &["<giornale.toml>"], m)?;
    let testo = std::fs::read_to_string(posizionali[0])
        .map_err(|e| Problema::Fallito(format!("{}: {e}", posizionali[0])))?;
    let tabella: toml::Table =
        toml::from_str(&testo).map_err(|e| Problema::Fallito(e.to_string()))?;
    Ok(riassunto(&tabella, m))
}

/// What a Giornale says, in the order a reader needs it: what to cite, how it
/// ended, what raised a flag, and only then the numbers.
fn riassunto(tabella: &toml::Table, m: &Messaggi) -> String {
    let stringa = |sezione: &str, chiave: &str| {
        tabella
            .get(sezione)
            .and_then(|s| s.get(chiave))
            .and_then(|v| v.as_str())
    };
    let mut righe = Vec::new();
    if let Some(citazione) = stringa("corsa", "citazione") {
        righe.push(format!("{} {citazione}", m.citazione));
    }
    // A Giornale with no `[conclusione]` is a Corsa that did not finish, and
    // the absence is the answer: nothing is invented in its place.
    righe.push(match stringa("conclusione", "esito") {
        Some(esito) => format!("{} {esito}", m.esito),
        None => format!("{} —", m.esito),
    });

    let campi = tabella
        .get("campo")
        .and_then(|c| c.as_array())
        .map(|a| a.as_slice())
        .unwrap_or_default();
    let con_bandiera = campi
        .iter()
        .filter(|c| {
            c.get("fuori_intervallo").and_then(|v| v.as_bool()) == Some(true)
                || c.get("bandiera").and_then(|v| v.as_bool()) == Some(true)
        })
        .count()
        + tabella
            .get("verifica_ombra")
            .and_then(|v| v.get("bandiera"))
            .and_then(|v| v.as_bool())
            .map(usize::from)
            .unwrap_or(0);
    righe.push(if con_bandiera == 0 {
        m.nessuna_bandiera.to_owned()
    } else {
        (m.verifiche_con_bandiera)(con_bandiera)
    });

    if !campi.is_empty() {
        righe.push(m.campi.to_owned());
        for campo in campi {
            let nome = campo.get("campo").and_then(|v| v.as_str()).unwrap_or("—");
            let unita = campo.get("unita").and_then(|v| v.as_str()).unwrap_or("");
            let numero = |chiave: &str| campo.get(chiave).and_then(|v| v.as_float());
            righe.push(
                match (numero("minimo"), numero("massimo"), numero("media")) {
                    (Some(minimo), Some(massimo), Some(media)) => {
                        format!("  {nome}: {minimo} … {massimo}, {media} {unita}")
                    }
                    // No minimum was invented for a field with nothing in it, and
                    // none is invented here either.
                    _ => format!("  {nome}: {}", m.campo_senza_dato),
                },
            );
        }
    }
    righe.join("\n")
}
