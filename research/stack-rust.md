# Ecosistema Rust per CLIMESH — geospaziale, meteo, binario singolo

**Ticket**: [#4](https://github.com/maeurong/CLIMESH/issues/4) — vincoli dalla mappa [#1](https://github.com/maeurong/CLIMESH/issues/1).
**Data ricerca**: 2026-08-31. **Repo**: CLIMESH, branch `main`, HEAD `7f23d42`. Nessun `Cargo.toml` presente, nessuna dipendenza già scelta nel codice.
**Metodo**: interrogazione diretta della API di crates.io (`/api/v1/crates/<nome>`, `/api/v1/crates/<nome>/<ver>/dependencies`) e lettura dei `Cargo.toml` e dei README dai repository sorgente via `raw.githubusercontent.com`; statistiche dei repository via API GitHub. Le classifiche e gli articoli di terzi non sono stati usati come fonte.

**Convenzione di marcatura** (la stessa di `research/envi-met.md`)
- `[P]` = confermato da fonte primaria (crates.io, `Cargo.toml`/README del progetto, API GitHub, documentazione ufficiale di rustc/cargo).
- `[S]` = fonte secondaria.
- `[I]` = inferenza mia, non confermata da fonte.

---

## 0. Risposta in una riga

Sì, l'ecosistema regge, ma **non nel modo in cui lo regge Python**: non esiste un equivalente Rust puro di GDAL. La strada praticabile è comporre crate mono-formato e scrivere in casa i due pezzi che non esistono maturi (parser EPW, writer NetCDF se servirà). Tutta la catena qui proposta è Rust puro senza dipendenze native, con **una sola eccezione contenuta** (SQLite vendorizzato, se si vuole leggere GeoPackage), e sta dentro il vincolo del binario unico. `[I]` sulla sintesi; i singoli fatti sotto sono `[P]`.

## 1. Tabella riassuntiva

Cifre di download al 2026-08-31 da crates.io (`recent_downloads` = ultimi 90 giorni). Tutte le righe sono `[P]`.

| Area | Crate | Versione | Ultima release | Licenza | Dip. native | Download 90 gg |
|---|---|---|---|---|---|---|
| GeoTIFF lettura | `geotiff` | 0.1.0 | 2025-06-10 | MIT | no | 7.364 |
| GeoTIFF scrittura | `tiff` | 0.11.3 | 2026-02-10 | MIT | no | 29.507.942 |
| Shapefile | `shapefile` | 0.9.0 | 2026-07-05 | MIT | no | 110.027 |
| GeoPackage (SQL) | `rusqlite` (feature `bundled`) | 0.40.2 | 2026-08-08 | MIT | **sì**, SQLite C vendorizzato e linkato statico | 31.521.605 |
| GeoPackage (geometrie) | `geozero` (feature `with-wkb`) | 0.15.1 | 2025-12-11 | MIT OR Apache-2.0 | no con le feature di default | 458.855 |
| Riproiezione CRS | `proj4rs` | 0.1.10 | 2026-03-06 | MIT OR Apache-2.0 | no | 531.647 |
| Tipi geometrici | `geo-types` / `geo` | 0.7.20 / 0.33.1 | 2026-08-02 / 2026-04-20 | MIT OR Apache-2.0 | no | 5.542.055 / 4.342.673 |
| EPW | **da scrivere in casa** | — | — | — | no | — |
| Posizione solare | `solar-positioning` | 0.5.3 | 2026-08-12 | MIT | no | 8.353 |
| Griglia N-dim | `ndarray` | 0.17.2 | 2026-01-10 | MIT OR Apache-2.0 | no (`blas` è opt-in) | 25.194.146 |
| Parallelismo | `rayon` | 1.12.0 | 2026-04-14 | MIT OR Apache-2.0 | no | 116.574.460 |
| Server HTTP | `axum` + `tokio` + `tower-http` | 0.8.9 / 1.53.1 / 0.7.1 | 2026-04-14 / 2026-07-20 / 2026-08-31 | MIT | no | 110M / 212M / 134M |
| Asset statici nel binario | `rust-embed` | 8.12.0 | 2026-07-08 | MIT | no | 13.849.182 |
| Grafici nel browser | ECharts (JS) embedded, opz. `charming` solo con feature di default | 0.6.0 | 2025-06-17 | MIT OR Apache-2.0 | no (**sì** con feature `ssr`) | 234.864 |
| Lettura `.INX` | `quick-xml` | 0.42.0 | 2026-08-22 | MIT | no | 101.267.205 |
| CLI | `clap` | 4.6.6 | 2026-08-06 | MIT OR Apache-2.0 | no | 221.636.172 |
| Data/ora | `chrono` (o `jiff`) | 0.4.45 / 0.2.35 | 2026-06-04 / 2026-07-25 | MIT OR Apache-2.0 | no | 164.927.510 |
| Cross-compilazione | `cargo-zigbuild` + `cargo-xwin` | 0.23.3 / 0.23.1 | 2026-08-27 / 2026-08-13 | MIT | toolchain, non runtime | 1.531.094 / 799.070 |

**Licenze e copyleft** `[I]`: tutte MIT, Apache-2.0, `MIT OR Apache-2.0` o Unlicense/MIT. Sono tutte compatibili con GPL-3 e AGPL-3 in senso *upstream* (permissiva assorbita da copyleft). Attenzione a un solo punto: Apache-2.0 è compatibile con GPLv3 ma **non** con GPLv2; siccome la mappa fissa GPL-3/AGPL-3, il problema non si pone. `nalgebra` è Apache-2.0 puro, non dual, se mai servisse.

---

## 2. Raster georeferenziati: GeoTIFF e NetCDF

### 2.1 GDAL è fuori — confermato

Il README di `georust/gdal` dice testualmente: *"Building this crate assumes a compatible version of GDAL is installed with the corresponding header files and shared libraries"*, e che `gdal-sys` è generato con `bindgen`. `[P]` — https://github.com/georust/gdal/blob/master/README.md

Quindi: header e shared library di sistema, `bindgen` in fase di build. Incompatibile col binario singolo senza installazione. Il crate `gdal` è vivo (0.19.0 del 2025-12-23, MIT, 481.834 download in 90 gg `[P]` crates.io), ma è la strada sbagliata per questo progetto. Stessa conclusione per `geozero` se si attivasse la feature `with-gdal` o `with-geos`: sono feature **opzionali e disattivate di default** (`default = ["with-geo", "with-geojson", "with-svg", "with-wkt"]`), quindi `geozero` con le feature di default resta Rust puro. `[P]` — https://github.com/georust/geozero/blob/main/geozero/Cargo.toml

### 2.2 Lettura GeoTIFF

**`geotiff` 0.1.0** (georust), MIT, ultima release 2025-06-10, 21.200 download totali, 132 stelle, 6 contributori, ultimo push 2026-07-31. Rust puro: dipende da `tiff`, `geo-types`, `num_enum`, `half`. `[P]` — crates.io + https://github.com/georust/geotiff/blob/main/Cargo.toml

Due avvertenze dal suo stesso README, entrambe `[P]`:
- *"The purpose of this library is to simply read GeoTIFFs, nothing else"* e *"In its current state, it works for very basic GeoTIFFs"*. È **solo lettura**.
- *"do expect breaking changes post v0.1.0, as we may decide to do another redesign to work towards asynchronous reading"*.

L'API esposta è `GeoTiff::read(path)` più `get_value_at::<T>(&Coord{x,y}, band)` — accesso pixel per pixel. `[P]` (README) Per leggere un DSM intero cella per cella con quella API il costo per-chiamata va misurato. `[I]`: probabile che convenga leggere gli strip direttamente col crate `tiff` e usare `geotiff` solo per i tag geografici, oppure prendere `georaster`.

**Alternativa: `georaster` 0.2.0**, MIT/Apache-2.0, ultima release 2025-01-11, solo 1.375 download in 90 gg. Dipende da `tiff` 0.9, `geo` e `geodesy` opzionali. `[P]` — crates.io. Meno mantenuto di `geotiff`; per ora non lo raccomando.

### 2.3 Scrittura GeoTIFF — il pezzo interessante

**Non serve un crate GeoTIFF per scrivere.** Il crate `tiff` 0.11.3 (image-rs, MIT, 2026-02-10, 29,5 milioni di download in 90 gg, *"TIFF decoding and encoding library in pure Rust"*, dipendenze `half` + `quick-error` + codec opzionali) espone:
- `DirectoryEncoder::write_tag(&mut self, tag: Tag, value: T)`, raggiungibile da un `ImageEncoder` tramite `.encoder()`. `[P]` — https://github.com/image-rs/image-tiff/blob/master/src/encoder/mod.rs righe 313 e 947
- Un enum `Tag` che **conosce già i tag GeoTIFF**: `ModelPixelScaleTag = 33550`, `ModelTiepointTag = 33922`, `GeoKeyDirectoryTag = 34735`, più una variante `Tag::Unknown(u16)` per tutto il resto (`GeoDoubleParams` 34736, `GeoAsciiParams` 34737, `GDAL_NODATA` 42113). `[P]` — https://github.com/image-rs/image-tiff/blob/master/src/tags.rs righe 80-153

`[I]`: scrivere un GeoTIFF `f32` monobanda valido, con CRS e trasformazione affine, è quindi qualche decina di righe sopra `tiff`, senza crate aggiuntivi. È l'opzione più conservativa: `tiff` è di fatto infrastruttura dell'ecosistema Rust, non un crate di nicchia.

### 2.4 Due stack GeoTIFF nuovi, entrambi da guardare con cautela

**`geotiff-rust`** di `roteiro-gis` (crate `tiff-core`, `tiff-reader`, `tiff-writer`, `geotiff-core`, `geotiff-reader`, `geotiff-writer`, v0.8.1 del 2026-08-13, metadati crates.io `MIT OR Apache-2.0`). Il README promette esattamente ciò che serve: *"Pure-Rust TIFF/BigTIFF and GeoTIFF/COG readers and writers. No C libraries, no build scripts"*, con lettura in `ndarray::ArrayD<f32>` e un `GeoTiffBuilder` con `.epsg()`, `.pixel_scale()`, `.origin()`, `.nodata()`, `.compression()`. `[P]` — https://github.com/roteiro-gis/geotiff-rust/blob/main/README.md

Cautele `[P]`: repository creato il **2026-03-20** (cinque mesi di vita), 13 stelle, 3 contributori, e la API GitHub riporta licenza `NOASSERTION` — cioè il file di licenza nel repo non è riconosciuto, in contraddizione con i metadati crates.io. `[I]`: da non adottare finché la licenza non è chiarita; utile come piano B se `tiff` grezzo si rivelasse scomodo.

**Whitebox Next Gen** di John Lindsay / Whitebox Geospatial Inc. (crate `wbprojection` 0.3.3, `wbvector` 0.1.6, `wbtopology` 0.2.1, tutti `MIT OR Apache-2.0`, aggiornati fra il 2026-07-30 e il 2026-08-04). Il README dichiara la stessa filosofia di questo progetto: *"Whitebox is full-stack: all foundational plumbing — GeoTIFF I/O, map projections, raster abstraction, vector I/O, LiDAR parsing, and topology — is implemented in this codebase rather than delegated to external libraries"* e *"avoids dependencies that would pull in C/C++ linkage"*. Copre GeoTIFF e COG, GeoPackage raster, 11 formati vettoriali fra cui GeoPackage e FlatGeobuf, e riproiezione raster. `[P]` — https://github.com/jblindsay/whitebox_next_gen/blob/main/README.md

Cautele `[P]`: repository creato il **2026-03-31**, 90 stelle, **1 solo contributore**, nessun file di licenza rilevato alla radice, modello **open-core** con un'estensione commerciale proprietaria fuori dal workspace, e il README dichiara esplicitamente lo sviluppo *"Human–AI collaborative"*. `[I]`: bus factor 1 su una dipendenza di base è un rischio serio per un progetto di tesi che deve restare riproducibile fra anni. Da tenere d'occhio, non da adottare ora.

### 2.5 NetCDF

Il crate `netcdf` 0.12.1 (georust, MIT OR Apache-2.0, 2026-07-26, 39.551 download in 90 gg) dipende da `netcdf-sys`, cioè dalla libreria C `netcdf-c`. Il README: *"This crate depends on the C library `netcdf-c` which must be installed on the machine, along with libraries such as `hdf5`"*. Esiste una via d'uscita: *"An alternative to using the system libraries is to enable `static` feature of this crate, which compiles `libnetcdf` from source. The `static` feature requires `cmake`, a `c++` compiler and more to be installed on the build machine."* `[P]` — https://github.com/georust/netcdf/blob/master/README.md

Traduzione `[I]`: con `--features static` il **binario finale resta unico**, ma la **build** acquista cmake, un compilatore C++ e sottomoduli git, e la cross-compilazione diventa un problema aperto. In più il README avverte che l'accesso è serializzato da un mutex globale perché `netcdf-c` non è thread-safe — irrilevante per la scrittura di output, rilevante se mai si volesse leggere in parallelo.

**Raccomandazione** `[I]`: **non mettere NetCDF nella prima versione.** I deliverable della mappa #1 sono mappe raster georeferenziate e serie temporali: GeoTIFF più CSV li coprono. Se in seguito servirà un cubo 4D, le opzioni in ordine di preferenza sono (a) scrivere un writer NetCDF-3 classic in casa — il formato è un header binario semplice e documentato; (b) `zarrs` 0.23.14 (MIT OR Apache-2.0, 2026-08-14, 66.110 download in 90 gg `[P]`), Rust puro ma produce una directory, non un file; (c) `netcdf --features static` accettando il costo di build.

---

## 3. Vettoriale: shapefile, GeoPackage, riproiezione

### 3.1 Shapefile

**`shapefile` 0.9.0** (tmontaigu), MIT, 2026-07-05, 110.027 download in 90 gg, 76 stelle, 13 contributori, ultimo push 2026-07-24. Rust puro, si appoggia a `dbase` 0.8.0 (stesso autore, stessa data, 236.327 download in 90 gg) per la tabella attributi. `[P]` — crates.io + https://github.com/tmontaigu/shapefile-rs

`[I]`: è il candidato migliore senza rivali seri — è di fatto l'unica implementazione shapefile Rust matura e multi-contributore, ha integrazione opzionale con `geo-types`, ed è quello che `geozero` stesso usa dietro la feature `with-shp`.

### 3.2 GeoPackage

Non esiste un crate GeoPackage maturo. `gpkg` 0.1.0 è fermo al 2022-05-22 con 53 download in 90 giorni: morto. `[P]` — crates.io

Un GeoPackage è però un database SQLite con geometrie in un incapsulamento binario attorno a WKB. Le due vie:

- **`geozero` con feature `with-gpkg`**, che tira `sqlx` con backend `sqlite`. `[P]` — Cargo.toml di geozero. `[I]`: `sqlx` è asincrono e pesante per una lettura di file locale.
- **`rusqlite` 0.40.2 con feature `bundled`** (MIT, 2026-08-08, 31,5 milioni di download in 90 gg) più `geozero` con `with-wkb` per decodificare le geometrie. `[P]` su versioni e licenze. `[I]` sulla combinazione: è più semplice, sincrona, e `bundled` compila SQLite dai sorgenti vendorizzati e lo linka staticamente, quindi **il binario resta unico**. Il prezzo è `libsqlite3-sys` con un build script e un compilatore C richiesto in fase di build.

**Raccomandazione** `[I]`: `rusqlite` con `bundled`. E siccome la mappa #1 dà come via primaria OpenStreetMap più DEM/DSM, e shapefile/GeoPackage come seconda, **GeoPackage può slittare dopo la prima release**: è l'unica dipendenza C dell'intero stack e vale la pena rimandarla finché non serve davvero.

### 3.3 Riproiezione fra sistemi di coordinate

Due mondi:

- **`proj` 0.31.0** (georust, MIT OR Apache-2.0, 2025-08-29): binding alla libreria C PROJ. Stesso problema di GDAL. Fuori. `[P]` crates.io
- **`proj4rs` 0.1.10** (3liz, MIT OR Apache-2.0, 2026-03-06, 531.647 download in 90 gg, 78 stelle, 13 contributori, ultimo push 2026-07-16). Il README: *"This is a pure Rust implementation of the PROJ.4 project"* e, esplicitamente, *"No installation of external C libraries such as `libproj` or `sqlite3` is needed."* `[P]` — https://github.com/3liz/proj4rs/blob/main/README.md

Le dipendenze `wasm-bindgen`, `js-sys`, `web-sys` e `console_log` che compaiono nell'elenco di crates.io **non riguardano le build native**: nel `Cargo.toml` sono sotto `[target.wasm32-unknown-unknown.dependencies]`. `[P]` — https://github.com/3liz/proj4rs/blob/main/proj4rs/Cargo.toml

Limiti dichiarati dagli autori, tutti `[P]` dallo stesso README:
- *"This port implements the PROJ.4 API, which means there's no 3D/4D/orthometric transformation ATM"*;
- *"This crate does not provide support for WKT"* — serve il crate separato `proj4wkt` per convertire WKT in stringa proj;
- l'unità angolare naturale sono i **radianti**, non i gradi. Fonte di errori silenziosi.

`[I]`: per CLIMESH serve trasformare fra WGS84 geografiche e una proiezione metrica (UTM 32N per l'Umbria, o Web Mercator per i tile). `proj4rs` copre esattamente questo. Il `Cargo.toml` nel repo dichiara già `version = "0.2.0"` non ancora pubblicata su crates.io: aspettarsi un salto di versione. `[P]` sul numero di versione nel repo.

Alternative minori `[P]`: `geodesy` 0.15.0 (MIT OR Apache-2.0, 2026-03-03, 55.831 download in 90 gg) è ben curato ma con un modello concettuale proprio; `utm` 0.1.6 è fermo al 2022 e fa solo UTM. `[I]`: `utm` basterebbe per il caso Bastia, ma non per un import OSM generico.

---

## 4. File meteo EPW

**Non esiste un crate EPW usabile.** L'unico è `epw-rs` 0.1.4: ultima release 2025-01-15, **32 download negli ultimi 90 giorni**, 1 stella su GitHub, 1 contributore, ultimo push 2025-01-15, MIT. `[P]` — crates.io + https://github.com/hamiltonkibbe/epw-rs. Ha anche una dipendenza opzionale da `polars`, sproporzionata. `[P]` — Cargo.toml.

**Raccomandazione: scrivere il parser in casa.** `[I]` sulla raccomandazione, `[P]` sui numeri che la sostengono, misurati sul file reale in `materiale università/ITA_Perugia.161810_IGDG.epw`:

- 8 righe di intestazione, con chiavi note e in chiaro: `LOCATION`, `DESIGN CONDITIONS`, `TYPICAL/EXTREME PERIODS`, `GROUND TEMPERATURES`, `HOLIDAYS/DAYLIGHT SAVINGS`, `COMMENTS 1`, `COMMENTS 2`, `DATA PERIODS`;
- 8.760 righe di dati (8.768 righe totali nel file), **35 campi separati da virgola ciascuna**, posizionali;
- la riga `LOCATION` porta già latitudine 43.08, longitudine 12.50, fuso +1.0 e quota 213 m, cioè tutto ciò che serve alla posizione solare.

Sopra `csv` 1.4.0 (BurntSushi, Unlicense/MIT, 2025-10-17 `[P]`) oppure anche solo `str::split(',')`, il parser è una struct con 35 campi e una manciata di righe di validazione. La dipendenza da un crate abbandonato costerebbe più di quanto risparmi. Da gestire esplicitamente `[I]`: i valori sentinella di mancanza (`9999`, `999999999`, `99.0` visibili nella prima riga di dati del file di Perugia) e l'ora convenzionale 1..24 dell'EPW, che non è l'ora 0..23 di `chrono`.

---

## 5. Posizione solare

**`solar-positioning` 0.5.3** (klausbrunner), MIT, ultima release 2026-08-12, 8.353 download in 90 gg, 5 stelle, 2 contributori. Dipendenze: solo `chrono` e `libm`, entrambe opzionali. `[P]` — crates.io + dependencies API.

Perché è il candidato migliore, tutto `[P]` dal README (https://github.com/klausbrunner/solarpositioning-rs/blob/main/README.md):
- implementa **SPA di Reda e Andreas** (doi:10.1016/j.solener.2003.12.003) e in alternativa **Grena/ENEA** (doi:10.1016/j.solener.2012.01.024);
- *"More than 1000 test points are included to validate against the reference code and other sources"*;
- *"This library is not based on or derived from code published by NREL, ENEA or other parties. It is an implementation precisely following the algorithms described in the respective papers."* — questo risolve il problema di licenza: il codice di riferimento NREL non è liberamente ridistribuibile, questa riscrittura MIT sì, ed è quindi legalmente assorbibile in un progetto GPL-3;
- espone `spa_time_dependent_parts(datetime, delta_t)` più `spa_with_time_dependent_parts(lat, lon, elev, refraction, &parts)`, cioè **calcola una sola volta la parte dipendente dal tempo e la riusa su molte coordinate**. Per un dominio a griglia con molti passi temporali è esattamente la forma giusta;
- supporta `no_std` con `libm`.

Avvertenza `[P]`, dallo stesso README: *"APIs may still evolve. Breaking changes may occur in minor version updates. You'll probably want to pin to a specific version in production code."* Da pinnare esatta in `Cargo.toml`.

Concorrenti scartati `[P]` su versioni e date:
- `spa` 0.5.1: ultima release 2024-02-11, Apache-2.0, 423.272 download in 90 gg. Molto più usato ma fermo da due anni e mezzo, e la sua API non separa la parte tempo-dipendente. `[I]`
- `sunrise` 3.0.0 (2026-01-01, MIT, 459.704 download): calcola alba e tramonto, non azimut ed elevazione istantanei. Fuori ambito. `[P]`
- `sun` 0.3.1 (2024-10-18): algoritmo semplificato, precisione insufficiente per l'ombreggiamento. `[I]`
- `astro` 2.0.0: **ultima release 2016-05-22**. Fuori. `[P]`
- `pvlib-rust` 0.1.6 (2026-04-20, Apache-2.0, **89 download in 90 gg**): porting di pvlib-python, troppo giovane. `[P]`

`[I]`: dato che il gate di validazione della mappa #1 richiede di confrontare l'ombreggiamento *"contro la posizione solare calcolata analiticamente"*, avere due algoritmi indipendenti nello stesso crate (SPA e Grena3) è un vantaggio diretto: uno verifica l'altro a costo zero.

---

## 6. Calcolo su griglia e parallelismo

**`ndarray` 0.17.2**, MIT OR Apache-2.0, ultima release 2026-01-10, 25,2 milioni di download in 90 gg. `[P]` crates.io. Dal suo `Cargo.toml` `[P]` (https://github.com/rust-ndarray/ndarray/blob/master/Cargo.toml):
- dipendenze obbligatorie tutte Rust puro: `num-integer`, `num-traits`, `num-complex`, `matrixmultiply`, `rawpointer`;
- `rayon` è una **feature opzionale** (`rayon = { version = "1.10.0", optional = true }`), che abilita gli iteratori paralleli sugli assi;
- `blas` è opzionale e disattivata di default; è l'**unica** cosa che porterebbe una dipendenza nativa (`cblas-sys` + `libc`). Non attivarla.

**`rayon` 1.12.0**, MIT OR Apache-2.0, ultima release 2026-04-14, **116,5 milioni di download in 90 gg**. `[P]` crates.io. Nessuna dipendenza nativa: usa i thread di sistema.

`[I]` sul dimensionamento rispetto al budget di 60 secondi della mappa #1. Il dominio di riferimento (verificato in `materiale università/LAB1.INX`, `<grids-I>50</grids-I> <grids-J>50</grids-J> <grids-Z>25</grids-Z>`, passo 1 m) è 62.500 celle, di cui 2.500 celle di suolo. Con due giorni da 48 h e un passo orario si parla di ~2.500 × 48 ≈ 120.000 valutazioni di MRT, ciascuna con un fascio di raggi verso la volta celeste. Anche a 200 direzioni di cielo per cella sono ~24 milioni di intersezioni raggio-geometria: un ordine di grandezza che su 8 core con `rayon` sta in pochi secondi. **Il vincolo di 60 secondi non è quello che seleziona lo stack**; è largamente rispettabile con `ndarray` + `rayon`. Questa è una stima a spanne, non una misura.

Se servirà accelerare l'intersezione raggio-geometria `[P]` su versioni: `parry3d` 0.30.2 (Apache-2.0, 2026-08-07, 633.107 download in 90 gg) offre BVH e query di raycast; `rstar` 0.13.0 (georust, MIT OR Apache-2.0, 2026-05-24, 13 milioni di download in 90 gg) è un R*-tree generico; `bvh` 0.12.0 (MIT, 2025-11-16) ha però solo 25.250 download in 90 gg. `[I]`: su una griglia regolare come questa, una traversata voxel DDA scritta a mano probabilmente batte qualunque BVH e non aggiunge dipendenze — da valutare al momento, non ora.

`nalgebra` 0.35.0 (Apache-2.0, 2026-05-24) serve solo se ci saranno rotazioni e trasformazioni 3D dense; per la rotazione di 21 gradi del dominio bastano due seni e due coseni. `[I]`

---

## 7. Server web incorporato e asset statici

### 7.1 Server

**`axum` 0.8.9** (tokio-rs), MIT, ultima release 2026-04-14, 110,5 milioni di download in 90 gg, sopra **`tokio` 1.53.1** (MIT, 2026-07-20, 212 milioni) e **`tower-http` 0.7.1** (MIT, 2026-08-31, 134 milioni). Tutto Rust puro. `[P]` crates.io.

Alternative `[P]`: `actix-web` 4.15.0 (MIT OR Apache-2.0, 2026-08-21, 9,9 milioni in 90 gg) — valido ma con un runtime proprio; `warp` 0.4.3 (MIT, 2026-05-04, 5 milioni) — stesso autore di hyper, meno usato di axum; `tiny_http` 0.12.0 (MIT OR Apache-2.0, **ultima release 2022-10-06**, 13,8 milioni in 90 gg) — sincrono e minimale, fermo da quattro anni ma ancora molto scaricato.

**Raccomandazione** `[I]`: `axum`. Non per prestazioni — un server locale per una UI monoutente non ha carico — ma perché è lo standard di fatto, la documentazione è abbondante, e `tower-http` dà gratis compressione e CORS. `[I]` sul contro: `tokio` porta dentro un runtime asincrono completo, che il resto di CLIMESH (calcolo sincrono e parallelo con rayon) non usa. Se si volesse un binario davvero minimo, `tiny_http` con un thread pool sarebbe sufficiente; è però fermo dal 2022. Vale la pena metterlo sul tavolo per una decisione esplicita.

### 7.2 Asset statici nel binario

**`rust-embed` 8.12.0**, MIT, ultima release 2026-07-08, 13,8 milioni di download in 90 gg. `[P]` crates.io. Macro derive che incorpora una directory nel binario a compile time in release, e la legge dal filesystem in debug — che significa ricaricare la UI senza ricompilare durante lo sviluppo.

Alternative `[P]`: `include_dir` 0.7.4 — 15,1 milioni di download in 90 gg ma **ultima release 2024-06-17**; `memory-serve` 2.3.0 (2026-07-05, 75.586 download) e `static-serve` 0.6.3 (2026-07-19, 18.099), entrambi specifici per axum con compressione precalcolata a compile time; `tower-serve-static` 0.1.2 (2026-05-08, 164.690).

**Raccomandazione** `[I]`: `rust-embed`, per volume d'uso e per il comportamento debug/release. `memory-serve` è tecnicamente più raffinato (precomprime gli asset in brotli a compile time) ma è un crate molto meno usato per un guadagno che su localhost non si vede.

`[I]` sul disegno: il binario apre una porta su `127.0.0.1`, serve gli asset embedded alla radice, espone i risultati come JSON o come PNG generati al volo, e apre il browser di default. Nessuna installazione, nessun server esterno, nessun Node.

---

## 8. Grafici nel browser

Il problema si spacca in due, e le due metà vogliono risposte diverse. `[I]`

**Serie temporali e diagrammi**: farli **nel browser** con una libreria JavaScript incorporata negli asset. Candidati `[P]` (API GitHub, 2026-08-31):
- **uPlot**, MIT, 10.461 stelle, ultimo push 2026-04-22. Minuscola (dell'ordine dei 50 KB) e velocissima su serie lunghe.
- **Apache ECharts**, Apache-2.0, 67.205 stelle, ultimo push 2026-08-04. Molto più completa, molto più grande.
- **Plotly.js**, MIT, 18.312 stelle, ultimo push 2026-08-30. Familiare a chi viene da Python, ma è la più pesante delle tre.

Se si sceglie ECharts, **`charming` 0.6.0** (MIT OR Apache-2.0, 2025-06-17, 234.864 download in 90 gg) permette di costruire la specifica ECharts in Rust con tipi controllati dal compilatore e serializzarla in JSON per il browser. `[P]` crates.io. **Attenzione seria** `[P]` dal suo `Cargo.toml` (https://github.com/yuankunzhang/charming/blob/main/charming/Cargo.toml): `default = ["html"]` porta solo `handlebars`, ma la feature **`ssr` tira `deno_core` 0.378**, cioè il motore V8 embedded. `[I]`: attivare `ssr` farebbe esplodere il binario di decine di megabyte e vanificherebbe il vincolo. Usare `charming` **solo con le feature di default**, o direttamente `serde_json`.

**Mappe raster di MRT e comfort**: `[I]` il modo più economico è che il server produca un PNG colorato con `image` 0.25.10 (MIT OR Apache-2.0, 2026-03-10, 45,9 milioni di download in 90 gg) o `png` 0.18.1 (MIT OR Apache-2.0, 2026-02-14, 64,6 milioni) e una scala di colori da **`colorous` 1.0.16** (dtolnay, Apache-2.0, 2025-03-03, 259.607 download in 90 gg — porta le palette di d3-scale-chromatic, viridis inclusa), e che il browser lo mostri come immagine sovrapposta. `[P]` su versioni e licenze. In alternativa il server manda l'array grezzo e il browser dipinge su `<canvas>`: più interattivo, più codice JS.

**Da non usare per questo scopo** `[P]`: `plotters` 0.3.7 (MIT, ultima release 2024-09-08, 48,5 milioni di download in 90 gg) — ottimo, ma rende in PNG/SVG lato server, che è il modello di matplotlib che la mappa #1 vuole abbandonare; utile semmai per le "figure pronte per una relazione". `poloto` 19.1.2 è fermo al 2023-07-09. `rerun` 0.36.3 è uno strumento di visualizzazione a sé, sproporzionato.

---

## 9. Compilazione incrociata e dimensione del binario

### 9.1 Livelli di supporto ufficiali

Dalla documentazione ufficiale di rustc, dove **Tier 1 with Host Tools** significa *"guaranteed to work"* con test automatici a ogni cambiamento. `[P]` — https://doc.rust-lang.org/rustc/platform-support.html

| Target | Livello |
|---|---|
| `x86_64-unknown-linux-gnu` | Tier 1 with Host Tools |
| `x86_64-pc-windows-msvc` | Tier 1 with Host Tools |
| `x86_64-pc-windows-gnu` (MinGW) | Tier 1 with Host Tools |
| `aarch64-apple-darwin` (Apple Silicon) | Tier 1 with Host Tools |
| `x86_64-apple-darwin` (Intel Mac) | Tier 2 with Host Tools |
| `x86_64-unknown-linux-musl` | Tier 2 with Host Tools |

`[I]`: tutte le piattaforme che interessano a CLIMESH sono Tier 1 o Tier 2 con host tools. Non c'è nessun rischio di piattaforma.

### 9.2 Come si compila da Linux

- **Verso Windows**: `x86_64-pc-windows-gnu` con `rustup target add` più il linker MinGW è la via più semplice, ed è Tier 1. Verso MSVC serve **`cargo-xwin` 0.23.1** (MIT, 2026-08-13, 799.070 download in 90 gg `[P]` crates.io), che scarica le librerie MSVC. Nota `[P]`: `cargo-zigbuild` **non copre Windows** — il suo README dice testualmente *"Currently only Linux and macOS targets are supported"* (https://github.com/rust-cross/cargo-zigbuild/blob/main/README.md).
- **Verso macOS**: **`cargo-zigbuild` 0.23.3** (MIT, 2026-08-27, 1,53 milioni di download in 90 gg `[P]`) usa `zig cc` come linker e supporta anche il target speciale `universal2-apple-darwin` per il binario universale Intel+ARM. `[P]` (README). Richiede però l'SDK macOS via `SDKROOT`; il progetto distribuisce immagini Docker *"which has macOS SDK pre-installed"*. `[P]` (README). `[I]`: qui c'è una questione di licenza dell'SDK Apple, non tecnica, che vale la pena non toccare — la strada pulita è **compilare i binari macOS su runner macOS di GitHub Actions**, che sono licenziati per farlo, e usare la cross-compilazione da Linux solo per Windows e Linux.
- **`cross` 0.2.5**: `[P]` crates.io — **ultima release su crates.io 2023-02-04**, 303.269 download in 90 gg. Il repository è attivo ma la versione pubblicata è vecchia di tre anni e mezzo, e richiede Docker o Podman. `[I]`: preferire `cargo-zigbuild` + `cargo-xwin`, oppure semplicemente una matrice GitHub Actions con runner nativi, che è la soluzione più noiosa e più affidabile.

Trappola `[P]` dal README di `cargo-zigbuild`: *"`-C target-feature=+crt-static` for statically linking to a glibc version is not supported. Use a `*-musl` target instead if you need a fully static binary."* `[I]`: per un binario Linux davvero portabile che gira ovunque senza badare alla glibc, il target è `x86_64-unknown-linux-musl`.

### 9.3 Dimensione realistica

Non ho potuto misurare direttamente: **nessun toolchain Rust è installato in questo ambiente** (`cargo` e `rustup` non trovati) e il repo non ha ancora codice. Ho quindi misurato gli artefatti di release di progetti Rust confrontabili — CLI con server HTTP e asset web incorporati — via API GitHub. `[P]`:

| Progetto | Versione | Target | Dimensione |
|---|---|---|---|
| `miniserve` (server HTTP con UI incorporata) | 0.35.0 | x86_64-unknown-linux-gnu | 2,2 MB |
| `miniserve` | 0.35.0 | x86_64-pc-windows-msvc | 2,0 MB |
| `miniserve` | 0.35.0 | x86_64-apple-darwin | 6,1 MB |
| `dufs` (server HTTP con UI incorporata) | 0.46.0 | x86_64-unknown-linux-musl | 2,7 MB (compresso) |
| `ripgrep` | 15.2.0 | x86_64-unknown-linux-musl | 2,1 MB (compresso) |

**Stima `[I]`, non misura**: CLIMESH aggiunge a un profilo tipo `miniserve` il codice numerico (`ndarray`, `rayon`, il motore radiativo), l'I/O geospaziale (`tiff`, `shapefile`, `proj4rs` con le sue tabelle di definizione CRS), e gli asset JS della UI. Con `--release`, `lto = true`, `codegen-units = 1` e `strip = true` in `[profile.release]` mi aspetto **fra 8 e 20 MB**, dominati dagli asset incorporati più che dal codice: ECharts minificato da solo è dell'ordine del megabyte, uPlot di poche decine di kilobyte. Il numero va misurato al primo binario reale, non prima.

---

## 10. Cosa resta da decidere (non deciso qui)

`[I]`, tutte raccomandazioni da portare al thread principale, non scelte fatte:

1. **`axum` + `tokio` oppure `tiny_http` sincrono.** La differenza è un runtime asincrono completo dentro un programma che per il resto è sincrono e CPU-bound. `axum` è lo standard, `tiny_http` è più onesto rispetto al carico reale ma è fermo dal 2022.
2. **Quando introdurre GeoPackage.** È l'unica dipendenza C dell'intero stack (SQLite vendorizzato). Rimandarla mantiene la build completamente Rust puro più a lungo.
3. **uPlot o ECharts.** Un ordine di grandezza di differenza sul peso degli asset, a fronte di funzioni che forse non servono.
4. **Se pinnare `solar-positioning` a 0.5.3 esatta** — gli autori avvertono di rotture in minor version, e la posizione solare è il fondamento del gate di validazione.
5. **Se scrivere il writer GeoTIFF direttamente sopra `tiff`** (raccomandato) o aspettare che `geotiff-writer` di roteiro-gis chiarisca la licenza.

---

## 11. Fonti primarie consultate

- crates.io API: `https://crates.io/api/v1/crates/<nome>` e `.../<versione>/dependencies` — versioni, date di release, licenze, download, alberi di dipendenza. Interrogata il 2026-08-31.
- GitHub API `repos/<owner>/<repo>` e `repos/<owner>/<repo>/contributors` — stelle, contributori, data di creazione, ultimo push, licenza rilevata.
- `Cargo.toml` e README dai repository: [image-tiff](https://github.com/image-rs/image-tiff), [georust/geotiff](https://github.com/georust/geotiff), [georust/gdal](https://github.com/georust/gdal), [georust/netcdf](https://github.com/georust/netcdf), [georust/geozero](https://github.com/georust/geozero), [3liz/proj4rs](https://github.com/3liz/proj4rs), [tmontaigu/shapefile-rs](https://github.com/tmontaigu/shapefile-rs), [klausbrunner/solarpositioning-rs](https://github.com/klausbrunner/solarpositioning-rs), [hamiltonkibbe/epw-rs](https://github.com/hamiltonkibbe/epw-rs), [rust-ndarray/ndarray](https://github.com/rust-ndarray/ndarray), [yuankunzhang/charming](https://github.com/yuankunzhang/charming), [rust-cross/cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild), [cross-rs/cross](https://github.com/cross-rs/cross), [roteiro-gis/geotiff-rust](https://github.com/roteiro-gis/geotiff-rust), [jblindsay/whitebox_next_gen](https://github.com/jblindsay/whitebox_next_gen).
- Documentazione ufficiale rustc: https://doc.rust-lang.org/rustc/platform-support.html
- Materiale del progetto: `materiale università/ITA_Perugia.161810_IGDG.epw` (struttura EPW misurata), `materiale università/LAB1.INX` (dimensioni del dominio).
