# SOLWEIG / UMEP — quanto è riusabile per CLIMESH e quanto costa

**Scopo**: rispondere al ticket [#2](https://github.com/maeurong/CLIMESH/issues/2) — quanto del modello radiativo di SOLWEIG è riusabile per CLIMESH, e quanto costa farlo girare sul caso di riferimento (Casa Evolutiva, Bastia Umbra).
**Data ricerca**: 2026-08-31. **Repo**: CLIMESH, branch `main`, HEAD `7f23d42` (`README.md` contiene solo `# CLIMESH`; unico altro documento `research/envi-met.md`, 527 righe).

**Convenzione di marcatura** (la stessa di `research/envi-met.md`)
- `[P]` = confermato da fonte primaria (codice sorgente del progetto, documentazione ufficiale UMEP, testo delle licenze, metadati PyPI/GitHub).
- `[M]` = **misurato da me in questa sessione**, con comando e macchina dichiarati in § 5.1.
- `[S]` = fonte secondaria.
- `[I]` = inferenza mia, non confermata da fonte.

---

## 0. Risposta in breve

Tre fatti cambiano il quadro rispetto a come il ticket era stato formulato.

1. **Esiste già una reimplementazione di SOLWEIG in Rust**, dentro l'organizzazione UMEP ufficiale: [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig), GPL-3.0, ~17,6 k righe di Rust + 1,3 k di WGSL (shader GPU) + ~17,8 k righe di Python di orchestrazione, con binding PyO3 e accelerazione GPU opzionale via `wgpu`. È pubblicata su PyPI come `solweig` 0.1.0b95 (25/08/2026), con wheel per Linux, macOS (x86_64 e arm64) e Windows. `[P]`
2. **Il codice di calcolo di SOLWEIG non dipende da QGIS** — mai dipeso, nella sostanza. Il modulo di calcolo di riferimento importa solo `numpy` e moduli fratelli; QGIS sta nel layer di interfaccia. Esistono oggi tre estrazioni QGIS-free ufficiali di UMEP, di cui una attiva e mantenuta. `[P]`
3. **Il costo sul caso di riferimento non è un problema**: ho ricostruito la geometria di Bastia dal file `LAB1.INX` (50×50 celle a 1 m, 616 piante) e ho eseguito 48 ore di simulazione con il pacchetto `solweig`. **1,7 s con GPU, 36,4 s su CPU sola.** `[M]` Il budget di progetto è 60 s per il caso completo: rientra con GPU con tre ordini di grandezza di margine, e **non rientra** su CPU sola quando si contano tutti e quattro i run del caso completo (§ 5.4).

Conseguenza per la scelta fra "riusare / reimplementare / riusare l'impianto concettuale": la domanda non è più *se* SOLWEIG sia portabile in Rust — **qualcuno l'ha già portato, sotto la stessa licenza che CLIMESH ha scelto**. La domanda diventa quale forma di riuso costi meno (§ 7).

---

## 1. Quali algoritmi usa SOLWEIG

SOLWEIG = *Solar and Longwave Environmental Irradiance Geometry*. Modello **2.5D**: opera su raster di altezza, non su una griglia volumetrica; calcola i flussi radiativi e la Tmrt **all'altezza del pedone**, non in un volume 3D. `[P]` — [`specs/OVERVIEW.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/OVERVIEW.md), [UMEP docs, SOLWEIG](https://umep-docs.readthedocs.io/en/latest/processor/Outdoor%20Thermal%20Comfort%20SOLWEIG.html) ("As SOLWEIG is a 2.5D model, large grids will take a relatively long time to compute").

### 1.1 Pipeline

SVF è **precalcolato una volta** (dipende solo dalla geometria). Poi, per ogni passo temporale: `[P]` — `specs/OVERVIEW.md`

```
ombre → temperatura del suolo → GVF → ritardo termico → radiazione (K↓ K↑ Kside, L↓ L↑ Lside) → Tmrt
```

UTCI e PET sono post-processing separati.

### 1.2 Ombreggiamento

Ray marching sul DSM, un raggio per pixel verso il sole. `[P]` — [`specs/shadows.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/shadows.md)

```
L = h / tan(α)                                # lunghezza d'ombra
sunlit[y,x] = 1 se altezza_propagata <= DSM[y,x]
```

- Ombre binarie, nessuna penombra. `[P]`
- Caso zenit (altitudine ≥ 89,5°) trattato a parte per evitare `tan(90°)`. `[P]`
- Portata massima dell'ombra limitata da `min_sun_elev_deg` (default 3°) e `max_shadow_distance_m` (default 1000 m nel layer Python di tiling). `[P]` — `specs/technical.md`

Origine metodologica dichiarata da UMEP: **Ratti & Richens (1990)**, sviluppata poi in **Lindberg & Grimmond (2011)**, **Lindberg et al. (2016)**, **Wallenberg et al. (2020, 2023)**. `[P]` — [UMEP docs](https://umep-docs.readthedocs.io/en/latest/processor/Outdoor%20Thermal%20Comfort%20SOLWEIG.html)

### 1.3 Sky View Factor

Metodo **patch-based**: emisfero celeste diviso in anelli di altitudine × settori azimutali; SVF = somma dei pesi di angolo solido dei patch visibili. `[P]` — [`specs/svf.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/svf.md)

- Riferimento metodologico: **Robinson & Stone (1990)**, *Solar Radiation Modelling in the Urban Context*, Building and Environment 25(3):201-209. `[P]`
- Quattro configurazioni: 145 / **153 (default)** / 305 / 609 patch. Per 153: 8 bande di altitudine (6°…90°) con 31, 30, 28, 24, 19, 13, 7, 1 suddivisioni azimutali. `[P]`
- Produce **17 raster**: SVF isotropo, 4 direzionali (N/E/S/W), le stesse varianti per la vegetazione, e le varianti "vegetazione che blocca l'ombra dell'edificio", più 3 matrici 3D di ombra per patch bitpacked in uint8 usate dal cielo anisotropo. `[P]`

### 1.4 Onda corta

- **Split diretto/diffuso**: correlazione di **Reindl, Beckman & Duffie (1990)**, Solar Energy 45(1):1-7, su tre regimi di clearness index. Fatto in preprocessing, fuori dal kernel. `[P]` — [`specs/radiation.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/radiation.md)
- **Cielo anisotropo**: modello di **Perez et al. (1993)**, Solar Energy 50(3):235-245 — background isotropo + brillamento circumsolare + brillamento all'orizzonte. `[P]`
- L'impatto della diffusa anisotropa sulla Tmrt è il contenuto di **Wallenberg et al. (2020)**, [doi:10.1016/j.uclim.2020.100589](https://doi.org/10.1016/j.uclim.2020.100589). `[P]` — UMEP docs
- Riflessione dal suolo (K↑) via **GVF (Ground View Factor)** e frazioni soleggiate delle facce urbane — **Lindberg, Grimmond & Martilli (2015)**, *Sunlit fractions on urban facets*, Urban Climate 12:65-84. `[P]`

### 1.5 Onda lunga

- Emissività del cielo: formulazione di **Jonsson et al. (2006)**; modello a bande di **Martin & Berdahl** per l'emissività per patch. `[P]` — `specs/radiation.md`, `rust/src/emissivity_models.rs`, `rust/src/patch_radiation.rs`
- Schema anisotropo per l'onda lunga: **Wallenberg et al. (2023)**, [doi:10.1007/s00484-023-02441-3](https://doi.org/10.1007/s00484-023-02441-3). `[P]` — UMEP docs
- L↑ dal suolo tramite GVF; L da pareti tramite modello sinusoidale di temperatura di parete. `[P]`

### 1.6 Temperatura superficiale

Non è un bilancio energetico completo: è una **parametrizzazione con ritardo termico esponenziale** (`TsWaveDelay`). `[P]` — [`specs/ground_temperature.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/ground_temperature.md)

```
T_ground(t) = T_current × (1 − w) + T_previous × w,  w = exp(−33,27 × Δt)
```

costante di tempo τ ≈ 43 minuti. Riferimenti: **Lindberg, Onomura & Grimmond (2016)**, Int J Biometeorol 60:1439-1452; **Lindberg & Grimmond (2011)**.

Il documento stesso ammette che la parametrizzazione **non è validata**: "The current parameterization is empirical and may need adjustment for specific climates or surface materials", con una checklist di validazioni ancora tutte da spuntare. `[P]` — `specs/ground_temperature.md`

Esiste uno **schema 2026a opt-in** (force-restore + OHM di **Grimmond, Cleugh & Oke 1991**, con marcia a 20 azimut per l'onda lunga uscente) che sostituisce il GVF; è **spento di default e non ancora validato contro misure**. `[P]`

### 1.7 Tmrt e comfort

```
Sstr = abs_k × (…K…) + abs_l × (…L…)
Tmrt = (Sstr / (abs_l × σ))^0,25 − 273,15,  clampato a [−50, 80] °C
```
con `abs_k = 0,70` e `abs_l = 0,97` da **ISO 7726:1998** Tab. 4. Riferimenti aggiuntivi: **Höppe (1992)**; corpo come **cilindro** anziché parallelepipedo secondo **Holmer et al. (2015)**. `[P]` — [`specs/tmrt.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/tmrt.md), UMEP docs

UTCI: approssimazione polinomiale. PET: solutore iterativo del bilancio energetico umano con parametri corporei configurabili. `[P]` — `specs/utci.md`, `specs/pet.md`

### 1.8 Cosa SOLWEIG non fa

Non calcola temperatura dell'aria, umidità, vento: sono **input** dal file meteo, esattamente come nell'ambito fisico già deciso per CLIMESH nel ticket #1. Non c'è CFD, non c'è griglia verticale, non c'è modello di suolo profondo, non c'è fisiologia vegetale. `[P]` (assenza verificata su `specs/` e sull'albero `rust/src/`)

> `[I]` La sovrapposizione fra l'ambito fisico di SOLWEIG e l'ambito fisico deciso per CLIMESH è **quasi perfetta**. Le differenze rimanenti riguardano l'ergonomia (CLI, binario singolo, import OSM, lettura INX, giornale della corsa), non la fisica.

---

## 2. Quanto è scorporabile da QGIS

### 2.1 Il kernel non importa QGIS

Il modulo di calcolo di riferimento, `functions/SOLWEIGpython/Solweig_2025a_calc_forprocessing.py` in `UMEP-dev/UMEP-processing`, importa **solo `numpy` e moduli fratelli del pacchetto**; nessun `qgis`, nessun `PyQt`. `[P]` (verificato via `gh api repos/UMEP-dev/UMEP-processing/contents/...`, righe 1-32)

QGIS compare nel layer sopra: `Solweig_run.py` accetta un `feedback` con la nota esplicita "*To communicate with qgis gui. **Set to None if standalone***". `[P]` — `umep-solweig/src/umep_solweig/functions/SOLWEIGpython/Solweig_run.py:62`

### 2.2 Le tre estrazioni QGIS-free esistenti

| Repo | Cos'è | Stato | Licenza dichiarata |
|---|---|---|---|
| [`UMEP-dev/UMEP`](https://github.com/UMEP-dev/UMEP) | plugin QGIS menu-based | attivo (push 27/08/2026) | GPL-3.0 |
| [`UMEP-dev/UMEP-processing`](https://github.com/UMEP-dev/UMEP-processing) | provider QGIS Processing; **implementazione di riferimento** del modello | attivo (push 27/08/2026) | GPL-3.0 |
| [`UMEP-dev/umep-core`](https://github.com/UMEP-dev/umep-core) | port Python senza QGIS ("modified to work without QGIS to facilitate Python workflows") | **archiviato**, esplicitamente "superseded by UMEP-dev/solweig" | GitHub e README dicono GPL-3.0, `pyproject.toml` dice AGPL-3.0, PyPI (`umep` 0.0.1b47) dice AGPL-3.0 |
| [`UMEP-dev/umep-solweig`](https://github.com/UMEP-dev/umep-solweig) | "A python package to run the SOLWEIG algorithm", 111 file `.py` | ultimo commit 03/07/2026; **non presente su PyPI** (HTTP 404 su `pypi.org/pypi/umep-solweig/json`) | GPL-3.0-only |
| [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig) | **reimplementazione Rust + PyO3 + GPU**, con plugin QGIS opzionale in `qgis_plugin/` | attivo, 69 commit dal 01/06/2026, v0.1.0b95 del 25/08/2026 | GPL-3.0 |

`[P]` — tutte le righe sopra da `gh api search/repositories?q=org:UMEP-dev`, dai `README.md`/`pyproject.toml`/`LICENSE` dei rispettivi cloni, e dall'API PyPI.

### 2.3 Prova diretta di uso fuori da QGIS

In questa sessione ho installato `solweig` da PyPI in un venv Python 3.13 **senza alcuna installazione di QGIS**, e ho eseguito un caso completo. Dipendenze installate: `numpy`, `rasterio`, `pyproj`, `shapely`, `pillow`, `pyparsing`, `tqdm`. `[M]`

Quindi la risposta alla domanda "esiste già un uso di SOLWEIG fuori da QGIS?" è: **sì, tre volte, e una di queste è la strada su cui UMEP sta dichiaratamente convergendo.**

### 2.4 Ma non è un binario, ed è una libreria Python

Due limiti concreti rispetto alla destinazione di CLIMESH ("binario Rust singolo senza installazione"): `[P]`

- Il pacchetto `solweig` **non ha entry point da riga di comando** (nessun `[project.scripts]` in `pyproject.toml`): è una libreria, si guida da Python.
- Il crate Rust si chiama `rustalgos`, ha `crate-type = ["cdylib"]` e **non è pubblicato su crates.io** (`crates.io/api/v1/crates/rustalgos` → "crate `rustalgos` does not exist"). Non è quindi consumabile con `cargo add`.

Ingombro: il venv completo pesa 247 MB; il solo `.so` Rust pesa 6,4 MB. `[M]`

---

## 3. Input richiesti e formati

| Input | Obbligatorio | Cos'è | Formato |
|---|---|---|---|
| **DSM** | sì | altezze terreno + edifici (m) | GeoTIFF o array numpy |
| **Location** | sì | lat, lon, offset UTC | ricavabile dal CRS del DSM o dall'EPW |
| **Weather** | sì | Ta, RH, radiazione globale (opz. vento per UTCI/PET) | **EPW nativo**, o costruito a mano |
| **CDSM** | no | altezza chiome (0 dove non c'è vegetazione) | GeoTIFF |
| **TDSM** | no | altezza zona-tronco; se assente si genera al 25 % della chioma | GeoTIFF |
| **DEM** | no | quote del solo terreno, per separare edifici da suolo | GeoTIFF |
| **Land cover** | no | classi di superficie (pavimentato, erba, acqua…) nel formato standard UMEP | GeoTIFF |

`[P]` — `README.md` di `UMEP-dev/solweig` § *Inputs and outputs*; [UMEP docs § Spatial data](https://umep-docs.readthedocs.io/en/latest/processor/Outdoor%20Thermal%20Comfort%20SOLWEIG.html).

Note rilevanti per CLIMESH:

- **L'EPW è letto nativamente** (`Weather.from_epw(path, start=..., end=...)`, parser Python puro). L'EPW di Perugia già presente in `materiale università/` funziona senza conversioni: l'ho usato. `[M]`
- Nella versione QGIS il SVF va prodotto da un preprocessore separato e passato come **zip**; nel pacchetto nuovo `SurfaceData.prepare()` calcola pareti + SVF e li mette in cache su disco con un fingerprint (run "caldi" in ~50 ms). `[P]` — UMEP docs § Spatial data; `docs/getting-started/quick-start.md`
- Output: GeoTIFF per passo temporale (Tmrt, shadow, UTCI, PET, K↓/K↑/L↓/L↑) più griglie aggregate (medie/max/min, ore di sole, ore sopra soglia UTCI) e serie temporali su **POI** puntuali. `[P]`
- Tutti i DSM devono avere **stessa estensione e stesso pixel size**; la documentazione UMEP sconsiglia griglie sopra ~4·10⁶ pixel senza tiling. `[P]` — UMEP docs § Remarks

**Ponte INX → SOLWEIG**: il file `LAB1.INX` contiene le matrici `zTop`, `zBottom`, `buildingNr`, `terrainheight` (tutte `matrix-data` 50×50 in CSV) e 616 blocchi `<3Dplants>` con `rootcell_i/j/k` e `plantID`. Ho costruito DSM e CDSM da quelle matrici con ~30 righe di parsing regex. `[M]` La mappatura INX → (DSM, DEM, CDSM) è quindi **meccanica**, con un'unica lacuna: **l'altezza delle piante non è nel file INX** (sta nel database piante di ENVI-met, `plantID` come `020027` = ".Pine Tree (middle)"). Nel benchmark ho usato 12 m nominali per tutte.

---

## 4. Vegetazione: cosa si perde rispetto al LAD di ENVI-met

| Aspetto | SOLWEIG | ENVI-met |
|---|---|---|
| Rappresentazione | due raster 2.5D: **CDSM** (cima chioma) + **TDSM** (cima zona-tronco) | **LAD** 3D, m²/m³ one-sided, per grid box della chioma |
| Attenuazione della luce | **trasmissività costante**: 0,03 leaf-on, 0,5 leaf-off; periodo leaf-on default giorno 100→300; conifere sempre leaf-on | estinzione dipendente dal profilo LAD, per cella |
| Geometria interna della chioma | euristica **"pergola"**: 4 test canopy/trunk su due passi consecutivi del raggio; `0 < somma < 4` → ombra, `somma = 4` (raggio interamente dentro lo strato di chioma) → nessuna ombra | ray tracing attraverso il volume fogliare |
| Fisiologia | **nessuna**: niente temperatura fogliare, niente resistenza stomatica, niente traspirazione, niente bilancio idrico | bilancio energetico fogliare per cella, Deardorff o A-gs, radici 3D, Darcy |
| Effetto sulla Tmrt | solo via ombra e via SVF vegetato | ombra + raffrescamento evaporativo dell'aria |

`[P]` per la colonna SOLWEIG — [`specs/shadows.md`](https://github.com/UMEP-dev/solweig/blob/main/specs/shadows.md) §§ *Canopy Transmissivity*, *Vegetation Shadow Algorithm*, *Trunk Zone Ratio*; UMEP docs § Spatial data ("Transmissivity … Default value is set to 3 % according to Konarska et al. (2013)").
`[P]` per la colonna ENVI-met — `research/envi-met.md` § 2.5, che cita https://envi-met.info/doku.php?id=kb:lad.

La combinazione in Python è:

```
shadow = bldg_sh − (1 − veg_sh) × (1 − transmissivity)
```

> `[I]` La semplificazione è **coerente con l'ambito già deciso per CLIMESH**. Quello che il LAD compra in più è il raffrescamento evaporativo, cioè un effetto sulla **temperatura dell'aria** — la grandezza che il ticket #1 ha esplicitamente messo fuori (perché le differenze fra scenari, 0,21 °C d'estate, stanno dentro la barra d'errore di ENVI-met). Ai fini della Tmrt, quello che conta della vegetazione è **dove cade l'ombra**, e quello SOLWEIG lo fa con geometria esatta.
> `[I]` Il costo reale della semplificazione, per il caso Bastia, è che le 5 specie distinte (pino, platano, tiglio, pioppo bianco, betulla — 616 istanze) diventano una sola trasmissività e una sola coppia altezza/rapporto-tronco per pianta. Se serve differenziare, il modello lo permette solo variando CDSM/TDSM pixel per pixel, non la trasmissività, che nel pacchetto è **uno scalare per run**.

---

## 5. Quanto costa — misurato

### 5.1 Provenienza delle misure

Tutte le misure sotto vengono da:

- **cwd** `/home/mario/climesh-research` (fuori dal repo CLIMESH: nessun file scritto dentro il progetto oltre a questo report).
- **pacchetto** `solweig` **0.1.0b95** installato da PyPI in `.venv` Python **3.13.15** (`uv pip install solweig`). Clone di riferimento per il codice: `/home/mario/climesh-research/solweig`, HEAD `02246ab71a3a8b127d740dde9640449ee9d558ff` (25/08/2026, tag `v0.1.0b95`).
- **macchina**: Intel Core i7-12700K (20 thread), 7 GB di RAM visibili alla VM, WSL2 su Linux 6.18.33.2; GPU **NVIDIA GeForce RTX 4060 Ti 8 GB** raggiunta da `wgpu` via backend **Vulkan** (`solweig.get_gpu_limits()` → `{'backend': 'Vulkan', ...}`).
- **script**: `/home/mario/climesh-research/bench/bastia_bench.py` (caso Bastia), `bench/scaling.py` (scaling), `bench/veg_isolate.py` (isolamento del costo vegetazione). Comandi: `./.venv/bin/python bench/<script>.py`, e `FORCE_CPU=1 ./.venv/bin/python bench/<script>.py` per la variante CPU (che chiama `solweig.disable_gpu()`).
- **input**: geometria estratta da `materiale università/LAB1.INX` (matrici `zTop` 50×50, max 6,0 m, 280 celle-edificio; `terrainheight` piatto a 0; 616 piante da `<3Dplants>` a 12 m nominali, tronco al 25 % di default); meteo da `materiale università/ITA_Perugia.161810_IGDG.epw`, 48 passi orari 15-16 luglio; `Location(latitude=43.07, longitude=12.56, utc_offset=1)`.

**Ogni misura è una singola esecuzione, senza ripetizioni né statistica.** La macchina è un desktop, non il laptop-tipo dell'utente primario.

### 5.2 Caso di riferimento Bastia — 50×50 celle a 1 m, 48 h, con 616 alberi

| | `prepare()` (pareti + SVF) | 48 passi | totale | per passo |
|---|---|---|---|---|
| **GPU (Vulkan, RTX 4060 Ti)** | 0,24 s | 1,48 s | **1,73 s** | 0,031 s |
| **CPU sola (20 thread)** | 23,0 s | 5,84 s | **28,9 s** | 0,122 s |

`[M]` Tmrt media 27,7 °C, massima 61,0 °C (coerenti con un 15 luglio umbro; **non validate**, la geometria è approssimata).

Una seconda esecuzione dello stesso script su CPU, con DSM senza terreno sommato e chiome a 12 m (`bench/veg_isolate.py`), dà `prepare` 29,2 s + calcolo 7,2 s = **36,4 s**: la dispersione fra run CPU è del 20-25 %. `[M]`

### 5.3 Dove va il tempo: il SVF con vegetazione

Stesso dominio 50×50, 48 passi, **solo CPU**: `[M]`

| | `prepare()` | 48 passi | totale |
|---|---|---|---|
| senza vegetazione | 8,1 s | 3,5 s | 11,6 s |
| con 616 alberi | 29,2 s | 7,2 s | 36,4 s |

**La vegetazione triplica il costo del precalcolo SVF e raddoppia il costo per passo.** Su GPU lo stesso precalcolo costa 0,24 s: è il singolo punto in cui l'accelerazione rende di più (~×100).

> `[I]` 8 s di SVF su CPU per soli 2500 pixel non sono spiegabili con il numero di pixel: il costo è dominato dai 153 patch × la marcia dei raggi con portata massima ampia. Il che significa che **il SVF ha un costo quasi fisso finché il dominio è piccolo**, e diventa proporzionale ai pixel solo più su (§ 5.4 lo conferma: da 100 a 400 celle di lato, cioè ×16 pixel, il `prepare` CPU passa da 4,6 a 7,6 s).

### 5.4 Scaling e tetto di progetto — 24 passi

Città sintetica ottenuta piastrellando la geometria di Bastia, senza vegetazione: `[M]`

| dominio | pixel size | GPU prepare | GPU calc | GPU tot | CPU prepare | CPU calc | CPU tot |
|---|---|---|---|---|---|---|---|
| 100×100 celle (**200×200 m a 2 m — tetto di progetto**) | 2 m | 0,26 s | 1,00 s | **1,27 s** | 4,58 s | 1,97 s | **6,55 s** |
| 200×200 celle | 2 m | 0,22 s | 1,39 s | 1,60 s | 6,18 s | 2,83 s | 9,01 s |
| 400×400 celle | 2 m | 0,45 s | 2,78 s | 3,22 s | 7,60 s | 6,56 s | 14,2 s |
| 1000×1000 celle (1 Mpixel) | 1 m | 2,92 s | 16,0 s | 18,9 s | 15,8 s | 42,9 s | **58,7 s** |

**Il tetto di progetto (200×200 m a 2 m, 24 h, sotto 10 minuti) è superato di due ordini di grandezza**: 1,3 s su GPU, 6,6 s su CPU. `[M]`

I "performance target" dichiarati nel repo (ombre < 1 s/Mpixel, SVF < 30 s/Mpixel, giornata intera < 5 min/Mpixel, tutti su GPU) sono coerenti con quanto misurato: 1 Mpixel × 24 passi in 18,9 s su GPU. `[P]` per i target — `specs/technical.md` § Performance Targets; `[M]` per il confronto.

### 5.5 Il caso Bastia *completo* e il budget di 60 s

Il ticket #1 definisce il caso completo come **2 scenari × 2 giorni da 48 h**. Con la stessa geometria: `[I]` (aritmetica sulle misure, non misurato come run unico)

- **con GPU**: 2 × `prepare` (0,24 s) + 4 × 48 passi (1,5 s) ≈ **6,5 s** → dentro il budget, largamente.
- **CPU sola**: 2 × `prepare` (29 s) + 4 × 48 passi (7,2 s) ≈ **87 s** → **fuori dal budget di 60 s**, su un desktop a 20 thread. Su un laptop da studente sarebbe peggio.

> Questo è il risultato operativamente più importante della ricerca. **La fisica di SOLWEIG entra nel budget; il precalcolo SVF con vegetazione su CPU sola no.** O il SVF vegetato si accelera (GPU, o algoritmo diverso, o cache su disco fra scenari che condividono la vegetazione), oppure il vincolo dei 60 s su laptop senza GPU non è raggiungibile con questo schema. `[I]`
> `[I]` Nota mitigante: i due scenari di Bastia condividono gran parte della geometria, e il SVF è cacheable su fingerprint (funzione già presente nel pacchetto). Se cambia solo la vegetazione fra i due scenari, il ricalcolo è comunque totale.

---

## 6. Licenza

### 6.1 Cosa dicono le fonti

- `UMEP-dev/solweig`: file `LICENSE` = testo integrale della **GNU GPL versione 3**; `pyproject.toml` → `license = { text = "GPL-3.0" }`; metadato PyPI → `GPL-3.0`; `CITATION.cff` → `license: GPL-3.0`, autori **Fredrik Lindberg** (Univ. Göteborg), **C. Sue B. Grimmond** (Univ. Reading), UMEP Developers. `[P]`
- `UMEP`, `UMEP-processing`, `umep-solweig`: GPL-3.0 (`umep-solweig` dichiara esplicitamente `GPL-3.0-only`). `[P]`
- `umep-core`: **incoerente** — LICENSE e README dicono GPL-3.0, `pyproject.toml` e PyPI dicono AGPL-3.0. Repo archiviato; irrilevante se non lo si usa. `[P]`

### 6.2 Cosa comporta per CLIMESH

CLIMESH ha già deciso GPL-3 o AGPL-3 (ticket #1). Questo elimina il problema alla radice: **il copyleft di SOLWEIG non vincola nulla che CLIMESH non si sia già imposto da solo.**

- CLIMESH sotto **GPL-3**: combinazione diretta ammessa, nessuna condizione aggiuntiva oltre a mantenere le note di copyright e distribuire il sorgente.
- CLIMESH sotto **AGPL-3**: ammesso esplicitamente dalla **§ 13 della GPLv3** — *"Notwithstanding any other provision of this License, you have permission to link or combine any covered work with a work licensed under version 3 of the GNU Affero General Public License into a single combined work, and to convey the resulting work"*. `[P]` — https://www.gnu.org/licenses/gpl-3.0.txt, riga 552 e segg. Il "GPL-3.0-only" di `umep-solweig` **non** è un ostacolo: la § 13 è dentro la GPLv3 stessa, non è una clausola "or later".

### 6.3 Le forme di riuso e cosa costano in termini di licenza

| Forma | Ammessa? | Cosa comporta |
|---|---|---|
| **Linking / vendoring del crate Rust dentro CLIMESH** | sì | l'opera combinata è GPL-3 (o AGPL-3 via § 13). Serve preservare copyright e note, e distribuire il sorgente del combinato. |
| **Dipendenza Python opzionale, invocata come libreria** | sì | stesso regime del linking se distribuita insieme; se è solo una dipendenza di sviluppo/test **non distribuita**, il problema non si pone proprio. |
| **Processo separato (`fork`/`exec`, pipe, file su disco)** | sì | la FAQ FSF: il criterio dipende "*both on the mechanism of communication (exec, pipes, rpc, function calls within a shared address space, etc.) and the semantics of the communication*"; pipe, socket e argomenti di riga di comando sono "*communication mechanisms normally used between two separate programs*", **salvo** che la semantica sia "intimate enough, exchanging complex internal data structures". `[P]` — https://www.gnu.org/licenses/gpl-faq.html (§ *MereAggregation*, § *GPLPlugins*) |
| **Riscrittura a partire dai paper** | sì | gli algoritmi pubblicati nei paper di Lindberg et al. non sono coperti dal copyright del codice UMEP. |
| **Riscrittura a partire da `specs/`** | attenzione | i file `specs/*.md` di `UMEP-dev/solweig` **sono parte del repo GPL-3**: contengono formule, costanti (33,27; 3,0459e-4; soglie di clamp) e descrizioni di euristiche non pubblicate nei paper (per esempio l'euristica "pergola"). Trascriverli è più vicino alla derivazione che alla reimplementazione indipendente. `[I]` |

> **Non è consulenza legale.** § 6.3 riporta il testo della licenza e la FAQ del titolare della licenza, non un parere.

> `[I]` Nota pratica: dato che CLIMESH è già copyleft per scelta, **le distinzioni fini della § 6.3 non hanno conseguenze operative**. L'unica riga che conta davvero è l'ultima: se si sceglie di reimplementare, conviene sapere che parte di ciò che serve **non sta nei paper** ma solo nel codice e nelle sue spec.

---

## 7. Le opzioni di riuso — raccomandazione

**Questa sezione è una raccomandazione, non una decisione.** La scelta è del thread principale / di Mario.

### Opzione A — SOLWEIG come oracolo di validazione, non come motore

CLIMESH implementa la propria fisica in Rust; `solweig` (PyPI) resta una **dipendenza di sviluppo** usata in CI per generare i raster di riferimento su cui confrontare l'output di CLIMESH.

- **Pro**: soddisfa il gate di validazione del ticket #1 alla lettera e con zero attrito (wheel su tutte e tre le piattaforme, EPW nativo, headless, 1,7 s per caso); non tocca il binario distribuito, quindi non compromette il "binario Rust singolo"; nessun vincolo di licenza sulla distribuzione perché la dipendenza non viene distribuita.
- **Contro**: CLIMESH deve comunque scrivere ombre, SVF, GVF, radiazione, Tmrt, UTCI/PET da zero — circa quello che `UMEP-dev/solweig` ha in 17,6 k righe di Rust.

### Opzione B — vendoring del crate Rust dentro CLIMESH

Copiare `rust/src/` dentro CLIMESH, togliere lo strato PyO3, esporre le funzioni native.

- **Fattibilità**: il codice ha già la forma giusta. Le funzioni `#[pyfunction]` sono wrapper sottili che convertono i tipi e delegano a funzioni interne con firme `ndarray` pure — per esempio `calculate_shadows_rust(azimuth_deg: f32, altitude_deg: f32, scale: f32, max_local_dsm_ht: f32, dsm_view: ArrayView2<f32>, …)`, `pub(crate)`, in `rust/src/shadowing.rs:191`. `[P]` L'orchestratore `pipeline.rs` (2021 righe) tocca tipi Python in sole 62 righe. `[M]` (`grep -c "Py\|py\." pipeline.rs`)
- **Lavoro richiesto**: `[I]` cambiare `crate-type` da `cdylib` a `rlib`, rendere pubbliche le funzioni interne, sostituire i `pyclass` bundle (`SvfBundle`, `StateBundle`, `SurfaceBundle`, `PropertiesBundle`, ~336 righe in `pipeline_bundles.rs`) con struct semplici, e **riscrivere in Rust il layer Python di orchestrazione** (~17,8 k righe: caricamento raster, cache SVF, tiling, accumulatori, export GeoTIFF, EPW). Quest'ultimo è il pezzo grosso, ma è anche il pezzo che CLIMESH dovrebbe scrivere comunque.
- **Pro**: si eredita la fisica validata, gli shader GPU, e — non secondario — i test di parità contro il Python di riferimento.
- **Contro**: si eredita anche un fork da mantenere allineato a un upstream in beta e a contributore singolo.

### Opzione C — reimplementazione dai paper

Come da formulazione originaria del ticket.

- **Contro**: § 6.3 ultima riga — costanti ed euristiche che servono davvero (pergola, correzione dell'ultimo anello 3,0459e-4, decadimento 33,27, bucketing direzionale dei patch) **non stanno nei paper**. Reimplementare "dai paper" e poi validare contro SOLWEIG significa inseguire differenze di cui la letteratura non dà conto.

### Opzione D — processo separato

Scartata di fatto: il pacchetto non ha CLI, e distribuire un interprete Python + 247 MB di dipendenze contraddice frontalmente "binario singolo senza installazione". `[M]` per i 247 MB, `[P]` per l'assenza di CLI.

### Raccomandazione

> **A come base, con B come opzione aperta sui moduli più costosi.** `[I]`
>
> A è quasi gratis e va fatta comunque, perché il gate di validazione del ticket #1 la richiede esplicitamente. B è la scorciatoia che rende discutibile riscrivere da zero shadow casting e SVF: sono i due moduli dove il codice esistente è già Rust puro su `ndarray`, dove la GPU serve davvero (§ 5.3), e dove reimplementare vuol dire riprodurre euristiche non pubblicate.
>
> **Quello che non raccomando è C in purezza.** Se la ragione per riscrivere è "voglio capire la fisica", vale; se è "voglio un risultato confrontabile con SOLWEIG", il confronto sarà più facile partendo dal loro codice che dai loro paper.

---

## 8. Cosa questo cambia per il gate di validazione

Il ticket #1 fissa come gate "il confronto contro SOLWEIG, che è libero e già citato in letteratura". La ricerca aggiunge due numeri che vanno tenuti presenti.

**SOLWEIG stesso ha un errore.** Validazione ufficiale del pacchetto contro misure di campo a tre siti di Göteborg (Kronenhuset cortile chiuso, Gustav Adolfs torg piazza aperta, GVC; 7 giorni, 85 ore di osservazione): `[P]` — [`VALIDATION.md`](https://github.com/UMEP-dev/solweig/blob/main/VALIDATION.md), v0.1.0b95

| | Kronenhuset | Gustav Adolfs | GVC |
|---|---|---|---|
| Tmrt RMSE (°C) | 6,6 | 5,7–7,3 | 2,4–6,9 |
| Tmrt R² | 0,52 | 0,80–0,88 | 0,65–0,99 |
| Tmrt bias (°C) | +2,6 | +0,6 … +3,7 | +1,4 … +5,8 |

Il documento segnala anche un bias sistematico: **L↓ modellata 18-55 W/m² sopra le osservazioni a tutti i siti**, attribuito "*to the published Ldown formulation rather than to calibration*". `[P]`

> `[I]` Conseguenze per il gate. (1) "Concordare con SOLWEIG" e "essere giusto" non sono la stessa cosa: SOLWEIG sbaglia la Tmrt di 2,4-7,3 °C RMSE contro misure. (2) Un gate sul **Tmrt finale** con tolleranza dell'ordine di 1 °C sarebbe più stretto dell'accordo di SOLWEIG con la realtà, e finirebbe per misurare la fedeltà alle scelte implementative altrui. (3) Il gate più informativo è **per componente**: ombra binaria (confronto esatto contro posizione solare analitica, come già previsto), SVF, poi K↓/K↑/L↓/L↑ separatamente, come fa la validazione di Kronenhuset. (4) Se CLIMESH riproduce anche il bias di +18-55 W/m² su L↓, avrà riprodotto un errore noto, non validato un modello.

Un aspetto pratico a favore: la validazione di `UMEP-dev/solweig` è **self-contained e gira in CI** — DSM, DEM, CDSM, land cover, met file, POI in GeoJSON stanno dentro `tests/validation/` del repo, sotto GPL-3. `[P]` Sono 7 giorni di misure radiative reali a Göteborg, direttamente riusabili come dataset di validazione per CLIMESH. Questo copre parzialmente la voce "Validazione contro misure di campo reali" che il ticket #1 elenca fra le cose non ancora specificate.

---

## 9. Salute del progetto — caveat da pesare

`UMEP-dev/solweig` è il pezzo su cui si regge quasi tutta questa risposta. Va guardato in faccia: `[P]`

- Repo **creato il 27/06/2025**; versione **0.1.0b95**, cioè ancora beta; 11 stelle, 4 fork, 3 issue aperte.
- **Un solo contributore**: `songololo`, 375 commit su 375. Bus factor 1.
- Auto-dichiarazione nel README: "*This package is an **experimental**, compatibility-focused implementation of the SOLWEIG model, **not the reference implementation**. The science is taken from UMEP, where it continues to be developed, and parity tests pin this package against the reference Python. The API is stabilising but may change.*"
- Sviluppo assistito da AI: il repo contiene `CLAUDE.md`, `PRINCIPLES.md`, `INVARIANTS.md`, `AUDIT.md`, `ARCHITECTURE_REVIEW.md`.
- Contrappesi reali: `CITATION.cff` attribuisce Lindberg e Grimmond, il repo sta **nell'organizzazione UMEP-dev ufficiale**, `umep-core` (l'estrazione Python precedente) è stato archiviato **indicando questo repo come successore**, e ci sono test di parità e golden test contro il Python di riferimento più una validazione contro misure che gira in CI.
- L'implementazione **di riferimento** resta `UMEP-dev/UMEP-processing`, attiva (push 27/08/2026, 11 issue aperte).

> `[I]` Il rischio non è che il codice sia sbagliato — i test di parità e la validazione in CI sono più di quanto la media dei progetti accademici offra. Il rischio è di **continuità**: un contributore singolo su un progetto beta. Questo pesa contro l'opzione B (fork da mantenere) meno di quanto sembri (il fork è un'istantanea, non un abbonamento) e pesa contro l'opzione A più di quanto sembri (se il pacchetto smette di uscire su PyPI per Python futuri, l'oracolo di validazione si degrada). Mitigazione ovvia per A: pinnare la versione e, se serve, vendorare il wheel.

---

## 10. Fonti

**Primarie — codice e repo**
- https://github.com/UMEP-dev/solweig — clone locale `/home/mario/climesh-research/solweig`, HEAD `02246ab71a3a8b127d740dde9640449ee9d558ff` (25/08/2026, `v0.1.0b95`). File citati: `README.md`, `LICENSE`, `CITATION.cff`, `pyproject.toml`, `ARCHITECTURE.md`, `VALIDATION.md`, `rust/Cargo.toml`, `rust/src/shadowing.rs`, `rust/src/pipeline.rs`, `specs/*.md`, `docs/physics/*.md`
- https://github.com/UMEP-dev/UMEP-processing — implementazione di riferimento; `functions/SOLWEIGpython/Solweig_2025a_calc_forprocessing.py`, `functions/SOLWEIGpython/Solweig_run.py` (letti via `gh api .../contents/...`)
- https://github.com/UMEP-dev/umep-core — clone locale, HEAD `0e8df1a81867158223728e230ea5dca2f10a95ce` (archiviato)
- https://github.com/UMEP-dev/umep-solweig — clone locale, HEAD `82f9fbff03b69ead25e2e4e9143f433990ccd74f`
- https://github.com/UMEP-dev/UMEP
- https://pypi.org/pypi/solweig/json , https://pypi.org/pypi/umep/json , https://crates.io/api/v1/crates/rustalgos

**Primarie — documentazione**
- https://umep-docs.readthedocs.io/en/latest/processor/Outdoor%20Thermal%20Comfort%20SOLWEIG.html
- https://umep-docs.readthedocs.io/en/latest/OtherManuals/SOLWEIG.html (manuale completo, **non letto in questa sessione**)
- https://umep-dev.github.io/solweig/

**Primarie — licenze**
- https://www.gnu.org/licenses/gpl-3.0.txt (§ 13, riga 552)
- https://www.gnu.org/licenses/gpl-faq.html (§§ *MereAggregation*, *GPLPlugins*)

**Secondarie — letteratura del modello** (citate dalle spec e dalla doc UMEP; **non ho letto i testi**, solo i riferimenti)
- Lindberg F, Holmer B, Thorsson S (2008) *SOLWEIG 1.0 — Modelling spatial variations of 3D radiant fluxes and mean radiant temperature in complex urban settings*, Int J Biometeorol 52(7):697-713 — https://doi.org/10.1007/s00484-008-0162-7
- Lindberg F, Grimmond CSB (2011) *The influence of vegetation and building morphology on shadow patterns and mean radiant temperatures in urban areas*, Theor Appl Climatol 105:311-323 — https://doi.org/10.1007/s00704-010-0382-8
- Lindberg F, Grimmond CSB, Martilli A (2015) *Sunlit fractions on urban facets*, Urban Climate 12:65-84
- Lindberg F, Onomura S, Grimmond CSB (2016) *Influence of ground surface characteristics on the mean radiant temperature in urban areas*, Int J Biometeorol 60:1439-1452
- Lindberg F et al. (2018) *Urban Multi-scale Environmental Predictor (UMEP)*, Environ Model Softw 99:70-87 — https://doi.org/10.1016/j.envsoft.2017.09.020
- Konarska J et al. (2014) *Transmissivity of solar radiation through crowns of single urban trees*, Theor Appl Climatol 117:363-376 — https://doi.org/10.1007/s00704-013-1000-3
- Wallenberg N et al. (2020) *The Influence of Anisotropic Diffuse Shortwave Radiation on Mean Radiant Temperature*, Urban Climate 31 — https://doi.org/10.1016/j.uclim.2020.100589
- Wallenberg N et al. (2023) *An anisotropic parameterization scheme for longwave irradiance*, Int J Biometeorol — https://doi.org/10.1007/s00484-023-02441-3
- Robinson D, Stone A (1990) *Solar Radiation Modelling in the Urban Context*, Building and Environment 25(3):201-209
- Perez R, Seals R, Michalsky J (1993) *All-weather model for sky luminance distribution*, Solar Energy 50(3):235-245
- Reindl DT, Beckman WA, Duffie JA (1990) *Diffuse fraction correlations*, Solar Energy 45(1):1-7
- Holmer B et al. (2015) *How to transform the standing man from a box to a cylinder*, ICUC9
- Grimmond CSB, Cleugh HA, Oke TR (1991) *An objective urban heat storage model*, Atmos Environ 25B(3):311-326
- Ratti C, Richens P (1990) — citato da UMEP come origine del metodo delle ombre
- Höppe P (1992), Wetter und Leben 44:147-151 ; Jonsson et al. (2006) ; ISO 7726:1998

**Interne al progetto**
- `research/envi-met.md` §§ 2.4, 2.5, 2.6 (per il confronto LAD / IVS)
- `materiale università/LAB1.INX`, `materiale università/ITA_Perugia.161810_IGDG.epw`
- Ticket [#1](https://github.com/maeurong/CLIMESH/issues/1) (mappa: budget, gate, ambito), [#2](https://github.com/maeurong/CLIMESH/issues/2)

---

## 11. Caveat

1. **Le misure sono singole esecuzioni su un desktop**, non su un laptop da studente: i7-12700K a 20 thread e RTX 4060 Ti. La colonna "CPU sola" è il proxy migliore per un laptop, e va considerata **ottimistica**. Nessuna ripetizione, nessuna deviazione standard; la dispersione osservata fra due run CPU sullo stesso caso è del 20-25 %.
2. **La geometria del benchmark è approssimata**: il DSM viene dalle matrici reali di `LAB1.INX`, ma l'altezza delle 616 piante non è nel file (sta nel database di ENVI-met) e l'ho fissata a 12 m per tutte. Il costo di ombreggiamento e SVF vegetato cresce con l'altezza delle chiome: numeri sensibili a questo parametro.
3. **Le Tmrt del benchmark (media 27,7 °C, max 61,0 °C) non sono un risultato di modello**, sono un sottoprodotto del cronometro. Nessun confronto con la relazione `RELAZIONI/LA01.pdf` è stato fatto.
4. La GPU usata è raggiunta **via WSL2/Vulkan**. Su una macchina Windows nativa o su un laptop con GPU integrata i rapporti GPU/CPU saranno diversi, plausibilmente meno favorevoli.
5. **Non ho letto** il manuale completo di SOLWEIG (https://umep-docs.readthedocs.io/en/latest/OtherManuals/SOLWEIG.html) né i paper originali di Lindberg: i riferimenti in § 1 vengono dalle spec del repo e dalla pagina UMEP, non dalla lettura degli articoli. Se una formula è load-bearing per l'implementazione, va verificata sulla fonte.
6. **§ 6 non è consulenza legale.** Riporta il testo della GPLv3 e la FAQ FSF.
7. **La stima di lavoro dell'opzione B (§ 7) è un'inferenza mia**, basata sulla lettura delle firme e sul conteggio delle righe, non su un tentativo di estrazione.
8. `umep-core` ha licenza **incoerente** fra LICENSE (GPL-3.0) e `pyproject.toml`/PyPI (AGPL-3.0). Il repo è archiviato: non usarlo.
9. **Questo file è un rapporto di ricerca.** La § 7 è una raccomandazione, non una decisione presa.
