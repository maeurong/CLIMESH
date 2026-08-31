# Nucleo di calcolo — piano di implementazione

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Portare CLIMESH dal lettore `.INX` che esiste oggi a una Corsa completa sul caso di riferimento che scrive campi e Giornale, misurando il budget dei 60 secondi.

**Architecture:** Cinque strati, ciascuno verificabile senza il successivo. Gli oggetti del dominio sono la verità e stanno in un Progetto su disco; la Derivazione li trasforma nei raster co-registrati che il Motore pretende; il Motore è il nucleo Rust di `UMEP-dev/solweig`, riusato e non riscritto; il Giornale registra ingressi, scelte e verifiche. Nessuna superficie utente: questo piano si guida dai test.

**Tech Stack:** Rust 2021. Dipendenze aggiunte, tutte Rust puro, nessuna libreria nativa: `serde` con derive, `toml`, `ndarray`, `sha2`. Il Motore arriva come dipendenza git pinnata.

**Spec:** [`docs/spec.md`](../../spec.md)

## Global Constraints

- **Nessuna dipendenza nativa.** Prima di aggiungere un crate, verificare su crates.io che non tiri dietro una libreria C. Niente GDAL, PROJ, NetCDF.
- **Codice, commenti e messaggi di commit in inglese.** Documentazione e file di dati in italiano.
- **Il vocabolario di [`CONTEXT.md`](../../../CONTEXT.md) è vincolante** anche nei nomi dei tipi: `Progetto`, `Griglia`, `Scenario`, `Periodo`, `Corsa`, `Giornale`, `Edificio`, `Albero`, `Superficie`, `PuntoDiOsservazione`, `Provenienza`, `Derivazione`, `Motore`. I nomi di dominio restano in italiano; tutto il resto della lingua del codice è inglese.
- **Si disegna in oggetti, si calcola in raster** ([ADR 0001](../../adr/0001-oggetti-e-raster.md)). Nessuna funzione può prendere un raster e restituire un oggetto di dominio.
- **Le righe corrono da nord.** Riga `0` di ogni raster è la più settentrionale. `inx::Matrix::at` fa già la conversione dagli indici ENVI-met; non reimplementarla.
- **Test prima dell'implementazione.** Ogni task comincia da un test che fallisce.
- **Ambiente:** `export PATH=$HOME/.cargo/bin:$PATH`, poi `cargo test` normale. Il giro con musl che questo piano prescriveva non serve più: il linker C è installato.
- **`materiale università/` resta fuori dal repository.** I test che lo leggono si saltano da soli quando non c'è.

---

## Struttura dei file

| File | Responsabilità |
|---|---|
| `src/dominio.rs` | i tipi del dominio e nient'altro: nessuna lettura, nessun calcolo |
| `src/progetto.rs` | lettura e scrittura del Progetto su disco, in TOML |
| `src/da_inx.rs` | costruisce un Progetto da un `.INX`; unico punto che conosce entrambi |
| `src/specie.rs` | la tabella delle specie: altezza di chioma, frazione di tronco, caducità |
| `src/derivazione.rs` | oggetti in raster co-registrati |
| `src/motore.rs` | l'unico punto che conosce l'API del nucleo riusato |
| `src/giornale.rs` | il Giornale: registrazione, verifiche per Corsa, impronta |
| `src/inx.rs` | esiste già, non si tocca |

`motore.rs` è l'unico file che nomina il crate del Motore. Se un giorno il fork andasse sostituito, il resto del programma non se ne accorge.

---

### Task 1: Il Progetto su disco

I tipi del dominio, e un Progetto che si scrive e si rilegge identico.

**Files:**
- Create: `src/dominio.rs`
- Create: `src/progetto.rs`
- Modify: `src/lib.rs`
- Modify: `Cargo.toml`
- Test: `tests/progetto.rs`

**Interfaces:**
- Consumes: niente.
- Produces: `dominio::{Griglia, Scenario, Periodo, Edificio, Albero, Superficie, TipoSuperficie, PuntoDiOsservazione, Provenienza, FonteAltezza, Data, Progetto}`; `progetto::{scrivi, leggi, ProgettoError}`.

- [ ] **Step 1: Aggiungere le dipendenze**

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

Verificare che nessuna delle due tiri dipendenze native:

```bash
cargo tree -e normal | grep -iE "sys$|cc |bindgen" || echo "nessuna dipendenza nativa"
```

- [ ] **Step 2: Scrivere il test che fallisce**

`tests/progetto.rs`:

```rust
use climesh::dominio::*;
use climesh::progetto;

fn progetto_di_prova() -> Progetto {
    Progetto {
        nome: "bastia".into(),
        griglia: Griglia { nx: 50, ny: 50, passo_m: 1.0, crs: "EPSG:4326".into(),
                           origine: (12.56, 43.07), rotazione_gradi: 21.0 },
        punti: vec![PuntoDiOsservazione { id: 1, cella: (35, 14), etichetta: "corte".into() }],
        scenari: vec![Scenario {
            nome: "stato-di-fatto".into(),
            derivato_da: None,
            terreno_m: vec![0.0; 2500],
            edifici: vec![Edificio {
                celle: vec![(10, 10), (10, 11)],
                altezza_m: 6.0,
                provenienza: Provenienza { origine: "rilievo di laboratorio".into(),
                                           altezza: FonteAltezza::Rilievo },
            }],
            alberi: vec![Albero {
                cella: (5, 4), specie: "020027".into(), altezza_m: 12.0,
                frazione_tronco: 0.25,
                provenienza: Provenienza { origine: "LAB1.INX".into(),
                                           altezza: FonteAltezza::Predefinito },
            }],
            superfici: vec![Superficie { celle: vec![(0, 0)], tipo: TipoSuperficie::Erba }],
        }],
        periodi: vec![Periodo {
            nome: "luglio-2021".into(),
            meteo: "ITA_Perugia.161810_IGDG.epw".into(),
            inizio: Data { anno: 2021, mese: 7, giorno: 15 },
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

/// TOML mette ogni chiave scritta dopo una tabella dentro quella tabella. Se
/// l'ordine dei campi scivola, il round-trip smette di essere fedele: questo
/// test rilegge il testo, non solo la struttura.
#[test]
fn the_manifest_keeps_its_values_before_its_tables() {
    let dir = tempdir_di_prova("ordine-toml");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    let testo = std::fs::read_to_string(dir.join("progetto.toml")).unwrap();
    let prima_tabella = testo.find('[').unwrap_or(testo.len());
    assert!(testo.find("scenari =").unwrap() < prima_tabella,
            "i valori devono precedere le tabelle:\n{testo}");
    let periodo = std::fs::read_to_string(dir.join("periodi/luglio-2021.toml")).unwrap();
    assert!(periodo.find("ore =").unwrap() < periodo.find("[inizio]").unwrap(),
            "ore finirebbe dentro [inizio]:\n{periodo}");
}

#[test]
fn a_project_directory_without_its_manifest_names_the_missing_file() {
    let dir = tempdir_di_prova("senza-manifesto");
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(err.to_string().contains("progetto.toml"), "messaggio: {err}");
}

#[test]
fn a_scenario_referenced_by_the_manifest_but_absent_on_disk_is_named() {
    let dir = tempdir_di_prova("scenario-mancante");
    progetto::scrivi(&dir, &progetto_di_prova()).unwrap();
    std::fs::remove_file(dir.join("scenari/stato-di-fatto.toml")).unwrap();
    let err = progetto::leggi(&dir).expect_err("deve fallire");
    assert!(err.to_string().contains("stato-di-fatto"), "messaggio: {err}");
}

#[test]
fn a_grid_with_no_cells_is_rejected_rather_than_written() {
    let dir = tempdir_di_prova("griglia-vuota");
    let mut p = progetto_di_prova();
    p.griglia.nx = 0;
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(err.to_string().contains("griglia"), "messaggio: {err}");
}

#[test]
fn a_terrain_that_does_not_match_the_grid_is_rejected() {
    let dir = tempdir_di_prova("terreno-corto");
    let mut p = progetto_di_prova();
    p.scenari[0].terreno_m.truncate(10);
    let err = progetto::scrivi(&dir, &p).expect_err("deve fallire");
    assert!(err.to_string().contains("2500") && err.to_string().contains("10"),
            "il messaggio deve riportare attese e trovate: {err}");
}

/// Una cartella pulita sotto `target/`, così i test non lasciano nulla in giro
/// e non serve una dipendenza per i file temporanei.
fn tempdir_di_prova(nome: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
```

- [ ] **Step 3: Eseguire il test e verificare che fallisca**

Run: `cargo test --test progetto`
Expected: FAIL, `unresolved import climesh::dominio`.

- [ ] **Step 4: Scrivere i tipi del dominio**

`src/dominio.rs`:

```rust
//! The domain types. This module reads nothing and computes nothing.
//!
//! Names come from CONTEXT.md and are binding: the domain speaks Italian, the
//! rest of the code speaks English.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Extent, step and coordinate system, shared by every raster of the Progetto.
///
/// It lives on the Progetto and not on the Scenario: two Scenari with different
/// grids could not be compared, and comparing them is the whole point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Griglia {
    pub nx: usize,
    pub ny: usize,
    pub passo_m: f64,
    /// The coordinate system `origine` is expressed in.
    ///
    /// Known gap: `passo_m` is metres while a geographic `crs` puts `origine` in
    /// degrees, so the two do not yet belong to the same system. Nothing in this
    /// plan reprojects, so nothing breaks here; the first georeferenced export
    /// will hit it, and it is recorded as an open question in the spec.
    pub crs: String,
    /// Lower-left corner in the coordinate system named by `crs`.
    pub origine: (f64, f64),
    pub rotazione_gradi: f64,
}

impl Griglia {
    pub fn celle(&self) -> usize {
        self.nx * self.ny
    }
}

/// Where an object comes from, and which of its attributes were surveyed rather
/// than estimated. It rides on the object because it outlives any single Corsa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenienza {
    pub origine: String,
    pub altezza: FonteAltezza,
}

/// Which link of the fallback chain supplied a height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FonteAltezza {
    Rilievo,
    ModelloDiSuperficie,
    NumeroDiPiani,
    Predefinito,
}

/// Cell indices are `(row, col)` with row 0 the northernmost, matching every
/// raster in the program.
pub type Cella = (usize, usize);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edificio {
    pub celle: Vec<Cella>,
    pub altezza_m: f32,
    pub provenienza: Provenienza,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Albero {
    pub cella: Cella,
    /// ENVI-met plant id where the object came from an `.INX`, otherwise a
    /// species name from `specie.rs`.
    pub specie: String,
    pub altezza_m: f32,
    /// Trunk-zone top as a fraction of canopy height.
    pub frazione_tronco: f32,
    pub provenienza: Provenienza,
}

/// `Ord` perché la Derivazione raggruppa le celle per tipo in una `BTreeMap`:
/// un ordine stabile tiene stabile anche l'ordine in cui le Superfici finiscono
/// nel file, che il contratto di riproducibilità classifica come esito discreto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TipoSuperficie {
    Pavimentato,
    Erba,
    Acqua,
    TerrenoNudo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Superficie {
    pub celle: Vec<Cella>,
    pub tipo: TipoSuperficie,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PuntoDiOsservazione {
    pub id: u32,
    pub cella: Cella,
    pub etichetta: String,
}

/// A calendar date. A dependency for three fields would not pay for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub anno: i32,
    pub mese: u32,
    pub giorno: u32,
}

impl Data {
    /// Day of the year, 1 to 366.
    pub fn giorno_dell_anno(&self) -> u32 {
        const CUMULATI: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let bisestile = (self.anno % 4 == 0 && self.anno % 100 != 0) || self.anno % 400 == 0;
        let extra = u32::from(bisestile && self.mese > 2);
        CUMULATI[(self.mese - 1) as usize] + self.giorno + extra
    }
}

/// The place in one arrangement: everything that does not change with time.
///
/// A Scenario is self-contained. `derivato_da` records which other Scenario it
/// was created from, and stays an annotation: no live inheritance, because a
/// change to a parent must never silently change an already published result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub nome: String,
    pub derivato_da: Option<String>,
    /// Ground elevation per cell, in written order, row 0 northernmost.
    pub terreno_m: Vec<f32>,
    pub edifici: Vec<Edificio>,
    pub alberi: Vec<Albero>,
    pub superfici: Vec<Superficie>,
}

/// The weather file plus the date range and forcing parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Periodo {
    pub nome: String,
    pub meteo: PathBuf,
    pub ore: u32,
    pub direzione_vento_gradi: Option<f64>,
    /// Ultimo perché `Data` serializza come tabella, e in TOML ogni chiave
    /// scritta dopo una tabella finisce dentro quella tabella.
    pub inizio: Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Progetto {
    pub nome: String,
    pub griglia: Griglia,
    pub punti: Vec<PuntoDiOsservazione>,
    pub scenari: Vec<Scenario>,
    pub periodi: Vec<Periodo>,
}
```

- [ ] **Step 5: Scrivere lettura e scrittura**

`src/progetto.rs`:

```rust
//! The Progetto on disk: a directory whose manifest is the truth.
//!
//! Layout:
//! ```text
//! progetto/
//! ├── progetto.toml     grid, observation points, scenario and period names
//! ├── scenari/<nome>.toml
//! └── periodi/<nome>.toml
//! ```
//! Scenarios and periods live in their own files because a scenario carries a
//! cell-sized terrain array and hundreds of objects: keeping them in the manifest
//! would make the one file a reader is meant to skim unreadable.

use crate::dominio::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ProgettoError {
    Io { percorso: PathBuf, causa: std::io::Error },
    Sintassi { percorso: PathBuf, causa: String },
    Griglia(String),
    Terreno { scenario: String, attese: usize, trovate: usize },
}

impl fmt::Display for ProgettoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { percorso, causa } => write!(f, "{}: {causa}", percorso.display()),
            Self::Sintassi { percorso, causa } => write!(f, "{}: {causa}", percorso.display()),
            Self::Griglia(msg) => write!(f, "griglia non valida: {msg}"),
            Self::Terreno { scenario, attese, trovate } => write!(
                f,
                "scenario {scenario}: il terreno ha {trovate} celle, la griglia ne vuole {attese}"
            ),
        }
    }
}

impl std::error::Error for ProgettoError {}

/// The manifest: everything but the scenarios and periods themselves.
#[derive(Serialize, Deserialize)]
struct Manifesto {
    nome: String,
    /// I nomi prima delle tabelle: vedi la nota su `Periodo::inizio`.
    scenari: Vec<String>,
    periodi: Vec<String>,
    griglia: Griglia,
    punti: Vec<PuntoDiOsservazione>,
}

fn valida(p: &Progetto) -> Result<(), ProgettoError> {
    if p.griglia.nx == 0 || p.griglia.ny == 0 {
        return Err(ProgettoError::Griglia("nx e ny devono essere maggiori di zero".into()));
    }
    for s in &p.scenari {
        if s.terreno_m.len() != p.griglia.celle() {
            return Err(ProgettoError::Terreno {
                scenario: s.nome.clone(),
                attese: p.griglia.celle(),
                trovate: s.terreno_m.len(),
            });
        }
    }
    Ok(())
}

fn scrivi_toml<T: Serialize>(percorso: &Path, valore: &T) -> Result<(), ProgettoError> {
    let testo = toml::to_string_pretty(valore)
        .map_err(|e| ProgettoError::Sintassi { percorso: percorso.into(), causa: e.to_string() })?;
    if let Some(genitore) = percorso.parent() {
        std::fs::create_dir_all(genitore)
            .map_err(|e| ProgettoError::Io { percorso: genitore.into(), causa: e })?;
    }
    std::fs::write(percorso, testo)
        .map_err(|e| ProgettoError::Io { percorso: percorso.into(), causa: e })
}

fn leggi_toml<T: for<'a> Deserialize<'a>>(percorso: &Path) -> Result<T, ProgettoError> {
    let testo = std::fs::read_to_string(percorso)
        .map_err(|e| ProgettoError::Io { percorso: percorso.into(), causa: e })?;
    toml::from_str(&testo)
        .map_err(|e| ProgettoError::Sintassi { percorso: percorso.into(), causa: e.to_string() })
}

pub fn scrivi(dir: impl AsRef<Path>, p: &Progetto) -> Result<(), ProgettoError> {
    valida(p)?;
    let dir = dir.as_ref();
    let manifesto = Manifesto {
        nome: p.nome.clone(),
        scenari: p.scenari.iter().map(|s| s.nome.clone()).collect(),
        periodi: p.periodi.iter().map(|x| x.nome.clone()).collect(),
        griglia: p.griglia.clone(),
        punti: p.punti.clone(),
    };
    scrivi_toml(&dir.join("progetto.toml"), &manifesto)?;
    for s in &p.scenari {
        scrivi_toml(&dir.join("scenari").join(format!("{}.toml", s.nome)), s)?;
    }
    for x in &p.periodi {
        scrivi_toml(&dir.join("periodi").join(format!("{}.toml", x.nome)), x)?;
    }
    Ok(())
}

pub fn leggi(dir: impl AsRef<Path>) -> Result<Progetto, ProgettoError> {
    let dir = dir.as_ref();
    let manifesto: Manifesto = leggi_toml(&dir.join("progetto.toml"))?;
    let mut scenari = Vec::with_capacity(manifesto.scenari.len());
    for nome in &manifesto.scenari {
        scenari.push(leggi_toml(&dir.join("scenari").join(format!("{nome}.toml")))?);
    }
    let mut periodi = Vec::with_capacity(manifesto.periodi.len());
    for nome in &manifesto.periodi {
        periodi.push(leggi_toml(&dir.join("periodi").join(format!("{nome}.toml")))?);
    }
    let p = Progetto {
        nome: manifesto.nome,
        griglia: manifesto.griglia,
        punti: manifesto.punti,
        scenari,
        periodi,
    };
    valida(&p)?;
    Ok(p)
}
```

`src/lib.rs`:

```rust
//! CLIMESH: simulazione del microclima urbano per il comfort termico all'aperto.

pub mod dominio;
pub mod inx;
pub mod progetto;
```

- [ ] **Step 6: Eseguire i test e verificare che passino**

Run: `cargo test --test progetto`
Expected: PASS, 5 test.

Poi `cargo clippy --all-targets -- -D warnings` e `cargo fmt --check`.

- [ ] **Step 7: Commit**

```bash
git add src/dominio.rs src/progetto.rs src/lib.rs Cargo.toml Cargo.lock tests/progetto.rs
git commit -m "feat: the Progetto on disk, with the manifest as the truth"
```

---

### Task 2: Dal `.INX` al Progetto

Il lettore che esiste smette di essere un lettore di file e diventa un costruttore di Progetto.

> **Scritto dopo Task 1**, contro i tipi che esistono davvero: `Edificio.impronta: Vec<Rettangolo>` e `Superficie.impronta` in metri dall'origine della Griglia, `Albero.posizione_m` e `PuntoDiOsservazione.posizione_m` come `Posizione = (f64, f64)`. Non esiste più nessun `Cella`. Leggi `src/dominio.rs` prima di scrivere: è la fonte, non questo file.

**Files:**
- Create: `src/specie.rs`
- Create: `src/da_inx.rs`
- Create: `src/bin/costruisci_caso.rs`
- Modify: `src/lib.rs`
- Test: `tests/da_inx.rs`

**Interfaces:**
- Consumes: `dominio::*`, `inx::{read_inx, Inx, Matrix, Plant, Geometry}`, `progetto::scrivi`.
- Produces: `da_inx::progetto_da_inx(letto: &Inx, nome_progetto: &str, nome_scenario: &str) -> Progetto`; `specie::{nome, altezza_di_chioma_m, frazione_tronco, e_decidua, ALTEZZA_PREDEFINITA_M, FRAZIONE_TRONCO_PREDEFINITA}`.

**La conversione che decide tutto.** Il `.INX` parla in celle 1-based con `j = 1` a sud; il dominio parla in metri dall'angolo sud-ovest della Griglia. La cella `(i, j)` copre il rettangolo che va da `((i-1)·passo, (j-1)·passo)` a `(i·passo, j·passo)`. Un albero radicato in `(i, j)` sta al **centro** della sua cella, cioè `((i-0.5)·passo, (j-0.5)·passo)`.

Attenzione: `inx::Matrix::at(i, j)` prende già indici ENVI-met e fa il ribaltamento nord-sud. **Non reimplementarlo**, e non indicizzare `cells` a mano se non ti serve davvero l'ordine di scrittura.

**Cosa deve essere vero alla fine:**

1. **La tabella delle specie** copre le cinque del caso — `020027` Pine Tree (middle), `020060` London Plane Tree (middle), `0000PR` Tilia, `0000PA` Populus Alba, `020111` Hanging Birch (middle) — con altezza di chioma, frazione di tronco e caducità. I nomi vengono da `casi/bastia/valori-di-riferimento.toml`, che li ha letti dal file vero: un nome inventato qui diventa un nome sbagliato in una relazione. Platano e tiglio sono decidui, il pino no. Una specie sconosciuta prende i valori predefiniti e **non** è decidua, perché togliere ombra che il modello non sa giustificare è l'errore peggiore.
2. **Le altezze degli Edifici sono `FonteAltezza::Rilievo`**: vengono da una matrice del file. **Le altezze degli Alberi sono `FonteAltezza::Predefinito`**: non stanno nel `.INX`, stanno nel database piante di ENVI-met, che non abbiamo.
3. **Il nome del Progetto è un parametro**, non `location.name`: nel caso di riferimento quel campo dice `bergamo` per un modello di Bastia Umbra, refuso noto e registrato fra le incongruenze.
4. **Le celle costruite che condividono l'altezza formano un Edificio.** Il campo che numera gli edifici nel `.INX` spezza tre blocchi costruiti in sette identificativi, alcuni di una cella sola: raggrupparle per altezza è più vicino a ciò che un lettore chiama "un edificio".
5. **Il generatore `costruisci_caso` produce l'intero caso**, Periodi compresi, ed è **idempotente**: due esecuzioni lasciano lo stesso albero di file. Niente file scritti a mano accanto a file generati, perché la seconda esecuzione li cancellerebbe — e i Periodi sono ciò che il cancello di Task 5 legge.
6. **Il secondo Scenario è ricostruito, non rilevato.** `LAB1.INX` contiene solo lo stato di fatto. Quello degli interventi si costruisce dalla descrizione della relazione — filari sul confine, verde addossato agli edifici, specchio d'acqua nella corte — **in modo deterministico, senza generatori casuali**, perché la geometria di uno Scenario è un esito discreto e due macchine devono produrre lo stesso file. Ogni oggetto aggiunto porta in Provenienza che è ricostruito dalla descrizione e non rilevato. Serve a misurare il carico di calcolo, non a pubblicare risultati, e il codice lo deve dire.
7. **I Periodi** vengono da `casi/bastia/valori-di-riferimento.toml`: 15/07/2021 e 15/01/2021, 48 ore, direzione del vento 45° d'estate e 180° d'inverno, meteo l'EPW di Perugia. Non stanno nel `.INX` perché il forcing vive nel `.SIMX`, che non è nel materiale.
8. **I tre Punti di osservazione** della relazione, in indici ENVI-met `(15;18)`, `(25;35)`, `(37;15)`, convertiti in metri con la stessa regola del centro cella.
9. `casi/bastia/progetto/corse/` finisce in `.gitignore`: nasce dall'esecuzione.

## Ingressi degeneri

Ogni riga è un test, e ogni test deve fallire se togli la difesa:

- `.INX` senza la matrice degli edifici → Scenario senza Edifici, mai un panico e mai un edificio a quota zero
- `.INX` senza la sezione dei suoli → nessuna Superficie, che non è la stessa cosa di un dominio tutto di terreno nudo
- pianta con `plantID` sconosciuto → altezza e tronco predefiniti, non decidua, e la Provenienza lo dice
- pianta radicata fuori dalla griglia dichiarata → segnalata, mai una posizione fuori dall'estensione che poi `progetto::valida` rifiuta
- `.INX` con `dx` diverso da `dy` → o è supportato davvero, o è rifiutato con un messaggio che lo dice; mai una conversione che assume l'uno per l'altro
- `.INX` di dimensione diversa da 50×50 → convertito lo stesso, nessuna dimensione cablata
- generatore eseguito due volte di fila → stesso albero di file, byte per byte
- generatore eseguito senza `materiale università/` → errore che nomina il file e spiega che non è ridistribuibile, codice d'uscita diverso da zero, nessun Progetto a metà
- Progetto generato → `progetto::leggi` lo rilegge senza errori, e i due Scenari hanno nomi diversi

**Accettazione:** `cargo run --bin costruisci_caso` scrive `casi/bastia/progetto` con due Scenari e due Periodi; `cargo test` resta verde compresi i 18 test di `inx` e i 31 di `progetto`; `clippy` e `fmt` puliti. I test che leggono il materiale del corso si saltano da soli quando manca, stampando il motivo.

---

### Task 3: La Derivazione

> ⚠️ **Il codice di questo task precede il cambio di modello e non compila più.**
> Gli oggetti non hanno più indici di cella: `Edificio.impronta` e
> `Superficie.impronta` sono `Vec<Rettangolo>` in metri, `Albero.posizione_m` e
> `PuntoDiOsservazione.posizione_m` sono `Posizione`, `Cella` non esiste,
> `Griglia::celle()` restituisce `Option<usize>` e `Data::giorno_dell_anno()`
> pure. Leggi i tipi in `src/dominio.rs`, che è la fonte. Il testo qui sotto
> resta valido per **intento, interfacce e ingressi degeneri**; il codice va
> riscritto al momento del dispaccio, come è stato fatto per Task 2.

Da oggetti a raster co-registrati. Verificabile per intero senza il Motore.

**Files:**
- Create: `src/derivazione.rs`
- Modify: `src/lib.rs`, `Cargo.toml`
- Test: `tests/derivazione.rs`

**Interfaces:**
- Consumes: `dominio::{Progetto, Scenario, Periodo, Griglia}`, `specie::*`.
- Produces: `derivazione::{Raster, Raster2D, RasterDiScenario, deriva, Stagione, ScelteDiDerivazione}`.

- [ ] **Step 1: Aggiungere `ndarray`**

```toml
ndarray = "0.16"
```

Versione confermata: il Motore dichiara `ndarray 0.16.1` con feature `rayon` in `rust/Cargo.toml:12`. **Non salire alla 0.17**, che esiste ed è semver-incompatibile: sarebbero due tipi diversi e `ArrayView2<f32>` non combacerebbe.

`ndarray` è Rust puro e produce `Array2<f32>`, che è il tipo che il Motore accetta senza conversioni.

**La versione non era una nostra scelta ed è già stata accertata.** Quanto segue resta come traccia del perché: `ndarray` non ha ancora la 1.0, quindi due versioni minori diverse sono due tipi diversi e `ArrayView2<f32>` non combacia. Prima di fissare questa riga, leggere la versione dichiarata dal Motore — è il dossier che il ticket di ricerca produce prima di ogni altro task:

```bash
curl -sL https://raw.githubusercontent.com/UMEP-dev/solweig/main/rust/Cargo.toml | grep -i ndarray
```

Pinnare la stessa versione minore. Se differisce da `0.16`, correggere qui e annotarlo, perché è un vincolo che arriva da fuori e non si vede nel codice.

- [ ] **Step 2: Scrivere il test che fallisce**

`tests/derivazione.rs`:

```rust
use climesh::derivazione::{self, Stagione};
use climesh::dominio::*;

fn scenario_di_prova() -> (Griglia, Scenario) {
    let griglia = Griglia { nx: 4, ny: 4, passo_m: 1.0, crs: "EPSG:4326".into(),
                            origine: (0.0, 0.0), rotazione_gradi: 0.0 };
    let scenario = Scenario {
        nome: "prova".into(),
        derivato_da: None,
        terreno_m: vec![2.0; 16],
        edifici: vec![Edificio {
            celle: vec![(1, 1)],
            altezza_m: 6.0,
            provenienza: Provenienza { origine: "test".into(), altezza: FonteAltezza::Rilievo },
        }],
        alberi: vec![
            Albero { cella: (0, 0), specie: "020027".into(), altezza_m: 12.0,
                     frazione_tronco: 0.45,
                     provenienza: Provenienza { origine: "test".into(),
                                                altezza: FonteAltezza::Predefinito } },
            Albero { cella: (0, 1), specie: "020060".into(), altezza_m: 15.0,
                     frazione_tronco: 0.30,
                     provenienza: Provenienza { origine: "test".into(),
                                                altezza: FonteAltezza::Predefinito } },
        ],
        superfici: vec![Superficie { celle: vec![(3, 3)], tipo: TipoSuperficie::Acqua }],
    };
    (griglia, scenario)
}

#[test]
fn the_surface_model_is_ground_plus_buildings() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.superficie[[1, 1]], 8.0, "2 m di terreno più 6 m di edificio");
    assert_eq!(r.superficie[[2, 2]], 2.0, "dove non c'è edificio resta il terreno");
    assert_eq!(r.terreno[[1, 1]], 2.0, "il modello di terreno ignora gli edifici");
}

#[test]
fn canopy_height_is_measured_from_the_ground_it_stands_on() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.chiome[[0, 0]], 14.0, "2 m di terreno più 12 m di chioma");
    assert!((r.tronchi[[0, 0]] - (2.0 + 12.0 * 0.45)).abs() < 1e-5);
}

#[test]
fn the_winter_derivation_drops_deciduous_trees_and_keeps_evergreens() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::SenzaFoglie);
    assert_eq!(r.chiome[[0, 0]], 14.0, "il pino resta");
    assert_eq!(r.chiome[[0, 1]], 0.0, "il platano esce dal raster");
    assert_eq!(r.scelte.chiome_escluse, 1);
}

#[test]
fn the_summer_derivation_drops_nothing() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.scelte.chiome_escluse, 0);
}

#[test]
fn a_terrain_of_the_wrong_shape_is_recorded_rather_than_silently_flattened() {
    let (g, mut s) = scenario_di_prova();
    s.terreno_m.truncate(3);
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert!(r.scelte.terreno_sostituito, "la sostituzione va nel Giornale");
    assert_eq!(r.terreno.dim(), (g.ny, g.nx));
}

#[test]
fn every_raster_has_the_shape_of_the_grid() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    for (nome, a) in [("superficie", &r.superficie), ("terreno", &r.terreno),
                      ("chiome", &r.chiome), ("tronchi", &r.tronchi)] {
        assert_eq!(a.dim(), (g.ny, g.nx), "raster {nome} fuori forma");
    }
    assert_eq!(r.classi.dim(), (g.ny, g.nx));
}

#[test]
fn surface_classes_land_where_the_objects_are() {
    let (g, s) = scenario_di_prova();
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.classi[[3, 3]], derivazione::CLASSE_ACQUA);
    assert_eq!(r.classi[[0, 0]], derivazione::CLASSE_PREDEFINITA);
}

#[test]
fn an_object_outside_the_grid_is_ignored_rather_than_panicking() {
    let (g, mut s) = scenario_di_prova();
    s.alberi.push(Albero { cella: (99, 99), specie: "020027".into(), altezza_m: 12.0,
                           frazione_tronco: 0.45,
                           provenienza: Provenienza { origine: "test".into(),
                                                      altezza: FonteAltezza::Predefinito } });
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.scelte.oggetti_fuori_griglia, 1);
}

#[test]
fn two_trees_on_one_cell_leave_the_taller_canopy() {
    let (g, mut s) = scenario_di_prova();
    s.alberi.push(Albero { cella: (0, 0), specie: "0000PA".into(), altezza_m: 8.0,
                           frazione_tronco: 0.25,
                           provenienza: Provenienza { origine: "test".into(),
                                                      altezza: FonteAltezza::Predefinito } });
    let r = derivazione::deriva(&g, &s, Stagione::ConFoglie);
    assert_eq!(r.chiome[[0, 0]], 14.0, "vince la chioma più alta");
}
```

- [ ] **Step 3: Eseguire il test e verificare che fallisca**

Run: `cargo test --test derivazione`
Expected: FAIL, `unresolved import climesh::derivazione`.

- [ ] **Step 4: Scrivere la Derivazione**

`src/derivazione.rs`:

```rust
//! Objects into the co-registered rasters the Motore consumes.
//!
//! This is the step ADR 0001 exists for: the engine takes a single leaf
//! transmissivity per Corsa, but species lives on the Albero, so a leaf-off
//! Periodo drops deciduous canopies from the raster instead. That distinction
//! would not even be expressible with rasters as the truth.

use crate::dominio::*;
use crate::specie;
use ndarray::Array2;

pub type Raster = Array2<f32>;

/// Surface class codes. The list is closed because the engine only accepts known
/// classes, and a closed list lets CLIMESH say so before the computation rather than after.
pub const CLASSE_PREDEFINITA: u8 = 1;
pub const CLASSE_PAVIMENTATO: u8 = 2;
pub const CLASSE_ERBA: u8 = 5;
pub const CLASSE_ACQUA: u8 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stagione {
    ConFoglie,
    SenzaFoglie,
}

/// What the Derivazione decided, for the Giornale to record.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScelteDiDerivazione {
    pub chiome_escluse: usize,
    pub oggetti_fuori_griglia: usize,
    pub celle_costruite: usize,
    pub celle_con_chioma: usize,
    /// Set when the Scenario's terrain did not match the Griglia and a flat
    /// ground was used instead. A modelling choice taken by the program is a
    /// thing the Giornale records, never a thing it swallows.
    pub terreno_sostituito: bool,
}

pub struct RasterDiScenario {
    /// Ground plus buildings.
    pub superficie: Raster,
    /// Ground only.
    pub terreno: Raster,
    /// Canopy top, absolute. Zero where there is no canopy.
    pub chiome: Raster,
    /// Trunk-zone top, absolute. Zero where there is no canopy.
    pub tronchi: Raster,
    pub classi: Array2<u8>,
    pub scelte: ScelteDiDerivazione,
}

fn classe_di(t: TipoSuperficie) -> u8 {
    match t {
        TipoSuperficie::Pavimentato => CLASSE_PAVIMENTATO,
        TipoSuperficie::Erba => CLASSE_ERBA,
        TipoSuperficie::Acqua => CLASSE_ACQUA,
        TipoSuperficie::TerrenoNudo => CLASSE_PREDEFINITA,
    }
}

pub fn deriva(g: &Griglia, s: &Scenario, stagione: Stagione) -> RasterDiScenario {
    let forma = (g.ny, g.nx);
    let mut scelte = ScelteDiDerivazione::default();

    let terreno = match Array2::from_shape_vec(forma, s.terreno_m.clone()) {
        Ok(a) => a,
        Err(_) => {
            scelte.terreno_sostituito = true;
            Array2::zeros(forma)
        }
    };
    let mut superficie = terreno.clone();
    let mut chiome = Array2::zeros(forma);
    let mut tronchi = Array2::zeros(forma);
    let mut classi = Array2::from_elem(forma, CLASSE_PREDEFINITA);

    let dentro = |(r, c): Cella| r < g.ny && c < g.nx;

    for e in &s.edifici {
        for &cella in &e.celle {
            if !dentro(cella) {
                scelte.oggetti_fuori_griglia += 1;
                continue;
            }
            superficie[[cella.0, cella.1]] = terreno[[cella.0, cella.1]] + e.altezza_m;
            scelte.celle_costruite += 1;
        }
    }

    for a in &s.alberi {
        if !dentro(a.cella) {
            scelte.oggetti_fuori_griglia += 1;
            continue;
        }
        if stagione == Stagione::SenzaFoglie && specie::e_decidua(&a.specie) {
            scelte.chiome_escluse += 1;
            continue;
        }
        let base = terreno[[a.cella.0, a.cella.1]];
        let cima = base + a.altezza_m;
        if cima > chiome[[a.cella.0, a.cella.1]] {
            chiome[[a.cella.0, a.cella.1]] = cima;
            tronchi[[a.cella.0, a.cella.1]] = base + a.altezza_m * a.frazione_tronco;
        }
    }
    scelte.celle_con_chioma = chiome.iter().filter(|&&v| v > 0.0).count();

    for sup in &s.superfici {
        for &cella in &sup.celle {
            if !dentro(cella) {
                scelte.oggetti_fuori_griglia += 1;
                continue;
            }
            classi[[cella.0, cella.1]] = classe_di(sup.tipo);
        }
    }

    RasterDiScenario { superficie, terreno, chiome, tronchi, classi, scelte }
}
```

- [ ] **Step 5: Eseguire i test e verificare che passino**

Run: `cargo test --test derivazione`
Expected: PASS, 8 test. Poi `cargo clippy --all-targets -- -D warnings`.

- [ ] **Step 6: Commit**

```bash
git add src/derivazione.rs src/lib.rs Cargo.toml Cargo.lock tests/derivazione.rs
git commit -m "feat: derive co-registered rasters from the objects of a Scenario"
```

---

### Task 4: Il Motore, primo collegamento

> ⚠️ **Il codice di questo task precede il cambio di modello e non compila più.**
> Gli oggetti non hanno più indici di cella: `Edificio.impronta` e
> `Superficie.impronta` sono `Vec<Rettangolo>` in metri, `Albero.posizione_m` e
> `PuntoDiOsservazione.posizione_m` sono `Posizione`, `Cella` non esiste,
> `Griglia::celle()` restituisce `Option<usize>` e `Data::giorno_dell_anno()`
> pure. Leggi i tipi in `src/dominio.rs`, che è la fonte. Il testo qui sotto
> resta valido per **intento, interfacce e ingressi degeneri**; il codice va
> riscritto al momento del dispaccio, come è stato fatto per Task 2.

L'ombra viene dal nucleo riusato invece che da noi, e si verifica contro la geometria.

**Files:**
- Create: `src/motore.rs`
- Create: `src/sole.rs`
- Modify: `src/lib.rs`, `Cargo.toml`
- Test: `tests/motore.rs`

**Interfaces:**
- Consumes: `derivazione::Raster`. `RasterDiScenario` serve solo a Task 5.
- Produces: `sole::{posizione, PosizioneSolare}`; `motore::{ombre, VersioneMotore, versione}`. `ombre` restituisce la **frazione illuminata**, non l'ombra: 1 al sole, 0 in ombra, come il campo `bldg_sh` del Motore.

- [ ] **Step 1: Preparare il fork del Motore**

Il crate del Motore è dichiarato `cdylib` con funzioni `pub(crate)`: oggi nessun programma Rust può consumarlo. Serve un fork che esponga un'API nativa. La richiesta a monte si apre in parallelo, ma **questo piano non dipende dal suo esito**.

```bash
gh repo fork UMEP-dev/solweig --clone --fork-name solweig --remote=false
cd solweig && git checkout -b api-rust-nativa
```

**Il dossier `research/motore-api.md` dice cosa serve davvero, e non è solo `rlib`.** Tre ostacoli, in ordine di rischio crescente:

1. `crate-type = ["cdylib"]` in `rust/Cargo.toml` diventa `["cdylib", "rlib"]`.
2. I moduli in `rust/src/lib.rs:3-26` sono tutti `mod` privati: rendere pubblico `shadowing` e la funzione non basta se il modulo non è raggiungibile. Vanno resi `pub mod` quelli che servono, e vanno esportati i tipi che compaiono nella firma pubblica — `ShadowingResultRust` è `pub(crate)` a `shadowing.rs:151`.
3. **L'ostacolo che decide il task:** `pyo3` è dichiarato con `extension-module` **non opzionale**, e la guida di PyO3 avverte che con quella feature attiva i binari non compilano. Va resa opzionale dietro una feature, spenta di default:

```toml
[features]
default = []
python = ["pyo3/extension-module"]

[dependencies]
pyo3 = { version = "...", optional = true, default-features = false }
```

**Verificare che compili prima di andare avanti**, perché è qui che il piano scopre se l'integrazione è praticabile:

```bash
cargo build --release -p rustalgos --no-default-features
```

Se non compila, **fermarsi e riferire**: il ripiego è il vendoring dei file sorgente dentro il nostro crate, che la GPL-3 di entrambe le parti rende lecito, ed è una decisione da prendere con Mario, non da improvvisare.

La firma è nota e sta nel dossier. Verificarla comunque contro il sorgente prima di scrivere il chiamante, perché il pin potrebbe essere avanzato:

```bash
grep -n "fn calculate_shadows_rust" -A 18 rust/src/shadowing.rs
```

Poi:

```bash
cargo build --release -p rustalgos && git commit -am "expose a native Rust API" && git push -u origin api-rust-nativa
gh pr create --repo UMEP-dev/solweig --title "Expose a native Rust API alongside the Python extension" \
  --body "Adds rlib to crate-type and makes the kernel entry points public, so the crate can be consumed by other Rust programs. No behaviour change; the cdylib build is untouched."
```

- [ ] **Step 2: Scrivere il test che fallisce**

`tests/motore.rs`:

```rust
use climesh::derivazione::Raster;
use climesh::{motore, sole};
use ndarray::Array2;

/// Un cubo alto 10 m al centro di una griglia piana da 21×21 a 1 m.
fn dominio_con_una_torre() -> Raster {
    let mut a: Raster = Array2::zeros((21, 21));
    a[[10, 10]] = 10.0;
    a
}

#[test]
fn the_sun_is_up_at_noon_in_july_and_down_at_midnight() {
    let mezzogiorno = sole::posizione(196, 12.0, 43.07, 12.56, 1.0);
    assert!(mezzogiorno.altezza_gradi > 60.0, "altezza a mezzogiorno: {}", mezzogiorno.altezza_gradi);
    let notte = sole::posizione(196, 0.0, 43.07, 12.56, 1.0);
    assert!(notte.altezza_gradi < 0.0, "altezza a mezzanotte: {}", notte.altezza_gradi);
}

#[test]
fn the_sun_sits_in_the_south_at_solar_noon() {
    let p = sole::posizione(196, 13.0, 43.07, 12.56, 1.0);
    assert!((p.azimut_gradi - 180.0).abs() < 15.0, "azimut: {}", p.azimut_gradi);
}

#[test]
fn nothing_is_lit_when_the_sun_is_below_the_horizon() {
    let dsm = dominio_con_una_torre();
    let ombra = motore::ombre(&dsm, 1.0, 180.0, -5.0);
    assert!(ombra.iter().all(|&v| v == 0.0), "sotto l'orizzonte è tutto in ombra");
}

#[test]
fn a_flat_domain_under_a_high_sun_is_entirely_lit() {
    let piano: Raster = Array2::zeros((21, 21));
    let ombra = motore::ombre(&piano, 1.0, 180.0, 60.0);
    assert!(ombra.iter().all(|&v| v > 0.99), "senza ostacoli non c'è ombra");
}

/// La verifica che il gate del progetto chiede: l'ombra deve cadere dove la
/// geometria dice, confrontata con la posizione solare calcolata a parte.
#[test]
fn the_shadow_of_a_tower_falls_north_when_the_sun_is_south() {
    let dsm = dominio_con_una_torre();
    // Sole a sud, 45 gradi: una torre di 10 m proietta 10 m d'ombra verso nord.
    let ombra = motore::ombre(&dsm, 1.0, 180.0, 45.0);
    assert!(ombra[[5, 10]] < 0.5, "a 5 celle a nord deve esserci ombra: {}", ombra[[5, 10]]);
    assert!(ombra[[15, 10]] > 0.5, "a sud della torre deve esserci sole: {}", ombra[[15, 10]]);
    assert!(ombra[[10, 3]] > 0.5, "a ovest, fuori dall'ombra, deve esserci sole");
}

#[test]
fn the_engine_version_is_recorded_and_pinned() {
    let v = motore::versione();
    assert!(!v.crate_version.is_empty(), "la versione del Motore deve essere leggibile");
    assert!(!v.git_rev.is_empty(), "il tag pinnato deve essere leggibile");
}
```

- [ ] **Step 3: Eseguire il test e verificare che fallisca**

Run: `cargo test --test motore`
Expected: FAIL, `unresolved import climesh::motore`.

- [ ] **Step 4: Scrivere la posizione solare e il collegamento**

`src/sole.rs`:

```rust
//! Solar position, NOAA's approximation.
//!
//! CLIMESH computes this itself rather than asking the engine: the shadow check
//! in the Giornale compares the engine's shadow against a position derived
//! independently, and a check that shares its source with the thing it checks
//! verifies nothing.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosizioneSolare {
    /// Degrees above the horizon; negative when the sun is down.
    pub altezza_gradi: f64,
    /// Degrees clockwise from north.
    pub azimut_gradi: f64,
}

pub fn posizione(
    giorno_dell_anno: u32,
    ora_locale: f64,
    latitudine_gradi: f64,
    longitudine_gradi: f64,
    fuso_ore: f64,
) -> PosizioneSolare {
    let lat = latitudine_gradi.to_radians();
    let g = 2.0 * std::f64::consts::PI / 365.0
        * (giorno_dell_anno as f64 - 1.0 + (ora_locale - 12.0) / 24.0);

    let eqt = 229.18
        * (0.000_075 + 0.001_868 * g.cos() - 0.032_077 * g.sin()
            - 0.014_615 * (2.0 * g).cos() - 0.040_849 * (2.0 * g).sin());
    let decl = 0.006_918 - 0.399_912 * g.cos() + 0.070_257 * g.sin()
        - 0.006_758 * (2.0 * g).cos() + 0.000_907 * (2.0 * g).sin()
        - 0.002_697 * (3.0 * g).cos() + 0.001_48 * (3.0 * g).sin();

    let ora_vera = ora_locale + (eqt + 4.0 * longitudine_gradi - 60.0 * fuso_ore) / 60.0;
    let angolo_orario = ((ora_vera - 12.0) * 15.0).to_radians();

    let altezza = (lat.sin() * decl.sin() + lat.cos() * decl.cos() * angolo_orario.cos()).asin();
    let azimut = angolo_orario
        .sin()
        .atan2(angolo_orario.cos() * lat.sin() - decl.tan() * lat.cos())
        + std::f64::consts::PI;

    PosizioneSolare {
        altezza_gradi: altezza.to_degrees(),
        azimut_gradi: azimut.to_degrees().rem_euclid(360.0),
    }
}
```

`src/motore.rs`:

```rust
//! The only module that names the reused radiative kernel.
//!
//! If the fork ever has to be replaced, nothing else in the program notices.
//! The dependency is pinned to a tag: the Giornale cites the engine's published
//! validation by version, and a version that can move underfoot makes that
//! citation worthless.

use crate::derivazione::Raster;

#[derive(Debug, Clone, PartialEq)]
pub struct VersioneMotore {
    pub crate_version: String,
    pub git_rev: String,
}

pub fn versione() -> VersioneMotore {
    VersioneMotore {
        crate_version: env!("CLIMESH_MOTORE_VERSION").to_string(),
        git_rev: env!("CLIMESH_MOTORE_REV").to_string(),
    }
}

/// Sunlit fraction per cell: 1.0 in full sun, 0.0 in shade.
///
/// `azimut_gradi` runs clockwise from north, `altezza_gradi` above the horizon.
/// Below the horizon everything is shaded, which the kernel does not decide for
/// us because it is a modelling choice, not a geometric one.
pub fn ombre(dsm: &Raster, passo_m: f64, azimut_gradi: f64, altezza_gradi: f64) -> Raster {
    if altezza_gradi <= 0.0 {
        return Raster::zeros(dsm.dim());
    }
    // `max_local_dsm_ht` è il **rilievo** del modello di superficie, massimo meno
    // minimo, non il massimo: è la quota di cui il raggio deve salire prima di
    // essere sicuramente sopra ogni ostacolo. Passare il massimo su un dominio
    // con terreno a quota non nulla marcia più del necessario.
    let massimo = dsm.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let minimo = dsm.iter().copied().fold(f32::INFINITY, f32::min);
    let rilievo = massimo - minimo;

    // L'azimut del nucleo è gradi da nord in senso orario, come il nostro, e la
    // riga 0 è a nord in entrambi: nessuna conversione. Verificato nel dossier
    // `research/motore-api.md` § 2 su spec e docstring del Motore.
    let esito = rustalgos::shadowing::calculate_shadows_rust(
        azimut_gradi as f32,
        altezza_gradi as f32,
        passo_m as f32,
        rilievo,
        dsm.view(),
        None, // chiome
        None, // tronchi
        None, // cespugli
        None, // muri
        None, // esposizioni
        None, // schema muri
        None, // schema esposizioni
        false, // uscite complete sui muri: non servono per le ombre
        0.0,   // altezza solare minima: la soglia la decidiamo noi, sopra
        f32::INFINITY, // distanza massima d'ombra: nessun taglio
    );

    // `bldg_sh` vale 1 dove c'è sole: è già la frazione illuminata che il resto
    // del programma si aspetta, non l'ombra.
    esito.bldg_sh
}
```

Il numero e l'ordine dei parametri vengono da `research/motore-api.md` § 1, letti nel sorgente del Motore al commit `02246ab7`. Se il pin che si finisce per usare differisce, adattare **solo** questa chiamata: è l'unico punto del programma che conosce quella firma.

`Cargo.toml` — **aggiungere** questa riga alla sezione `[dependencies]` esistente, che a questo punto contiene già `serde`, `toml` e `ndarray`. Non sostituire la sezione:

```toml
rustalgos = { git = "https://github.com/maeurong/solweig", tag = "api-rust-nativa-v1", package = "rustalgos" }
```

`build.rs`, per rendere la versione del Motore leggibile a compilazione:

```rust
fn main() {
    // La versione e il rev pinnati finiscono nel Giornale. Si leggono qui perché
    // a runtime il lock file non è disponibile.
    let lock = std::fs::read_to_string("Cargo.lock").unwrap_or_default();
    let mut versione = String::from("sconosciuta");
    let mut rev = String::from("sconosciuto");
    let mut dentro = false;
    for riga in lock.lines() {
        if riga.starts_with("name = \"rustalgos\"") {
            dentro = true;
        } else if dentro && riga.starts_with("version = ") {
            versione = riga.trim_start_matches("version = ").trim_matches('"').to_string();
        } else if dentro && riga.starts_with("source = ") {
            rev = riga.rsplit('#').next().unwrap_or("sconosciuto").trim_matches('"').to_string();
            break;
        }
    }
    println!("cargo:rustc-env=CLIMESH_MOTORE_VERSION={versione}");
    println!("cargo:rustc-env=CLIMESH_MOTORE_REV={rev}");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
```

- [ ] **Step 5: Eseguire i test e verificare che passino**

Run: `cargo test --test motore`
Expected: PASS, 6 test.

Se `the_shadow_of_a_tower_falls_north_when_the_sun_is_south` fallisce, la convenzione di azimut del nucleo non coincide con la nostra: **non ruotare il test**, correggere la conversione in `motore::ombre` e annotare la convenzione trovata nel commento del modulo. Il test dice il fatto geometrico, che non è negoziabile.

- [ ] **Step 6: Commit**

```bash
git add src/motore.rs src/sole.rs src/lib.rs build.rs Cargo.toml Cargo.lock tests/motore.rs
git commit -m "feat: wire the reused radiative kernel and verify its shadows"
```

---

### Task 5: Il Giornale e il cancello dei 60 secondi

> ⚠️ **Il codice di questo task precede il cambio di modello e non compila più.**
> Gli oggetti non hanno più indici di cella: `Edificio.impronta` e
> `Superficie.impronta` sono `Vec<Rettangolo>` in metri, `Albero.posizione_m` e
> `PuntoDiOsservazione.posizione_m` sono `Posizione`, `Cella` non esiste,
> `Griglia::celle()` restituisce `Option<usize>` e `Data::giorno_dell_anno()`
> pure. Leggi i tipi in `src/dominio.rs`, che è la fonte. Il testo qui sotto
> resta valido per **intento, interfacce e ingressi degeneri**; il codice va
> riscritto al momento del dispaccio, come è stato fatto per Task 2.

La prima Corsa completa sul caso di riferimento, con il suo Giornale, cronometrata.

**Files:**
- Create: `src/giornale.rs`
- Create: `src/corsa.rs`
- Modify: `src/lib.rs`, `Cargo.toml`
- Test: `tests/corsa.rs`

**Interfaces:**
- Consumes: tutto il precedente.
- Produces: `corsa::{esegui, esegui_caso_di_riferimento, Campi, Esito}`; `giornale::{Giornale, Impronta, Ingresso, Inviluppo, ConteggioProvenienza, conta_provenienza, inviluppo}`.

- [ ] **Step 1: Aggiungere `sha2`**

```toml
sha2 = "0.10"
```

Serve per l'impronta, che è calcolata dal contenuto: due Corse con la stessa impronta *sono* la stessa Corsa.

- [ ] **Step 2: Scrivere il test che fallisce**

`tests/corsa.rs`:

```rust
use climesh::dominio::*;
use climesh::{corsa, giornale};

#[test]
fn the_fingerprint_is_the_same_for_the_same_inputs_and_differs_otherwise() {
    let ing = |nome: &str, somma: &str| giornale::Ingresso {
        nome: nome.into(), somma: somma.into(),
    };
    let luglio = [ing("progetto", "aa"), ing("luglio", "bb")];
    let gennaio = [ing("progetto", "aa"), ing("gennaio", "bb")];
    let a = giornale::Impronta::calcola(&luglio, "1.0", "abc123");
    let b = giornale::Impronta::calcola(&luglio, "1.0", "abc123");
    let c = giornale::Impronta::calcola(&gennaio, "1.0", "abc123");
    let d = giornale::Impronta::calcola(&luglio, "1.1", "abc123");
    assert_eq!(a, b, "stessi ingressi, stessa impronta");
    assert_ne!(a, c, "un Periodo diverso è una Corsa diversa");
    assert_ne!(a, d, "una versione diversa del binario è una Corsa diversa");
    assert_eq!(a.corta().len(), 12, "l'impronta corta si legge e si incolla");

    // Lo stesso nome con contenuto diverso è una Corsa diversa: è il caso che
    // le sole stringhe non distinguerebbero.
    let modificato = [ing("progetto", "aa"), ing("luglio", "ZZ")];
    assert_ne!(a, giornale::Impronta::calcola(&modificato, "1.0", "abc123"),
               "un ingresso modificato cambia l'Impronta");
}

#[test]
fn the_journal_counts_provenance_by_link_of_the_chain() {
    let scenario = Scenario {
        nome: "misto".into(), derivato_da: None, terreno_m: vec![0.0; 4],
        edifici: vec![
            edificio_con(FonteAltezza::Rilievo),
            edificio_con(FonteAltezza::NumeroDiPiani),
            edificio_con(FonteAltezza::NumeroDiPiani),
            edificio_con(FonteAltezza::Predefinito),
        ],
        alberi: vec![], superfici: vec![],
    };
    let conteggio = giornale::conta_provenienza(&scenario);
    assert_eq!(conteggio.rilievo, 1);
    assert_eq!(conteggio.numero_di_piani, 2);
    assert_eq!(conteggio.predefinito, 1);
    assert_eq!(conteggio.modello_di_superficie, 0);
}

#[test]
fn a_field_outside_its_plausible_envelope_raises_a_flag() {
    let sano = giornale::inviluppo("tmrt", &[10.0, 25.0, 40.0], -40.0, 90.0);
    assert!(!sano.fuori_intervallo);
    assert_eq!(sano.minimo, 10.0);
    assert_eq!(sano.massimo, 40.0);
    let assurdo = giornale::inviluppo("tmrt", &[10.0, 500.0], -40.0, 90.0);
    assert!(assurdo.fuori_intervallo, "500 °C di temperatura radiante non è plausibile");
}

#[test]
fn a_field_with_holes_reports_the_fraction_of_missing_cells() {
    let inv = giornale::inviluppo("tmrt", &[1.0, f32::NAN, 3.0, f32::NAN], -40.0, 90.0);
    assert!((inv.frazione_senza_dato - 0.5).abs() < 1e-6);
    assert_eq!(inv.minimo, 1.0, "i buchi non entrano nel minimo");
}

#[test]
fn a_journal_opened_and_never_closed_still_says_where_the_run_stopped() {
    let dir = dir_di_prova("corsa-interrotta");
    let mut g = giornale::Giornale::apri(&dir, "prova").expect("apertura");
    g.annota("derivazione", "raster prodotti");
    g.fallisci("il file meteo non esiste");
    let v = rileggi(&dir);
    assert_eq!(v["passi"][0]["passo"].as_str(), Some("derivazione"));
    assert_eq!(v["conclusione"]["esito"].as_str(), Some("fallita"));
    assert_eq!(v["conclusione"]["errore"].as_str(), Some("il file meteo non esiste"));
}

/// Un Giornale che non è TOML valido non si rilegge, e la vista nella pagina e
/// l'appendice stampata sono rese di questo file: se non si rilegge, non
/// esistono. Il caso che lo rompe è un testo con virgolette dentro.
#[test]
fn a_journal_stays_valid_toml_even_with_quotes_in_the_text() {
    let dir = dir_di_prova("virgolette");
    let mut g = giornale::Giornale::apri(&dir, "prova \"strana\"").expect("apertura");
    g.annota("derivazione", "campo = \"ombra\", con virgolette e \\ barra");
    g.verifica(&giornale::inviluppo("ombra", &[0.0, 1.0], 0.0, 1.0));
    g.concludi(1.5);
    let v = rileggi(&dir);
    assert_eq!(v["etichetta"].as_str(), Some("prova \"strana\""));
    assert_eq!(v["verifiche"][0]["campo"].as_str(), Some("ombra"));
    assert_eq!(v["conclusione"]["esito"].as_str(), Some("riuscita"));
}

/// Finché la Corsa non è finita, `[conclusione]` non c'è: l'assenza è lo stato,
/// e non esiste una seconda chiave che possa contraddirla.
#[test]
fn a_journal_in_progress_has_no_conclusion_at_all() {
    let dir = dir_di_prova("in-corso");
    let mut g = giornale::Giornale::apri(&dir, "prova").expect("apertura");
    g.annota("derivazione", "raster prodotti");
    assert!(rileggi(&dir).get("conclusione").is_none());
}

fn dir_di_prova(nome: &str) -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(nome);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rileggi(dir: &std::path::Path) -> toml::Value {
    let testo = std::fs::read_to_string(dir.join("giornale.toml")).unwrap();
    testo.parse::<toml::Value>()
        .unwrap_or_else(|e| panic!("il Giornale non è TOML valido: {e}\n{testo}"))
}

/// Il cancello del progetto. Gira solo con il materiale presente e con
/// `--release`: un cronometro su una build di debug non dice niente.
#[test]
#[ignore = "cancello: eseguire con --release e il materiale del caso"]
fn the_reference_case_runs_under_the_budget() {
    let inizio = std::time::Instant::now();
    let esito = corsa::esegui_caso_di_riferimento().expect("il caso deve girare");
    let durata = inizio.elapsed();
    assert_eq!(esito.corse, 4,
               "due Scenari per due Periodi: il secondo Scenario è ricostruito, \
                e serve a misurare il carico, non a pubblicare risultati");
    assert!(durata.as_secs_f64() < 60.0, "budget sforato: {:.1} s", durata.as_secs_f64());
}

fn edificio_con(fonte: FonteAltezza) -> Edificio {
    Edificio {
        celle: vec![(0, 0)], altezza_m: 3.0,
        provenienza: Provenienza { origine: "test".into(), altezza: fonte },
    }
}
```

- [ ] **Step 3: Eseguire il test e verificare che fallisca**

Run: `cargo test --test corsa`
Expected: FAIL, `unresolved import climesh::giornale`.

- [ ] **Step 4: Scrivere il Giornale**

`src/giornale.rs`:

```rust
//! The Giornale: what a Corsa recorded about itself.
//!
//! One TOML file in the Corsa's directory. The view in the page and the printed
//! appendix are renderings of this file, never parallel artefacts — two surfaces
//! saying the same thing drift apart, and the printed one drifts first.
//!
//! It is opened at the start and appended to, so a Corsa that dies leaves a
//! Giornale saying how far it got. There is no checkpoint of the computation:
//! at sixty seconds the remedy is to press it again.

use crate::dominio::{FonteAltezza, Scenario};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Computed from the content, so two Corse with the same Impronta *are* the same
/// Corsa. It answers the only question a reviewer actually asks: are we looking
/// at the same result?
/// One input file, named and hashed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ingresso {
    pub nome: String,
    pub somma: String,
}

impl Ingresso {
    /// Reads the file and hashes it. A file that cannot be read hashes as
    /// `assente`, so a missing input changes the Impronta instead of being
    /// silently equal to a present one.
    pub fn da_file(percorso: &Path) -> Self {
        let somma = match std::fs::read(percorso) {
            Ok(b) => format!("{:x}", Sha256::digest(&b)),
            Err(_) => "assente".to_string(),
        };
        Self { nome: percorso.display().to_string(), somma }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Impronta(String);

impl Impronta {
    /// `ingressi` are the input files: each one contributes its path **and the
    /// SHA-256 of its content**, per spec §7. Names alone would give two
    /// different Progetti with the same Scenario name the same Impronta, which
    /// would defeat the only question the Impronta exists to answer.
    pub fn calcola(ingressi: &[Ingresso], versione_binario: &str, rev_motore: &str) -> Self {
        let mut h = Sha256::new();
        for i in ingressi {
            h.update(i.nome.as_bytes());
            h.update([0u8]);
            h.update(i.somma.as_bytes());
            h.update([0u8]);
        }
        h.update(versione_binario.as_bytes());
        h.update([0u8]);
        h.update(rev_motore.as_bytes());
        Self(format!("{:x}", h.finalize()))
    }

    /// The first twelve characters: enough to compare by eye, short enough to
    /// paste into a report.
    pub fn corta(&self) -> String {
        self.0.chars().take(12).collect()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ConteggioProvenienza {
    pub rilievo: usize,
    pub modello_di_superficie: usize,
    pub numero_di_piani: usize,
    pub predefinito: usize,
}

pub fn conta_provenienza(s: &Scenario) -> ConteggioProvenienza {
    let mut c = ConteggioProvenienza::default();
    for e in &s.edifici {
        match e.provenienza.altezza {
            FonteAltezza::Rilievo => c.rilievo += 1,
            FonteAltezza::ModelloDiSuperficie => c.modello_di_superficie += 1,
            FonteAltezza::NumeroDiPiani => c.numero_di_piani += 1,
            FonteAltezza::Predefinito => c.predefinito += 1,
        }
    }
    c
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Inviluppo {
    pub campo: String,
    pub minimo: f32,
    pub massimo: f32,
    pub media: f32,
    pub frazione_senza_dato: f32,
    pub fuori_intervallo: bool,
}

/// Min, max, mean and the fraction of missing cells, with a flag when the field
/// leaves a physically plausible range. Cheap enough to run on every Corsa.
pub fn inviluppo(campo: &str, valori: &[f32], minimo_plausibile: f32, massimo_plausibile: f32) -> Inviluppo {
    let validi: Vec<f32> = valori.iter().copied().filter(|v| v.is_finite()).collect();
    let n = validi.len();
    let (minimo, massimo, media) = if n == 0 {
        (f32::NAN, f32::NAN, f32::NAN)
    } else {
        (
            validi.iter().copied().fold(f32::INFINITY, f32::min),
            validi.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            validi.iter().sum::<f32>() / n as f32,
        )
    };
    Inviluppo {
        campo: campo.to_string(),
        minimo,
        massimo,
        media,
        frazione_senza_dato: 1.0 - n as f32 / valori.len().max(1) as f32,
        fuori_intervallo: n > 0 && (minimo < minimo_plausibile || massimo > massimo_plausibile),
    }
}

/// A TOML basic string, quotes and backslashes escaped. Everything written to
/// the Giornale goes through here or through `toml::to_string`.
fn stringa_toml(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 2);
    out.push('"');
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Opened at the start of a Corsa and appended to as it goes.
pub struct Giornale {
    percorso: PathBuf,
}

impl Giornale {
    pub fn apri(dir: &Path, etichetta: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let percorso = dir.join("giornale.toml");
        let mut f = std::fs::File::create(&percorso)?;
        writeln!(f, "# Giornale della Corsa. Scritto man mano: se la Corsa non")?;
        writeln!(f, "# arriva in fondo, quel che c'è dice fin dove è arrivata.")?;
        // Nessun `esito` qui: un file appeso non può riscrivere una riga di sopra,
        // e due esiti nello stesso file sarebbero una contraddizione permanente.
        // Finché `[conclusione]` non c'è, la Corsa non è finita.
        writeln!(f, "etichetta = {}", stringa_toml(etichetta))?;
        Ok(Self { percorso })
    }

    fn appendi(&mut self, testo: &str) {
        if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&self.percorso) {
            let _ = write!(f, "{testo}");
        }
    }

    pub fn annota(&mut self, passo: &str, dettaglio: &str) {
        let testo = format!(
            "\n[[passi]]\npasso = {}\ndettaglio = {}\n",
            stringa_toml(passo),
            stringa_toml(dettaglio)
        );
        self.appendi(&testo);
    }

    /// Records a per-Corsa check as a table of its own, serialised rather than
    /// printed: the `Debug` of a struct carrying a `String` writes unescaped
    /// quotes, and a Giornale that is not valid TOML cannot be read back by the
    /// page or by anyone else.
    pub fn verifica(&mut self, inv: &Inviluppo) {
        match toml::to_string(inv) {
            Ok(t) => self.appendi(&format!("\n[[verifiche]]\n{t}")),
            Err(e) => self.annota("verifica", &format!("non serializzabile: {e}")),
        }
    }

    pub fn fallisci(&mut self, errore: &str) {
        let testo = format!(
            "\n[conclusione]\nesito = \"fallita\"\nerrore = {}\n",
            stringa_toml(errore)
        );
        self.appendi(&testo);
    }

    pub fn concludi(&mut self, secondi: f64) {
        self.appendi(&format!("\n[conclusione]\nesito = \"riuscita\"\nsecondi = {secondi:.3}\n"));
    }
}
```

- [ ] **Step 5: Scrivere l'orchestrazione della Corsa**

`src/corsa.rs` compone i pezzi: legge il Progetto, deriva i raster una volta **per Scenario** — non per Corsa, perché lo sky view factor dipende solo dalla geometria e sul caso di riferimento questo è ciò che separa i 60 secondi dagli 87 — chiama il Motore per ogni passo temporale del Periodo, e scrive campi e Giornale.

```rust
//! Runs a Corsa: one Scenario computed for one Periodo.
//!
//! Derivation is cached by `(Scenario, Stagione)`, not by Scenario alone: the
//! leaf-off Periodo drops deciduous canopies, so the canopy raster genuinely
//! differs between seasons. On the reference case the two Periodi fall either
//! side of the leaf-on window, so four Corse need four derivations and the cache
//! saves nothing here — it pays off as soon as a Scenario gets a second Periodo
//! in the same season, which is what a parametric study looks like.
//!
//! The saving the budget actually leans on is finer and does not exist yet: the
//! building sky view factor is season-independent and can be computed once per
//! Scenario, while the vegetated one is not. Measured on the reference geometry,
//! that is 8.1 s against 29.2 s. It belongs to the task that adds the sky view
//! factor, and until then this module must not claim it.

use crate::derivazione::{self, RasterDiScenario, Stagione};
use crate::dominio::*;
use crate::giornale::{self, Giornale, Impronta};
use crate::motore;
use crate::sole;

pub struct Campi {
    pub ombra_per_ora: Vec<derivazione::Raster>,
}

pub struct Esito {
    pub corse: usize,
    pub secondi: f64,
}

fn stagione_di(p: &Periodo) -> Stagione {
    let giorno = p.inizio.giorno_dell_anno();
    if (100..=300).contains(&giorno) { Stagione::ConFoglie } else { Stagione::SenzaFoglie }
}

pub fn esegui(
    dir_corsa: &std::path::Path,
    progetto: &Progetto,
    scenario: &Scenario,
    periodo: &Periodo,
    raster: &RasterDiScenario,
) -> std::io::Result<Campi> {
    let etichetta = format!("{} · {}", scenario.nome, periodo.nome);
    let mut g = Giornale::apri(dir_corsa, &etichetta)?;
    let impronta = Impronta::calcola(
        &[
            giornale::Ingresso::da_file(&periodo.meteo),
            giornale::Ingresso {
                nome: format!("scenario:{}", scenario.nome),
                somma: format!("{:x}", sha2::Sha256::digest(
                    toml::to_string(scenario).unwrap_or_default().as_bytes())),
            },
        ],
        env!("CARGO_PKG_VERSION"),
        &motore::versione().git_rev,
    );
    g.annota("impronta", &impronta.corta());
    g.annota("derivazione", &format!("{:?}", raster.scelte));

    let inizio = std::time::Instant::now();
    let mut ombra_per_ora = Vec::with_capacity(periodo.ore as usize);
    for ora in 0..periodo.ore {
        let p = sole::posizione(
            periodo.inizio.giorno_dell_anno() + ora / 24,
            (ora % 24) as f64,
            progetto.griglia.origine.1,
            progetto.griglia.origine.0,
            1.0,
        );
        ombra_per_ora.push(motore::ombre(
            &raster.superficie,
            progetto.griglia.passo_m,
            p.azimut_gradi,
            p.altezza_gradi,
        ));
    }

    let tutte: Vec<f32> = ombra_per_ora.iter().flat_map(|a| a.iter().copied()).collect();
    g.verifica(&giornale::inviluppo("ombra", &tutte, 0.0, 1.0));
    g.concludi(inizio.elapsed().as_secs_f64());
    Ok(Campi { ombra_per_ora })
}

/// Il caso di riferimento completo: due Scenari per due Periodi.
pub fn esegui_caso_di_riferimento() -> std::io::Result<Esito> {
    let progetto = crate::progetto::leggi("casi/bastia/progetto")
        .expect("il caso di riferimento deve essere un Progetto valido");
    let inizio = std::time::Instant::now();
    let mut corse = 0;
    // La cache è per (Scenario, Stagione): due Periodi nella stessa stagione
    // riusano la derivazione, due in stagioni diverse no, perché il raster delle
    // chiome cambia davvero.
    let mut cache: std::collections::HashMap<(String, Stagione), RasterDiScenario> =
        std::collections::HashMap::new();
    for scenario in &progetto.scenari {
        for periodo in &progetto.periodi {
            let chiave = (scenario.nome.clone(), stagione_di(periodo));
            let raster = cache.entry(chiave).or_insert_with(|| {
                derivazione::deriva(&progetto.griglia, scenario, stagione_di(periodo))
            });
            let dir = std::path::Path::new("casi/bastia/progetto/corse")
                .join(format!("{}-{}", scenario.nome, periodo.nome));
            esegui(&dir, &progetto, scenario, periodo, raster)?;
            corse += 1;
        }
    }
    Ok(Esito { corse, secondi: inizio.elapsed().as_secs_f64() })
}
```

- [ ] **Step 6: Eseguire i test e verificare che passino**

Run: `cargo test --test corsa`
Expected: PASS, 5 test. Il sesto è `#[ignore]`.

- [ ] **Step 7: Attraversare il cancello**

```bash
cargo test --release --test corsa -- --ignored --nocapture
```

Expected: PASS, con la durata stampata.

**Se sfora, fermarsi.** Non ottimizzare a caso: misurare prima dove va il tempo, e se il collo di bottiglia è la derivazione ripetuta invece del calcolo, la cache per Scenario non sta funzionando. Se è il calcolo, la decisione da riaprire è quella sul Motore, non quella sull'architettura sopra. Il piano si ferma qui finché il numero non torna: tutto ciò che verrebbe dopo presuppone che il budget regga.

- [ ] **Step 8: Commit**

```bash
git add src/giornale.rs src/corsa.rs src/lib.rs Cargo.toml Cargo.lock tests/corsa.rs
git commit -m "feat: run a Corsa and record it in its Giornale"
```

---

## Cosa questo piano non fa

Sono sottosistemi separati, ciascuno con il proprio piano, ognuno dei quali produce software funzionante da sé:

- **Le superfici** — riga di comando, pagina nel browser, foglio di stampa.
- **La validazione** — parità contro l'implementazione di riferimento in integrazione continua, e il confronto con le misure di Göteborg recuperate dal repository del Motore a un tag pinnato, mai copiate nel nostro.
- **L'import da OpenStreetMap** — con la catena di ripiego delle altezze, che è dove `FonteAltezza::ModelloDiSuperficie` e `NumeroDiPiani` smettono di essere varianti mai costruite. Il tipo esiste già da Task 1 perché il Giornale deve contarle; la catena che sceglie appartiene al lettore che ne ha bisogno, non a questo piano.
- **La catena completa fino a Tmrt e agli indici di comfort.** Task 4 collega il Motore sulle ombre, che è la fetta più piccola che dimostra che l'integrazione regge ed è dove il collo di bottiglia si manifesta per primo. Il resto della catena è meccanico una volta che il collegamento funziona, e va pianificato dopo aver visto il cronometro.
