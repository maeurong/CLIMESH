//! Every string the interface shows, in both languages, in one place.
//!
//! The rule is [`CLAUDE.md`](../CLAUDE.md)'s: **no string is nested in the
//! code**. A message written where it is printed is a message that exists in
//! one language, and adding the second one later means finding all of them —
//! which is the moment a project decides it is monolingual after all.
//!
//! There is no framework and no catalogue file to load. [`Messaggi`] is a
//! struct, the two languages are two constants of it, and a message added to
//! one and forgotten in the other does not compile. That is the whole
//! mechanism, and it buys the only guarantee that matters here: the two
//! languages cannot drift apart.
//!
//! **The technical terms of the domain stay English in both**: *sky view
//! factor*, *mean radiant temperature*, *UTCI*. Translating them would help
//! nobody who reads them.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lingua {
    Italiano,
    Inglese,
}

impl Lingua {
    /// The language named by a code, `None` for one this program does not have.
    ///
    /// Only the two, and by prefix: `it`, `it-IT`, `it_IT.UTF-8` are the same
    /// answer.
    pub fn dal_codice(codice: &str) -> Option<Self> {
        let codice = codice.to_ascii_lowercase();
        if codice.starts_with("it") {
            Some(Self::Italiano)
        } else if codice.starts_with("en") {
            Some(Self::Inglese)
        } else {
            None
        }
    }

    /// The language of the environment, English when it says nothing this
    /// program understands.
    ///
    /// English and not Italian: a user whose machine is set to a third language
    /// is more likely to read English than Italian, and the project's own
    /// documents are the place where Italian is the default.
    pub fn dall_ambiente(variabili: impl Fn(&str) -> Option<String>) -> Self {
        ["CLIMESH_LINGUA", "LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .find_map(|nome| variabili(nome).and_then(|v| Self::dal_codice(&v)))
            .unwrap_or(Self::Inglese)
    }

    pub fn messaggi(self) -> &'static Messaggi {
        match self {
            Self::Italiano => &ITALIANO,
            Self::Inglese => &INGLESE,
        }
    }
}

impl fmt::Display for Lingua {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Italiano => "it",
            Self::Inglese => "en",
        })
    }
}

/// What the command line says, in one language.
///
/// Parametrised messages are function pointers rather than templates with
/// placeholders: a template is a string whose arity nothing checks, and the
/// two languages would be free to disagree about how many holes it has.
pub struct Messaggi {
    pub uso: &'static str,
    pub descrizione_costruisci: &'static str,
    pub descrizione_esegui: &'static str,
    pub descrizione_interroga: &'static str,
    pub opzioni: &'static str,

    pub comando_assente: &'static str,
    pub comando_ignoto: fn(&str) -> String,
    pub opzione_ignota: fn(&str) -> String,
    pub opzione_senza_valore: fn(&str) -> String,
    pub argomento_di_troppo: fn(&str) -> String,
    pub manca_argomento: fn(&str) -> String,
    pub lingua_ignota: fn(&str) -> String,

    pub progetto_scritto: fn(&str) -> String,
    pub scenari_e_periodi: fn(usize, usize) -> String,
    pub senza_periodi: &'static str,

    pub corse_eseguite: fn(usize) -> String,
    pub tempo_totale: fn(f64) -> String,
    pub corsa_riuscita: fn(&str) -> String,
    pub corsa_fallita: fn(&str, &str) -> String,
    pub giornale_in: fn(&str) -> String,

    pub citazione: &'static str,
    pub esito: &'static str,
    pub verifiche_con_bandiera: fn(usize) -> String,
    pub nessuna_bandiera: &'static str,
    pub campi: &'static str,
    pub campo_senza_dato: &'static str,
    pub punti: &'static str,
    /// The label of a Punto, its hours in the sun, the hours of the Periodo,
    /// and the per cent of the sky it sees.
    pub punto_sole_e_cielo: fn(&str, f64, usize, f64) -> String,

    pub errore: &'static str,
}

pub const ITALIANO: Messaggi = Messaggi {
    uso: "uso: climesh <comando> [opzioni]",
    descrizione_costruisci: "  costruisci <modello.inx> <cartella>   un Progetto da un file .INX",
    descrizione_esegui: "  esegui <cartella>                     tutte le Corse del Progetto",
    descrizione_interroga: "  interroga <giornale.toml>             che cosa dice un Giornale",
    opzioni: "opzioni: --lingua it|en   --aiuto   --versione",

    comando_assente: "manca il comando",
    comando_ignoto: |c| format!("«{c}» non è un comando di climesh"),
    opzione_ignota: |o| format!("«{o}» non è un'opzione di questo comando"),
    opzione_senza_valore: |o| format!("«{o}» vuole un valore dopo di sé"),
    argomento_di_troppo: |a| format!("«{a}» è un argomento di troppo"),
    manca_argomento: |a| format!("manca {a}"),
    lingua_ignota: |l| format!("«{l}» non è una lingua di questo programma: it oppure en"),

    progetto_scritto: |d| format!("Progetto scritto in {d}"),
    scenari_e_periodi: |s, p| {
        format!(
            "{s} {}, {p} {}",
            if s == 1 { "Scenario" } else { "Scenari" },
            if p == 1 { "Periodo" } else { "Periodi" }
        )
    },
    senza_periodi: "nessun Periodo: senza file meteo non c'è niente da eseguire",

    corse_eseguite: |n| format!("{n} {}", if n == 1 { "Corsa" } else { "Corse" }),
    tempo_totale: |s| format!("in {s:.2} s"),
    corsa_riuscita: |e| format!("  {e}: riuscita"),
    corsa_fallita: |e, errore| format!("  {e}: fallita — {errore}"),
    giornale_in: |p| format!("    giornale: {p}"),

    citazione: "citazione:",
    esito: "esito:",
    verifiche_con_bandiera: |n| {
        format!(
            "{n} {} con la bandiera alzata",
            if n == 1 { "verifica" } else { "verifiche" }
        )
    },
    nessuna_bandiera: "nessuna bandiera alzata",
    campi: "campi:",
    campo_senza_dato: "tutto senza dato",
    punti: "punti di osservazione:",
    punto_sole_e_cielo: |etichetta, ore, su, cielo| {
        format!("  {etichetta}: {ore:.1} h di sole su {su}, vede il {cielo:.0}% del cielo")
    },

    errore: "errore:",
};

pub const INGLESE: Messaggi = Messaggi {
    uso: "usage: climesh <command> [options]",
    descrizione_costruisci: "  costruisci <model.inx> <folder>       a Progetto from an .INX file",
    descrizione_esegui: "  esegui <folder>                       every Corsa of the Progetto",
    descrizione_interroga: "  interroga <giornale.toml>             what a Giornale says",
    opzioni: "options: --lingua it|en   --aiuto   --versione",

    comando_assente: "no command given",
    comando_ignoto: |c| format!("\"{c}\" is not a climesh command"),
    opzione_ignota: |o| format!("\"{o}\" is not an option of this command"),
    opzione_senza_valore: |o| format!("\"{o}\" wants a value after it"),
    argomento_di_troppo: |a| format!("\"{a}\" is one argument too many"),
    manca_argomento: |a| format!("missing {a}"),
    lingua_ignota: |l| format!("\"{l}\" is not a language of this program: it or en"),

    progetto_scritto: |d| format!("Progetto written to {d}"),
    scenari_e_periodi: |s, p| {
        format!(
            "{s} {}, {p} {}",
            if s == 1 { "Scenario" } else { "Scenari" },
            if p == 1 { "Periodo" } else { "Periodi" }
        )
    },
    senza_periodi: "no Periodo: without a weather file there is nothing to run",

    corse_eseguite: |n| format!("{n} {}", if n == 1 { "Corsa" } else { "Corse" }),
    tempo_totale: |s| format!("in {s:.2} s"),
    corsa_riuscita: |e| format!("  {e}: done"),
    corsa_fallita: |e, errore| format!("  {e}: failed — {errore}"),
    giornale_in: |p| format!("    giornale: {p}"),

    citazione: "citation:",
    esito: "outcome:",
    verifiche_con_bandiera: |n| {
        format!(
            "{n} {} raised a flag",
            if n == 1 { "check" } else { "checks" }
        )
    },
    nessuna_bandiera: "no flag raised",
    campi: "fields:",
    campo_senza_dato: "no data at all",
    punti: "observation points:",
    punto_sole_e_cielo: |etichetta, ore, su, cielo| {
        format!("  {etichetta}: {ore:.1} h of sun out of {su}, sees {cielo:.0}% of the sky")
    },

    errore: "error:",
};

/// The names of the commands and options are **not** translated.
///
/// A command whose spelling changed with the environment would make a script
/// stop working on a machine set to another language, and a Corsa launched from
/// a shell script has to run the same everywhere. The words are the domain's,
/// which [`CONTEXT.md`](../CONTEXT.md) keeps in Italian; the sentences around
/// them are what this module translates.
pub const COSTRUISCI: &str = "costruisci";
pub const ESEGUI: &str = "esegui";
pub const INTERROGA: &str = "interroga";
