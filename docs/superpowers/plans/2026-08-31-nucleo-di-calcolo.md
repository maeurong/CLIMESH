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

Da oggetti in metri a raster co-registrati. Verificabile per intero senza il Motore.

> **Scritto dopo Task 1 e 2**, contro i tipi che esistono. Leggi `src/dominio.rs`: è la fonte.

**Files:**
- Create: `src/derivazione.rs`
- Modify: `src/lib.rs`, `Cargo.toml`
- Test: `tests/derivazione.rs`

**Interfaces:**
- Consumes: `dominio::{Griglia, Scenario, Periodo, Rettangolo, Posizione}`, `specie::e_decidua`.
- Produces: `derivazione::{Raster, RasterDiScenario, ScelteDiDerivazione, Stagione, deriva}` più le costanti delle classi di superficie.

**La dipendenza che arriva da fuori.** `ndarray` va pinnata alla versione del Motore, che è **0.16.1** con feature `rayon` — accertato nel dossier `research/motore-api.md`. `ndarray` non ha la 1.0, quindi due versioni minori diverse sono due tipi incompatibili e `ArrayView2<f32>` non combacerebbe. Usa `"0.16"`, non salire alla 0.17.

**La regola di copertura, che va scritta una volta e rispettata ovunque.** Una cella appartiene a un rettangolo quando **il suo centro** cade dentro, con gli intervalli **semiaperti** — minimo incluso, massimo escluso. Il centro perché è la convenzione che non raddoppia le celle sui bordi condivisi; il semiaperto perché due rettangoli adiacenti che condividono un lato devono coprire ogni cella **una volta sola**. Una regola diversa produce un modello plausibile e sbagliato, che è la trappola che `CLAUDE.md` segnala.

**Cosa deve essere vero alla fine:**

1. **Cinque raster co-registrati**, tutti della forma della Griglia: modello di superficie (terreno più edifici), modello di terreno, chiome, zona-tronco, classi di superficie. Riga 0 a nord, come ogni matrice del progetto.
2. **Le altezze sono assolute**, non relative: una chioma di 12 m su un terreno a 2 m sta a 14 m. Il Motore ragiona su quote, non su altezze locali.
3. **La zona-tronco** è la frazione dichiarata dell'altezza di chioma, misurata dal terreno.
4. **Il Periodo senza foglie esclude le specie decidue dal raster delle chiome**, e conta quante ne ha escluse. È la ragione concreta per cui esiste l'ADR 0001: il Motore accetta una sola trasmissività fogliare per Corsa, ma la specie vive sull'oggetto, quindi la distinzione si esprime qui, per pixel. Con i raster come verità non sarebbe stata nemmeno esprimibile.
5. **Due alberi sulla stessa cella lasciano la chioma più alta**, e la zona-tronco che le corrisponde — non quella dell'altro albero.
6. **`ScelteDiDerivazione` registra ciò che la Derivazione ha deciso**: chiome escluse, oggetti fuori griglia, celle costruite, celle con chioma, e la sostituzione del terreno se la sua lunghezza non corrispondeva. Sono le righe che il Giornale pubblicherà: una scelta di modellazione presa dal programma non si ingoia in silenzio.
7. **La stagione si decide dal Periodo**, non si passa a mano da chi chiama: la finestra con foglie del Motore è il giorno 100-300, e `Data::giorno_dell_anno` restituisce `Option`, quindi una data impossibile deve avere un comportamento scelto e testato, non un `unwrap`.

## Ingressi degeneri

Ogni riga è un test, e ogni test deve fallire se togli la difesa:

- rettangolo che esce dalla Griglia → coperte solo le celle dentro, mai un indice fuori intervallo
- rettangolo di area nulla, o con minimo e massimo invertiti → nessuna copertura e un conteggio che lo dice, mai una copertura al contrario
- due rettangoli che condividono un lato → ogni cella coperta una volta sola, verificabile contando
- albero esattamente sul confine fra due celle → assegnato a una sola, in modo deterministico e documentato
- albero fuori dall'estensione della Griglia → contato in `oggetti_fuori_griglia`, mai una posizione scartata in silenzio
- terreno di lunghezza diversa dalle celle → sostituito con terreno piatto **e registrato**, mai sostituito e basta
- Scenario senza alberi in un Periodo senza foglie → zero esclusioni, non un errore
- Scenario con soli alberi decidui in un Periodo senza foglie → raster delle chiome tutto a zero, e il conteggio pari al numero di alberi
- Griglia 1×1 → funziona; Griglia con `celle()` che restituisce `None` → rifiutata prima di allocare
- due Derivazioni sullo stesso Scenario e stessa Stagione → raster identici byte per byte, perché il contratto di riproducibilità classifica la copertura come esito discreto

**Accettazione:** `cargo test` verde in debug **e in release**, `clippy` con warning negati e `fmt` puliti. Il caso di riferimento deriva senza errori in entrambe le stagioni, e il raster delle chiome invernale ha meno celle non nulle di quello estivo.

---

### Task 4: Il Motore, vendorato e collegato

Le ombre vengono dal nucleo riusato invece che da noi, e si verificano contro la geometria.

> **Riscritto dopo la decisione di Mario del 2026-09-01: niente fork, niente richiesta a monte, vendoring diretto.** Il sorgente del nucleo entra in `vendor/solweig/`, con la sua licenza e la sua provenienza. Il motivo per cui questa via è praticabile è che aggira in un colpo solo i tre ostacoli che il dossier aveva trovato: `crate-type`, i moduli privati e `pyo3` con `extension-module` non opzionale. Non consumiamo il loro crate: prendiamo i file che servono.

**Files:**
- Create: `vendor/solweig/` — i sorgenti presi, con `LICENSE` e `PROVENIENZA.toml`
- Create: `src/sole.rs`, `src/motore.rs`
- Modify: `src/lib.rs`, `Cargo.toml`
- Test: `tests/motore.rs`

**Interfaces:**
- Produces: `sole::{PosizioneSolare, posizione}`; `motore::{ombre, VersioneMotore, versione}`. **`ombre` restituisce la frazione illuminata**, 1 al sole e 0 in ombra, come il campo `bldg_sh` del nucleo — non l'ombra.

#### 4a — Prendere il minimo che serve

Clonare `UMEP-dev/solweig` al commit `02246ab7`, che è quello letto nel dossier, e portare dentro **soltanto** i file necessari al percorso CPU delle ombre. Non l'intero crate: niente GPU, niente `pyo3`, niente catena Python. Il criterio è che compili e nient'altro.

Obblighi non negoziabili, perché è codice di altri:

- **Le intestazioni di licenza dei file presi non si toccano.**
- `vendor/solweig/LICENSE` porta il testo GPL-3 a monte.
- `vendor/solweig/PROVENIENZA.toml` dichiara, in forma leggibile da un programma: `upstream`, `commit`, `data`, `licenza`, `percorso_sorgente`, l'elenco dei file presi e **l'elenco esatto delle modifiche fatte**. Il lavoro pianificato `.github/workflows/vendor-check.yml` legge `upstream`, `commit` e `percorso_sorgente` da questo file: i nomi delle chiavi sono un contratto.
- Ogni modifica ai file presi è minima e dichiarata. Adattare per far compilare è lecito; migliorare non lo è, perché ogni divergenza è debito da riconciliare al prossimo aggiornamento.
- `README.md` e `PRODUCT.md` dicono che il nucleo è riusato: aggiornarli se la forma del riuso cambia il senso di quelle frasi.

**Se non compila, la via di riserva è decisa** e non va improvvisata: **riscrivere le ombre da zero**. Il calcolo dell'ombreggiamento su un campo di altezze è poche centinaia di righe, l'algoritmo è la marcia del raggio lungo l'azimut, e il test geometrico più sotto lo verifica indipendentemente da chi l'ha scritto. In quel caso `vendor/` sparisce, il lavoro pianificato di controllo si toglie, e la scelta va scritta in un ADR perché contraddice la risoluzione del ticket sul motore.

#### 4b — Collegare e verificare

**La firma è nota** e sta in `research/motore-api.md` § 1: quindici parametri, di cui dieci raster opzionali e tre scalari di taglio, e il ritorno è una struct. Verificala comunque contro il sorgente vendorato prima di scrivere il chiamante.

Due fatti che il piano aveva sbagliato prima del dossier, e che vanno rispettati:

- **`max_local_dsm_ht` è il rilievo**, massimo meno minimo, non il massimo. È la quota di cui il raggio deve salire per essere sicuramente sopra ogni ostacolo; passare il massimo marcia più del necessario su qualunque dominio il cui terreno non stia a zero.
- **L'azimut del nucleo coincide con il nostro** — gradi da nord in senso orario, riga 0 a nord — accertato su spec e docstring a monte. Nessuna conversione. Se il test geometrico fallisce, la conversione va corretta in `motore::ombre`, che è l'unico punto del programma a conoscerla: **il test non si ruota**, perché dice un fatto geometrico.

`sole.rs` calcola la posizione solare **per conto nostro**, non la chiede al nucleo: la verifica dell'ombra nel Giornale confronta l'ombra del Motore con una posizione derivata in modo indipendente, e un controllo che condivide la fonte con ciò che controlla non verifica niente.

## Ingressi degeneri

- sole sotto l'orizzonte → tutto in ombra, e la decisione è nostra e non del nucleo, perché è una scelta di modellazione e non un fatto geometrico
- dominio piatto con sole alto → tutto al sole, nessuna ombra dal nulla
- torre di 10 m, sole a sud a 45 gradi → ombra lunga 10 m **verso nord**, sole a sud della torre e sole a ovest: è la verifica geometrica del gate, e vale contro qualunque implementazione
- sole all'alba, altezza vicina a zero → nessun panico, nessuna divisione per zero, nessuna ombra di lunghezza infinita
- dominio con terreno a quota costante non nulla → stesse ombre del dominio a quota zero, che è il test che smaschera il rilievo confuso col massimo
- Griglia 1×1 → funziona
- versione del Motore leggibile a runtime → `versione()` restituisce commit e data della copia vendorata, che il Giornale citerà

**Accettazione:** i test geometrici passano, `cargo test` verde in debug e release, `clippy` e `fmt` puliti, e una singola chiamata a `ombre` su 50×50 va **misurata e riportata**: sopra i 300 ms il cancello dei 60 secondi è già perso, ed è meglio saperlo qui che a Task 5.

---

### Task 5: Il Giornale e il cancello dei 60 secondi

La prima Corsa completa sul caso di riferimento, con il suo Giornale, cronometrata.

> **Scritto dopo i task precedenti**, contro i tipi che esistono. Leggi `src/dominio.rs`, `src/derivazione.rs` e `src/motore.rs`: sono la fonte.

**Files:**
- Create: `src/giornale.rs`, `src/corsa.rs`
- Modify: `src/lib.rs`, `Cargo.toml`, `.gitignore`
- Test: `tests/corsa.rs`

**Interfaces:**
- Produces: `giornale::{Giornale, Impronta, Ingresso, Inviluppo, ConteggioProvenienza, conta_provenienza, inviluppo}`; `corsa::{esegui, esegui_caso_di_riferimento, Campi, Esito}`.

**Il Giornale è un file TOML nella cartella della Corsa, e una sola fonte.** La vista nella pagina e l'appendice stampata saranno **rese** di quel file, mai artefatti paralleli: due superfici che dicono la stessa cosa divergono, e diverge per prima quella che si guarda di meno.

**Cosa deve essere vero alla fine:**

1. **Si apre all'inizio e si scrive man mano.** Una Corsa che muore lascia un Giornale che dice fin dove è arrivata e con quale errore. **Nessun `esito` al livello superiore**: un file appeso non può riscrivere una riga di sopra, quindi due esiti nello stesso file sarebbero una contraddizione permanente. Finché `[conclusione]` non c'è, la Corsa non è finita, e l'assenza *è* lo stato.
2. **Resta TOML valido comunque.** Ogni testo che entra passa da un escaping o da `toml::to_string`. Stampare il `Debug` di una struct che contiene una `String` inietta virgolette non protette, e un Giornale che non si rilegge non esiste — perché la pagina e la stampa sono rese di questo file. **I test lo rileggono con il parser**, non cercano sottostringhe.
3. **Registra**: gli ingressi con le loro somme di controllo, la versione del binario, la versione e il commit del Motore vendorato, la Griglia, lo Scenario e il Periodo, le scelte della Derivazione, e il fornitore dei valori meteorologici marcato come **ingresso e non risultato**.
4. **Riporta le verifiche per Corsa**, che costano poco: l'ombra confrontata con la posizione solare calcolata in modo indipendente; il conteggio della Provenienza per anello della catena; l'inviluppo di ogni campo — minimo, massimo, media — con una bandiera se esce da un intervallo fisicamente plausibile; la frazione di celle senza dato.
5. **Cita le verifiche per rilascio** invece di rieseguirle: parità contro l'implementazione di riferimento e confronto con le misure di campo sfonderebbero il budget da sole. Il Giornale dice *questo binario usa il nucleo alla versione X*. È la ragione tecnica per cui il commit vendorato è pinnato: se potesse muoversi sotto i piedi, la citazione non varrebbe niente.
6. **Ogni Corsa ha due nomi**: un'**Impronta** calcolata dal contenuto — somme di controllo degli ingressi, versione del binario, commit del Motore, parametri — per cui due Corse con la stessa Impronta *sono* la stessa Corsa; e un'**etichetta** scelta da chi la lancia, perché nessuno chiama una corsa `a3f9c1`. Il Giornale porta anche una riga di citazione già scritta.
7. **Il contratto di riproducibilità è a due livelli**, e va scritto nel Giornale: gli esiti **discreti** — quali celle sono in ombra, i conteggi, l'ordine di qualunque cosa — sono identici su qualunque macchina, e un esito discreto che dipende dalla piattaforma è un difetto da correggere; le grandezze **continue** rientrano entro una tolleranza dichiarata. Promettere l'identità bit a bit sarebbe falso.
8. **Nessun checkpoint del calcolo.** A sessanta secondi il rimedio è premere di nuovo. Se il budget saltasse, questa decisione va riaperta insieme a quello: sono la stessa decisione.
9. **La Derivazione è in cache per `(Scenario, Stagione)`**, non per Scenario soltanto: il Periodo senza foglie toglie le chiome decidue, quindi il raster cambia davvero con la stagione. Sul caso di riferimento i due Periodi cadono ai due lati della finestra fogliare, quindi **la cache qui non risparmia niente** e va detto invece di lasciar credere il contrario; paga quando uno Scenario prende un secondo Periodo nella stessa stagione, che è ciò che è uno studio parametrico.
10. `casi/bastia/progetto/corse/` è già in `.gitignore`: nasce dall'esecuzione.

## Ingressi degeneri

- Giornale con virgolette, barre rovesce e ritorni a capo nel testo → resta TOML valido e si rilegge, verificato **col parser**
- Corsa interrotta a metà → `[conclusione]` assente, i passi già fatti presenti, nessun secondo `esito`
- Corsa fallita → `[conclusione]` con esito fallita e l'errore leggibile
- campo tutto `NaN` → inviluppo che lo dice, frazione senza dato pari a 1, nessun minimo inventato
- campo con valori fuori dall'intervallo plausibile → bandiera alzata, e il valore riportato comunque
- stessi ingressi due volte → stessa Impronta; un ingresso modificato di un byte → Impronta diversa
- file di ingresso assente al momento del calcolo dell'Impronta → contribuisce come assente, mai identico a uno presente
- Periodo la cui `Data::giorno_dell_anno()` è `None` → comportamento scelto e testato, mai un `unwrap`
- due Corse che condividono Scenario e Stagione → una sola Derivazione, verificabile contando

## Il cancello

```bash
cargo test --release --test corsa -- --ignored --nocapture
```

Il caso di riferimento completo — due Scenari per due Periodi, quattro Corse, 50×50 celle a 1 m, 48 ore — **sotto i 60 secondi su sola CPU**, misurato in release perché un cronometro su una build di debug non dice niente.

**Se sfora, il piano si ferma qui.** Non ottimizzare a caso: prima misurare **dove** va il tempo, perché la decisione da riaprire dipende dalla risposta. Se domina la Derivazione ripetuta, il difetto è nella cache e sta dentro questo piano. Se domina il calcolo del Motore, si riapre la scelta del motore e con essa quella sull'assenza di checkpoint, che ne discende. La ripartizione del tempo va nel rapporto in entrambi i casi, e la decisione è di Mario, non del subagente.

## Cosa questo piano non fa

Sono sottosistemi separati, ciascuno con il proprio piano, ognuno dei quali produce software funzionante da sé:

- **Le superfici** — riga di comando, pagina nel browser, foglio di stampa.
- **La validazione** — parità contro l'implementazione di riferimento in integrazione continua, e il confronto con le misure di Göteborg recuperate dal repository del Motore a un tag pinnato, mai copiate nel nostro.
- **L'import da OpenStreetMap** — con la catena di ripiego delle altezze, che è dove `FonteAltezza::ModelloDiSuperficie` e `NumeroDiPiani` smettono di essere varianti mai costruite. Il tipo esiste già da Task 1 perché il Giornale deve contarle; la catena che sceglie appartiene al lettore che ne ha bisogno, non a questo piano.
- **La catena completa fino a Tmrt e agli indici di comfort.** Task 4 collega il Motore sulle ombre, che è la fetta più piccola che dimostra che l'integrazione regge ed è dove il collo di bottiglia si manifesta per primo. Il resto della catena è meccanico una volta che il collegamento funziona, e va pianificato dopo aver visto il cronometro.
