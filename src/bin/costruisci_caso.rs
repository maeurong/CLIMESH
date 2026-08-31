//! Builds the whole reference case — Casa Evolutiva, Bastia Umbra — as a
//! Progetto on disk, from the `.INX` of the course material plus the values
//! transcribed in `casi/bastia/valori-di-riferimento.toml`.
//!
//! Usage: `cargo run --bin costruisci_caso -- [cartella-di-uscita] [modello.INX]`
//!
//! Nothing here is written by hand next to what it generates: the generator owns
//! the Scenari and the Periodi, and removes the ones it no longer writes, so a
//! second run leaves the same tree of files as the first. It removes them after
//! the write and never before, because a Progetto deleted by a run that then
//! fails is a Progetto nobody has any more.

use std::collections::HashSet;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use climesh::da_inx::{
    celle, centro_cella_m, maschere_superfici, progetto_da_inx, unisci_rettangoli,
};
use climesh::dominio::*;
use climesh::inx::{read_inx, Inx};
use climesh::progetto;
use climesh::specie;
use serde::Deserialize;

const RADICE: &str = env!("CARGO_MANIFEST_DIR");
const RIFERIMENTI: &str = "casi/bastia/valori-di-riferimento.toml";
const USCITA_PREDEFINITA: &str = "casi/bastia/progetto";
const NOME_PROGETTO: &str = "bastia";
/// The Scenario `LAB1.INX` holds, verified by reading the file and not deduced
/// from its name; see `[verifica_scenario]` in the reference values.
const STATO_DI_FATTO: &str = "stato-di-fatto";
const INTERVENTI: &str = "interventi";

/// The parts of `valori-di-riferimento.toml` the generator reads. Everything the
/// file holds beyond these is documentation for a reader, not input.
#[derive(Deserialize)]
struct Riferimenti {
    ingressi: Ingressi,
    punti_di_osservazione: Vec<PuntoDiRiferimento>,
    periodi: Vec<PeriodoDiRiferimento>,
}

#[derive(Deserialize)]
struct Ingressi {
    modello: PathBuf,
    meteo: PathBuf,
}

#[derive(Deserialize)]
struct PuntoDiRiferimento {
    id: u32,
    i: usize,
    j: usize,
}

#[derive(Deserialize)]
struct PeriodoDiRiferimento {
    nome: String,
    giorno: toml::value::Datetime,
    durata_ore: u32,
    direzione_vento_gradi: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let radice = Path::new(RADICE);
    let uscita = match std::env::args().nth(1) {
        Some(percorso) => PathBuf::from(percorso),
        None => radice.join(USCITA_PREDEFINITA),
    };
    let riferimenti: Riferimenti =
        toml::from_str(&std::fs::read_to_string(radice.join(RIFERIMENTI))?)?;
    let modello = match std::env::args().nth(2) {
        Some(percorso) => PathBuf::from(percorso),
        None => radice.join(&riferimenti.ingressi.modello),
    };

    if !modello.exists() {
        eprintln!(
            "{}: non c'è.\n\
             È il modello del corso, che non è ridistribuibile e resta fuori dal repository \
             (vedi «materiale università/» in .gitignore).\n\
             Rimetti il file al suo posto, oppure indicane un altro:\n    \
             cargo run --bin costruisci_caso -- {} percorso/del/modello.INX",
            modello.display(),
            uscita.display()
        );
        std::process::exit(2);
    }
    let letto = read_inx(&modello)?;

    let mut caso = progetto_da_inx(&letto, NOME_PROGETTO, STATO_DI_FATTO)?;
    let passo_m = caso.griglia.passo_m;
    caso.punti = riferimenti
        .punti_di_osservazione
        .iter()
        .map(|p| PuntoDiOsservazione {
            id: p.id,
            posizione_m: centro_cella_m(p.i, p.j, passo_m),
            etichetta: format!("punto {} — cella {};{}", p.id, p.i, p.j),
        })
        .collect();
    caso.periodi = riferimenti
        .periodi
        .iter()
        .map(|p| periodo(p, &riferimenti.ingressi.meteo))
        .collect::<Result<_, _>>()?;
    caso.scenari
        .push(interventi(&caso.scenari[0], &letto, passo_m));

    // The write validates the whole Progetto before it touches the directory, so
    // a case that does not hold together leaves the previous one where it was.
    progetto::scrivi(&uscita, &caso)?;
    togli_orfani(&uscita, "scenari", caso.scenari.iter().map(|s| &s.nome))?;
    togli_orfani(&uscita, "periodi", caso.periodi.iter().map(|p| &p.nome))?;

    println!(
        "{}: {} Scenari, {} Periodi, {} punti di osservazione",
        uscita.display(),
        caso.scenari.len(),
        caso.periodi.len(),
        caso.punti.len()
    );
    let meteo = radice.join(&riferimenti.ingressi.meteo);
    if !meteo.exists() {
        println!(
            "{}: i Periodi scritti nominano questo file, che qui non c'è. Viene dal materiale \
             del corso, che non è ridistribuibile: chi clona il repository trova il percorso e \
             non il file.",
            meteo.display()
        );
    }
    Ok(())
}

/// Removes the files of `sotto` that the Progetto no longer names.
///
/// After the write and not before, so that a run which fails leaves the previous
/// Progetto whole. Only the `.toml` files of the two directories the generator
/// writes: everything else under the Progetto, the Corse first of all, belongs to
/// whoever put it there. A removal that fails is reported and not swallowed —
/// a Scenario left behind under an old name is the orphan this exists to avoid.
fn togli_orfani<'a>(
    uscita: &Path,
    sotto: &str,
    nomi: impl Iterator<Item = &'a String>,
) -> std::io::Result<()> {
    let attesi: HashSet<String> = nomi.map(|nome| format!("{nome}.toml")).collect();
    let cartella = uscita.join(sotto);
    let voci = match std::fs::read_dir(&cartella) {
        Ok(voci) => voci,
        Err(errore) if errore.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(errore) => return Err(errore),
    };
    for voce in voci {
        let percorso = voce?.path();
        let nome = percorso
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        if percorso.is_file() && nome.ends_with(".toml") && !attesi.contains(&nome) {
            std::fs::remove_file(&percorso)?;
        }
    }
    Ok(())
}

/// The forcing lives in the `.SIMX` file, which the course material does not
/// have, so a Periodo comes from the values transcribed from the report.
fn periodo(r: &PeriodoDiRiferimento, meteo: &Path) -> Result<Periodo, String> {
    let giorno = r
        .giorno
        .date
        .ok_or_else(|| format!("periodo «{}»: «giorno» non è una data", r.nome))?;
    Ok(Periodo {
        nome: r.nome.clone(),
        meteo: meteo.to_path_buf(),
        ore: r.durata_ore,
        direzione_vento_gradi: Some(r.direzione_vento_gradi),
        inizio: Data {
            anno: i32::from(giorno.year),
            mese: u32::from(giorno.month),
            giorno: u32::from(giorno.day),
        },
    })
}

/// The pool of the reconstructed Scenario, in ENVI-met cell indices.
///
/// Written down and not derived. In `LAB1.INX` the two six-metre duplexes stand
/// at i 11-23 and i 32-44 over j 26-32, and the courtyard between them is free of
/// buildings from i 24 to i 31; its northern rows j 29-32 are already planted, so
/// what is left is j 26-28. A model of another shape has no courtyard at those
/// indices, so cells outside the grid, built on, or already planted are dropped,
/// and a model with none of them free gets no pool at all.
const VASCA_I: RangeInclusive<usize> = 24..=31;
const VASCA_J: RangeInclusive<usize> = 26..=28;

/// The species the reconstruction plants. The report calls them H2 along the
/// boundary and H4 against the buildings, but the `plantID` behind either code is
/// in the ENVI-met plant database, which the course material does not have.
/// Inventing `0000H2` would put a six-character identifier that looks like an
/// ENVI-met one into a Progetto somebody may publish, so the reconstruction uses
/// two species the model itself contains and says, in the Provenienza of the
/// Scenario, that they stand in for the report's.
const SIEPE_DI_CONFINE: &str = "020060";
const VERDE_ADDOSSATO: &str = "0000PR";

/// The Scenario of the proposed mitigations, rebuilt from the description in the
/// report — hedges along the boundary, greenery against the buildings, a pool in
/// the courtyard — because its own `.INX` is not in the course material.
///
/// Rebuilt and not surveyed: it exists to measure how much work a second Scenario
/// costs the engine, not to publish a result, and the Provenienza of the Scenario
/// says so once for every object that does not say otherwise. Nothing here draws a
/// random number: the geometry of a Scenario is a discrete outcome, and two
/// machines must write the same file.
///
/// The report also puts vegetation on the roofs of the duplexes. An Albero
/// carries a position and no elevation, so that part is not expressible and is
/// left out rather than quietly placed on the ground.
fn interventi(fatto: &Scenario, letto: &Inx, passo_m: f64) -> Scenario {
    let (nx, ny) = (letto.geometry.grids_i, letto.geometry.grids_j);
    let costruita = |i: usize, j: usize| {
        letto
            .z_top
            .as_ref()
            .and_then(|z| z.at(i, j))
            .is_some_and(|&h| h > 0.0)
    };
    let radicate: HashSet<(usize, usize)> = letto.plants.iter().map(|p| (p.i, p.j)).collect();
    let libera = |i: usize, j: usize| !costruita(i, j) && !radicate.contains(&(i, j));

    let vasca: Vec<(usize, usize)> = VASCA_J
        .rev()
        .flat_map(|j| VASCA_I.map(move |i| (i, j)))
        .filter(|&(i, j)| i <= nx && j <= ny && libera(i, j))
        .collect();
    let nella_vasca = |i: usize, j: usize| vasca.contains(&(i, j));

    let mut scenario = fatto.clone();
    scenario.nome = INTERVENTI.to_owned();
    scenario.derivato_da = Some(fatto.nome.clone());
    scenario.provenienza = Provenienza {
        origine: format!(
            "ricostruito dalla descrizione della relazione, non rilevato: serve a misurare il \
             carico di calcolo, non a pubblicare un risultato. Le siepi di confine sono {} e il \
             verde addossato agli edifici è {}, due specie del modello al posto dei codici H2 e \
             H4 della relazione, il cui plantID non sta nel materiale del corso",
            nome_di(SIEPE_DI_CONFINE),
            nome_di(VERDE_ADDOSSATO)
        ),
        altezza: FonteAltezza::Predefinito,
    };
    // What was carried over from the surveyed Scenario is not reconstructed, and
    // an object that inherited its provenance has to say so here instead.
    for albero in &mut scenario.alberi {
        albero.provenienza.get_or_insert_with(|| Provenienza {
            origine: format!("rilevato: viene dallo Scenario «{}»", fatto.nome),
            altezza: fatto.provenienza.altezza,
        });
    }

    let pianta = |specie_id: &str, i: usize, j: usize| Albero {
        posizione_m: centro_cella_m(i, j, passo_m),
        specie: specie_id.to_owned(),
        altezza_m: specie::altezza_di_chioma_m(specie_id),
        frazione_tronco: specie::frazione_tronco(specie_id),
        provenienza: None,
    };
    for (i, j) in celle(nx, ny) {
        if !libera(i, j) || nella_vasca(i, j) {
            continue;
        }
        let addossata = costruita(i + 1, j)
            || costruita(i, j + 1)
            || (i > 1 && costruita(i - 1, j))
            || (j > 1 && costruita(i, j - 1));
        if i == 1 || i == nx || j == 1 || j == ny {
            scenario.alberi.push(pianta(SIEPE_DI_CONFINE, i, j));
        } else if addossata {
            scenario.alberi.push(pianta(VERDE_ADDOSSATO, i, j));
        }
    }

    if !vasca.is_empty() {
        // The pool replaces the ground under it: two Superfici over one cell would
        // leave the Derivazione to pick, and it has nothing to pick with. The cells
        // go before the rectangles are formed, because a rectangle already merged
        // over a hundred cells is no longer something a cell can be taken out of.
        let mut maschere = maschere_superfici(letto);
        for (_, maschera) in &mut maschere {
            maschera.retain(|cella| !vasca.contains(cella));
        }
        maschere.push((TipoSuperficie::Acqua, vasca));
        maschere.sort_by_key(|(tipo, _)| *tipo);
        scenario.superfici = maschere
            .into_iter()
            .filter(|(_, maschera)| !maschera.is_empty())
            .map(|(tipo, maschera)| Superficie {
                tipo,
                impronta: unisci_rettangoli(&maschera, passo_m),
            })
            .collect();
    }
    scenario
}

/// The catalogue name of a species the reconstruction uses, or its plant id where
/// the table does not know it. The table is the only source for either.
fn nome_di(plant_id: &str) -> String {
    match specie::nome(plant_id) {
        Some(nome) => format!("{plant_id} {nome}"),
        None => plant_id.to_owned(),
    }
}
