# MRT e indici di comfort termico — formulazioni, implementazioni aperte, sensibilità

**Scopo**: istruire la scelta di CLIMESH su (a) quale formulazione di temperatura media radiante calcolare, (b) quale indice di comfort produrre, (c) quali implementazioni aperte riusare, (d) se il vento semplificato sia accettabile. Risponde al ticket [#3](https://github.com/maeurong/CLIMESH/issues/3).

**Data ricerca**: 2026-08-31. **Repo**: `/mnt/c/Users/mario/GitHub/CLIMESH`, branch `main`, HEAD `7f23d42` (working tree: solo `README.md` tracciato; `research/` e `materiale università/` non tracciati).

**Convenzione di marcatura**
- `[P]` = confermato da fonte primaria (norma, paper originale, sito o codice sorgente del progetto).
- `[S]` = fonte secondaria (paper di terzi, review, documentazione di riuso).
- `[I]` = inferenza mia, non confermata da fonte.
- `[M]` = **misura fatta da me in questa sessione**, con metodo e provenienza dichiarati in §4.1.

**Nota metodologica.** La skill `research` prescrive di delegare a un agente in background: non ne ho la possibilità (nessun tool di dispatch disponibile in questa sessione), quindi la ricerca è stata svolta in linea. Le pagine web non-markdown sono state lette con `defuddle`. Non erano disponibili nell'ambiente `numpy`, `scipy`, `pip` né `gfortran`: per questo la sensibilità di UTCI è stata calcolata traducendo il polinomio ufficiale in Python puro (§4.1), mentre **la sensibilità di PET non è stata misurata** e resta un vuoto dichiarato (§4.6).

---

## 0. Risposta in breve

1. **MRT**: la formulazione da adottare è quella **a sei direzioni con fattori di proiezione angolare**, nella variante modellistica di **SOLWEIG** (Lindberg, Holmer & Thorsson 2008). È il metodo che ISO 7726 definisce per la misura, che SOLWEIG implementa a partire da geometria e radiazione, e che rende commensurabili modello e misura. Gli altri metodi (globo termometro, formulazione NWP di Di Napoli) sono per misura di campo e per scala globale, non per una griglia urbana a 1 m.
2. **Indice**: **UTCI come indice primario**, **PET come secondario**. UTCI ha un'unica implementazione autoritativa (polinomio di Bröde, sorgente Fortran pubblicato) e quindi è riproducibile bit-a-bit fra strumenti; PET no — esistono almeno quattro varianti mutuamente incompatibili. PMV va escluso come indice principale.
3. **Implementazioni**: esistono e sono citabili. Per CLIMESH la più rilevante è **`UMEP-dev/solweig`**, riscrittura Rust ufficiale di SOLWEIG con Tmrt, UTCI e PET, GPL-3.0 — stessa licenza e stesso linguaggio del progetto.
4. **Vento**: il vento semplificato è **accettabile per l'estate, ed è una debolezza reale in inverno**. Misurato sul polinomio ufficiale: d(UTCI)/dv vale circa **−1 K per m/s** in condizioni estive e circa **−3 K per m/s** in condizioni invernali (§4.2). Va dichiarato, non nascosto.
5. **Segnale sole/ombra**: ΔTmrt di **20–35 K**, contro un RMSE di SOLWEIG contro misure di campo di **2,4–7,3 K**. Rapporto segnale/errore fra 3 e 10. Il segnale è largamente sopra la barra d'errore — al contrario della differenza di 0,21 °C sulla temperatura dell'aria che la relazione del caso studio dichiara.

---

## 1. Temperatura media radiante

### 1.1 Definizione

La MRT è *"la temperatura uniforme di un ipotetico recinto nero che produrrebbe lo stesso scambio radiativo netto con il soggetto dell'ambiente reale, non uniforme"*. È la definizione di ISO 7726:1998 ed è ripresa testualmente in letteratura. `[S]` — Di Napoli et al. 2020, §Introduction: https://pmc.ncbi.nlm.nih.gov/articles/PMC7295834/

La forma generale, che tutte le formulazioni specializzano:

```
MRT = [ (1/σ) · Σ_i ( E_i + α_ir · D_i / ε_p ) · F_i ]^0.25
MRT* = [ MRT^4 + f_p · α_ir · I* / (ε_p · σ) ]^0.25
```

dove `E_i = ε_i σ T_i^4` è l'emissione della superficie *i*, `D_i` la radiazione solare diffusa e riflessa da essa, `F_i` i fattori angolari, `I*` la radiazione diretta su una superficie normale al raggio, `f_p` il fattore di proiezione del corpo, `σ = 5,67·10⁻⁸ W/m²K⁴`, `ε_p = 0,97` (emissività del corpo vestito), `α_ir = 0,7` (assorbimento onda corta del corpo vestito). `[P]` — Di Napoli et al. 2020, eq. 1 e 2, https://doi.org/10.1007/s00484-020-01900-5

### 1.2 Metodo a sei direzioni con fattori di proiezione angolare (ISO 7726)

È il metodo di riferimento sia per la misura sia per la modellazione urbana. Si misurano (o si calcolano) i flussi di onda corta `K` e onda lunga `L` nelle sei direzioni — su, giù, N, E, S, O — e si pesano con i fattori di proiezione angolare della postura umana:

```
S_str = α_k · Σ K_i F_i + α_l · Σ L_i F_i
T_mrt = ( S_str / (α_l · σ) )^0.25 − 273,15
```

Fattori di proiezione, per orientamento incognito `[P]` — codice sorgente UMEP, `processor/solweig_algorithm.py:803-811` (https://github.com/UMEP-dev/UMEP-processing/blob/main/processor/solweig_algorithm.py) e riscrittura Rust `rust/src/tmrt.rs:10-17` (https://github.com/UMEP-dev/solweig/blob/main/rust/src/tmrt.rs):

| Postura | F_up = F_down | F_side (×4) | F_cyl | Quota di riferimento |
|---|---|---|---|---|
| In piedi | 0,06 | 0,22 | 0,28 | 1,1 m |
| Seduto | 0,166666 | 0,166666 | 0,20 | 0,75 m |

I valori 0,06 / 0,22 sono quelli di ISO 7726 per persona in piedi con orientamento non noto. `[S]` (la norma è a pagamento; il PDF campione pubblicato da iTeh non è estraibile come testo — la verifica diretta della clausola non è stata possibile in questa sessione). Confermati però `[P]` dal codice di due implementazioni indipendenti citate sopra.

**Input richiesti**: sei flussi K e sei flussi L per punto. In misura servono due pyranometri e due pyrgeometri montati su tre assi ortogonali. In modello servono ombreggiamento, sky view factor direzionali, temperature di superficie di suolo e pareti, e un modello di cielo (isotropo o anisotropo).

**Coefficienti di assorbimento** `[P]` — https://umep-dev.github.io/solweig/physics/tmrt/ (tabella "Absorption Coefficients", attribuita a ISO 7726 Tabella 4):

| Coefficiente | Valore | Significato |
|---|---|---|
| `abs_k` (α_k) | 0,70 | assorbimento onda corta, corpo vestito grigio medio |
| `abs_l` (α_l = ε_p) | 0,97 | assorbimento/emissività onda lunga |

> **Attenzione — incoerenza fra implementazioni.** La documentazione nuova di SOLWEIG dà `abs_l = 0,97` `[P]`, ma il file di parametri di default del plugin QGIS UMEP dà `absL = 0,95` `[P]` — https://github.com/UMEP-dev/UMEP-processing/blob/main/processor/parametersforsolweig.json (`Tmrt_params.Value.absL`). `[I]` La differenza è piccola sul risultato (Tmrt ∝ (1/α_l)^0.25, cioè ~0,5 % in kelvin assoluti, ≈ 0,4 K a 40 °C) ma è il tipo di divergenza silenziosa che CLIMESH deve fissare esplicitamente in configurazione e registrare nel giornale della corsa.

### 1.3 Globo termometro (metodo di misura, non di modello)

Formulazione ISO 7726 per convezione forzata, quella usata in gran parte della letteratura di campo:

```
T_mrt = [ (T_g + 273,15)^4 + (1,1·10^8 · v_a^0.6) / (ε · D^0.4) · (T_g − T_a) ]^0.25 − 273,15
```

con `T_g` temperatura di globo, `D` diametro del globo, `ε` emissività. `[S]` — resa esplicita, con questa notazione, in Sci Rep 2025 eq. 1: https://www.nature.com/articles/s41598-025-33440-6

Rilevante per CLIMESH solo come metodo di validazione futura contro misure di campo. Nota: dipende da `v_a`, quindi anche la *misura* di riferimento della MRT eredita l'errore sul vento.

### 1.4 SOLWEIG — la formulazione operativa da imitare

SOLWEIG applica il metodo a sei direzioni a una griglia raster, dichiarando esplicitamente di seguire lo stesso approccio usato per *osservare* la Tmrt (Höppe 1992). `[P]` — https://umep-docs.readthedocs.io/en/latest/OtherManuals/SOLWEIG.html §12.2.1

Forma esatta, dal codice `[P]` (https://github.com/UMEP-dev/UMEP-processing/blob/main/functions/TreePlanter/SOLWEIG1D/Solweig1D_2023a_calc.py:407-446):

```python
# corpo umano come cilindro, cielo anisotropo
Sstr = absK * (Kside*Fcyl + (Kdown+Kup)*Fup + (Knorth+Keast+Ksouth+Kwest)*Fside) \
     + absL * ((Ldown+Lup)*Fup + Lside*Fcyl + (Lnorth+Least+Lsouth+Lwest)*Fside)
# corpo umano come cubo (formulazione originale 2008)
Sstr = absK * ((Kdown+Kup)*Fup + (Knorth+Keast+Ksouth+Kwest)*Fside) \
     + absL * (Ldown*Fup + Lup*Fup + Lnorth*Fside + Least*Fside + Lsouth*Fside + Lwest*Fside)

Tmrt = (Sstr / (absL * SBC))**0.25 - 273.2
```

Tre varianti selezionabili: **cubo** (Lindberg et al. 2008), **cilindro** con cielo isotropo, **cilindro** con cielo anisotropo (Wallenberg et al. 2020, 2023). La variante cilindro è stata introdotta in v2015a. `[P]` — manuale UMEP §12.2.2.5, con riferimento a Holmer et al. 2015.

**Input richiesti da SOLWEIG** `[P]` — manuale UMEP §12.2.2:
- DSM di suolo+edifici (m s.l.m.), stessa estensione e risoluzione per tutti i raster;
- DSM di vegetazione: CDSM (cima chioma) + TDSM (zona tronco), in m dal suolo;
- DEM del solo terreno **oppure** un raster di land cover per individuare i pixel edificio;
- opzionale: raster di ground cover per lo schema di temperatura di superficie (Lindberg et al. 2016);
- meteo: temperatura dell'aria, umidità relativa, radiazione globale onda corta; componenti diretta e diffusa (se assenti, sottomodello Reindl et al. 1990); **velocità del vento richiesta solo se si vogliono PET e UTCI** ai punti d'interesse;
- parametri ambiente: albedo ed emissività di suolo e pareti (default: pareti 0,20/0,90; suolo 0,15/0,95);
- parametri umani: `abs_k`, `abs_l`, postura.

Output: griglie di Tmrt, K↓, K↑, L↓, L↑ e ombra, più serie complete ai POI.

> **Nota di attualità**: la versione 2026a introdurrà uno schema di temperatura del suolo Force/Restore, fisicamente più fondato del precedente schema lineare, e capace di stimare Ts anche di notte — cosa che lo schema attuale non fa. `[P]` — manuale UMEP §12.2.2.3. Chi copia lo schema attuale copia un modello che i suoi autori stanno sostituendo.

### 1.5 Formulazione NWP (Di Napoli et al. 2020) — fuori scala per CLIMESH

Deriva la MRT dai flussi radiativi di un modello meteorologico globale (ECMWF): `ssrd`, `ssr`, `dsrp`, `strd`, `fdir`, `strr`, più il coseno dell'angolo zenitale. Implementata in `thermofeel` di ECMWF. `[P]` — https://github.com/ecmwf/thermofeel/blob/main/thermofeel/thermofeel.py:235-265

Nessuna geometria urbana: nessuna ombra da edificio, nessun view factor locale. Serve a mappe continentali, non a una piazza. `[I]` Per CLIMESH è utile solo come formulazione di confronto e come sorgente di forcing su larga scala.

### 1.6 ENVI-met (AVF / IVS)

Trattata nel report `research/envi-met.md` §2.4. In sintesi: ray tracing 3D una tantum con risoluzione 10°×10° (648 facet), quattro view factor scalari nella variante AVF; nella variante IVS si memorizzano puntatori alla superficie specifica vista, al costo di ~10⁹ record per un dominio 200×200×35. `[P]` — https://envi-met.info/doku.php?id=kb:ivs

### 1.7 Confronto pratico

| Metodo | Input | Risoluzione spaziale | Costo | Ruolo per CLIMESH |
|---|---|---|---|---|
| Sei direzioni misurato (ISO 7726) | 6 K + 6 L da radiometri | punto | strumentazione | validazione futura |
| Globo termometro | T_g, T_a, v_a, D, ε | punto | strumento singolo, economico | validazione futura; dipende da v_a |
| **SOLWEIG** | DSM + CDSM/TDSM + meteo + albedo/emissività | griglia, 1 m | ombre + SVF + patch di cielo | **formulazione da adottare** |
| Di Napoli / thermofeel | flussi radiativi NWP | ~km | trascurabile | fuori scala |
| ENVI-met IVS | geometria 3D voxel + materiali | griglia 3D | memoria ~10⁹ record | riferimento di confronto |

---

## 2. Indici di comfort

### 2.1 PMV (Predicted Mean Vote)

**Cosa misura**: il voto medio previsto su scala di sensazione termica da −3/−4 (molto freddo) a +3/+4 (molto caldo), 0 = neutro. Deriva dal modello di Fanger (1970/1972), normato in ISO 7730. Da PMV si ricava linearmente il PPD (percentuale di insoddisfatti).

Equazione base `[P]` — https://envi-met.info/doku.php?id=apps:biomet_pmv:

```
PMV = [0,028 + 0,303 · exp(−0,036 · M/A_Du)] · (H/A_Du − E_d − E_sw − E_re − L − R − C)
```

**Input**: T_a, T_mrt, pressione di vapore, velocità del vento locale (tutti alla quota biometeorologica di 1,6 m), più isolamento del vestiario, produzione metabolica e fattore di lavoro meccanico. La persona di riferimento è fissa: uomo, 35 anni, 1,75 m, 75 kg. `[P]` (stessa pagina)

**Perché non va bene outdoor** — critica esplicita del vendor che pure lo implementa `[P]`:
- è stato sviluppato per situazioni **indoor stazionarie**; l'estensione outdoor (VDI 3787 Parte 2) aggiunge radiazione solare e onda lunga e ammette velocità superiori a quelle di una stanza, ma resta discutibile se la funzione empirica che lega bilancio energetico e sensazione termica sia valida fuori dal contesto in cui è stata tarata;
- outdoor il PMV **esce regolarmente dall'intervallo [−4, +4]**, cioè fuori dai dati sperimentali di Fanger;
- nel modello PMV la temperatura del vestiario è **l'unico** parametro che reagisce alle condizioni ambientali: la temperatura cutanea dipende solo dall'attività.

`[I]` Conclusione: PMV è utile solo come output di compatibilità con chi lo pretende. Non è l'indice su cui costruire CLIMESH.

### 2.2 PET (Physiological Equivalent Temperature)

**Cosa misura**: *la temperatura dell'aria alla quale, in un ambiente interno tipico (senza vento né radiazione solare), il bilancio termico del corpo umano si chiude con la stessa temperatura del nucleo e della pelle che si hanno nelle condizioni esterne complesse da valutare*. `[P]` — Höppe 1999, IJB 43:71-75, https://doi.org/10.1007/s004840050118, citato testualmente in https://envi-met.info/doku.php?id=apps:biomet_pet

Si basa sul **MEMI** (Munich Energy-balance Model for Individuals, Höppe 1984), a sua volta estensione del modello a 2 nodi di Gagge et al. (1971). Bilancio: `M + W + R + C + E_D + E_Re + E_Sw + S = 0`.

**Ambiente interno di riferimento** `[P]` — docstring di `pythermalcomfort.models.pet_steady`, https://github.com/CenterForTheBuiltEnvironment/pythermalcomfort/blob/master/pythermalcomfort/models/pet_steady.py:34-38:

```
tdb = tr, v = 0,1 m/s, pressione di vapore = 12 hPa, clo = 0,9, met = 1,37 met + metabolismo basale
```

**Input**: T_a, T_mrt, v, RH, met, clo, più — a differenza di UTCI — età, sesso, peso, altezza, postura, pressione atmosferica.

**Il problema di PET: non esiste "un" PET.** Questo è il punto decisivo, ed è documentato da fonte primaria:
- PET è rimasto a lungo senza documentazione completa: per implementarlo bisognava leggere Gagge 1971, Höppe 1984 (in tedesco) e il codice sorgente allegato a VDI 3787 Parte 2. `[P]` — envi-met, pagina PET
- Walther & Goestchel (2018), *The P.E.T. comfort index: Questioning the model*, Building and Environment 137:1-10, hanno documentato le equazioni e segnalato **assunzioni non logiche, errori nel set di equazioni originale ed errori di codifica nel codice pubblicato da VDI 3787**. `[P]` — https://doi.org/10.1016/j.buildenv.2018.03.054
- ENVI-met ha corretto parte di questi errori e avverte: *"PET values calculated by ENVI-met may differ from values calculated by other programs, but we see no sense in copying wrong code"*. `[P]`
- Con la release Winter 22/23 ENVI-met ha introdotto **PET\*** ("PET Reviewed"), che differisce **sia** dal PET originale **sia** dal PET già corretto delle versioni precedenti. `[P]`
- `pythermalcomfort` non implementa il PET originale ma **il modello Walther & Goestchel 2018**. `[P]` — docstring citata sopra, righe 41-45.

`[I]` Conseguenza operativa per CLIMESH: due valori di PET provenienti da strumenti diversi non sono confrontabili senza dichiarare quale variante è stata usata. Se CLIMESH produce PET, deve dichiarare la variante nel giornale della corsa, e il confronto con ENVI-met su PET è strutturalmente ambiguo.

### 2.3 UTCI (Universal Thermal Climate Index)

**Cosa misura**: la temperatura dell'aria di un ambiente di riferimento che, secondo il modello, produce la stessa risposta fisiologica dinamica dell'ambiente reale. La risposta umana è simulata dal modello multi-nodo **UTCI-Fiala** di termoregolazione, accoppiato a un **modello adattivo di vestiario**. `[P]` — Bröde et al. 2012, IJB 56:481-494, abstract: https://doi.org/10.1007/s00484-011-0454-1

**Ambiente di riferimento** `[P]` (stesso abstract): umidità relativa 50 % **ma con pressione di vapore limitata a 20 hPa**, aria calma, temperatura radiante uguale alla temperatura dell'aria.

**Costruzione dell'indice** `[P]`: analisi in componenti principali su 7 parametri di strain fisiologico (temperatura del nucleo, cutanea media, cutanea del viso, sudorazione, bagnatura cutanea, flusso ematico cutaneo, brivido) dopo 30 e 120 minuti di esposizione; la combinazione lineare spiega due terzi della varianza totale della risposta multidimensionale.

**Approssimazione operativa**: polinomio di 6° ordine, 210 coefficienti, che approssima l'offset `UTCI − T_a` a partire da quattro input. `[P]` — sorgente Fortran ufficiale, `UTCI_a002.f90`, scaricato da https://utci.org/resources/UTCI%20Program%20Code.zip

```
UTCI = Ta + f(Ta, D_Tmrt, va, Pa)      D_Tmrt = Tmrt − Ta,  Pa = ehPa/10  [kPa]
```

**Intervalli di validità dichiarati** `[P]` — `ReadMe_UTCI_a002.txt` nello stesso zip, sezione METHOD:

| Input | Intervallo |
|---|---|
| temperatura dell'aria T_a | −50 … +50 °C |
| temperatura media radiante | da 30 °C sotto a 70 °C sopra T_a |
| velocità del vento **a 10 m** | **0,5 … 17 m/s** |
| pressione di vapore | < 50 hPa (o 100 % UR) |

Il limite sul vento è ribadito, in maiuscolo di fatto, sia nel ReadMe sia sulla pagina del calcolatore ufficiale: *"The given polynomial approximation limits the application of this procedure to values of wind speed between 0.5 and 17 m/s!"* `[P]` — https://utci.org/utci_calc.php

**Scala di stress termico** `[P]` — https://umep-dev.github.io/solweig/physics/utci/:

| UTCI (°C) | Categoria |
|---|---|
| > 46 | stress da caldo estremo |
| 38…46 | molto forte |
| 32…38 | forte |
| 26…32 | moderato |
| 9…26 | nessuno stress termico |
| 0…9 | leggero stress da freddo |
| −13…0 | moderato |
| −27…−13 | forte |
| −40…−27 | molto forte |
| < −40 | estremo |

### 2.4 Quale indice usa la letteratura, e chi raccomanda cosa

Le raccomandazioni disponibili **non concordano**, e questo va detto:

- **Fischereit & Schlünzen (2018)**, *Evaluation of thermal indices for their applicability in obstacle-resolving meteorology models*, IJB, open access. Applicano 11 criteri e 6 caratteristiche a **165 indici** rivedendo le pubblicazioni originali. Risultato: **solo quattro indici** sono applicabili globalmente nella forma attuale a vari ambienti urbani soddisfacendo anche i requisiti dei modelli obstacle-resolving. `[P]` — abstract: https://doi.org/10.1007/s00484-018-1591-6
- **ENVI-met** si allinea a quel paper e **raccomanda PET**: *"they suggest […] to use PET as a thermal comfort indicator. We totally agree with this suggestion."* `[P]` — https://envi-met.info/doku.php?id=apps:biomet
- **ENVI-met sconsiglia esplicitamente UTCI** nella versione basata su regressione, e la motivazione è **il vento**: *"We do not recommend to use UTCI in the regression-based version based on using 2m (1.6m) level wind speeds extrapolated to 10m. In a complex urban environment, wind speeds at pedestrian level are unique and cannot be related to some above-roof general quantity."* `[P]` — https://envi-met.info/doku.php?id=apps:biomet_utci
- **Charalampopoulos & Nouri (2019)**, Atmosphere 10(10):580, CC-BY, analisi di sensibilità con Generalized Additive Models su THI, HUMIDEX, PET, mPET, UTCI, PT: *"UTCI is very sensitive under low radiation condition, and PET/mPET present higher sensitivity when the weather is dominated by high radiation and air temperature."* `[S]` — https://doi.org/10.3390/atmos10100580
- **Fröhlich & Matzarakis (2016)**, Theor Appl Climatol 124:179-187, analisi quantitativa di sensibilità in condizioni calde e ventose (Doha). `[S]` — https://doi.org/10.1007/s00704-015-1410-5 — **non letto: paywall Springer, abstract oscurato anche su Semantic Scholar.** Da recuperare se serve un numero pubblicato sulla sensibilità di PET al vento.
- SOLWEIG/UMEP calcola **entrambi**, PET e UTCI, e non sceglie per l'utente. `[P]` — https://github.com/UMEP-dev/solweig

`[I]` La mia lettura: l'obiezione di ENVI-met a UTCI è corretta **nel contesto di ENVI-met**, dove esiste un campo di vento risolto al livello pedone che va estrapolato a 10 m con un `z0` sconosciuto. In CLIMESH la situazione è rovesciata: il vento arriva dall'EPW, cioè **già a 10 m** (§4.4), e quindi UTCI lo consuma senza alcuna estrapolazione, mentre PET richiederebbe di scendere a 1,1 m — cioè di inventare esattamente il profilo che CLIMESH non risolve. **L'argomento di ENVI-met contro UTCI diventa, in CLIMESH, un argomento a favore.**

### 2.5 Costo computazionale

Dichiarazione del vendor, coerente con la natura dei tre modelli `[P]` — https://envi-met.info/doku.php?id=apps:biomet:

- **UTCI**: valutazione di un polinomio, veloce;
- **PMV**: soluzione iterativa di **una** equazione di bilancio energetico;
- **PET**: soluzione iterativa di **due** bilanci (outdoor e indoor di riferermento), *"much more calculation time expensive"*.

`[I]` Con il budget di 60 s per il caso Bastia (50×50×48 h), UTCI su griglia è gratuito; PET su griglia va misurato prima di darlo per scontato. La riscrittura Rust di SOLWEIG mette PET su un solver iterativo parallelizzato con rayon (`rust/src/pet.rs`), il che suggerisce che sia fattibile, ma non ho un benchmark.

---

## 3. Implementazioni aperte e citabili

### 3.1 Il codice UTCI ufficiale e le sue condizioni d'uso

Scaricato e ispezionato in questa sessione: https://utci.org/resources/UTCI%20Program%20Code.zip — contiene `UTCI_a002.f90` (18.611 byte), `UTCI_a002.exe`, `ReadMe_UTCI_a002.txt`.

Testo rilevante, verbatim dal ReadMe `[P]`:

> UTCI, Version a 0.002, October 2009 — Copyright (C) 2009 Peter Broede
> Program for calculating UTCI Temperature (UTCI) **released for public use after termination of COST Action 730**
> […]
> USAGE: **This subroutine can be incorporated into your own applications**, the programs provided here may serve as an instructive example.
> The programs are distributed in the hope that they will be useful, but WITHOUT ANY WARRANTY […]

> **Caveat di licenza `[I]`, importante.** Il ReadMe e l'intestazione del `.f90` contengono la clausola di esclusione di garanzia e la limitazione di responsabilità **testualmente copiate dalla GPL** (fino all'espressione *"ANY OTHER PARTY WHO MODIFIES AND/OR CONVEYS THE PROGRAM AS PERMITTED ABOVE"*), ma **non contengono la clausola di concessione della licenza**. Non c'è un file `COPYING`, né una dichiarazione "GPL v2/v3". L'unico permesso esplicito è la frase *"This subroutine can be incorporated into your own applications"*. Questo è probabilmente sufficiente per l'uso, ma **non è una licenza open source formale** e non risolve la compatibilità con GPL-3 in modo pulito. La via priva di rischi per CLIMESH è ripartire da una reimplementazione già licenziata (§3.2), non dal Fortran originale.

Riferimento accademico da citare in ogni caso: **Bröde, P., Fiala, D., Błażejczyk, K., Holmér, I., Jendritzky, G., Kampmann, B., Tinz, B., Havenith, G. (2012)**, *Deriving the operational procedure for the Universal Thermal Climate Index (UTCI)*, Int. J. Biometeorol. 56:481-494. https://doi.org/10.1007/s00484-011-0454-1

### 3.2 Panorama delle implementazioni

Dati raccolti via GitHub API e crates.io il 2026-08-31 `[P]`:

| Progetto | Linguaggio | Licenza | Stelle | Ultimo push | Copre |
|---|---|---|---|---|---|
| [`UMEP-dev/UMEP-processing`](https://github.com/UMEP-dev/UMEP-processing) | Python | **GPL-3.0** | 14 | 2026-08-27 | SOLWEIG completo, Tmrt, PET, UTCI — implementazione di riferimento |
| [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig) | **Rust** + PyO3 | **GPL-3.0** | 11 | 2026-08-25 | Tmrt, UTCI, PET, ombre e SVF su GPU (wgpu) |
| [`CenterForTheBuiltEnvironment/pythermalcomfort`](https://github.com/CenterForTheBuiltEnvironment/pythermalcomfort) | Python | **MIT** | 222 | 2026-08-20 | 38 modelli: PMV (ISO/ASHRAE), PET (Walther-Goestchel), UTCI, SET, PHS, adaptive |
| [`ecmwf/thermofeel`](https://github.com/ecmwf/thermofeel) | Python | **Apache-2.0** | 96 | 2026-08-06 | UTCI, MRT (Di Napoli 2020), WBGT, PMV/PPD, wind chill. **Niente PET.** Maturità ECMWF "Graduated" |
| [`ladybug-tools/ladybug-comfort`](https://github.com/ladybug-tools/ladybug-comfort) | Python | **AGPL-3.0** | 17 | 2026-08-28 | UTCI, PMV, SET, adaptive, SolarCal |
| [`NairAssoc/thermalcomfort`](https://github.com/NairAssoc/thermalcomfort) (crates.io `thermalcomfort` 3.9.8) | **Rust** `no_std` | **MIT** | 0 | 2026-08-07 | port 1:1 di pythermalcomfort v3.9.8: PET, UTCI, PMV, SET, PHS |
| [`nvnsudharsan/SOLWEIG-GPU`](https://github.com/nvnsudharsan/SOLWEIG-GPU) | Python | GPL-3.0 | 27 | 2026-08-18 | SOLWEIG accelerato GPU |

**Compatibilità di licenza con la scelta GPL-3/AGPL-3 di CLIMESH** `[I]`: MIT e Apache-2.0 sono compatibili in ingresso verso GPL-3 (Apache-2.0 è compatibile con GPL-3, non con GPL-2). GPL-3.0 è compatibile con GPL-3. AGPL-3.0 in ingresso forzerebbe CLIMESH ad AGPL-3, cosa che la mappa già ammette come opzione. Nessuna delle opzioni è bloccante; `ladybug-comfort` è l'unica che vincola la scelta.

### 3.3 Osservazioni sul codice, dalla lettura diretta

- **`UMEP-dev/solweig` è, per CLIMESH, il candidato più interessante e va guardato prima di scrivere qualsiasi riga.** È la riscrittura Rust ufficiale della pipeline SOLWEIG: `rust/src/{tmrt,utci,pet,shadowing,skyview,perez,patch_radiation}.rs`, con shader WGSL per ombre, SVF e cielo anisotropo su GPU (`wgpu 27`), parallelismo `rayon`, `lto = "fat"`, `panic = "abort"`. Licenza GPL-3.0, stessa di CLIMESH. `[P]` — https://github.com/UMEP-dev/solweig/blob/main/rust/Cargo.toml
  - **Limite `[P]`**: `crate-type = ["cdylib"]` e nome pacchetto `rustalgos`. **Non è pubblicato su crates.io e non è utilizzabile come dipendenza Rust così com'è**: è un modulo di estensione Python. Per riusarlo da un binario Rust bisogna forkarlo e cambiare il tipo di crate.
  - **Autodichiarazione degli autori `[P]`**: *"This package is an experimental, compatibility-focused implementation of the SOLWEIG model, not the reference implementation."*
- **Il crate `thermalcomfort`** (MIT, `no_std`, WASM) è l'unica libreria Rust *pubblicata* che copra PET e UTCI, e sarebbe la scelta ovvia — ma ha **0 stelle, 355 download e un solo autore, prima release febbraio 2026** `[P]`. `[I]` Bus factor 1: usabile, ma da vendorare o da tenere sotto test di parità propri, non da trattare come dipendenza stabile.
- **Divergenza numerica trovata in `pythermalcomfort` `[M]`**: la funzione di pressione di vapore saturo dentro `models/utci.py` usa `np.log1p(tk)` dove il Fortran ufficiale usa `log(tk)` (https://github.com/CenterForTheBuiltEnvironment/pythermalcomfort/blob/master/pythermalcomfort/models/utci.py, funzione `exponential`). Misurato: la `es` risultante è **0,87–0,91 % più alta** di quella ufficiale, e l'UTCI ne risulta più alto di **0,03 K a 25 °C e 0,12 K a 40 °C** (RH 50 %). `[I]` Sembra un errore di traduzione, non una scelta; l'effetto è trascurabile rispetto alle altre incertezze, ma spiega perché i valori di `pythermalcomfort` non coincidono al centesimo con il riferimento e va tenuto presente in qualsiasi test di parità.
- **Precisione**: la riscrittura Rust di SOLWEIG valuta il polinomio UTCI in `f32` (`rust/src/utci.rs:8`), mentre il riferimento Fortran è in `double precision`. `[I]` Su un polinomio di 6° grado con 210 termini e cancellazioni, l'errore di arrotondamento f32 non è ovviamente trascurabile e andrebbe misurato prima di adottare quella scelta.

---

## 4. Sensibilità agli input — il punto centrale del ticket

### 4.1 Metodo delle misure `[M]`

Non essendo disponibili `numpy`, `scipy`, `pip` né `gfortran` nell'ambiente, ho estratto **l'espressione polinomiale dal file Fortran ufficiale** `UTCI_a002.f90` (Bröde, versione a 0.002, ottobre 2009, scaricata da utci.org in questa sessione), l'ho tradotta meccanicamente in Python (rimozione delle continuazioni `&`, conversione degli esponenti `D±nn` in `e±nn`) e valutata con `eval`. La funzione di pressione di vapore saturo è la subroutine `es` dello stesso file, trasposta con `log(tk)` come nell'originale.

Verifica di correttezza: `UTCI(Ta=25, Tmrt=25, v=1,0 m/s, RH=50 %) = 24,61 °C`, contro **24,6 °C** dichiarati nella docstring di `pythermalcomfort.models.utci`. `[P]` Coincide.

Script di lavoro: `<scratchpad>/utci_eval.py` — file temporaneo di sessione, **non** aggiunto al repo.

Tutti i valori seguenti sono `[M]`. Derivate calcolate a differenze finite centrate con passo ±0,1 m/s (o ±1 K).

### 4.2 Sensibilità di UTCI — tabelle

**Caso estivo tipo (Ta = 30 °C, RH = 45 %)** — condizioni plausibili per Bastia Umbra il 15/07 pomeriggio.

| v a 10 m [m/s] | 0,5 | 1,0 | 2,0 | 3,0 | 5,0 | 8,0 |
|---|---|---|---|---|---|---|
| UTCI al sole (Tmrt = 60 °C) | 37,8 | 37,6 | 36,7 | 35,6 | 33,7 | 32,6 |
| UTCI in ombra (Tmrt = 32 °C) | 30,5 | 30,4 | 29,7 | 28,8 | 27,2 | 26,5 |
| **Δ sole − ombra** | **7,2** | **7,2** | **7,0** | **6,8** | **6,5** | **6,1** |
| dUTCI/dv [K per m/s], al sole | −0,1 | −0,6 | −1,1 | −1,1 | −0,7 | −0,1 |
| dUTCI/dT_mrt [K/K], al sole | 0,251 | 0,250 | 0,247 | 0,243 | 0,235 | 0,224 |
| dUTCI/dT_a [K/K] | 0,800 | 0,816 | 0,859 | 0,910 | 1,027 | 1,206 |
| dUTCI per +10 % di UR | 0,71 | 0,65 | 0,57 | 0,52 | 0,48 | 0,51 |

**Caso di ondata di calore (Ta = 35 °C, RH = 40 %)**: Δ sole−ombra 8,5 → 7,3 K passando da 0,5 a 5 m/s; dUTCI/dv fra −0,4 e −0,9 K per m/s.

**Caso invernale (Ta = 5 °C, RH = 75 %)** — il 15/01 del caso studio.

| v a 10 m [m/s] | 0,5 | 1,0 | 2,0 | 3,0 | 5,0 |
|---|---|---|---|---|---|
| UTCI al sole (Tmrt = 25 °C) | 14,0 | 12,7 | 9,5 | 6,2 | −0,1 |
| UTCI in ombra (Tmrt = 0 °C) | 4,5 | 3,3 | 0,4 | −2,8 | −8,7 |
| **Δ sole − ombra** | **9,5** | **9,4** | **9,2** | **9,0** | **8,6** |
| **dUTCI/dv [K per m/s]** | **−2,38** | **−2,86** | **−3,34** | **−3,37** | **−2,79** |
| dUTCI/dT_mrt [K/K] | 0,376 | 0,374 | 0,369 | 0,363 | 0,353 |

Tre letture dirette:

1. **UTCI comprime la MRT di un fattore ~4 d'estate (0,25 K/K) e ~2,7 d'inverno (0,37 K/K).** Un ΔTmrt di 28 K fra sole e ombra si traduce in ~7 K di UTCI. Questo è il vero fattore di conversione fra il prodotto di CLIMESH e il numero che l'utente legge.
2. **La sensibilità al vento è 3 volte maggiore d'inverno che d'estate.** −3 K per m/s contro −1 K per m/s.
3. Sotto 0,5 m/s la derivata cambia segno (+0,1 K per m/s a Tmrt = 60): è un artefatto del polinomio fuori dal suo intervallo di validità, non fisica. **Confermare il clamp a 0,5 m/s è obbligatorio.**

### 4.3 Quanto pesa un errore sul vento

Errore commesso su UTCI se la velocità vera è `v` e il modello ne usa `2v` (fattore 2, ordine di grandezza realistico per un vento non risolto in un canyon):

| v vero → usato | Estate (Ta 30, Tmrt 60) | Inverno (Ta 5, Tmrt 25) |
|---|---|---|
| 0,5 → 1,0 m/s | −0,17 K | −1,32 K |
| 1,0 → 2,0 m/s | −0,85 K | −3,14 K |
| 2,0 → 4,0 m/s | −2,20 K | −6,66 K |
| 3,0 → 6,0 m/s | −2,51 K | −8,84 K |

**Equivalenza fra errore sul vento ed errore sulla MRT** (estate, Ta 30, Tmrt 60, partendo da v = 1 m/s):

| errore sul vento | errore su UTCI | equivale a un errore su T_mrt di |
|---|---|---|
| +0,5 m/s | −0,36 K | −1,4 K |
| +1,0 m/s | −0,85 K | −3,4 K |
| +2,0 m/s | −1,97 K | −7,9 K |

`[I]` Questa è la tabella di conversione da tenere a mente. D'estate, sbagliare il vento di 1 m/s costa quanto sbagliare la MRT di 3,4 K — cioè circa quanto l'RMSE di SOLWEIG sulla MRT stessa (§5.2). I due errori sono dello stesso ordine: il vento semplificato **non** è l'anello debole dominante in estate. D'inverno lo diventa.

### 4.4 L'ambiguità della quota di riferimento del vento — un problema sottovalutato

UTCI vuole il vento **a 10 m**. Gli strumenti esistenti fanno tre cose diverse:

- **ENVI-met** `[P]` (https://envi-met.info/doku.php?id=apps:biomet_utci) usa il profilo logaritmico
  `Wind_10m = ln(10/z0) / ln(z_level/z0) · Wind_z_level`
  e ammette che *"that doesn't make much sense as the z0 roughness value is unknown for most sites"*.
- **UMEP** `[P]` (https://github.com/UMEP-dev/UMEP-processing/blob/main/functions/SOLWEIGpython/Solweig_run.py:1107-1108) usa una legge di potenza con esponente 0,2, e scala **in modo diverso per i due indici**:
  ```python
  WsPET  = (1.1  / sensorheight) ** 0.2 * Ws[i]
  WsUTCI = (10.0 / sensorheight) ** 0.2 * Ws[i]
  ```
  con `sensorheight` = 10 m di default (`parametersforsolweig.json`, `Wind_Height.Value.magl`). Cioè: PET riceve il 64 % del vento che riceve UTCI.
- **`UMEP-dev/solweig` (Rust) non applica nessuna conversione**: `compute_pet_grid` e `compute_utci_grid` ricevono entrambi lo stesso scalare, documentato in entrambi i casi come *"Wind speed at 10m height"*. `[P]` — https://github.com/UMEP-dev/solweig/blob/main/pysrc/solweig/postprocess.py:25-56 e 59-90. `[I]` Rispetto a UMEP-processing questa è una divergenza di comportamento su PET, non segnalata nel README: PET riceve un vento ~55 % più alto.
- **`ladybug-comfort`** `[P]` documenta la regola pratica: *"this meteorological speed at 10 m is simply 1.5 times the speed felt at ground in the original Fiala model used to build UTCI"*, e applica il clamp a [0,5, 17] m/s. https://github.com/ladybug-tools/ladybug-comfort/blob/master/ladybug_comfort/utci.py:46-57

**Quanto costa questa ambiguità** `[M]`. Vento pedone di 1,0 m/s a 1,6 m, convertito a 10 m:

| Convenzione | fattore | v10 [m/s] | UTCI estate (Ta 30, Tmrt 60) | UTCI inverno (Ta 5, Tmrt 25) |
|---|---|---|---|---|
| log, z0 = 0,01 m | 1,36 | 1,36 | 37,3 | 11,6 |
| log, z0 = 0,10 m | 1,66 | 1,66 | 37,1 | — |
| log, z0 = 0,50 m (urbano) | 2,58 | 2,58 | 36,1 | 7,6 |
| log, z0 = 1,00 m | 4,90 | 4,90 | 33,8 | — |
| potenza `(10/1,6)^0.2` (UMEP) | 1,44 | 1,44 | 37,3 | — |

**La sola scelta di `z0` sposta UTCI di 3,5 K d'estate e 4 K d'inverno** — più dell'intero effetto di un albero, e senza che nessun dato la giustifichi.

`[I]` **Conseguenza per CLIMESH, e a mio avviso l'argomento più forte del ticket.** Il file EPW fornisce la velocità del vento **già a 10 m su terreno aperto**: EnergyPlus lo assume esplicitamente, con `z_met = 10 m`, `α_met = 0,14`, `δ_met = 270 m` perché *"most meteorological stations are located in an open field"* `[P]` — https://bigladdersoftware.com/epx/docs/23-2/engineering-reference/outside-surface-heat-balance.html, §Local Wind Speed Calculation. Se CLIMESH alimenta UTCI direttamente con il vento dell'EPW, **non compie nessuna estrapolazione**: consuma l'input alla quota per cui l'indice è definito. È l'unica catena in cui il vento non risolto non introduce un parametro inventato. Il momento in cui CLIMESH decide di calcolare anche PET è il momento in cui deve inventarsi un `z0` o un esponente, e reintrodurre esattamente il problema che ENVI-met denuncia.

### 4.5 Sensibilità di PET — quello che non ho potuto misurare

**Vuoto dichiarato.** Non ho prodotto una tabella di sensibilità per PET: nessuna implementazione eseguibile era disponibile nell'ambiente e le due fonti quantitative pubblicate non sono accessibili.

Quello che si può dire con fonte:
- PET/mPET sono **più sensibili** di UTCI quando dominano radiazione e temperatura dell'aria elevate; UTCI è **molto sensibile in condizioni di bassa radiazione**. `[S]` — Charalampopoulos & Nouri 2019, https://doi.org/10.3390/atmos10100580
- `[I]` Coerente con la mia misura: UTCI è più reattivo al vento in inverno/bassa radiazione (−3 K per m/s) che in estate (−1 K per m/s).
- `[I]` Struttura del modello: in PET il vento entra nel coefficiente di scambio convettivo `h_c` sia nel bilancio outdoor sia — via la ridefinizione dell'ambiente interno di riferimento a 0,1 m/s — nel confronto. Ci si attende quindi una sensibilità al vento **non nulla e dello stesso segno**, ma non ho una stima numerica e **non va inventata**.

Fonte da recuperare per chiudere il vuoto: Fröhlich & Matzarakis (2016), https://doi.org/10.1007/s00704-015-1410-5.

### 4.6 Il vento semplificato di CLIMESH è accettabile?

`[I]` — **raccomandazione, non decisione.**

**Sì per l'estate, con riserva dichiarata per l'inverno.** Argomenti, in ordine di peso:

1. **Precedente diretto**: SOLWEIG, che è lo strumento libero contro cui la mappa prevede di validare CLIMESH, fa **esattamente questo**. `compute_utci_grid` costruisce la griglia di vento con `np.full_like(tmrt, wind)`: un solo valore scalare per tutto il dominio, per ogni passo temporale. `[P]` — https://github.com/UMEP-dev/solweig/blob/main/pysrc/solweig/postprocess.py:55. La struttura dati `Weather` ha un unico campo scalare `ws`, default 1,0 m/s. `[P]` — `pysrc/solweig/models/weather.py:55, 111`.
2. **Ordine di grandezza dell'errore**: d'estate un errore di 1 m/s costa 0,85 K di UTCI, equivalente a 3,4 K di errore su MRT — comparabile all'errore che il modello radiativo ha comunque (§5.2). Non introduce un errore dominante.
3. **Nessuna estrapolazione**: il vento EPW è già a 10 m, la quota che UTCI richiede (§4.4).
4. **Contro, e va scritto nel report dell'utente**: d'inverno l'errore triplica. Con Ta = 5 °C e vento sbagliato di un fattore 2 si sbaglia UTCI di 3–9 K, cioè **si può cambiare categoria di stress termico**. Per il giorno 15/01/2021 del caso studio, le mappe di UTCI vanno accompagnate da un avvertimento esplicito, non da una legenda muta.
5. **Mitigazioni a costo quasi nullo**, tutte già praticate dagli strumenti letti:
   - clamp obbligatorio a [0,5; 17] m/s con marcatura dei pixel fuori intervallo (ENVI-met marca "No Data"; `UMEP-dev/solweig` restituisce `NaN` per `va ≤ 0` `[P]`, `rust/src/utci.rs:355-360`);
   - avviso a runtime se `ws = 0` — `UMEP-dev/solweig` lo fa già: *"UTCI is sensitive to wind speed near zero"* `[P]`, `pysrc/solweig/api.py:248-252`;
   - registrare nel giornale della corsa il vento usato, la sua quota e il fatto che sia uniforme;
   - **produrre una banda di incertezza invece di un singolo numero**: rieseguire l'indice con `v/2` e `2v` costa quanto valutare due volte un polinomio, cioè niente, e trasforma la debolezza in un'informazione onesta. `[I]` Questa è la mossa che rende il vento semplificato difendibile in una tesi.

---

## 5. Ordini di grandezza: segnale sole/ombra contro barre d'errore

### 5.1 Il segnale

- **Misura di campo, sei direzioni, piattaforma mobile MaRTy, Tempe (Arizona), 19 giugno 2016, 22 siti** `[P]` — Middel & Krayenhoff 2019, Sci. Total Environ., https://doi.org/10.1016/j.scitotenv.2019.06.085: T_a massima locale **48,5 °C**, **T_mrt massima 76,4 °C** in un canyon E-O. **Gli alberi riducono la T_mrt pomeridiana fino a 33,4 °C**, ma la aumentano fino a 5 °C dopo il tramonto. La radiazione onda lunga misurata dai sensori **laterali** domina il bilancio di T_mrt.
- **Freiburg**: siti ombreggiati da alberi con T_mrt inferiore di circa **30 °C** rispetto ai siti al sole in una giornata estiva calda. `[S]` — riportato in letteratura di sintesi sul raffrescamento degli alberi urbani (TDAG, *What we know and don't know about the cooling benefits of urban trees*, https://www.tdag.org.uk/uploads/4/2/8/0/4280686/what_is_known_and_not_know_cooling_benefits_of_urban_trees.pdf), che rimanda a Matzarakis et al. 1999. **Non ho letto la fonte primaria.**
- **Ordine di grandezza generale**: in condizioni soleggiate la MRT può superare la temperatura dell'aria **fino a 30 °C**. `[S]` — Di Napoli et al. 2020, §Introduction, che cita Jendritzky et al.

**Il segnale sole/ombra su MRT vale quindi 20–35 K.** Coerente con la premessa "15-30 °C" fissata nella mappa (issue #1).

### 5.2 La barra d'errore

**Validazione di `UMEP-dev/solweig` contro misure radiative di campo a Göteborg, tre siti, 31 test in CI** `[P]` — https://github.com/UMEP-dev/solweig/blob/main/VALIDATION.md, versione v0.1.0b95 del 2026-08-25:

| Metrica | Kronenhuset | Gustav Adolfs | GVC |
|---|---|---|---|
| RMSE su T_mrt [°C] | 6,6 | 5,7–7,3 | 2,4–6,9 |
| R² su T_mrt | 0,52 | 0,80–0,88 | 0,65–0,99 |
| bias su T_mrt [°C] | +2,6 | +0,6…+3,7 | +1,4…+5,8 |
| ore di osservazione | 12 | 43 | 30 |

Bias sistematico dichiarato e diagnosticato dagli autori `[P]`: *"Modelled L↓ sits 18 to 55 W/m² above the observations at every site. This traces to the published Ldown formulation rather than to calibration"*. Cioè: **la formulazione pubblicata di L↓ ha un bias positivo noto**, non è un problema di taratura. Chi la reimplementa eredita il bias.

### 5.3 Rapporto segnale/rumore

`[I]` — calcolo mio, dai numeri sopra.

| Grandezza | Segnale (differenza fra scenari o fra sole e ombra) | Errore del modello | Rapporto |
|---|---|---|---|
| **T_mrt** | 20–35 K | RMSE 2,4–7,3 K | **3…10** |
| **UTCI** (via dUTCI/dT_mrt ≈ 0,25) | 7…9 K | ~0,6…1,8 K da sola MRT, + ~0,9 K per 1 m/s di errore sul vento | **3…6** |
| Temperatura dell'aria (relazione LA01, per confronto) | 0,21 K d'estate, ~1 K d'inverno | MAE mediano ENVI-met 1,34 K, RMSE 1,51 K | **0,14…0,7** |

I dati della terza riga vengono dal contesto fissato nella mappa (issue #1) e dal report `research/envi-met.md` §2.11 (Tsoka et al. 2018). **Non ho riverificato personalmente il contenuto di `materiale università/RELAZIONI/LA01.pdf` in questa sessione** (nessun estrattore PDF disponibile nell'ambiente): il file esiste, 1.365.744 byte, ma i valori 0,21 °C e ~1 °C li prendo dal brief e dalla mappa, non da lettura diretta.

**Conclusione**: il segnale su MRT e sull'indice sta 3–10 volte sopra l'errore del modello. Quello sulla temperatura dell'aria sta **sotto**. La scelta di ambito fisico fatta nella mappa è confermata quantitativamente.

---

## 6. Raccomandazione

`[I]` — **raccomandazione, non decisione. La scelta resta alla sessione principale.**

1. **MRT**: adottare il metodo a **sei direzioni con fattori di proiezione angolare** nella formulazione SOLWEIG, partendo dalla variante **cilindro** (`Fcyl = 0,28`, `Fside = 0,22`, `Fup = 0,06`, in piedi a 1,1 m). Rendere `abs_k`, `abs_l` e la postura parametri espliciti in configurazione, non costanti nel codice — le implementazioni esistenti già divergono su `abs_l` (0,95 vs 0,97).
2. **Indice primario: UTCI.** Motivi: un'unica implementazione autoritativa e quindi riproducibilità fra strumenti; costo computazionale nullo; consuma il vento alla quota in cui l'EPW lo fornisce, senza estrapolazioni inventate. Contro: nessun parametro personale regolabile; validità limitata a 0,5–17 m/s.
3. **Indice secondario: PET**, dichiarando nel giornale della corsa **quale variante** (originale VDI, Walther-Goestchel 2018, o PET\* di ENVI-met) e sapendo che il confronto quantitativo con ENVI-met su PET è strutturalmente ambiguo. Da aggiungere solo dopo aver misurato il costo del solver iterativo contro il budget di 60 s.
4. **PMV**: non implementare, se non come output di compatibilità in una fase successiva.
5. **Riuso**: leggere `UMEP-dev/solweig` prima di scrivere codice. È Rust, è GPL-3, è degli autori di SOLWEIG, e copre già Tmrt, UTCI, PET, ombre e SVF su GPU. Il fork con `crate-type = ["rlib"]` è probabilmente il percorso più corto verso il primo risultato. Alternativa per i soli indici: il crate `thermalcomfort` (MIT), da vendorare vista la sua immaturità.
6. **Vento**: accettare l'input semplificato a 10 m dall'EPW, **senza estrapolazioni di quota**; clamp a [0,5; 17] m/s con marcatura dei pixel fuori intervallo; produrre di default una banda `UTCI(v/2) … UTCI(2v)` accanto al valore centrale; scrivere l'avvertimento sull'inverno nella documentazione e nell'output, non solo nel codice.

---

## 7. Caveat

1. **ISO 7726:1998 non è stata letta direttamente** (norma a pagamento; il PDF campione pubblicato non è estraibile come testo nell'ambiente). I valori 0,22/0,06 e i coefficienti 0,70/0,97 sono confermati da due implementazioni indipendenti che li attribuiscono alla norma, non dalla norma stessa.
2. **Höppe 1999 e Lindberg et al. 2008 non sono stati letti in testo pieno** (Springer dietro autenticazione; abstract non disponibili nemmeno via Semantic Scholar). Le loro formulazioni sono state ricostruite dal codice sorgente delle implementazioni ufficiali e dalla documentazione che le cita.
3. **Fröhlich & Matzarakis 2016** — la fonte quantitativa più diretta sulla sensibilità degli indici al vento — **non è stata letta**: paywall Springer e abstract oscurato. È il primo documento da recuperare.
4. **La sensibilità di PET non è stata misurata** (§4.5). Trattare qualsiasi affermazione su PET e vento come non quantificata.
5. **Le misure `[M]` valgono per il polinomio UTCI**, non per il modello Fiala completo da cui deriva. Il polinomio è un'approssimazione, e vicino ai bordi del suo dominio (in particolare sotto 0,5 m/s) diverge dalla fisica in modo visibile — l'inversione di segno di dUTCI/dv sotto 0,5 m/s ne è la prova.
6. **Le condizioni scelte per le tabelle di §4.2 sono mie**, plausibili per il caso studio ma non estratte dall'EPW di Perugia. Se servono numeri per il caso Bastia vanno rifatti sui valori reali dell'EPW.
7. **Licenza del codice UTCI ufficiale**: c'è un permesso esplicito di incorporazione ma nessuna licenza formale (§3.1). Se il punto diventa load-bearing serve un parere, non la mia lettura.
8. **`UMEP-dev/solweig` si autodichiara sperimentale** e a numero di versione `0.1.0b95`. La sua API *"is stabilising but may change"*.
9. **Questo file è un rapporto di ricerca.** §6 è una raccomandazione, non una decisione presa.

---

## 8. Fonti

**Primarie — norme e paper originali**
- ISO 7726:1998, *Ergonomics of the thermal environment — Instruments for measuring physical quantities*: https://www.iso.org/standard/14562.html (non letta, §7.1)
- Höppe, P. (1999), *The physiological equivalent temperature*, Int. J. Biometeorol. 43:71-75 — https://doi.org/10.1007/s004840050118
- Höppe, P. (1984), *Die Energiebilanz des Menschen*, Wiss. Mitt. Meteorol. Inst. Univ. München 49
- Gagge, A., Stolwijk, J., Nishi, Y. (1971), *An effective temperature scale based on a simple model of human physiological regulatory response*, ASHRAE Trans. 77(1):247-262
- Bröde, P. et al. (2012), *Deriving the operational procedure for the Universal Thermal Climate Index (UTCI)*, Int. J. Biometeorol. 56:481-494 — https://doi.org/10.1007/s00484-011-0454-1
- Lindberg, F., Holmer, B., Thorsson, S. (2008), *SOLWEIG 1.0*, Int. J. Biometeorol. 52:697-713 — https://doi.org/10.1007/s00484-008-0162-7
- Thorsson, S., Lindberg, F., Eliasson, I., Holmer, B. (2007), *Different methods for estimating the mean radiant temperature in an outdoor urban setting*, Int. J. Climatol. — https://doi.org/10.1002/joc.1537
- Holmer, B. et al. (2015), *How to transform the standing man from a box to a cylinder*, ICUC9 — http://www.meteo.fr/icuc9/LongAbstracts/bph5-2-3271344_a.pdf
- Walther, E., Goestchel, Q. (2018), *The P.E.T. comfort index: Questioning the model*, Building and Environment 137:1-10 — https://doi.org/10.1016/j.buildenv.2018.03.054
- Di Napoli, C. et al. (2020), *Mean radiant temperature from global-scale numerical weather prediction models*, Int. J. Biometeorol. — https://doi.org/10.1007/s00484-020-01900-5 (open access: https://pmc.ncbi.nlm.nih.gov/articles/PMC7295834/)
- Middel, A., Krayenhoff, E.S. (2019), *Micrometeorological determinants of pedestrian thermal exposure during record-breaking heat in Tempe, Arizona: Introducing the MaRTy observational platform*, Sci. Total Environ. — https://doi.org/10.1016/j.scitotenv.2019.06.085
- Fischereit, J., Schlünzen, K.H. (2018), *Evaluation of thermal indices for their applicability in obstacle-resolving meteorology models*, Int. J. Biometeorol. — https://doi.org/10.1007/s00484-018-1591-6
- VDI 3787 Parte 2 (2008), *Environmental meteorology — Methods for the human biometeorological evaluation of climate and air quality for urban and regional planning*

**Primarie — codice sorgente e documentazione ufficiale**
- UTCI, sito ufficiale COST Action 730: https://utci.org/ ; calcolatore e limiti: https://utci.org/utci_calc.php
- UTCI, sorgenti Fortran `UTCI_a002` (Bröde, ottobre 2009): https://utci.org/resources/UTCI%20Program%20Code.zip
- SOLWEIG, manuale UMEP: https://umep-docs.readthedocs.io/en/latest/OtherManuals/SOLWEIG.html
- SOLWEIG, documentazione fisica (Tmrt / UTCI / PET): https://umep-dev.github.io/solweig/physics/tmrt/ , .../utci/ , .../pet/
- `UMEP-dev/UMEP-processing` (GPL-3.0): https://github.com/UMEP-dev/UMEP-processing — in particolare `processor/solweig_algorithm.py`, `functions/SOLWEIGpython/Solweig_run.py`, `processor/parametersforsolweig.json`, `functions/TreePlanter/SOLWEIG1D/Solweig1D_2023a_calc.py`
- `UMEP-dev/solweig` (Rust, GPL-3.0): https://github.com/UMEP-dev/solweig — `rust/src/tmrt.rs`, `rust/src/utci.rs`, `rust/src/pet.rs`, `rust/Cargo.toml`, `pysrc/solweig/postprocess.py`, `pysrc/solweig/api.py`, `pysrc/solweig/models/weather.py`, `VALIDATION.md`
- `pythermalcomfort` (MIT): https://github.com/CenterForTheBuiltEnvironment/pythermalcomfort — `pythermalcomfort/models/pet_steady.py`, `pythermalcomfort/models/utci.py`
- `thermofeel` (Apache-2.0, ECMWF): https://github.com/ecmwf/thermofeel — `thermofeel/thermofeel.py`
- `ladybug-comfort` (AGPL-3.0): https://github.com/ladybug-tools/ladybug-comfort — `ladybug_comfort/utci.py`
- crate `thermalcomfort` (MIT): https://crates.io/crates/thermalcomfort — https://github.com/NairAssoc/thermalcomfort
- ENVI-met, Thermal Comfort: https://envi-met.info/doku.php?id=apps:biomet
- ENVI-met, PET e PET\*: https://envi-met.info/doku.php?id=apps:biomet_pet
- ENVI-met, UTCI e la sua critica al vento: https://envi-met.info/doku.php?id=apps:biomet_utci
- ENVI-met, PMV/PPD: https://envi-met.info/doku.php?id=apps:biomet_pmv
- EnergyPlus Engineering Reference, Local Wind Speed Calculation (z_met = 10 m): https://bigladdersoftware.com/epx/docs/23-2/engineering-reference/outside-surface-heat-balance.html

**Secondarie**
- Charalampopoulos, I., Nouri, A.S. (2019), *Investigating the Behaviour of Human Thermal Indices under Divergent Atmospheric Conditions: A Sensitivity Analysis Approach*, Atmosphere 10(10):580 — https://doi.org/10.3390/atmos10100580
- Fröhlich, D., Matzarakis, A. (2016), *A quantitative sensitivity analysis on the behaviour of common thermal indices under hot and windy conditions in Doha, Qatar*, Theor. Appl. Climatol. 124:179-187 — https://doi.org/10.1007/s00704-015-1410-5 (**non letto**)
- Sci. Rep. 15 (2025), *Comparative reliability assessment of PET and UTCI thermal comfort indices using Monte Carlo simulation in urban microclimates* — https://www.nature.com/articles/s41598-025-33440-6 (usato solo per la resa esplicita della formula del globo termometro; `[I]` la presentazione del polinomio UTCI in eq. 7 di quel paper è scorretta — mostra un polinomio in UR anziché in pressione di vapore — quindi il resto va preso con cautela)
- TDAG, *What we know and don't know about the cooling benefits of urban trees* — https://www.tdag.org.uk/uploads/4/2/8/0/4280686/what_is_known_and_not_know_cooling_benefits_of_urban_trees.pdf
- Tsoka, S. et al. (2018), *Sustainable Cities and Society* 43:55-76 — via `research/envi-met.md` §2.11

**Interne**
- `research/envi-met.md` — ricognizione ENVI-met, in particolare §2.4 (radiazione AVF/IVS), §2.8 (BIO-met), §2.11 (accuratezza)
- Issue [#1](https://github.com/maeurong/CLIMESH/issues/1) — mappa del progetto, premesse fissate
