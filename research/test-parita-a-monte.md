# I test a monte (`UMEP-dev/solweig`) — cosa contengono, contro cosa confrontano, cosa serve per eseguirli

**Scopo**: rispondere al ticket [#23](https://github.com/maeurong/CLIMESH/issues/23) della mappa
[#17](https://github.com/maeurong/CLIMESH/issues/17), per sbloccare
[#22 «Come si esegue la parità per componente»](https://github.com/maeurong/CLIMESH/issues/22).
La domanda dietro la domanda: CLIMESH **copia** il nucleo Rust a monte invece di dipenderne, quindi
confrontare il nostro sky view factor con il loro è tautologico. Serve sapere contro cosa si può
confrontare davvero.

**Data ricerca**: 2026-09-07.
**Repo CLIMESH**: ramo `research/test-parita-a-monte` da `main`, HEAD di partenza
`6f675ab88f1c5d4cdf3167b3ead48be1456aa76f` («Record the upstream request in ADR 0003»).
**Repo letto**: clone in `~/solweig-recon`, HEAD verificato con `git -C ~/solweig-recon rev-parse HEAD`
= `02246ab71a3a8b127d740dde9640449ee9d558ff`, cioè il commit pinnato in
[`vendor/solweig/PROVENIENZA.toml:9`](../vendor/solweig/PROVENIENZA.toml) (`tag_vicino = "v0.1.0b95"`).
Tutti i permalink qui puntano a quel commit.

**Ricerca documentale, nessuna esecuzione.** La loro suite non è stata installata né lanciata: serve
una toolchain Rust più una `maturin develop --release`, e il ticket chiede di leggere. Ogni numero che
si potrebbe sapere solo eseguendo è marcato `[V]` = *da verificare eseguendo*.

**Convenzione di marcatura** (la stessa di [`research/motore-api.md`](motore-api.md))
- `[P]` = letto nel sorgente o nella loro documentazione.
- `[I]` = inferenza mia dal codice, non verificata eseguendo.
- `[V]` = da verificare eseguendo: non lo si sa leggendo.

---

## 0. Risposta in breve

1. I livelli sono **quattro, non tre**: `tests/spec/` (proprietà fisiche e parità contro UMEP Python),
   `tests/golden/` (regressione contro fissaggi `.npy`), `tests/validation/` (tre siti di Göteborg con
   misure di campo), `tests/benchmarks/` (soglie di tempo e memoria). Il `tests/README.md` ne descrive
   tre e nomina come terzo un file, `tests/rustalgos/test_rustalgos.py`, **che al commit pinnato non
   esiste più**: in `tests/rustalgos/` restano solo tre `.ini` e un `.json` di configurazione. `[P]`
2. **I fissaggi stanno nel repository**, non si scaricano: 81 file sotto
   `tests/golden/fixtures/`, **37 MB** in totale, tutti tracciati da git (nessun Git LFS, `.gitattributes`
   non esiste). I raster sono **400 × 400** in `<f4` o `<f8`. `[P]`
3. **La verità a terra non è la stessa per tutti i fissaggi.** Ombre, SVF, GVF, radiazione, UTCI, PET e
   temperatura del suolo sono generati chiamando il **pacchetto pip `umep`**, cioè un'implementazione
   Python indipendente. Cielo anisotropo, temperatura dei muri e radiazione anisotropa sono generati
   **dal loro stesso Rust**: per quelli il golden è una fotografia, non un giudice. `[P]`
4. **Le tolleranze dichiarate nei loro due README non sono quelle del codice.** Il codice è
   l'autorità: SVF non vegetato `rtol=1e-5, atol=1e-5`, SVF vegetato `rtol=0.21, atol=0.18`
   (i README dicono `0.01 / 0.02`); ombre `1e-5 / 1e-5` (i README dicono `1e-6 / 1e-6`). `[P]`
5. `tests/spec/test_umep_parity.py` **non tocca né SVF né ombre né Tmrt**. Afferma due cose sole:
   che `create_patches` locale è *identica* (uguaglianza esatta) a quella di UMEP per le opzioni 1–4, e
   che `patch_steradians` coincide a `rtol=1e-6, atol=1e-8`. È la decomposizione del cielo in patch,
   non la fisica. `[P]`
6. La parità che conta davvero per la catena radiativa sta in **altri tre file**:
   `test_parity_extended.py` (findwalls bit-identico, Goodwin solo strutturale, `lside_veg`
   `rtol=1e-3, atol=1e-3`), `test_perez_parity.py` (Perez_v3, `atol=0.01` sulla luminanza) e
   `test_parity_2026a.py` (Lside 2026a, `rtol=1e-3, atol=1e-2`) contro una copia **Python** vendorata
   in `tests/reference/umep_2026a/`, 88 KB, GPL-3.0, con provenienza in testa a ogni file. `[P]`
7. **Girano senza QGIS**: nessun job di CI installa QGIS, e i test del plugin usano i mock in
   `tests/qgis_mocks.py`. **Girano senza GPU**: ogni test GPU si salta da sé con
   `if not solweig.is_gpu_available()`. Quello che *non* si può evitare è **compilare il loro Rust**:
   `tests/conftest.py` importa `solweig` già in `pytest_configure`. `[P]`
8. **I dati di Göteborg stanno lì dentro**, sotto `tests/validation/`, 3,8 MB per tre siti
   (Kronenhuset, Gustav Adolfs torg, GVC), rasters + met + misure CSV + POI GeoJSON. **Licenza: GPL-3.0**,
   quella del repository; non esiste una licenza dati separata, e nemmeno una nota di terzi. `[P]`
9. **Raccomandazione** (§ 7, marcata come tale): il cancello non tautologico per CLIMESH è il pacchetto
   pip `umep` — implementazione Python indipendente dal Rust che abbiamo copiato — e in seconda battuta
   le misure di campo. I fissaggi `.npy` restano utili come **vettori d'ingresso** (un DSM 400 × 400 con
   i suoi parametri), non come verdetto.

---

## 1. La mappa dei test al commit pinnato

| Cartella | File di test | Peso | Cosa fa |
|---|---:|---:|---|
| `tests/` (radice) | 40 | — | Unità: I/O, tiling, cache, plugin QGIS mockato, logging, errori |
| `tests/spec/` | 22 | 240 KB | Proprietà fisiche su dati sintetici **+ parità contro UMEP Python** |
| `tests/golden/` | 12 + generatore | 38 MB | Regressione contro `tests/golden/fixtures/` (37 MB) |
| `tests/validation/` | 5 | 3,8 MB | Tre siti di Göteborg contro misure di campo |
| `tests/benchmarks/` | 4 | 52 KB | Soglie di tempo e memoria |
| `tests/reference/` | — | 88 KB | **Codice Python di UMEP vendorato**, riferimento di parità |
| `tests/rustalgos/` | 0 | 16 KB | Solo `.ini` e `.json` di configurazione |

`[P]` — misurato con `du -sh` e `find` in `~/solweig-recon` al commit pinnato.

**Prima trappola per chi legge il loro `tests/README.md`.** Quel documento descrive una strategia a tre
livelli e chiama il terzo «Rust vs UMEP comparison, `tests/rustalgos/test_rustalgos.py`»
([`tests/README.md:76-95`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/tests/README.md)).
Quel file **non c'è**. La comparazione Rust-contro-UMEP non è sparita, si è spostata dentro
`tests/spec/` (i quattro file di parità del § 3) e dentro il generatore dei fissaggi. Chi cerca il
livello 3 dove il README lo manda non lo trova.

---

## 2. `tests/golden/` — i fissaggi e cosa dicono davvero

### 2.1 Cosa c'è

`[P]` — 81 file in `tests/golden/fixtures/`, 37 MB, tutti in git.

- **Ingressi**: `input_dsm.npy`, `input_cdsm.npy`, `input_tdsm.npy`, `input_bush.npy`,
  `input_wall_ht.npy`, `input_wall_asp.npy` — 400 × 400 `<f4`, 640 128 byte l'uno.
- **Parametri**: `input_params.npz` (530 byte) con due scalari `<f8`, `scale` e `amaxvalue`;
  `svf_metadata.npz` e `shadow_metadata.npz` (796 byte) con `dsm_shape`, `scale`, `amaxvalue`.
  I valori numerici non li ho letti: servirebbe numpy. `[V]`
- **Uscite attese**: SVF (`svf_total`, `svf_north/east/south/west`, `svf_veg`), ombre per tre posizioni
  del sole (mattina 90°/30°, mezzogiorno 180°/60°, pomeriggio 270°/45°), GVF, radiazione Kside/Lside
  isotropa e anisotropa, Tmrt isotropa e anisotropa, UTCI puntuale e a griglia, PET puntuale e a
  griglia, temperatura del suolo (tre casi), temperatura dei muri, cielo anisotropo
  (`aniso_sky_output.npz`, 4,4 MB, il file più grosso), schema del suolo 2026a
  (`fixtures/ground_scheme/scheme_iso_baseline.npz`, 43 KB).

### 2.2 Da dove viene la verità a terra — il punto che decide tutto

Il generatore
[`tests/golden/generate_fixtures.py`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/tests/golden/generate_fixtures.py)
importa, per riga: `umep.functions.SOLWEIGpython.solweig_runner_core` (18),
`umep.functions.svf_functions.svfForProcessing153` (19),
`shadowingfunction_wallheight_23` (20), poi dentro le singole funzioni `gvf_2018a` (200),
`Kside_veg_v2022a` / `Lside_veg_v2022a` (311-312), `utci_calculator` (442),
`PET_calculations` (530, 582), `TsWaveDelay_2015a` (751). `[P]`

Ma **tre generatori non chiamano UMEP affatto**: `generate_anisotropic_sky_fixtures` importa
`solweig.rustalgos.sky` (riga 842), `generate_wall_temp_fixtures` importa `solweig.rustalgos.ground`
(riga 970), `generate_aniso_radiation_fixtures` importa `solweig.rustalgos.vegetation` (riga 1060).
`[P]` Il loro stesso README lo dichiara — «Ground Truth: Rust (regression)» per cielo anisotropo e
temperatura dei muri — ma la riga va letta per quello che significa: **quei fissaggi non provano nulla
sulla fisica, provano solo che il Rust non è cambiato**. Per CLIMESH, che ha copiato quel Rust, sono
doppiamente tautologici.

### 2.3 Le tolleranze vere

I due documenti `tests/golden/README.md` e `tests/golden/REPORT.md` portano una tabella di tolleranze
identica fra loro e **diversa dal codice**. Il codice: `[P]`

| Grandezza | rtol | atol | Fonte |
|---|---|---|---|
| SVF totale e direzionali | 1e-5 | 1e-5 | `tests/golden/test_golden_svf.py:29-30` |
| SVF vegetato | **0.21** | **0.18** | `tests/golden/test_golden_svf.py:39-40` |
| Ombre (edifici, chiome, muri) | 1e-5 | 1e-5 | `tests/golden/test_golden_shadows.py:83-84` e seguenti |
| Lside 2026a vs Python vendorato | 1e-3 | 1e-2 | `tests/spec/test_parity_2026a.py:129-130` |
| Temperatura del suolo, passi concatenati | 1e-3 | 0.05 | `tests/spec/test_parity_2026a.py:302` |
| Onda lunga uscente | 1e-3 | 0.5 | `tests/spec/test_parity_2026a.py:401-402` |

Le tabelle dei README dicono invece «Shadows 1e-6 / 1e-6» e «SVF 0.01 / 0.02». Sono **stale**: non
descrivono nessuna riga di codice al commit pinnato.

Il perché della tolleranza al 21 % sul SVF vegetato è scritto nel docstring del test, e vale la pena
riportarlo perché è esattamente il genere di trappola in cui CLIMESH cadrebbe:
[`test_golden_svf.py:5-15`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/tests/golden/test_golden_svf.py)
dice che il fissaggio è generato da `svfForProcessing153`, che internamente usa
`shadowingfunction_20`, mentre il Rust usa l'equivalente di `wallheight_23`; l'accumulo in virgola
mobile su 153 patch diverge ai bordi della vegetazione, con differenza massima ~0,17 su ~0,11 % dei
pixel. C'è un `TODO` esplicito: rigenerare il fissaggio con la stessa funzione d'ombra. **Finché quel
TODO è aperto, il cancello sul SVF vegetato non è un cancello.** `[P]`

### 2.4 I fissaggi si possono rigenerare da zero?

Solo in parte. `generate_svf_fixtures` legge `input_dsm.npy`, `input_cdsm.npy`, `input_tdsm.npy` e
`input_params.npz` **già presenti nei fissaggi** e chiama `svfForProcessing153`: quello è riproducibile
con il solo repository più il pacchetto `umep`. `[P]` Invece `generate_shadow_fixtures` e
`generate_input_fixtures` costruiscono un `SolweigRunCore` da
`tests/rustalgos/test_config_shadows.ini`, che punta a `demos/data/Athens/DSM.tif`,
`temp/athens/CDSM.tif`, `temp/athens/walls/wall_hts.tif`, `temp/athens/svf/svfs.zip`. `[P]` La cartella
`temp/` **non esiste nel repository** (è ignorata da `.gitignore:35-40`, resta solo un `.gitkeep`), e il
percorso è scritto `demos/data/Athens/` con la maiuscola mentre la cartella vera è `demos/data/athens/`:
su un filesystem sensibile alle maiuscole quel percorso non si apre. `[I]` per la conseguenza,
`[P]` per le due premesse. Conclusione operativa: **il generatore non è eseguibile da un clone pulito
senza un passo di preparazione non documentato**, e i fissaggi degli ingressi vanno presi come sono.

Nota accessoria: il `REPORT.md` dei golden dichiara «Test data: Gothenburg demo site (234 × 223
pixels)», mentre i fissaggi in git sono 400 × 400 e il generatore punta ad Atene. Uno dei due è vecchio.
`[P]` per entrambe le letture, `[I]` per la contraddizione.

### 2.5 Quanti test sono

`tests/golden/README.md` dice **146**, `tests/golden/REPORT.md` dice **130**. Il conteggio vero
richiede `pytest --collect-only`, perché molti test sono parametrizzati. `[V]`

---

## 3. `tests/spec/` — proprietà fisiche e le quattro parità

22 file di test, 5 143 righe contando l'`__init__.py`. Due nature diverse mescolate nella stessa cartella.

**Proprietà fisiche su dati sintetici** (`test_shadows.py`, `test_svf.py`, `test_utci.py`,
`test_pet.py`, `test_gvf_properties.py`, `test_low_sun_angles.py`, `test_radiation_formulas.py`,
`test_clearness_index.py`, …). Verificano invarianti dichiarati in `specs/*.md`: SVF in `[0, 1]`,
terreno piatto ha SVF = 1, nessuna ombra con sole sotto l'orizzonte, edifici più alti fanno ombre più
lunghe. Non servono fissaggi, non serve UMEP. `[P]` — `tests/README.md:26-50`.

**Parità contro UMEP Python.** Quattro file, e sono la parte interessante:

| File | Contro cosa | Tolleranza | Righe |
|---|---|---|---|
| `test_umep_parity.py` | pip `umep`: `create_patches`, `patch_steradians` | esatta / `1e-6, 1e-8` | 95 |
| `test_perez_parity.py` | pip `umep`: `Perez_v3` | `atol=0.01` sulla luminanza, `rtol=1e-3, atol=1e-6` sulla somma; steradianti `1e-4, 1e-6` | 212 |
| `test_parity_extended.py` | pip `umep`: `findwalls`, Goodwin, `Lside_veg_v2022a` | esatta / strutturale / `1e-3, 1e-3` | 211 |
| `test_parity_2026a.py` | copia Python vendorata in `tests/reference/umep_2026a/` | `1e-3, 1e-2` (Lside), `1e-3, 0.05` (Tg), `1e-5, 1e-4` (init) | 489 |

`[P]` — righe citate per file nel § 4 e nel § 3.2.

### 3.1 `tests/reference/umep_2026a/` — la copia Python vendorata

Quattro moduli più `__init__.py`, 88 KB: `Lside_veg.py`, `Lvikt_veg.py`, `ground_surface.py`,
`Solweig_2026a_calc_forprocessing.py`. Ogni file apre con un blocco di provenienza esplicito: sorgente
`https://github.com/UMEP-dev/UMEP-processing`, percorso originale, ramo `main` preso il 2026-07-06,
commit a monte citati per nome, **licenza GPL-3.0**, e la ragione: il pacchetto pip `umep` distribuisce
solo la generazione 2025a, quindi il 2026a va vendorato finché a monte non pubblicano una release.
`[P]` — `tests/reference/umep_2026a/Lside_veg.py:1-15`. Il codice sotto il blocco è tenuto **verbatim**,
e `pyproject.toml:130` esclude `tests/reference` da ruff per questo.

Il modello è lo stesso che CLIMESH usa per `vendor/solweig/`: file di altri, non formattati, con la
provenienza scritta accanto. Loro lo fanno per il codice Python di riferimento nei test; noi per il
codice Rust in produzione.

### 3.2 Cosa dicono, in dettaglio

- **`findwalls`**: uguaglianza **esatta** (`rtol=0, atol=0`), è un port Python-su-Python.
  `test_parity_extended.py:72-73`.
- **Goodwin (aspetto dei muri)**: parità **solo strutturale** — «stessi pixel di muro identificati», non
  l'angolo per pixel. Il docstring lo motiva: il kernel `f32` diverge fino a ~90° sui pixel d'angolo
  dove il filtro ha candidati quasi pari, e la deriva «si lava via nell'aggregazione a valle;
  la validazione di campo di Göteborg passa invariata, che è il cancello di correttezza autorevole».
  `test_parity_extended.py:10-17`. **Questa frase è la chiave di lettura di tutto il ticket**: a monte
  stessi loro considerano le misure di campo il giudice, e la parità numerica un mezzo.
- **`lside_veg`**: `rtol=1e-3, atol=1e-3`, e il docstring dice perché stretta — «un cambio di costante
  qui storce silenziosamente la radiazione a onda lunga dalle quattro direzioni cardinali».
  `test_parity_extended.py:18-21, 204-205`.
- **Perez**: due liste di casi. `PEREZ_CASES` (8 condizioni atmosferiche) dove Rust e UMEP **devono**
  coincidere, e `PEREZ_EDGE_CASES` dove il Rust **diverge di proposito**, perché il port ha guardie per
  sole basso e diffusa bassa che a monte non ci sono, e senza quelle escono NaN/Inf.
  `test_perez_parity.py:20-40`. Un test asserisce che il Rust restituisca uniforme, un altro che UMEP
  *non* lo faccia: la divergenza è pinnata, non tollerata.

---

## 4. `tests/spec/test_umep_parity.py`, parola per parola

Il ticket chiede «cosa afferma esattamente». 95 righe, e afferma molto meno di quanto il nome
suggerisca. `[P]` —
[file intero](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/tests/spec/test_umep_parity.py).

Il docstring dichiara l'ambito: «moduli tenuti localmente (perché il codice di produzione non può
dipendere da UMEP) sono validati qui contro le implementazioni UMEP a monte». I moduli sono due:
`physics/create_patches.py` e `physics/patch_radiation.py`. `[P]` — righe 1-10.

Tre `pytest.importorskip` in testa (righe 17-29): senza il pacchetto `umep` installato **l'intero
modulo si salta**, silenziosamente, con `reason="UMEP package required for parity tests"`. `[P]`
Corollario non scritto: in un ambiente senza `umep` questo file passa senza eseguire nulla.

Le asserzioni:

1. `TestCreatePatchesParity::test_all_outputs_match` — per `patch_option` 1, 2, 3, 4, tutti e sette i
   valori di ritorno (`skyvaultalt`, `skyvaultazi`, `annulino`, `skyvaultaltint`, `patches_in_band`,
   `skyvaultaziint`, `azistart`) devono essere **identici** via `assert_array_equal`, nessuna
   tolleranza. Righe 34-61.
2. `test_patch_count` — opzione 1 → **145** patch, opzione 2 → **153**. Righe 61-65.
3. `TestPatchSteradiansParity::test_steradians_match` — per `patch_option` 1, 2, 3: si genera `lv` con
   il `Perez_v3` **di UMEP**, poi steradianti locali contro steradianti UMEP a
   `rtol=1e-6, atol=1e-8`. Righe 68-86.
4. `test_steradians_sum_to_2pi` — la somma degli steradianti vale `2π` entro `rtol=0.05`. Righe 88-95.

**Cosa non c'è dentro**: nessun SVF, nessuna ombra, nessuna Tmrt, nessun UTCI, nessun raster. È il
cancello sulla **discretizzazione del cielo**, che serve al cielo anisotropo. Che `PRINCIPLES.md` lo
citi come uno dei due pilastri del contratto scientifico
([`PRINCIPLES.md:28`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/PRINCIPLES.md)
e righe 76-81) è vero ma va ridimensionato: l'altro pilastro, i golden, è quello che copre la fisica.

---

## 5. I dati di Göteborg: dove, quanto, con quale licenza

### 5.1 Dove stanno

**Dentro il repository**, sotto `tests/validation/`, 3,8 MB, 51 file tracciati. `[P]` Tre siti:

| Sito | Cartella | Contenuto | Periodo | Risoluzione |
|---|---|---|---|---|
| Kronenhuset | `tests/validation/kronenhuset/` | `DSM_KR.tif`, `DEM_KR.tif`, `CDSM_KR.asc`, `landcover_KR.tif`, `MetFile_Prepared.txt`, `measurements_kr.csv` (2,1 KB), `poi.geojson`, `parametersforsolweig_KR.json` | 2005-10-07, 12 ore | 1 m, EPSG:3007 |
| Gustav Adolfs torg | `tests/validation/gustav_adolfs/` | come sopra più `wall_height_GA.tif`, `wall_aspect_GA.tif`, tre `MetFile_*.txt` | 2005-10-11, 2006-07-26, 2006-08-01, 43 ore | 2 m, EPSG:3006 |
| GVC | `tests/validation/gvc/` | come Gustav Adolfs, file `*_GVC_1m.tif` | 2010-07-07, 07-10, 07-12, 30 ore | 2 m, EPSG:3006 (i file dicono `1m` ma sono 2 m) |

`[P]` — `VALIDATION.md:54-127`. Il singolo file più grosso è `DEM_KR.tif`, 210 636 byte.

Le stesse basi geometriche di Kronenhuset esistono **anche** in `demos/data/Goteborg_SWEREF99_1200/`
(2,7 MB, con i `_KRbig`): il docstring del test di Kronenhuset dice ancora che la geodata «viene dalla
cartella demo», ma il gate del test è `_geodata_present = KR_DIR.exists() and (KR_DIR /
"DSM_KR.tif").exists()`, cioè guarda dentro `tests/validation/`.
`[P]` — `tests/validation/test_validation_kronenhuset.py:14-18` contro `:135`. Il docstring è vecchio.

### 5.2 Licenza

**GPL-3.0**, la stessa del repository: `pyproject.toml:7` (`license = { text = "GPL-3.0" }`),
`LICENSE` è il testo GPL v3, `README.md:419-421` lo conferma, `CITATION.cff:18` pure. **Non esiste una
licenza separata per i dati**, né un file di licenza dentro `tests/validation/`. `[P]`

L'unico documento di provenienza dei dati è
[`demos/data/Goteborg_SWEREF99_1200/CITATION.md`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/demos/data/Goteborg_SWEREF99_1200/CITATION.md):
dice che DSM, DEM, CDSM e uso del suolo «originano dal progetto UMEP e sono stati usati nello studio di
validazione originale di SOLWEIG», e chiede di citare Lindberg, Holmer & Thorsson (2008),
doi:10.1007/s00484-008-0162-7, più il paper UMEP, doi:10.1016/j.envsoft.2017.09.020. `[P]`
Nessuna cartella analoga esiste sotto `tests/validation/`: quel `CITATION.md` copre la geodata di
Kronenhuset, non i CSV di misura degli altri due siti. `[I]`

CLIMESH è anch'esso GPL-3.0 (`LICENSE`), quindi **la compatibilità non è un problema**; l'attribuzione
sì, e va scritta dove i dati verranno usati.

### 5.3 Cosa confrontano e con quali soglie

Le misure sono orarie al singolo punto di osservazione. Kronenhuset è l'unico sito che porta i
**componenti del bilancio radiativo** e non solo la Tmrt: `Kdown, Kup, Kn, Ke, Ks, Kw, Ldown, Lup, Ln,
Le, Ls, Lw, Sstr, Tmrt, Ta, RH`. `[P]` — `test_validation_kronenhuset.py:60-63`.

Soglie asserite nel codice — **larghe**, e dichiaratamente tali: `[P]`

| Sito | Grandezza | Soglia | Riga |
|---|---|---:|---|
| Kronenhuset | Tmrt RMSE | < 15,0 °C | `test_validation_kronenhuset.py:406` |
| Kronenhuset | Kdown RMSE | < 80 W/m² | `:562` |
| Kronenhuset | Ldown RMSE | < 65 W/m² | `:565` |
| Kronenhuset | Lup RMSE | < 40 W/m² | `:568` |
| Gustav Adolfs | Tmrt RMSE | < 20,0 °C | `test_validation_gustav_adolfs.py:274` |
| Gustav Adolfs | Kdown / Ldown RMSE | < 350 / < 85 W/m² | `:375, :378` |
| GVC | Tmrt RMSE | < 17,0 °C | `test_validation_gvc.py:257` |
| GVC | Kdown / Ldown RMSE | < 350 / < 75 W/m² | `:342, :345` |

I valori **effettivamente ottenuti** sono molto più stretti delle soglie, e stanno in `VALIDATION.md:33-42`:
Tmrt RMSE 2,4–7,3 °C, R² 0,52–0,99, bias da −1,4 a +5,8 °C, su 31 test che passano. `[P]` Le soglie
sono cancelli anti-regressione, non una misura di qualità: la qualità è nella tabella.

Due limiti noti che il documento dichiara e che **CLIMESH erediterà se riusa lo stesso metodo**: `[P]`

1. **Bias positivo sistematico su Ldown**, da +18 a +55 W/m² a ogni sito. Non è taratura: la
   formulazione di Jonsson et al. (2006) riempie l'emisfero non-cielo con emissione di muro a
   emissività 0,90 e temperatura dell'aria, e i muri in ombra sono più freddi dell'aria. La correzione
   empirica di −25 W/m² «è presente ma commentata in tutte le release UMEP (2021a, 2022a, 2025a) e non
   è applicata qui». `VALIDATION.md:140-157`.
2. **Sensibilità al bordo d'ombra**: al singolo pixel lo stato sole/ombra è binario, quindi uno
   spostamento minimo del bordo produce 600–800 W/m² di differenza fra due ore adiacenti.
   `VALIDATION.md:131-138`. È il motivo per cui la validazione punto-a-punto ha soglie larghe.

Terzo dato, utile a chi progetta la nostra catena: lo **schema del suolo 2026a**, opzionale e
disattivato di default, **peggiora** la Tmrt in modo drastico contro le stesse misure — RMSE da 6,6 a
22,0 °C a Kronenhuset, bias da +2,6 a +21,4 °C. `VALIDATION.md` § «UMEP 2026a ground scheme». `[P]`
Che a monte lo tengano spento di default è un'informazione che vale un ticket, non una nota a piè di
pagina.

---

## 6. Cosa serve per eseguirli

### 6.1 Ambiente

`[P]` — `pyproject.toml`, `.github/workflows/test.yml`, `tests/conftest.py`.

- **Python** `>=3.11, <3.14`. La CI gira la matrice 3.11 / 3.12 / 3.13 su `ubuntu-latest`.
- **Toolchain Rust** (`dtolnay/rust-toolchain@stable`) e **`maturin develop --release`**: obbligatorio,
  non aggirabile. `tests/conftest.py:16-28` fa `import solweig` dentro `pytest_configure` e **esce con
  `returncode=1`** se l'estensione è stata compilata in debug («5-20x più lente»).
- **Gestore**: `uv`. `uv sync --group test` per i test, `--group dev` per lint e grafici.
- **Il gruppo `test`** è: `pytest>=7.2.0`, `pandas-stubs`, `memory-profiler`, **`umep>=0.0.1a18`**,
  `scipy>=1.13.0`. Il lock risolve `umep` a **0.0.1b32**, che tira dietro `geopandas`, `matplotlib`,
  `numba`, `pandas`, `pvlib`, `rasterio`, `rioxarray`, `scipy`, `shapely`, `xarray`.
  `uv.lock`, blocco `name = "umep"`. Non è un ambiente leggero.
- **Runtime della libreria**: `numpy>=1.26`, `pyproj`, `shapely`, `rasterio`, `tqdm`, `pillow`.

### 6.2 QGIS: no

Nessuno dei nove job in `.github/workflows/test.yml` installa QGIS. `[P]` I test del plugin importano
`tests/qgis_mocks.py`, che dichiara in testa «Shared QGIS mock setup for testing plugin code without a
QGIS installation». `[P]` — `tests/qgis_mocks.py:1-5`. L'unico job che tocca il mondo geospaziale di
sistema è `test-qgis-compat`, e installa **GDAL** (`libgdal-dev gdal-bin python3-gdal`) più
`numpy 1.26`, per provare il backend alternativo — non QGIS.
`[P]` — `.github/workflows/test.yml:103-152`.

### 6.3 GPU: no

Tutti i test GPU si autoescludono. `tests/spec/test_gpu_cpu_parity.py:106-108` — «Skips if
`solweig.is_gpu_available()` is False (CI / no-GPU host)»; `test_aniso_gpu_parity.py` usa una fixture
`gpu_available` e ritorna presto; `tests/conftest.py:66-75` offre una fixture `cpu_only` che spegne la
GPU e la riaccende dopo. `[P]` La feature Cargo `gpu` è **di default** (`rust/Cargo.toml:25-26`, e tira
`wgpu 27.0`, `pollster`, `bytemuck`), quindi si compila comunque; se poi compilare `wgpu` richieda
librerie di sistema su una macchina spoglia è `[V]`.

C'è anche un interruttore d'ambiente, `SOLWEIG_NO_GPU=1`, con il suo test di regressione
(`tests/test_gpu_env_gate.py:1-12`). `[P]`

### 6.4 Come si lanciano

`[P]` — `tests/README.md:107-133`, `pyproject.toml` `[tool.poe.tasks]`, `.github/workflows/test.yml`.

```
uv run pytest tests/spec/ -v          # proprietà e parità
uv run pytest tests/golden/ -v        # regressione contro fissaggi
uv run pytest tests/ -m 'not slow'    # quello che gira in CI
```

I marcatori dichiarati in `pyproject.toml`: `slow` (calcolo SOLWEIG completo), `validation` (dataset
esterni), `gpu_perf_gate` (soglie tarate sull'hardware del manutentore, escluse dalla CI). I test di
validazione portano `pytest.mark.validation` a livello di modulo e `pytest.mark.slow` sui test di
pipeline; **il job `validation` della CI li lancia senza `-m`, quindi anche i lenti**.
`.github/workflows/test.yml:154-194`.

**Tre moduli golden non girano in CI, al commit pinnato.** `test_golden_svf.py:24`,
`test_golden_gvf.py:24` e `test_golden_wall_geometry.py:14` portano
`pytestmark = pytest.mark.slow` a livello di modulo; il job `test` gira `-m 'not slow'`, e **nessun job
nomina `tests/golden/`** (verificato con `grep -n "tests/golden" .github/workflows/*.yml`: l'unica
occorrenza è dentro un commento, a `test.yml:313`, che dice che i golden «hanno il loro job» — non ce
l'hanno). Gli altri nove moduli golden girano, di rimbalzo, perché il job `test` colleziona `tests/`
intera. `[P]` per le righe e per il grep, `[I]` per la conclusione. Detto altrimenti: **il cancello sul
SVF a monte non è verde, è spento.**

### 6.5 Tempo

`tests/README.md:110` dichiara «tutti gli spec test, veloci, ~10 secondi». Nient'altro è dichiarato, e
i tempi di CI non si leggono dal repository. Tempo della suite golden, della validazione e della
compilazione Rust: `[V]`.

### 6.6 Peso del clone

Un `git clone` completo costa molto più dei test: `docs/tutorials/` da solo pesa **1,3 GB** (notebook
con uscite incorporate), l'albero di lavoro 1,4 GB, `.git` altri 497 MB. `[P]` I test sono 42 MB. Se
serviranno i fissaggi o i dati di Göteborg, un checkout sparso di `tests/` più `demos/data/athens/` è
il modo di non tirarsi in casa un gigabyte e mezzo. `[I]`

---

## 7. Cosa di tutto questo è un cancello per CLIMESH, e cosa no

**Il problema, riformulato.** `vendor/solweig/PROVENIENZA.toml` dichiara tre file presi interi:
`shadowing.rs`, `skyview.rs`, `utci.rs`. Confrontare il nostro `calculate_svf_inner`
(`vendor/solweig/src/skyview.rs:287`) con il loro `skyview.rs` confronta una funzione con sé stessa. Lo
stesso vale per le ombre e per il polinomio UTCI. Quei tre non hanno bisogno di un test di parità:
hanno bisogno del `Vendor check`, che già esiste e confronta byte per byte.

Riordinati per quanto valgono come giudice indipendente:

| Riferimento | Indipendente dal nostro Rust? | Costo | Copre |
|---|---|---|---|
| **Pacchetto pip `umep`** (0.0.1b32) | **Sì**, implementazione Python separata | Pesante: numba, geopandas, rioxarray, xarray | SVF, ombre, GVF, Kside/Lside, UTCI, PET, Tg |
| **`tests/reference/umep_2026a/`** (88 KB, GPL-3.0) | **Sì**, Python verbatim da UMEP-processing | Nullo: quattro file, numpy e matplotlib | Lside 2026a, schema del suolo, onda lunga uscente |
| **Misure di campo di Göteborg** (3,8 MB) | **Sì**, sono osservazioni | Serve la catena intera | Tmrt e quattro componenti al punto |
| **Fissaggi golden da UMEP Python** (`svf_*`, `shadow_*`, `gvf_*`, `radiation_*`, `utci_*`, `pet_*`, `ground_temp_*`) | **Sì**, ma congelati: valgono quanto valeva UMEP il giorno della generazione | 37 MB, non copiabili da noi | come sopra, su un caso 400 × 400 |
| **Fissaggi golden da Rust** (`aniso_sky_*`, `wall_temp_*`, `radiation_aniso_*`) | **No** | — | nulla, per noi |
| **`tests/spec/test_umep_parity.py`** | Sì, ma copre solo la discretizzazione del cielo | Serve `umep` | `create_patches`, `patch_steradians` |
| **Proprietà fisiche di `tests/spec/`** | **Sì**: non confrontano, verificano invarianti | Nullo, si riscrivono in Rust | SVF ∈ [0,1], niente ombre con sole sotto l'orizzonte, ecc. |

**Raccomandazione — è una raccomandazione, non una decisione presa.** Se #22 deve scegliere un metro
per la parità per componente:

1. **Il metro primario è il pacchetto pip `umep`**, non il repository `UMEP-dev/solweig`. È l'unico
   riferimento eseguibile, indipendente dal codice che abbiamo copiato, e già installabile.
2. **Il vettore d'ingresso può essere il loro**, recuperato al commit pinnato e mai copiato da noi:
   `input_dsm.npy` + `input_cdsm.npy` + `input_tdsm.npy` + `input_params.npz`, 400 × 400, ~1,9 MB in
   tutto. È il caso su cui i loro numeri sono già pubblicati in `tests/golden/REPORT.md` (SVF totale:
   differenza massima 2,09e-06), quindi un nostro risultato fuori da quell'ordine di grandezza è un
   segnale, non rumore.
3. **Due trappole da portare dentro #22**, entrambe già costate a monte:
   (a) `svfForProcessing153` vuole altezze di vegetazione **relative** al DSM, il Rust le vuole
   **assolute** — `generate_fixtures.py:119-129` fa la conversione a mano e la commenta in maiuscolo;
   (b) SVF vegetato diverge dello 0,17 in assoluto perché a monte il fissaggio nasce da
   `shadowingfunction_20` e il Rust usa `wallheight_23`: **quel confronto non è like-for-like nemmeno
   per loro**, e il loro `TODO` è aperto.
4. **Il cancello che vale davvero, a monte come da noi, sono le misure di campo.** Lo dicono loro, in un
   commento che non era scritto per essere citato: la parità strutturale su Goodwin si accetta perché
   «la validazione di campo di Göteborg passa invariata, che è il cancello di correttezza autorevole
   per l'output scientifico» (`test_parity_extended.py:15-17`). Il ticket
   [#14](https://github.com/maeurong/CLIMESH/issues/14) sta già lì.

---

## 8. Derive trovate nella loro documentazione

Da tenere presenti da parte di chi leggerà quei documenti dopo di me: `[P]`

1. `tests/README.md:76-95` manda a `tests/rustalgos/test_rustalgos.py`, che non esiste.
2. `tests/golden/README.md` e `REPORT.md` portano tolleranze che non corrispondono al codice
   (§ 2.3).
3. Conteggio dei golden: 146 nel README, 130 nel REPORT.
4. `REPORT.md` dice «Gothenburg 234 × 223», i fissaggi sono 400 × 400 e il generatore punta ad Atene.
5. `test_validation_kronenhuset.py:16-19` dice che la geodata viene da `demos/data/…`, il gate del
   test guarda `tests/validation/kronenhuset/`.
6. `tests/rustalgos/test_config_shadows.ini` punta a `demos/data/Athens/` (maiuscola) e a `temp/…`,
   che nel repository non c'è.

Nessuna di queste tocca il codice sotto test. Tutte toccano chi prova a rieseguirlo.

---

## 9. Cosa resta da verificare eseguendo

- Il conteggio reale dei test per cartella (`pytest --collect-only`). `[V]`
- Il tempo della suite golden, della validazione, e della compilazione `maturin develop --release`. `[V]`
- I valori di `scale` e `amaxvalue` in `input_params.npz`, e la forma esatta in `svf_metadata.npz`.
  Servono numpy. `[V]`
- Se `wgpu 27.0` compili su una macchina Linux spoglia senza pacchetti di sviluppo grafici. `[V]`
- Se il job `validation` della CI passi davvero con il solo gruppo `test`: `test_timeseries_plots.py`
  importa `matplotlib` a riga 133 **senza** `importorskip`, e `matplotlib` non è nel gruppo `test` —
  ci arriva solo di rimbalzo, come dipendenza di `umep`. Regge, ma per caso. `[I]`
