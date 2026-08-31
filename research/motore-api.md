# Il Motore (`UMEP-dev/solweig`, crate `rustalgos`) — API, convenzioni, e cosa serve per consumarlo

**Scopo**: dossier preliminare richiesto da `docs/superpowers/plans/2026-08-31-nucleo-di-calcolo-dispatch.md` § 4, da consumare nei Task 3, 4a e 4b del piano `2026-08-31-nucleo-di-calcolo.md`.

**Data ricerca**: 2026-08-31.
**Repo CLIMESH**: branch `main`, HEAD `d7561cb` (`Cargo.toml` con `[dependencies]` vuota, nessuna dipendenza ancora entrata).
**Motore letto**: `UMEP-dev/solweig`, clone `--depth 1` di `main`, HEAD `02246ab71a3a8b127d740dde9640449ee9d558ff` (25/08/2026 15:21:29 UTC). Questo commit è **un solo commit avanti** rispetto al tag `v0.1.0b95` = `d3dfb2a1d0a7b284b4a530e8439f1966a917361e`, e l'unico file che cambia fra i due è `uv.lock` (verificato via `GET /repos/UMEP-dev/solweig/compare/d3dfb2a...02246ab` → `ahead_by: 1`, `files: [uv.lock]`). **Tutte le righe di `rust/src/` citate qui sono quindi identiche nel tag e in `main`**, e i permalink puntano al tag.

**Convenzione di marcatura** (la stessa di `research/solweig-riuso.md`)
- `[P]` = letto nel sorgente o in documentazione ufficiale primaria (codice del Motore, guida PyO3, API GitHub, API crates.io).
- `[I]` = inferenza mia, derivata dal codice ma **non** verificata compilando o eseguendo.

---

## 0. Risposta in breve

1. La firma è di **sedici parametri**, non cinque: l'ellissi del rapporto precedente nascondeva otto raster opzionali, un booleano e due scalari di limitazione. Il tipo di ritorno **non è un raster** ma `ShadowingResultRust`, una struct `pub(crate)` con nove campi. `[P]`
2. **Azimut: gradi da nord in senso orario** (0 = N, 90 = E, 180 = S, 270 = O). **Altezza: gradi sopra l'orizzonte** (0 = orizzonte, 90 = zenit). Il raster ha riga 0 = nord e colonna 0 = ovest. Il test geometrico del piano passa senza conversione. `[P]`
3. `ndarray = "0.16.1"` con la feature `rayon`. La riga `ndarray = "0.16"` che il Task 3 propone è corretta e va tenuta così: **esiste già la 0.17.2** su crates.io, e prenderla romperebbe la compatibilità dei tipi. `[P]`
4. Rendere `pub` la sola `calculate_shadows_rust` **non basta**: in `lib.rs` tutti i moduli sono dichiarati `mod x;` privati, quindi `rustalgos::shadowing::…` non esisterebbe comunque. `[P]`
5. C'è un ostacolo che il piano non ha previsto e che è più grosso di `crate-type`: `pyo3` è dichiarato con la feature `extension-module` **non opzionale**, e la guida ufficiale PyO3 dice che con quella feature "binaries, tests, and examples will fail to build". La richiesta a monte deve rendere quella feature opzionale, non solo aggiungere `rlib`. `[P]` per entrambe le premesse, `[I]` per la conseguenza sul nostro binario.
6. Il tag più recente è **`v0.1.0b95` del 25/08/2026**. I tag esistono davvero (≥ 100), non solo commit. `[P]`
7. **Nessuno ha chiesto la stessa cosa a monte**: zero pull request aperte, tre issue aperte e nessuna riguarda un'API Rust nativa. `[P]`

---

## 1. La firma completa di `calculate_shadows_rust`

`[P]` — [`rust/src/shadowing.rs:187-207`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L187-L207)

```rust
/// Internal Rust function for shadow calculations.
/// Operates purely on ndarray types.
#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_shadows_rust(
    azimuth_deg: f32,
    altitude_deg: f32,
    scale: f32,
    max_local_dsm_ht: f32,
    dsm_view: ArrayView2<f32>,
    veg_canopy_dsm_view_opt: Option<ArrayView2<f32>>,
    veg_trunk_dsm_view_opt: Option<ArrayView2<f32>>,
    bush_view_opt: Option<ArrayView2<f32>>,
    walls_view_opt: Option<ArrayView2<f32>>,
    aspect_view_opt: Option<ArrayView2<f32>>,
    walls_scheme_view_opt: Option<ArrayView2<f32>>,
    aspect_scheme_view_opt: Option<ArrayView2<f32>>,
    need_full_wall_outputs: bool,
    min_sun_elev_deg: f32,
    max_shadow_distance_m: f32,
) -> ShadowingResultRust
```

Significato dei parametri, nell'ordine. I primi dodici sono documentati dal wrapper PyO3 `calculate_shadows_wall_ht_25` `[P]` — [`rust/src/shadowing.rs:789-815`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L789-L815); gli ultimi tre non compaiono nella documentazione del wrapper e sono ricostruiti dal corpo della funzione.

| # | Parametro | Tipo | Significato |
|---|---|---|---|
| 1 | `azimuth_deg` | `f32` | Azimut del sole in gradi, 0 = N, 90 = E, 180 = S, 270 = O. `[P]` |
| 2 | `altitude_deg` | `f32` | Altezza del sole sopra l'orizzonte in gradi, 0 = orizzonte, 90 = zenit. `[P]` |
| 3 | `scale` | `f32` | **Dimensione del pixel in metri.** Convenzione di `solweig`, *opposta* a quella di UMEP a monte (che passa `1/pixel_size`). Nota esplicita nel codice a [`shadowing.rs:403-408`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L403-L408). `[P]` |
| 4 | `max_local_dsm_ht` | `f32` | **Rilievo locale in metri** (max − min del DSM), non elevazione assoluta. Limita la marcia del raggio in verticale (`max_local_dsm_ht >= dz` nella guardia del ciclo) e determina la portata orizzontale `height_reach_m = max_local_dsm_ht / tan(min_sun_elev_deg)`. `[P]` — [`shadowing.rs:420-427`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L420-L427); che sia il rilievo e non il massimo lo dice il chiamante Python, [`pysrc/solweig/models/surface.py:236-258`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/pysrc/solweig/models/surface.py#L236-L258) ("Uses local relief (max - min) instead of absolute elevation"), e il commento del test di parità GPU/CPU `tests/spec/test_gpu_cpu_parity.py:368` (`10.0,  # max_local_dsm_ht (relief)`). |
| 5 | `dsm_view` | `ArrayView2<f32>` | Digital Surface Model: terreno + edifici, in metri. Definisce la forma di tutti gli altri raster. |
| 6 | `veg_canopy_dsm_view_opt` | `Option<ArrayView2<f32>>` | CDSM, altezza della chioma. |
| 7 | `veg_trunk_dsm_view_opt` | `Option<ArrayView2<f32>>` | TDSM, altezza della zona di tronco sotto la chioma. |
| 8 | `bush_view_opt` | `Option<ArrayView2<f32>>` | Arbusti / vegetazione bassa. |
| 9 | `walls_view_opt` | `Option<ArrayView2<f32>>` | Altezza delle pareti. Se `None`, tutti gli output di parete restano `None`. |
| 10 | `aspect_view_opt` | `Option<ArrayView2<f32>>` | Orientamento (normale uscente) delle facce di parete. Va sempre insieme a `walls`. |
| 11 | `walls_scheme_view_opt` | `Option<ArrayView2<f32>>` | Secondo insieme di altezze di parete, usato dallo schema UMEP di temperatura di parete. |
| 12 | `aspect_scheme_view_opt` | `Option<ArrayView2<f32>>` | Orientamento appaiato a `walls_scheme`. |
| 13 | `need_full_wall_outputs` | `bool` | Se `false`, `wall_sh`, `wall_sh_veg`, `face_sh`, `face_sun` non vengono prodotti nel ramo zenit/notte. Il wrapper Python passa sempre `true`. `[P]` — [`shadowing.rs:937`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L937) |
| 14 | `min_sun_elev_deg` | `f32` | Elevazione solare minima usata per limitare la portata dell'ombra. Default del wrapper Python: **3.0**. `[P]` — [`shadowing.rs:938`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L938) |
| 15 | `max_shadow_distance_m` | `f32` | Distanza orizzontale massima dell'ombra in metri. **0.0 = nessun limite.** Default del wrapper Rust: 0.0; il layer Python di tiling passa 1000.0. `[P]` — [`shadowing.rs:939`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L939) e [`specs/shadows.md:60`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/specs/shadows.md#L60) |

### Il tipo di ritorno

Non è un raster. `[P]` — [`rust/src/shadowing.rs:150-161`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L150-L161)

```rust
/// Rust-native result struct for internal shadow calculations.
pub(crate) struct ShadowingResultRust {
    pub bldg_sh: Array2<f32>,
    pub veg_sh: Array2<f32>,
    pub veg_blocks_bldg_sh: Array2<f32>,
    pub wall_sh: Option<Array2<f32>>,
    pub wall_sun: Option<Array2<f32>>,
    pub wall_sh_veg: Option<Array2<f32>>,
    pub face_sh: Option<Array2<f32>>,
    pub face_sun: Option<Array2<f32>>,
    pub sh_on_wall: Option<Array2<f32>>,
}
```

Il campo che serve a CLIMESH è **`bldg_sh`**. I campi sono `pub` dentro una struct `pub(crate)`: basta cambiare la visibilità della struct, non quella dei campi.

### Polarità di `bldg_sh`: 1 = al sole, 0 = in ombra

Durante la marcia del raggio la variabile accumula il contrario (1 dove il raggio è ostruito), e viene **invertita in blocco alla fine**: `bldg_sh.par_mapv_inplace(|v| 1.0 - v);` `[P]` — [`shadowing.rs:569`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L569). Coerente con `specs/shadows.md:38-40` ("A pixel is sunlit if…"). I pixel dove il DSM non è finito restano `NaN`.

### Due rami che tornano prima

`[P]` — [`shadowing.rs:213-250`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L213-L250)

```rust
if altitude_deg >= 89.5 || altitude_deg < 0.0 { … tutto illuminato … }
```

Zenit e **notte** tornano entrambi il raster tutto a 1.0, cioè tutto al sole. Il commento lo dice esplicitamente: di notte la marcia scenderebbe sotto il terreno e nessun pixel risulterebbe in ombra, e il nucleo non se ne cura perché "shortwave is zeroed at night". **La guardia `if altezza_gradi <= 0.0 { return Raster::zeros(…) }` che il piano mette in `motore::ombre` è quindi necessaria e corretta**: senza, il test `nothing_is_lit_when_the_sun_is_below_the_horizon` fallirebbe.

---

## 2. Le convenzioni angolari e di griglia

Tutte e quattro le affermazioni seguenti sono lette, non presunte.

**Azimut: gradi da nord in senso orario.** `[P]`
- [`specs/OVERVIEW.md:146`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/specs/OVERVIEW.md#L146): "**Azimuth**: 0° = North, 90° = East, 180° = South, 270° = West".
- [`specs/technical.md:93`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/specs/technical.md#L93), stessa frase.
- [`specs/shadows.md:48`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/specs/shadows.md#L48), tabella degli input.
- Docstring del wrapper PyO3, [`shadowing.rs:802`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L802): "`azimuth_deg` - Sun azimuth in degrees (0=N, 90=E, 180=S, 270=W)".
- Il layer Python di posizione solare dichiara la stessa cosa: "sun.azimuth = azimuth angle in degrees, eastward from the north" — [`pysrc/solweig/physics/sun_position.py:58`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/pysrc/solweig/physics/sun_position.py#L58).

**Altezza solare: gradi sopra l'orizzonte.** 0° = orizzonte, 90° = zenit. `[P]` — `specs/OVERVIEW.md:147`, `specs/technical.md:94`, docstring a `shadowing.rs:803`.

**Orientamento della griglia.** Riga 0 = nord, righe crescenti verso sud; colonna 0 = ovest, colonne crescenti verso est. `[P]` — [`specs/technical.md:86-89`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/specs/technical.md#L86-L89) e `specs/OVERVIEW.md:145`.

**Verifica indipendente sul codice, non sulle spec.** `[I]` (derivazione mia dal sorgente, non da esecuzione) — nel ciclo di marcia, [`shadowing.rs:436-448`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L436-L448), `dx` indicizza le righe e `dy` le colonne, con

```rust
dy = sign_sin_azimuth * index;
dx = -1.0 * sign_cos_azimuth * (index / tan_azimuth).round().abs();
```

Con azimut 180°: `sin = 0`, `cos = −1`, quindi `dy = 0` e `dx = +index`. Lo slicing successivo ([`shadowing.rs:453-460`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/shadowing.rs#L453-L460)) con `dx > 0` dà sorgente `[dx..num_rows)` e destinazione `[0..num_rows−dx)`: l'altezza della riga `r` si propaga verso la riga `r − dx`, cioè verso **indici di riga minori, cioè verso nord**. Sole a sud, ombra a nord.

> **Conseguenza per Task 4**: il test `the_shadow_of_a_tower_falls_north_when_the_sun_is_south` dovrebbe passare passando l'azimut così com'è, senza conversione. `[I]` — è una derivazione dal codice, non un'esecuzione; la conferma vera è il test verde nel Task 4b.

---

## 3. La versione di `ndarray`

`[P]` — [`rust/Cargo.toml:12`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/Cargo.toml#L12)

```toml
ndarray = { version = "0.16.1", features = ["rayon"] }
```

Confermato anche dal lock file del Motore, `rust/Cargo.lock:519-521`: `name = "ndarray"`, `version = "0.16.1"`, sorgente crates.io.

Tre note operative.

1. **La riga proposta dal Task 3 (`ndarray = "0.16"`) è giusta e va lasciata così.** Il requisito caret `^0.16` risolve nell'ultima `0.16.x` disponibile, oggi la 0.16.1, cioè esattamente la stessa che risolve il Motore: un solo `ndarray` nel grafo, un solo tipo `ArrayView2<f32>`.
2. **Attenzione a non "aggiornare" a 0.17.** Su crates.io la versione massima è **0.17.2** `[P]` — `GET https://crates.io/api/v1/crates/ndarray`, campo `max_stable_version`. Siccome `ndarray` è pre-1.0, `0.17` e `0.16` sono semver-incompatibili: se il nostro `Cargo.toml` dicesse `"0.17"`, cargo compilerebbe due copie di `ndarray` e `dsm.view()` non sarebbe assegnabile al parametro `dsm_view` del Motore. L'errore che ne uscirebbe parla di "expected `ArrayView2<f32>`, found `ArrayView2<f32>`" e non è ovvio da leggere.
3. La feature `rayon` è attivata dal Motore. Per l'unificazione delle feature di cargo finirà attiva anche per noi, il che tira dentro `rayon` come dipendenza transitiva. Non cambia i tipi, ma non è più "solo `ndarray`" in termini di albero delle dipendenze. `[I]`

Altre dipendenze del Motore, per dimensionare l'albero: `pyo3 0.24.2`, `numpy 0.24.0`, `rayon 1.10.0`, e con la feature `gpu` (attiva di default) `wgpu 27.0`, `pollster 0.4`, `bytemuck 1.24`, più `windows 0.61` su Windows e `libc 0.2` su Linux. `[P]` — `rust/Cargo.toml:9-25`.

---

## 4. Le altre funzioni `pub(crate)`: quanto serve per arrivare a Tmrt e agli indici

Nel crate ci sono **39 funzioni `pub(crate)`** (`grep -rn "pub(crate) fn" rust/src/` sul clone). Non servono tutte. Sotto, quelle che stanno sulla catena da geometria a Tmrt e comfort, raggruppate per stadio della pipeline di `specs/OVERVIEW.md`. Tutti i riferimenti sono `[P]`, letti in `rust/src/`.

**Geometria precalcolata (una volta per scenario)**

| Funzione | File:riga | Nota |
|---|---|---|
| `compute_wall_aspect_pure` | `wall_aspect.rs:146` | Da DSM a altezza e orientamento delle pareti. Ritorna `Result<Array2<f32>, &'static str>`. |
| `binary_dilation_pure` | `morphology.rs:15` | Ausiliaria della precedente. |
| `calculate_svf_inner` | `skyview.rs:330` | **Non è `pub(crate)`: è privata (`fn`)**, e ritorna `PyResult<SvfIntermediate>`. È l'unico ingresso al calcolo dei 17 raster SVF senza passare da Python. Va resa `pub` *e* va tolto il `PyResult` dal ritorno, altrimenti la firma pubblica trascina `pyo3`. |
| `precompute_gvf_geometry_cpu` / `_gpu` | `gvf_geometry.rs:255` / `:368` | Cache geometrica del GVF, ritornano `GvfGeometryCache` (`pub(crate) struct` a `gvf_geometry.rs:37`). |

**Per passo temporale**

| Funzione | File:riga | Nota |
|---|---|---|
| `calculate_shadows_rust` | `shadowing.rs:191` | Ombre. Quella del § 1. |
| `compute_ground_temperature_pure` | `ground.rs:19` | Temperatura del suolo. Ritorna `GroundTempResult` (`pub(crate) struct`, `ground.rs:10`). |
| `ts_wave_delay_batch_pure` | `ground.rs:134` | Ritardo termico. Ritorna `TsWaveDelayBatchPureResult` (`ground.rs:115`). |
| `surface_temperature_calc_pure` | `ground_surface.rs:64` | Schema 2026a, opzionale. Ritorna `SurfaceTemperatureResult` (`ground_surface.rs:50`). |
| `outgoing_longwave_calc_pure` | `ground_surface.rs:365` | Schema 2026a, opzionale. Ritorna `OutgoingLongwaveResult` (`ground_surface.rs:273`). |
| `gvf_calc_pure` | `gvf.rs:175` | GVF, versione senza cache. Ritorna `GvfResultPure` (`gvf.rs:116`). |
| `gvf_calc_with_cache` / `_gpu` | `gvf.rs:370` / `:556` | Varianti con cache; la GPU ritorna `Result<GvfResultPure, String>`. |
| `compute_sunwall_mask` | `gvf.rs:152` | Ausiliaria del GVF. |
| `anisotropic_sky_pure` | `sky.rs:236` | Cielo anisotropo, 27 parametri. Ritorna `SkyResultPure` (`sky.rs:212`). |
| `cylindric_wedge_pure`, `cylindric_wedge_pure_masked` | `sky.rs:752`, `:756` | Geometria del corpo cilindrico. |
| `lside_veg_pure`, `lside_veg_variant_pure` | `vegetation.rs:270`, `:63` | L laterale con vegetazione. Ritornano `LsideVegPureResult` (`vegetation.rs:32`). |
| `kside_veg_isotropic_pure` | `vegetation.rs:348` | K laterale isotropo. Ritorna `KsideVegPureResult` (`vegetation.rs:336`). |
| `compute_esky`, `compute_kup`, `compute_ldown`, `compute_kdown`, `asvf_for_svf_cached`, `weighted_side_sum_four`, `lside_dirs_sum_aniso_from_lup`, `kside_dirs_sum_aniso_from_kup`, `side_sum_from_directional`, `patch_lut_for_option_cached`, `compute_ani_lum_from_packed` | `pipeline_radiation.rs:23, 37, 103, 153, 189, 257, 280, 291, 302, 336, 374` | Undici funzioni: sono i pezzi di radiazione che l'orchestratore incolla fra loro. |
| `create_patches`, `compute_steradians`, `patch_alt_azi_steradians_for_patch_option`, `perez_v3` | `perez.rs:97, 150, 220, 234` | Discretizzazione del cielo e modello di Perez. |
| `compute_tmrt_pure` | `tmrt.rs:56` | **Tmrt.** 18 parametri, ritorna `Array2<f32>`: nessun tipo proprio nella firma. |
| `compute_tmrt_from_dir_sums_pure` | `tmrt.rs:100` | Variante con le componenti direzionali già sommate. |

**Indici di comfort — già pronti**

`utci::utci_single` ([`utci.rs:288`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/utci.rs#L288)) e `pet::pet_calculate` ([`pet.rs:354`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/pet.rs#L354)) sono **già `pub`, prendono solo `f32`/`i32` e ritornano `f32`**. Portano l'attributo `#[pyfunction]`, che però non cambia la firma Rust. Sono consumabili appena il modulo diventa pubblico, senza nessun'altra modifica. Le varianti su griglia (`utci_grid`, `pet_grid`) sono invece PyO3 pure e vanno riscritte da noi con `rayon`: sono cicli su celle indipendenti, un `par_mapv_inplace` per lato.

`sun::sun_on_surface` e `sun::sun_on_surface_cached` (`sun.rs:7`, `:355`) sono già `pub`. **Attenzione al nome**: non calcolano la posizione del sole, servono al ray casting del GVF. **Il crate Rust non calcola affatto la posizione solare**: quella sta in Python, `pysrc/solweig/physics/sun_position.py`. Il modulo `sole::posizione` che il Task 4 prevede di scrivere in casa è quindi necessario, non ridondante. `[P]`

**Il buco: non esiste un orchestratore Rust puro**

`pipeline.rs` ha **zero** funzioni `pub(crate)`. L'unico orchestratore per passo temporale è `compute_timestep`, [`pipeline.rs:502`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/pipeline.rs#L502): è `#[pyfunction]` e prende `Python`, `PyReadonlyArray2/3` e cinque bundle `#[pyclass]` (`SvfBundle`, `SurfaceBundle`, `PropertiesBundle`, `StateBundle`, `GroundSchemeBundle`, in `pipeline_bundles.rs:29, 101, 141, 186, 269`), più tre struct di scalari `#[pyclass]` (`WeatherScalars`, `HumanScalars`, `ConfigScalars`, in `pipeline_scalars.rs:13, 102, 129`). **Non ha un gemello puro.** `[P]`

> `[I]` Conseguenza per il dimensionamento della richiesta a monte: chiedere "esponete la catena completa" significa chiedere due cose diverse. Le funzioni `_pure` di stadio sono già pronte e costano solo visibilità. L'orchestratore no: o lo si riscrive in casa chiamando i pezzi `_pure` uno per uno (~1500 righe di `compute_timestep` da rifare senza i tipi Python), oppure a monte va aggiunto un `compute_timestep_pure` che oggi non esiste. **La superficie sensata da chiedere è la prima**: moduli pubblici + funzioni `_pure` e loro struct di ritorno pubbliche, lasciando a noi l'orchestrazione. È anche la richiesta meno invasiva per chi la deve accettare.

---

## 5. Cosa serve esattamente perché il crate sia consumabile

Quattro cose, non una. Le prime due sono quelle che il piano prevede; la terza e la quarta no.

### 5.1 `crate-type`

`rust/Cargo.toml:7` `[P]`:

```toml
crate-type = ["cdylib"]        # oggi
crate-type = ["cdylib", "rlib"]  # necessario
```

### 5.2 I moduli, prima delle funzioni

**Questo il piano lo dà per scontato e non lo è.** In `rust/src/lib.rs:3-26` **tutti e ventiquattro i moduli sono dichiarati `mod x;`, privati** `[P]` — [`lib.rs:3-26`](https://github.com/UMEP-dev/solweig/blob/d3dfb2a1d0a7b284b4a530e8439f1966a917361e/rust/src/lib.rs#L3-L26). Non c'è nessun `pub mod`, nessun `pub use`. Rendere `pub` la funzione `calculate_shadows_rust` senza toccare `lib.rs` non produce nulla di raggiungibile: il percorso `rustalgos::shadowing::calculate_shadows_rust` che il Task 4 scrive richiede `pub mod shadowing;`.

Minimo per le sole ombre: `pub mod shadowing;` + `pub(crate) fn calculate_shadows_rust` → `pub fn` + `pub(crate) struct ShadowingResultRust` → `pub struct`.

### 5.3 I tipi propri del crate che comparirebbero nella firma pubblica

Vanno esportati anche loro. Per le sole ombre ne basta uno; per la catena completa sono undici. `[P]`, tutti letti in `rust/src/`:

| Tipo | File:riga | Compare come |
|---|---|---|
| `ShadowingResultRust` | `shadowing.rs:151` | ritorno di `calculate_shadows_rust` |
| `SvfIntermediate` | `skyview.rs:256` | ritorno di `calculate_svf_inner` — **già `pub struct`**, ma nel modulo privato |
| `GvfResultPure` | `gvf.rs:116` | ritorno delle tre `gvf_calc_*` |
| `GvfGeometryCache`, `AzimuthGeometry` | `gvf_geometry.rs:37`, `:13` | parametro e ritorno della precomputazione GVF |
| `SkyResultPure` | `sky.rs:212` | ritorno di `anisotropic_sky_pure` |
| `LsideVegPureResult`, `KsideVegPureResult` | `vegetation.rs:32`, `:336` | ritorni della vegetazione |
| `GroundTempResult`, `TsWaveDelayBatchPureResult` | `ground.rs:10`, `:115` | ritorni del suolo |
| `SurfaceTemperatureResult`, `OutgoingLongwaveResult` | `ground_surface.rs:50`, `:273` | ritorni dello schema 2026a |
| `PatchOptionLut` | `pipeline_radiation.rs:325` | ritorno di `patch_lut_for_option_cached` |

Nessuno di questi contiene tipi Python: sono `Array2<f32>`, `Array3<u8>`, `Option<…>` e scalari. Il loro campi sono già `pub`. `[P]`

Il caso storto è **`calculate_svf_inner`**, che ritorna `PyResult<SvfIntermediate>` (`skyview.rs:342`): l'unica funzione della catena la cui firma trascinerebbe `pyo3` anche dopo l'apertura. Va cambiata in `Result<SvfIntermediate, String>` o simile. Stessa cosa per `crop_svf_intermediate` (`skyview.rs:849`). `[P]`

### 5.4 L'ostacolo che il piano non ha visto: `pyo3/extension-module`

`rust/Cargo.toml:10` `[P]`:

```toml
pyo3 = { version = "0.24.2", features = ["extension-module", "abi3-py311"] }
```

La feature **non è opzionale**: è scritta direttamente nella dipendenza, non dietro un flag. (Il `pyproject.toml` la riattiva anche da maturin, riga 54: `features = ["pyo3/extension-module", "pyo3/abi3-py311", "gpu"]` — ridondante rispetto a `Cargo.toml`, ma conferma l'intenzione. `[P]`)

La guida ufficiale PyO3 v0.24.2 dice, testualmente `[P]` — [pyo3.rs/v0.24.2/building-and-distribution, sezione "The extension-module feature"](https://pyo3.rs/v0.24.2/building-and-distribution):

> PyO3's `extension-module` feature is used to disable linking to `libpython` on Unix targets. […] The downside of not linking to `libpython` is that binaries, tests, and examples (which usually embed Python) will fail to build. **If you have an extension module as well as other outputs in a single project, you need to use optional Cargo features to disable the `extension-module` when you're not building the extension module.**

> `[I]` **Non ho verificato compilando.** L'inferenza è: un `rlib` con `extension-module` attiva, linkato dentro il binario CLIMESH, lascia i simboli della C API di Python non risolti, e il link del binario fallisce se anche una sola unità di codegen di `rustalgos` che li referenzia viene ritenuta (in particolare l'inizializzatore `PyInit_rustalgos` generato da `#[pymodule]`). Non so dire senza provarlo se il linker li scarti come codice morto. **È la singola incognita che decide se Task 4b regge**, ed è verificabile in dieci minuti: clonare, mettere `crate-type = ["cdylib","rlib"]`, un `pub mod shadowing;`, e provare a linkarlo da un `main.rs` di tre righe.
>
> La forma della richiesta a monte che risolve il problema alla radice, e che è anche quella che la guida PyO3 raccomanda:
> ```toml
> [dependencies]
> pyo3 = { version = "0.24.2", default-features = false, optional = true }
> numpy = { version = "0.24.0", optional = true }
>
> [features]
> default = ["gpu", "python"]
> python = ["pyo3/extension-module", "pyo3/abi3-py311", "dep:pyo3", "dep:numpy"]
> ```
> con `#[cfg(feature = "python")]` su `#[pymodule]`, sui `#[pyfunction]` e sui `#[pyclass]`. Non è un cambio di comportamento per chi compila la wheel (la feature resta di default), ed è la modifica che rende il crate consumabile davvero, non solo raggiungibile. Come effetto collaterale toglie anche il vincolo che `pyo3-build-config` cerchi un interprete Python 3 in fase di build — comportamento documentato: "By default it will attempt to use […] any active Python virtualenv, the `python` executable, the `python3` executable" `[P]`, stessa pagina — vincolo che altrimenti la nostra CI erediterebbe (aggirabile con `PYO3_NO_PYTHON`, che la stessa pagina documenta come funzionante senza condizioni sui sistemi Unix).

### 5.5 Un dettaglio minore

`rust/Cargo.toml:28-33` dichiara `[profile.release]` con `panic = "abort"`, `lto = "fat"`, `codegen-units = 1`, `strip = "symbols"`. `[P]` Cargo ignora i profili delle dipendenze e usa solo quello del pacchetto radice, quindi questa sezione non ci arriva addosso; ma se CLIMESH volesse le stesse ottimizzazioni dovrebbe dichiararle nel proprio `Cargo.toml`. `[I]`

---

## 6. Su cosa pinnare

**Esistono tag veri.** L'API GitHub ne restituisce almeno 100 (`GET /repos/UMEP-dev/solweig/tags?per_page=100` → 100 elementi, il che significa che potrebbero essercene altri oltre la prima pagina), nella forma `v0.1.0bNN`. `[P]`

Il più recente: `[P]`

| Tag | Commit | Data pubblicazione release |
|---|---|---|
| **`v0.1.0b95`** | `d3dfb2a1d0a7b284b4a530e8439f1966a917361e` | **2026-08-25T15:29:00Z** |
| `v0.1.0b94` | `39160d9470…` | 2026-08-25T13:56:26Z |
| `v0.1.0b93` | `7ba2b17950…` | 2026-08-24T12:37:17Z |
| `v0.1.0b92` | `9447c117b3…` | 2026-07-09T20:11:04Z |

`v0.1.0b95` è un tag annotato (oggetto tag `2ee3c1e69e36289a6cd5525a7b832add213f3e62` → commit `d3dfb2a…`). Il messaggio del commit è `chore: bump to 0.1.0b95 — plugin Qt6 enum fix`. `[P]`

`main` è oggi `02246ab71a3a8b127d740dde9640449ee9d558ff` (`chore: sync uv.lock to 0.1.0b95`), un commit avanti sul solo `uv.lock`. `[P]`

> **Raccomandazione, non decisione** `[I]`: pinnare il nostro fork su `v0.1.0b95`. È l'ultimo tag, la `version` in `pyproject.toml` corrisponde (`0.1.0b95`), e il codice Rust è identico a `main`. La cadenza dei tag è irregolare — tre in due giorni ad agosto dopo un mese e mezzo di silenzio — quindi il fork andrà ribasato a mano, non c'è un ciclo di rilascio da seguire. Il `Cargo.toml` del crate dichiara `version = "0.1.0"` fisso da sempre, quindi **la versione del crate non è un identificatore utile**: il `build.rs` del Task 4 che legge `version` dal lock file leggerà sempre `0.1.0`; l'informazione che vale per il Giornale è il rev git, non la versione.

---

## 7. Se qualcuno l'ha già chiesto a monte

**No.** `[P]`

- **Pull request aperte: zero.** L'unica PR mai esistita è la #6 ("Dev"), chiusa il 2026-02-07. `GET /repos/UMEP-dev/solweig/issues?state=open|closed&per_page=100` restituisce 3 issue aperte e 10 elementi chiusi, di cui una sola PR.
- **Issue aperte: tre**, nessuna sull'API Rust nativa: #13 "Grid shape mismatch between DSM and surface grids in SOLWEIG QGIS Beta83" (23/08/2026), #7 "End-to-end test against real-world data" (20/02/2026), #1 "Rust version of SOLWEIG" (19/08/2025).
- Ricerca mirata: `GET /search/issues?q=repo:UMEP-dev/solweig+rust+api+in:title,body` → `total_count: 0`; `…+rlib+OR+crate-type` → `total_count: 0`.

La #1 è un thread di annuncio e coordinamento aperto da `songololo` (l'autore del port Rust) verso `biglimp` (Fredrik Lindberg, UMEP). I dieci commenti parlano di GPU, tiling, parità con il Python di riferimento e di estendere Rust ad altri strumenti UMEP. **Nessuno chiede un'API Rust per consumatori Rust esterni.** Un solo commento sfiora il tema, e va nella direzione opposta: `songololo`, 23/02/2026 — "I'm trying to keep everything bundled into the Rust binary so that it is easy to install from QGIS 4 or from Python". `[P]` — [issue #1](https://github.com/UMEP-dev/solweig/issues/1)

> `[I]` Il fork non è superfluo, e la richiesta a monte non ha precedenti su cui appoggiarsi. Due letture del contesto, entrambe ipotesi: il progetto ha un contributore principale attivo e reattivo (risponde entro giorni nel thread #1), il che gioca a favore di una PR ben confezionata; ma non ha mai considerato consumatori Rust, quindi la PR introdurrebbe un vincolo di compatibilità nuovo per loro, e la reazione non è prevedibile. Nel dubbio, il piano fa bene a dichiarare che **non dipende dall'esito**.

---

## 8. Correzioni che il Task 4 deve incassare

Non sono opinioni: sono differenze fra ciò che il piano scrive e ciò che il sorgente dice.

1. **La chiamata in `motore::ombre` ha cinque argomenti, ne servono sedici**, e il valore di ritorno non è il raster ma la struct. La forma corretta, salvo errori di battitura:

```rust
rustalgos::shadowing::calculate_shadows_rust(
    azimut_gradi as f32,
    altezza_gradi as f32,
    passo_m as f32,       // dimensione del pixel in metri: convenzione giusta
    rilievo,              // max - min, non max
    dsm.view(),
    None, None, None,     // veg_canopy, veg_trunk, bush
    None, None,           // walls, aspect
    None, None,           // walls_scheme, aspect_scheme
    false,                // need_full_wall_outputs
    3.0,                  // min_sun_elev_deg, default del wrapper Python
    0.0,                  // max_shadow_distance_m, 0 = nessun limite
).bldg_sh
```

2. **`max_local_dsm_ht` è il rilievo, non il massimo.** Il piano scrive `dsm.iter().copied().fold(0.0f32, f32::max)`. Sul dominio di prova (piano a 0 con una torre a 10) i due coincidono perché il minimo è 0, quindi i test passano lo stesso; su un DSM reale con quota assoluta la portata dell'ombra verrebbe sovrastimata di parecchio. Serve `max − min`.

3. **Lo step 1 del Task 4 va esteso.** "Cambiare `crate-type` e rendere `pub` la funzione" non produce un crate consumabile: mancano `pub mod shadowing;` in `lib.rs`, `pub struct ShadowingResultRust`, e soprattutto la feature `extension-module` opzionale (§ 5.4). Il comando `grep -n "fn calculate_shadows_rust" -A 12` che il piano prevede stampa **dodici righe su sedici parametri**: taglia la firma esattamente dove il rapporto precedente aveva messo l'ellissi. Serve `-A 20`.

4. **La guardia sotto l'orizzonte serve davvero** (§ 1, ultimo capoverso): il nucleo tratta la notte come "tutto al sole". Il commento del piano ("che il nucleo non decide per noi perché è una scelta di modello") descrive bene la situazione.

---

## 9. Cosa resta non verificato

Elenco esplicito, perché il Task 4b si rompe proprio qui.

1. **Che il crate linki dentro un binario Rust con `extension-module` attiva.** `[I]` — § 5.4. È il rischio numero uno e si chiude con una compilazione di prova. Non l'ho fatta perché il mandato di questa ricerca è di sola lettura e vieta esplicitamente di clonare per modificare.
2. **Che il test dell'ombra a nord passi senza conversione di azimut.** `[I]` — § 2. La derivazione dal codice è netta, ma resta una lettura di codice, non un'esecuzione.
3. **Se `cargo build` del Motore richieda un interprete Python nell'ambiente.** `[I]` — documentato da PyO3 come comportamento di default di `pyo3-build-config`, ma non provato sulla nostra macchina né in CI.
4. **Se la feature `gpu` (attiva di default, tira `wgpu 27.0`) vada disattivata da noi.** Non l'ho valutata. Se la si tiene, l'albero di dipendenze di CLIMESH cresce di parecchio e smette di essere "Rust puro senza librerie native"; se la si toglie con `default-features = false`, si perde l'accelerazione che il rapporto `solweig-riuso.md` § 5 misurava a 1,7 s contro 36,4 s. **È una decisione, non un dato, e non è mia.**
