# Quanto costa la catena piena — misura sul caso di Bastia

**Scopo**: rispondere al ticket wayfinder [#24](https://github.com/maeurong/CLIMESH/issues/24) della mappa [#17](https://github.com/maeurong/CLIMESH/issues/17), che aspetta un numero prima di decidere se il cielo anisotropo resta acceso ([#27](https://github.com/maeurong/CLIMESH/issues/27)).

**Data della misura**: 2026-09-07.
**Repo CLIMESH**: ramo `research/costo-catena-piena`, staccato da `main` a HEAD `6f675ab` («Record the upstream request in ADR 0003»).
**Motore letto ed eseguito**: `UMEP-dev/solweig` al commit `02246ab71a3a8b127d740dde9640449ee9d558ff`, che è il commit pinnato in `vendor/solweig/PROVENIENZA.toml` (chiave `commit`). Non un commit vicino: lo stesso.
**Macchina**: 12th Gen Intel Core i7-12700K, 20 thread, WSL2 su kernel 6.18.33.2, `rustc` 1.98.0, profilo `release` di CLIMESH (nessun `[profile.release]` in `Cargo.toml`, quindi i valori predefiniti di cargo).

**Convenzione di marcatura** (la stessa di `research/motore-api.md`)
- `[M]` = **misurato** su questa macchina, con il comando riportato accanto.
- `[A]` = **aritmetica** su quantità lette nel sorgente: esatta, non cronometrata.
- `[S]` = **stima**, estrapolata da un `[M]` con un ragionamento dichiarato. Non è una misura.

---

## 0. Risposta in breve

1. Il **passo fuso isotropo** costa **2,2–2,4 ms** per passo temporale sul dominio di Bastia, 2500 celle. Quarantotto passi costano **105–116 ms** per Corsa. `[M]`
2. Il **passo fuso anisotropo** costa **12,7–13,3 ms** per passo, **609–640 ms** per Corsa: **5,5 volte** l'isotropo sul totale del passo, **20 volte** sul solo blocco della radiazione. `[M]`
3. Le quattro Corse del caso di riferimento, catena piena e cielo anisotropo, stanno in **circa 2,7 s** contro un budget di **60**. Con cielo isotropo, in circa 0,65 s. `[S]`, con il metodo in § 5 — e sono limiti **superiori**, per la ragione di § 4.
4. **I 17 sky view factor non costano niente più dei 2.** Quindici escono già dalla stessa chiamata al nucleo che facciamo oggi e li buttiamo via; i due che mancano sono aritmetica elemento per elemento, e uno dei due CLIMESH lo calcola già. `[A]`
5. Le tre **matrici d'ombra per patch** occupano **150 000 byte** a 50 × 50 e **600 000 byte** al tetto di progetto di 200 × 200 m a 2 m. Sono impacchettate a bit lungo l'asse dei patch: 20 byte per cella per 153 patch, non 153. `[A]`
6. **La misura vecchia di 29-36 s non va ereditata, e adesso si sa perché.** Non è solo il numero di thread: è che il percorso Python chiama il nucleo **da fuori** dal pool di rayon. Lo stesso nucleo, sugli stessi raster, con `RAYON_NUM_THREADS=1`, costa **12 volte** di più chiamato fuori da `install` che dentro — e con il pool predefinito **1000 volte** di più. `[M]`, § 4.

---

## 1. Cosa è stato misurato, e con che cosa

Due programmi, tutti e due sul ramo, in `research/costo-catena-piena/`.

**`src/main.rs`** è un binario Rust che dipende da `climesh` per percorso. Legge `casi/bastia/progetto`, deriva i raster di ciascuna delle quattro Corse e cronometra quello che il Motore fa oggi: `motore::sky_view_factor` una volta per Corsa, `motore::ombre` una volta per ora. Scrive anche i raster in CSV, perché il programma Python li rilegga: la geometria misurata dalle due parti è così la stessa geometria, non una somiglianza.

```
cargo run --release --manifest-path research/costo-catena-piena/Cargo.toml -- casi/bastia/progetto
```

**`passo_fuso.py`** dà quei CSV a `solweig.calculate` di monte e cronometra il passo fuso. La ripartizione per blocco la stampa il nucleo stesso su stderr quando `SOLWEIG_TIMING=1` — [`rust/src/pipeline.rs:38`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/rust/src/pipeline.rs#L38) e la `eprintln!` a [`rust/src/pipeline.rs:1926`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/rust/src/pipeline.rs#L1926) — quindi i sei numeri per passo non sono nostri: sono la strumentazione di monte, letta e sommata.

`misura.sh` esegue i due in fila e rifà tutta la tabella.

**Come è stato costruito l'ambiente di monte**, che è la parte che vale la pena rifare o contestare:

- clone locale di `UMEP-dev/solweig` già al commit pinnato, copiato in una cartella di lavoro fuori dal repository;
- nella copia, e **solo** nella copia, la feature `gpu` è stata tolta da `pyproject.toml` (`[tool.maturin] features`) e da `rust/Cargo.toml` (`default = []`). Serviva a non compilare `wgpu 27`, e non cambia il percorso CPU: `vendor/solweig/PROVENIENZA.toml` dichiara che la nostra copia vendorata non compila nemmeno quella feature, quindi la copia costruita qui è nella **stessa** configurazione della nostra;
- `uv sync --no-default-groups --python 3.12`. Python 3.12 e non 3.13 perché `uv.lock` pinna `numpy 1.26.4`, che per 3.13 non ha ruota e vorrebbe un compilatore C++ che questa macchina non ha;
- ogni esecuzione con `RAYON_NUM_THREADS=1`, che mette il pool globale di monte nel numero di thread in cui CLIMESH tiene il proprio ([`src/motore.rs:295`](../src/motore.rs), `pool_a_un_thread`). Senza, i numeri non parlano del nostro caso: § 4.

---

## 2. Il passo fuso, misurato

Somme su 48 passi, in millisecondi, per Corsa. `[M]` — due esecuzioni complete della matrice; i valori sotto sono la seconda, la prima sta entro il 3 % su ogni cella tranne `gvf`, che è troppo piccola perché la differenza significhi qualcosa.

| Corsa | cielo | shadow | ground | gvf | delay | radiation | tmrt | **totale** | per passo |
|---|---|---|---|---|---|---|---|---|---|
| stato-di-fatto — estate | isotropo | 74,0 | 4,8 | 0,2 | 5,6 | 29,0 | 0,4 | **116,1** | 2,42 |
| stato-di-fatto — estate | anisotropo | 79,3 | 4,8 | 0,6 | 6,4 | 543,0 | 0,6 | **636,4** | 13,26 |
| stato-di-fatto — inverno | isotropo | 68,9 | 4,4 | 0,2 | 5,0 | 24,4 | 0,1 | **105,6** | 2,20 |
| stato-di-fatto — inverno | anisotropo | 71,0 | 4,8 | 0,3 | 6,4 | 522,6 | 0,4 | **609,2** | 12,69 |
| interventi — estate | isotropo | 74,5 | 4,4 | 0,3 | 5,3 | 28,3 | 0,1 | **114,4** | 2,38 |
| interventi — estate | anisotropo | 78,7 | 4,8 | 0,4 | 6,4 | 546,5 | 0,3 | **639,7** | 13,33 |
| interventi — inverno | isotropo | 69,3 | 4,5 | 0,3 | 5,0 | 24,3 | 0,3 | **105,2** | 2,19 |
| interventi — inverno | anisotropo | 71,7 | 4,8 | 0,1 | 6,4 | 534,7 | 0,4 | **621,2** | 12,94 |

Quattro letture che il ticket chiedeva:

**Il cielo anisotropo pesa 5,5 volte l'isotropo** sul totale del passo, e **20,3 volte** sul solo blocco `radiation` (2146,8 ms contro 106,0 ms, sommando le quattro Corse). Tutto il resto del passo non se ne accorge: `ground`, `delay` e `tmrt` sono identici entro il rumore. È un interruttore che accende **un** blocco, non una catena diversa.

**Il blocco `gvf` non compare** perché la geometria del ground view factor si calcola una volta e poi viaggia in una cache (`gvf_cache`, [`pipeline.rs:509`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/rust/src/pipeline.rs#L509)). A 2500 celle costa meno di un millisecondo la prima volta.

**Lo `shadow` di monte è dieci volte il nostro** — 69-79 ms per Corsa contro i 6-10 ms che `motore::ombre` misura sugli stessi raster. Non è una differenza di algoritmo: è la stessa funzione. § 4 dice cos'è.

**Le ore di notte costano quasi zero**: `shadow=0.0ms` e `total` fra 0,6 e 0,8 ms, contro i 3-5 ms delle ore basse del mattino. Il passo fuso non ha una scorciatoia notturna come la nostra `ALTEZZA_MINIMA_GRADI` ([`src/motore.rs:59`](../src/motore.rs)), ma il nucleo delle ombre esce presto da solo.

---

## 3. Quello che CLIMESH fa oggi, rimisurato

`[M]` — stesso binario, stessa macchina, stesso giro.

| Corsa | strati di chioma | sky view factor | ombre, 48 ore | ore sopra 2° | per ora di giorno |
|---|---|---|---|---|---|
| stato-di-fatto — estate | 1 | 29,4 ms | 6,4 ms | 28 | 0,230 ms |
| stato-di-fatto — inverno | 2 | 51,3 ms | 9,4 ms | 18 | 0,521 ms |
| interventi — estate | 1 | 28,1 ms | 6,7 ms | 28 | 0,238 ms |
| interventi — inverno | 2 | 53,8 ms | 9,5 ms | 18 | 0,528 ms |

Somma delle quattro Corse, senza Giornale né lettura dei file: **0,186–0,195 s** su tre esecuzioni.

E il caso completo dal binario, `/usr/bin/time ./target/release/climesh esegui casi/bastia/progetto`, tre giri: **0,51 / 0,53 / 0,42 s**. Il numero di 0,47 s che la mappa #17 porta è confermato; la differenza fra 0,19 e 0,47 è Giornale, lettura dei file e avvio.

L'inverno costa il doppio dell'estate su tutte e due le voci perché ha due Strati di chioma e il nucleo si percorre una volta per strato — è la ragione dichiarata in [`src/motore.rs:184`](../src/motore.rs), `sky_view_factor`.

---

## 4. Il tranello, e cos'era davvero

Il ticket avvisava che i 29-36 s della issue #2 venivano dal percorso Python col pool rayon predefinito e non vanno ereditati. Vero, e c'è un secondo strato sotto.

Il nucleo divide i suoi cicli interni con `par_azip!`. Chiamato da un thread che **non** è un worker di rayon, ogni divisione è una consegna al pool e un ritorno. `motore::ombre` avvolge ogni chiamata in `pool_a_un_thread().install(…)`, quindi la chiusura gira già **sul** thread del pool e le divisioni restano dove sono. Il percorso Python di monte non installa niente.

Il programma Rust misura le due cose sullo stesso nucleo, sugli stessi raster, nella stessa esecuzione. `[M]`

| | dentro `install` | fuori, `RAYON_NUM_THREADS=1` | fuori, pool predefinito (20 thread) |
|---|---|---|---|
| stato-di-fatto — estate | 0,191 ms/ora | 2,281 ms/ora — **11,9×** | 244,9 ms/ora — **1066×** |
| stato-di-fatto — inverno | 0,486 | 4,797 — 9,9× | 386,9 — 742× |
| interventi — estate | 0,213 | 2,749 — 12,9× | 237,9 — 998× |
| interventi — inverno | 0,493 | 4,775 — 9,7× | 380,9 — 723× |

Due conseguenze.

**La prima**: 2,28 ms per ora di giorno fuori da `install` con un thread solo è esattamente il conto che torna con il `shadow=74,0 ms` di monte su ~30 passi di giorno. La differenza fra il nostro numero e il loro è spiegata per intero, e non resta niente di misterioso da attribuire all'algoritmo.

**La seconda**: i 1066× della terza colonna sono i «quattro ordini di grandezza» che `PROVENIENZA.toml` aveva già registrato sullo sky view factor. Adesso si vede che non è una proprietà dello sky view factor: è una proprietà di **come si chiama il nucleo**, e vale per le ombre allo stesso modo. Su un dominio di 2500 celle il costo del fork-join di rayon è di due o tre ordini di grandezza sopra l'aritmetica che divide.

Perciò tutti i numeri di § 2 sono **limiti superiori** per CLIMESH: portano dentro la consegna al pool, che nel nostro binario `install` toglierebbe.

---

## 5. La proiezione contro il budget

`[S]`, e la marcatura conta: sotto non c'è nessuna misura del passo fuso dentro `install`, perché il passo fuso da dentro `install` non lo si può ancora chiamare.

Il metodo è la sostituzione. Della catena piena, `shadow` è l'unico blocco per cui abbiamo il numero da tutte e due le parti; per quel blocco si prende il nostro, per il resto si prende quello di monte così com'è, penalità di consegna compresa.

Quattro Corse, cielo **anisotropo**:

| voce | ms | da dove |
|---|---|---|
| sky view factor, una volta per Corsa | 162,6 | `[M]` CLIMESH, § 3 |
| ombre, 48 ore per Corsa | 32,0 | `[M]` CLIMESH, § 3 |
| passo fuso meno il suo `shadow` | 2205,8 | `[M]` monte, § 2 |
| Giornale, lettura dei file, avvio | ~280 | `[M]` differenza fra 0,47 s e 0,19 s, § 3 |
| **totale** | **~2,68 s** | contro **60 s** |

Con cielo **isotropo** la terza riga diventa 154,6 ms e il totale **~0,63 s**.

**Il cielo anisotropo consuma il 4,5 % del budget, l'isotropo l'1 %.** Il margine è di ventidue volte sull'opzione cara, e il vero costo sarà più basso di così perché 2205,8 ms è un limite superiore.

Al **tetto di progetto** — 200 × 200 m a 2 m, cioè 100 × 100 = 10 000 celle, 24 ore, budget 10 minuti — `[S]` con due regole di scala dichiarate: i blocchi elemento per elemento vanno con il numero di celle, la marcia del raggio va con il cubo del lato perché ogni cella marcia lungo un dominio più lungo. Quattro volte le celle e metà delle ore danno ×2 sul passo fuso, ×4 sulle ombre e ×8 sullo sky view factor, per **circa 1,5 s per Corsa** anisotropa contro 600. Non è il vincolo che stringe.

**Cosa si è liberato.** La mappa #17 teneva in caldo la cache dello sky view factor come cosa da riprendere «se la catena piena stringe il budget». Non stringe: la catena piena anisotropa lascia inutilizzati cinquantasette secondi su sessanta.

---

## 6. I 17 sky view factor contro i 2

Il ticket dice «è la stessa chiamata, quindi probabilmente nulla, ma "probabilmente" non è una misura». Non è una misura ed è meglio: è aritmetica sulle strutture. `[A]`

`SvfIntermediate`, che `calculate_svf_inner` restituisce, porta **quindici** raster `Array2<f32>` — [`vendor/solweig/src/skyview.rs:213`](../vendor/solweig/src/skyview.rs). Sono `svf` con i suoi quattro direzionali, `svf_veg` con i suoi quattro, `svf_veg_blocks_bldg_sh` con i suoi quattro. CLIMESH ne legge due (`svf` e `svf_veg`) e lascia cadere gli altri tredici: [`src/motore.rs:184`](../src/motore.rs). Non c'è nessun ramo nel nucleo che ne calcoli meno se se ne chiedono meno, quindi **i tredici che buttiamo li stiamo già pagando**.

I due che mancano per arrivare ai 17 di `SvfBundle` non escono dal nucleo: li deriva Python, elemento per elemento, in `pysrc/solweig/components/svf_resolution.py`.

- `svfbuveg = svf − (1 − svf_veg)(1 − psi)`, poi tagliato in [0, 1] — [riga 141](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/pysrc/solweig/components/svf_resolution.py#L141). **CLIMESH calcola già esattamente questo**: `motore::detrazione` a [`src/motore.rs:268`](../src/motore.rs) è il termine che quella formula sottrae, e la funzione lo dichiara citando lo stesso file di monte.
- `svfalfa = arcsin(√(1 − clip(svf + svf_veg − 1)))` — [riga 100](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/pysrc/solweig/components/svf_resolution.py#L100). Due `clip`, una radice e un arcoseno su 2500 celle, una volta per Scenario.

**Costo dei 17 contro i 2: zero calcolo aggiuntivo nel nucleo, un arcoseno su 2500 celle una volta per Scenario, e 150 000 byte di memoria in più** (15 raster × 2500 celle × 4 byte) che oggi allochiamo comunque e liberiamo subito.

---

## 7. Quanto occupano le matrici d'ombra

`[A]` — sulle strutture, non su un profilatore.

Le tre matrici sono `Array3<u8>` di forma `(righe, colonne, ceil(patch / 8))`: impacchettate a bit lungo l'asse dei patch, un bit per patch. Lo dice `SvfIntermediate::zeros` con `pack_bytes` a [`vendor/solweig/src/skyview.rs:267`](../vendor/solweig/src/skyview.rs), e il passo fuso lo ricontrolla all'ingresso (`expected_pack = n_patches.div_ceil(8)`, [`pipeline.rs:673`](https://github.com/UMEP-dev/solweig/blob/02246ab71a3a8b127d740dde9640449ee9d558ff/rust/src/pipeline.rs#L673)).

Con l'opzione 2 del cielo, quella che CLIMESH passa (`PATCH_DEL_CIELO`, [`src/motore.rs:170`](../src/motore.rs)), i patch sono **153**: 31 + 30 + 28 + 24 + 19 + 13 + 7 + 1, sommati dalla tabella di `create_patches` a [`vendor/solweig/src/skyview.rs:156`](../vendor/solweig/src/skyview.rs). Quindi `ceil(153/8) = 20` byte per cella per matrice.

| dominio | celle | una matrice | le tre | se non fossero impacchettate |
|---|---|---|---|---|
| Bastia, 50 × 50 a 1 m | 2 500 | 50 000 B (48,8 KiB) | **150 000 B (146,5 KiB)** | 1 147 500 B |
| tetto, 200 × 200 m a 2 m | 10 000 | 200 000 B (195,3 KiB) | **600 000 B (585,9 KiB)** | 4 590 000 B |

Due avvertenze che il numero da solo non dice.

**Con più di uno Strato di chioma le matrici non sono tre.** `bldg_sh_matrix` è la stessa a ogni passata, ma `veg_sh_matrix` e `veg_blocks_bldg_sh_matrix` sono per strato, per la stessa ragione per cui lo è tutto il resto: il nucleo dice *se* una cella è coperta da vegetazione, mai *da quale*. Le Corse invernali di Bastia hanno due strati, quindi cinque matrici: **250 000 B**, e **1 000 000 B** al tetto di progetto.

**Il passo fuso ne vuole tre, non cinque.** Come due strati diventino i tre array che `compute_timestep` accetta è la stessa domanda di `psi` e `conifer` che la mappa #17 chiama «il fatto che decide più cose di tutti». Questa nota non la risponde e non la tocca: misura il costo, non la cucitura.

---

## 8. Cosa non è stato stabilito

Elenco esplicito, perché il ticket #27 deciderà sopra questi numeri e deve sapere dove finiscono.

1. **Quanto del passo fuso sopravvive dentro `install`.** Il fattore 12 di § 4 è misurato sulle **ombre** e su nient'altro. Il blocco `radiation` anisotropo passa da `rust/src/sky.rs` e `rust/src/vegetation.rs`, che usano rayon anche loro, ma quante consegne al pool faccia per passo non è stato contato. Perciò § 5 è un limite superiore e non una previsione: il numero vero sta fra 2,68 s e qualcosa di sensibilmente più basso, e per saperlo servirebbe chiamare il passo fuso da Rust — che è la cucitura che [UMEP-dev/solweig#14](https://github.com/UMEP-dev/solweig/issues/14) aspetta.
2. **La vegetazione a due strati non è stata misurata sul passo fuso**, perché il passo fuso non la accetta: ha un `psi` solo. Il programma Python fonde gli strati prendendo la chioma più alta cella per cella, che è una semplificazione fatta per poter misurare, non una proposta. Il costo di qualunque cosa #27 scelga al suo posto non è in questa nota.
3. **Il meteo è sintetico.** Il file EPW del caso sta sotto `materiale università/`, che è in `.gitignore`. `passo_fuso.py` usa `900 · sin(altezza solare)` con l'altezza calcolata dal codice solare di monte. La forma della giornata è giusta, i valori no. Il costo di un passo dipende dal fatto che il sole sia sopra l'orizzonte, non da quanti W/m² porta, quindi la sostituzione non tocca la conclusione — ma è una sostituzione.
4. **Le pareti non ci sono.** `SurfaceData` è stato costruito senza `wall_height` né `wall_aspect`, quindi `config.has_walls` era falso e la marcia del raggio non ha calcolato `wall_sun`. La mappa #17 registra che quei due raster non li produciamo; il loro costo, dentro il passo fuso e prima di esso, resta da misurare.
5. **Lo schema del suolo UMEP 2026a non è stato acceso.** Tutte le misure sono sul percorso classico (`ground_scheme = None`), che è la linea di base dichiarata byte per byte da monte.
6. **Nessun risultato è stato verificato.** Questa è una nota di tempi. Che la Tmrt che esce sia giusta lo dovrà dire la validazione della mappa, non questa misura.
7. **Il numero di ore di giorno differisce fra i due programmi** — 28 contro 30 in estate, perché CLIMESH taglia sotto 2° di altezza solare e monte calcola il sole a metà intervallo. Sono due ore di crepuscolo su 48, le più economiche della giornata; non muovono nessuna delle conclusioni, ma le tabelle di § 2 e § 3 non hanno lo stesso denominatore e non vanno divise l'una per l'altra.

---

## 9. Come rifare la misura

```sh
# 1. l'ambiente di monte, una volta
git clone https://github.com/UMEP-dev/solweig /tmp/solweig
cd /tmp/solweig && git checkout 02246ab71a3a8b127d740dde9640449ee9d558ff
#    togliere "gpu" da [tool.maturin] features in pyproject.toml
#    e mettere default = [] in rust/Cargo.toml
uv sync --no-default-groups --python 3.12

# 2. la matrice, dalla radice di CLIMESH
PY=/tmp/solweig/.venv/bin/python sh research/costo-catena-piena/misura.sh
```

I CSV che il programma Rust scrive in `research/costo-catena-piena/raster/` non stanno sul ramo: sono derivati dai file del Progetto e il programma li rifà. `misura.sh` lo esegue per primo apposta.
