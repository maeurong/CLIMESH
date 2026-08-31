# ENVI-met — ricognizione tecnica completa

**Scopo**: quadro di riferimento per progettare un sostituto/alternativa open source (progetto CLIMESH).
**Data ricerca**: 2026-08-31. **Repo**: CLIMESH, branch `main`, HEAD `7f23d42` (repo vuoto: solo `README.md` con `# CLIMESH`).
**Versione ENVI-met di riferimento**: V5.9.0, rilasciata il 5 dicembre 2025.

**Convenzione di marcatura**
- `[P]` = confermato da fonte primaria (sito/wiki ufficiale ENVI-met, repo del progetto citato, GitLab/GitHub del progetto).
- `[S]` = fonte secondaria (paper peer-reviewed di terzi, listing commerciali, stampa).
- `[I]` = inferenza mia, non confermata da fonte.

---

## 1. Cos'è e a chi serve

### 1.1 Definizione

ENVI-met è un modello olistico tridimensionale non-idrostatico per la simulazione delle interazioni superficie-pianta-aria (*surface-plant-air interactions*), usato prevalentemente in ambiente urbano. Risoluzione orizzontale tipica 1–10 m, orizzonte temporale tipico 24–48 h, time step 1–5 s; con risorse adeguate si possono simulare mesi o un anno intero. `[P]` — https://envi-met.info/doku.php?id=intro:modelconcept

Non è "un CFD con qualche add-on": è un modello che accoppia in un unico loop CFD RANS, bilancio radiativo, modello di suolo, modello di vegetazione fisiologica, fisica degli edifici e dispersione inquinanti. Questo accoppiamento è la sua ragion d'essere e il motivo per cui è difficile da replicare. `[P]` (stessa pagina)

### 1.2 Storia e sviluppatore

- Sviluppato con continuità **dal 1994** dal geografo/climatologo tedesco **Michael Bruse**. `[S]` — https://en.wikipedia.org/wiki/ENVI-met
- Paper fondativo: **Bruse, M. & Fleer, H. (1998)**, *Simulating surface–plant–air interactions inside urban environments with a three dimensional numerical model*, **Environmental Modelling & Software 13(3–4): 373–384**. `[S]` — https://www.sciencedirect.com/science/article/abs/pii/S1364815298000425
- 2014: fondazione di **ENVI-met GmbH** (Michael e Daniela Bruse). Dal 2018 **Helge Simon** socio; **Tim Sinsel** contributore stabile. `[S]` — Wikipedia (sopra)
- **Settembre 2024: acquisizione da parte di One Click LCA** (azienda finlandese di software LCA per costruzioni). Termini non divulgati. `[P]` — https://envi-met.com/news/envi-met-acquired-by-one-click-lca/ ; `[S]` — https://tech.eu/2024/09/17/one-click-lca-acquires-envi-met/
- **Conseguenza operativa già visibile**: dalla versione **5.7.2 (febbraio 2025) serve un account One Click LCA per usare ENVI-met**, con Single Sign-On; vecchio sistema di licenza dismesso dal 31/12/2025. `[P]` — https://envi-met.info/doku.php?id=files:start e https://envi-met.info/doku.php?id=files:download

> Questo è un dato strategico: il vendor lock-in si è **irrigidito** nell'ultimo anno (account cloud obbligatorio per un software desktop). È esattamente il tipo di attrito che rende appetibile un'alternativa open.

### 1.3 Edizioni, licenza, prezzi

| Edizione | Licenza | Limiti noti |
|---|---|---|
| **LITE** | gratuita, **Creative Commons BY-NC-SA**, non commerciale | dominio max **50 × 50 grid**; meno feature di calcolo; niente dati dinamici facciate; niente Solar Access dettagliato; **niente parallel computing** |
| **BASIC** / **STUDENT** | a pagamento / studenti | **niente parallel computing** |
| **BUSINESS** | abbonamento annuale, rinnovo automatico, licenze team o singolo utente | tutti i moduli |
| **SCIENCE** / **Universities** | ricerca ed educazione, abbonamento o termine fisso | fino a **50 dispositivi per dipartimento**, incluso home use per studenti e docenti |
| **Students** | 1 anno, prova di iscrizione obbligatoria | **niente parallel computing** |

Fonti: `[P]` https://envi-met.info/doku.php?id=intro:modelconcept (LITE CC BY-NC-SA, 50×50) ; `[P]` https://envi-met.info/doku.php?id=kb:parallel ("The BASIC and STUDENT editions do not support parallel computing") ; `[P]` https://envi-met.info/doku.php?id=filereference:output:start (cartelle DYNAMIC/Solar Access "not in ENVI-met LITE") ; `[P]` https://envi-met.com/pricing/ (tipologie licenza).

**Prezzi**: ENVI-met **non pubblica un listino**. La pagina pricing ufficiale rimanda a "GET IN TOUCH"; la FAQ ufficiale rimanda a `license[at]envi-met.com`. `[P]` — https://envi-met.com/pricing/ , https://envi-met.info/doku.php?id=kb:faq

Numeri indicativi da listing di terze parti (**da verificare, bassa affidabilità**):
- Capterra: starting price **€290 per utente / anno**, piano "Basic"; nessuna prova gratuita, nessuna versione gratuita indicata. `[S]` — https://www.capterra.com/p/10008080/ENVI-met/
- GetApp: **$290 per utente / anno**. `[S]` — https://www.getapp.com/operations-management-software/a/envi-met/
- AlternativeTo riporta **$242/mese** in abbonamento. `[S]` — https://alternativeto.net/software/envi-met/about

> I tre valori sono incoerenti fra loro (€290/anno vs $242/mese = ~$2.900/anno). `[I]` L'ipotesi più plausibile è che €290/anno sia l'edizione BASIC/Student e che le licenze BUSINESS full stiano nell'ordine delle migliaia di euro/anno, ma **non ho una fonte primaria**. Se il numero è load-bearing per una decisione, va richiesto un preventivo diretto.

### 1.4 Chi lo usa e per cosa

Utenti dichiarati: architetti, architetti del paesaggio, urbanisti, ingegneri, ricercatori. `[P]` — https://en.wikipedia.org/wiki/ENVI-met (aree applicative), https://envi-met.com/pricing/

Casi d'uso tipici (case study pubblicati dal vendor): comfort termico outdoor (Kolkata, Lagos, Lima), comfort pedonale (Dubai), *cool corridor* e spazi camminabili (Abu Dhabi), mitigazione del calore (Milano/PoliMi), impatto del verde (Bolzano), cambiamento climatico (Parma). `[P]` — https://envi-met.com/pricing/ (sezione Case Studies, con booklet PDF scaricabili)

Ambiti scientifici: effetti dell'urbanizzazione (impermeabilizzazione, materiali, morfologia) su microclima urbano e salute; pianificazione climate-adaptive. `[S]` — Wikipedia + review Tsoka 2018, Liu 2021.

Deliverable tipici prodotti dagli utenti `[I]` (inferito da output disponibili e case study):
- mappe 2D a quota pedonale (1.5 m) di temperatura dell'aria, PET/UTCI, velocità del vento, MRT;
- confronti scenario-base vs scenario-progetto (ΔT, ΔPET) su un giorno tipo caldo;
- mappe di ombreggiamento/solar access su suolo e facciate;
- serie temporali su punti "receptor";
- report per stakeholder/committenza — dalla V5.8.0 esiste un **Web-Based Reporting Tool cloud** per utenti business. `[P]` — https://envi-met.info/doku.php?id=apps:updates

---

## 2. Fisica e modelli numerici

Tutta questa sezione è `[P]` salvo dove indicato; fonte principale https://envi-met.info/doku.php?id=intro:modelconcept più le pagine specifiche linkate.

### 2.1 Discretizzazione e schema numerico

- **Griglia**: griglia ortogonale **Arakawa C**. La topografia è rappresentata marcando celle come "piene di suolo". Conseguenza diretta: **solo strutture dritte e rettangolari**; superfici curve o inclinate vanno approssimate a gradini. Per la superficie del terreno esposizione e inclinazione esatte sono comunque considerate nel bilancio energetico.
- **Metodo**: **finite difference** per le PDE. Schema **parzialmente implicito, parzialmente esplicito** a seconda del sottosistema. **Advezione e diffusione atmosferiche sono completamente implicite** — è la scelta che consente time step relativamente grandi restando numericamente stabile.
- **Griglia verticale**: opzioni **splitting** (la cella più bassa viene divisa in 5) e **telescoping** (stretching progressivo sopra una quota di partenza). Esempio dal manuale: per raggiungere >200 m di altezza dominio servono 100 celle a 2 m; con telescoping 20% da 60 m → 45 celle; con splitting e risoluzione 5 m → 41 celle; combinando entrambi → **22 celle**. `[P]` — https://envi-met.info/doku.php?id=apps:spaces
- **Modello 1D di raccordo**: sopra il top del 3D (tipicamente 50–200 m) subentra un **modello 1D fino a 2500 m**, che fornisce anche i profili verticali per il bordo di inflow. `[P]` — https://envi-met.info/doku.php?id=kb:modellayout
- **Nesting grids**: banda di celle a risoluzione crescente attorno al dominio core, `Δxy(n) = Δxy(main) · n`. Servono solo per allontanare i bordi. **Il vendor oggi sconsiglia di usarle** e raccomanda invece di allargare il dominio normale. Nelle nesting non si possono piazzare edifici; il suolo è una scacchiera di due profili A/B; la radiazione può essere mediata sul dominio principale per evitare surriscaldamento irrealistico. `[P]` — https://envi-met.info/doku.php?id=kb:nesting
- **Condizioni al contorno laterali**: closed/forced, open, cyclic. `[P]` — https://envi-met.info/doku.php?id=kb:nesting (rimando a `kb:lbc`)
- **Regola pratica sui bordi**: nessun edificio nelle prime celle di bordo; distanza bordo–primo edificio ≈ metà dell'altezza dell'edificio, cioè tipicamente **4–8 celle libere per lato**. `[P]` — https://envi-met.info/doku.php?id=apps:spaces

### 2.2 Time step e ordine di risoluzione dentro il ciclo

Il time step principale **non è fisso**: è adattivo in funzione dell'altezza solare, divisa in 3 intervalli con due soglie `heightA`/`heightB` e un time step per intervallo (il più piccolo con sole alto). In versioni precedenti era fisso a 10 s. `[P]` — https://envi-met.info/doku.php?id=timesteps

**I sottomoduli non girano a ogni time step** — questo è il cuore dell'architettura di accoppiamento, e va capito bene per replicarla. Intervalli di default `[P]` — https://envi-met.info/doku.php?id=timing :

```
01: Update Surface Data each  ? sec             =30.0
02: Update Wind and Turbulence each ? sec       =900
03: Update Radiation and Shadows each ? sec     =600
04: Update Plant Data each ? sec                =600
```

- superficie (temperatura, umidità): ogni **30 s**
- campo di vento + turbolenza: ogni **900 s** (15 min)
- posizione del sole, ombre, flussi radiativi dal cielo: ogni **600 s**
- temperatura fogliare, resistenza stomatica, parametri pianta: ogni **600 s**

Esiste anche una modalità **wind continuo** (intervallo = 0, con turbulence mode 2) in cui il campo di flusso è una normale variabile prognostica calcolata a ogni step — molto lenta, il vendor stesso la definisce "test option […] not really tested". `[P]` — https://envi-met.info/doku.php?id=timing

> `[I]` Questa asimmetria di frequenze è la chiave del compromesso costo/accuratezza di ENVI-met: l'atmosfera avanza rapidamente su un campo di vento congelato per 15 minuti simulati. Un sostituto che voglia essere più veloce ha qui il margine principale; uno che voglia essere più accurato paga qui il prezzo.

### 2.3 CFD e turbolenza

- **Equazioni**: Navier-Stokes **RANS non-idrostatiche**, risolte per ogni cella e ogni time step. La vegetazione entra come **forza di trascinamento (drag)** nel campo di vento. Con la feature *Single Wall* si simula il flusso dentro strutture complesse o semi-aperte. Per la fisica degli edifici, il flusso è calcolato in prossimità di ogni segmento di facciata/tetto.
- **Turbolenza**: chiusura **E-ε di ordine 1.5** (modello k-ε), due equazioni prognostiche per energia cinetica turbolenta E e dissipazione ε; i coefficienti di scambio K si ottengono dalla **relazione di Prandtl-Kolmogorov**. In condizioni di vento debole si può usare un **modello di mixing length del 1° ordine** (il k-ε "spesso fallisce" lì).
- **Nota di stabilità dichiarata dal vendor**: c'è un **feedback non lineare fra Km e il sistema TKE-ε** — se Km diventa instabile, il sistema TKE-ε lo diventa a sua volta e ne genera di peggiori al ciclo successivo. `[P]` — https://envi-met.info/doku.php?id=kb:turbulence (la pagina è però marcata "*this information is outdated*" — vedi §2.11 sulla staleness della doc)

### 2.4 Radiazione — il pezzo più interessante

ENVI-met offre **due schemi** per la radiazione secondaria (onda lunga emessa e onda corta riflessa). `[P]` — https://envi-met.info/doku.php?id=kb:ivs

**AVF (Averaged View Factor)** — schema storico:
- ray tracing 3D da ogni centro cella, un raggio ogni **10° in zenit e 10° in azimut** → sfera di **18 × 36 = 648 facet**;
- calcolato **una sola volta per simulazione** (gli oggetti visti non cambiano);
- per ogni cella si memorizzano solo 4 view factor scalari: σ_Sky, σ_Veg, σ_Bldg, σ_Grnd;
- la radiazione secondaria è approssimata combinando questi view factor con **valori medi su tutto il dominio** → gli effetti dei materiali diversi si perdono nella media.

**IVS (Indexed View Sphere)** — default nelle versioni full, introdotto in V5:
- oltre al *tipo* di oggetto visto si memorizza un **puntatore di riferimento** alla specifica superficie di edificio, cluster di piante, superficie di suolo;
- si aggiunge un **fattore di visibilità** che tiene conto dell'ostruzione parziale da fogliame;
- il calcolo dei puntatori resta *one-shot*; a runtime si aggiornano solo i flussi riflessi/emessi;
- **costo memoria dichiarato**: per un dominio di soli **200 × 200 × 35 celle** servono **~1 miliardo di record** con risoluzione angolare 10°/10°. Per questo esiste un "height cap" che riduce la risoluzione angolare sopra una soglia definita dall'utente.
- Riferimento scientifico: **Simon et al. 2021**, *Advances in Simulating Radiative Transfer in Complex Environments*, Applied Sciences 11(12):5449, open access. `[S]` — https://doi.org/10.3390/app11125449

Copertura complessiva della radiazione: onda corta e onda lunga con ombreggiamento da geometrie complesse, riflessioni multiple, effetto della vegetazione su tutti i flussi, diffusione della radiazione nelle chiome. `[P]` — modelconcept

> `[I]` L'IVS è, in sostanza, un radiosity con indicizzazione persistente dei contributi. È il singolo componente più costoso in memoria e probabilmente il più difficile da replicare bene in un sostituto open. Un miliardo di record per un dominio medio-piccolo è il vincolo dimensionante dell'intera architettura.

### 2.5 Vegetazione

- Piante verticali semplici (erba, mais) **e** geometrie 3D complesse (alberi). Ogni pianta è trattata come specie individuale con bilancio idrico integrato e reazione a stress termico e idrico. `[P]` — modelconcept
- **LAD (Leaf Area Density)**, unità m² di superficie fogliare per m³ di aria, contata **one-sided** (solo un lato della foglia è superficie attiva, coerentemente con la posizione degli stomi e con il fatto che un lato è in scia). Il vendor ammette esplicitamente che *"the original LAD profiles provided by ENVI-met are rather hand made and based on only a few reference profiles"*. Metodi consigliati per ricavare profili LAD: misure ottiche, raccolta foglie, approcci analitici (Lalic & Mihailovic 2004; Stadt & Lieffers 2000; Ross et al. 2000; Meir et al. 2000). `[P]` — https://envi-met.info/doku.php?id=kb:lad
- **Temperatura fogliare**: risoluzione del bilancio energetico della superficie fogliare per **ogni grid box della chioma**, in funzione delle condizioni meteo e fisiologiche locali. `[P]` — modelconcept
- **Resistenza stomatica — due modelli selezionabili** `[P]` — https://envi-met.info/doku.php?id=plantmodel :
  ```
  01: Stomata res. approach (1=Deardorff, 2=A-gs)  =2
  02: Background CO2 concentration [ppm]           =350
  ```
  - **Deardorff (1978)**: scala un valore massimo esplicito di resistenza stomatica in funzione dell'input solare e della disponibilità idrica;
  - **A-gs (Jacobs 1994)**: calcola il tasso di fotosintesi, da lì la domanda di CO2, e da lì lo stato degli stomi. **Raccomandato dal vendor** come più fondato fisiologicamente. Default = 2.
  - Documento tecnico dedicato: https://envi-met.info/documents/new_a_gs.pdf `[P]`
- **Radiazione nella chioma**: ray tracing per ombreggiamento e schermatura termica onda lunga. `[P]`
- **Albero**: applicazione dedicata alla creazione/gestione di vegetazione 3D e **QSM**; genera chiome anche via regole procedurali (**sistemi di Lindenmayer / L-system**). `[P]` — https://envi-met.info/doku.php?id=apps:start , https://envi-met.info/doku.php?id=kb:lad
- **TreePass**: modulo per fabbisogni di salute della pianta e wind risk assessment (meccanica dell'albero, rischio danni da tempesta). **Dichiarato ancora in sviluppo**. `[P]` — modelconcept

### 2.6 Suolo

- Temperatura di superficie e distribuzione della temperatura nel suolo per suoli naturali e materiali sigillati, **fino a −4 m di profondità**. Materiale diverso selezionabile **per ogni layer verticale**. Conducibilità termica dei suoli naturali calcolata in funzione del contenuto idrico effettivo. `[P]` — modelconcept
- **Bilancio idrico**: stato idraulico risolto dinamicamente su **legge di Darcy**, con evaporazione, scambio interno al suolo, e **assorbimento radicale**. `[P]`
- **Modello di radici 3D** accoppiato: la disponibilità idrica nella zona radicale regola la traspirazione effettiva. `[P]`
- **Corpi idrici**: trattati come tipo speciale di suolo, con trasmissione/assorbimento della radiazione onda corta nell'acqua. **Limiti dichiarati esplicitamente dal vendor**: nessun secondo bilancio energetico sul fondo, nessun mixing turbolento → **il riscaldamento di corpi poco profondi è sottostimato** e l'uso è ristretto ad acque ferme (laghi). `[P]` — modelconcept

### 2.7 Edifici e superfici costruite

- **Geometria 3D piena**, senza limiti di complessità entro la struttura cubica di base; supporto a **single thin walls** applicabili a qualunque cella (pensiline, pergole, fermate bus). `[P]`
- **Materiali**: in Detailed Design Mode ogni superficie di muro/tetto ha il proprio tipo, composto da **fino a 3 layer** di materiali diversi (trasmissione solare, capacità termica, conducibilità). `[P]`
- **Fisica dell'edificio ad alta risoluzione**: ogni segmento di parete/tetto ha il proprio modello termodinamico con **7 nodi prognostici**; il nodo esterno si aggiorna in continuo rispetto alle variabili meteo alla facciata e allo stato termico degli oggetti nel campo di vista; i nodi interni seguono la **legge di Fourier**. `[P]`
- **Building energy performance**: temperatura indoor calcolata dinamicamente **in parallelo** alla simulazione outdoor, per ogni edificio, con feedback costante fra indoor e outdoor e fra edifici. Le versioni recenti includono un modello di zoning iniziale. `[P]`
- **Green Wall System (GWS) / green roof**: integrato nella dinamica del bilancio energetico di facciata, dai rampicanti ai living wall veri e propri, con layer di substrato. `[P]`

### 2.8 Comfort termico (BIO-met / "Thermal Comfort")

Post-processore che gira **dopo** la simulazione ENVI-core. Indici supportati: **PMV/PPD**, **PET\*** (PET reviewed), **dPET** (dynamic thermal comfort), **UTCI**, **SET\*** (ASHRAE 55-2013 senza modifiche). MRT è tra le grandezze calcolate. `[P]` — https://envi-met.info/doku.php?id=apps:biomet

Costo computazionale relativo, dichiarato: **UTCI è regressivo e veloce**; **PMV richiede la soluzione iterativa di un bilancio energetico**; **PET ne richiede due** ed è "much more calculation time expensive". Il modulo supporta calcolo parallelo. `[P]`

Il vendor **raccomanda PET**, allineandosi a Fischereit & Schlünzen (2018), Int. J. Biometeorology (open access). `[P]` — https://link.springer.com/article/10.1007/s00484-018-1591-6

Avvertenza esplicita del vendor: un indice di comfort è valido solo se i dati microclimatici di input lo sono. `[P]`

### 2.9 Inquinanti

Rilascio, dispersione e deposizione sincroni di **fino a 6 inquinanti** contemporaneamente: particolato (con sedimentazione e deposizione su foglie e superfici), gas passivi inerti, e **gas reattivi del ciclo NO–NO₂–O₃** (fotochimica). `[P]` — modelconcept. Output nella cartella `Pollutants` con tassi di conversione chimica. `[P]` — https://envi-met.info/doku.php?id=filereference:output:start

### 2.10 Accoppiamento fra moduli

Struttura di accoppiamento ricostruita da fonti primarie `[P]`, con la sequenza esatta dentro un time step `[I]` (non ho trovato un diagramma di flusso ufficiale del loop):

1. **1D model** (fino a 2500 m) → fornisce profili verticali e condizioni di inflow al 3D.
2. **Campo di vento + turbolenza** — aggiornato ogni 900 s di default; congelato in mezzo.
3. **Radiazione + ombre + posizione sole** — ogni 600 s; IVS/AVF calcolati una volta sola all'inizio per la parte geometrica.
4. **Superficie e suolo** — ogni 30 s; il suolo fornisce disponibilità idrica alla vegetazione.
5. **Vegetazione** — ogni 600 s; le foglie sono sorgenti/pozzi di calore sensibile e vapore per l'atmosfera, e assorbono acqua dal suolo via radici.
6. **Edifici** — bilancio a 7 nodi per segmento, accoppiato al modello indoor, con feedback all'atmosfera.
7. **Atmosfera** (T, umidità specifica, inquinanti) — advezione e diffusione **implicite**, a ogni time step, sui campi congelati sopra.

### 2.11 Limiti noti e critiche in letteratura

**Limiti strutturali dichiarati dal vendor stesso** `[P]`:
- solo strutture dritte e rettangolari; superfici curve/inclinate approssimate a gradini (conseguenza della Arakawa C-grid);
- **non si può fermare e riprendere una simulazione**: "If you cancel your simulation, you have to start from the beginning" — https://envi-met.info/doku.php?id=kb:faq
- **non si può distribuire un run su più PC** ("Can I distribute my ENVI-met run over several PCs? — Not at the moment") — stessa pagina;
- non progettato per simulare un'intera città; niente opzioni di land use misto per cella;
- corpi idrici modellati in modo semplificato (§2.6);
- floating point error / division by zero come modalità di fallimento **attesa**: il vendor le descrive come parte del mestiere ("these runtime errors belong to numerical modeling like woodchip to carpenters") — https://envi-met.info/doku.php?id=kb:faq

**Bug rilevanti recenti, dal changelog ufficiale** `[P]` — https://envi-met.info/doku.php?id=apps:updates :
- **V5.9.0**: *"ENVI-core: Fixed a major bug in conservation of energy — this leads to noticeable changes in the results"*. Un bug di conservazione dell'energia corretto a fine 2025 significa che **tutti i risultati prodotti con versioni ≤5.8.0 sono quantitativamente diversi da quelli attuali**.
- **V5.9.0**: metodo di scambio termico parete-aria cambiato "to a more accurate approach".
- **V5.8.0**: fix di un problema nel modulo greening e correzione della quantità di onda corta riflessa.

**Accuratezza — meta-analisi** `[S]`:
- **Tsoka et al. (2018)**, *Sustainable Cities and Society* 43:55–76, meta-analisi su **52 studi di valutazione**: mediana **MAE 1.34 °C** e **RMSE 1.51 °C** sulla temperatura dell'aria; R² fra 0.60 e 0.97 sulle temperature superficiali; calo di picco della temperatura dell'aria mediano **1.0 °C** per alberi urbani e **2.0 °C** per pavimentazioni fredde + alberi extra. https://www.sciencedirect.com/science/article/abs/pii/S2210670718307649
- Bias sistematici riportati in letteratura: sottostima della temperatura dell'aria, disallineamento nel calcolo dei flussi radiativi, **sovrastima della turbolenza**; radiazione sovrastimata al mattino e pomeriggio e sottostimata a mezzogiorno. `[S]`
- **MRT**: la V5 con IVS ha migliorato ma la letteratura riporta sottostima sistematica della MRT in condizioni estive sia in ombra sia al sole; studi più recenti trovano invece assenza di sottostima legata all'ombra e una tendenza alla **sovrastima** della MRT diurna in aree non ombreggiate. Quadro non consolidato. `[S]` — https://www.sciencedirect.com/science/article/abs/pii/S2212095522001973 (Sinsel/Simon, MRT schemes) ; https://www.sciencedirect.com/science/article/pii/S0360132325009485 (Hong Kong)
- **Validazione comparativa 5.6 vs 5.9** contro misure in-situ ad alta risoluzione: esiste un paper del 2026 (ScienceDirect S2590162126000675) — **non sono riuscito a leggerne l'abstract** (403 su ScienceDirect). Da recuperare, è probabilmente la fonte più aggiornata sull'accuratezza. `[S]` (esistenza confermata dai risultati di ricerca, contenuto non verificato)
- **Il modello non è grid-independent** e i limiti della griglia verticale sollevano dubbi di accuratezza in alcuni casi. `[S]` — https://www.sciencedirect.com/science/article/abs/pii/S2212095518301007 (Crank et al., "Evaluating the ENVI-met microscale model for suitability in analysis of targeted urban heat mitigation strategies")

**Staleness della documentazione ufficiale** `[P]` — problema reale e verificato:
- la pagina Turbulence Model è marcata *"—- this information is outdated —-"*;
- la pagina "Running ENVI-met" parla ancora di **32 Bit Version**, **un solo core**, e credito Azure di **EUR 150** — contenuto chiaramente pre-2018, mentre altre pagine dicono che il 32 bit non esiste più e che il parallelo c'è dalla 4.3;
- `kb:simple_full_forcing` e `filereference:start` restituiscono "This topic does not exist yet"; `filereference:inx` è **una pagina vuota** (solo il titolo).

> `[I]` Il fatto che la reference ufficiale del formato `.INX` — il formato di input principale — sia una pagina vuota è significativo: **il formato INX non è pubblicamente documentato**, a differenza di EDX/EDT ed EML che lo sono in dettaglio. Chi vuole interoperare con INX deve fare reverse engineering o partire dai plugin open esistenti (§3.7).

---

## 3. Architettura software e I/O

### 3.1 Stack tecnologico

**Confermato da fonte primaria** — https://envi-met.info/doku.php?id=intro:modelconcept , sezione "Programming language":
- **ENVI-met è scritto in Object Pascal, con Delphi, per Windows.**
- Esiste **anche una versione completa in C++**, ma il vendor dichiara di essere tornato a Object Pascal perché non mostrava benefici.
- Citazione testuale rilevante: *"Switching the core code from one language to another is about 3 days of work"*.

Altre proprietà `[P]`:
- **Solo Windows 64 bit**, minimo Windows 10. Niente 32 bit, **niente Linux, niente macOS**. — https://envi-met.info/doku.php?id=files:download , https://envi-met.info/doku.php?id=kb:faq
- Nessuna versione Linux → *"Can I run ENVI-met on any Supercomputer? Yes, if the Supercomputer runs under or supports WINDOWS."* — https://envi-met.info/doku.php?id=kb:parallel
- **Parallelismo multi-core dalla V4.3 (Winter1718)**, solo edizioni SCIENCE/BUSINESS. **Non tutto il modello è parallelo**: le interazioni fra elementi non sono parallelizzabili, quindi la CPU non è mai al 100%; su modelli piccoli (~50×50×20) l'overhead può rendere il parallelo **più lento** del single-core. Soglia indicata: sotto **40×40×30 celle** meglio single core. `[P]` — https://envi-met.info/doku.php?id=kb:parallel , https://envi-met.info/doku.php?id=apps:envimet_core
- **Nessuna menzione di GPU** in nessuna pagina ufficiale che ho letto. `[I]` Assumo che non ci sia accelerazione GPU.
- Il vendor rivendica riduzioni di tempo di simulazione del **20–50% in V5.8.0** e di **fino al 50% ulteriore in V5.9.0** rispetto a 5.8.0, per ottimizzazione di "background processes". `[P]` — https://envi-met.info/doku.php?id=apps:updates

### 3.2 Componenti dell'ecosistema

Il vendor è esplicito: *"There is no 'big' central application doing all the work — ENVI-met is a collection of several stand-alone applications"*. `[P]` — https://envi-met.info/doku.php?id=apps:start

**Applicazioni principali** `[P]`:
| Componente | Ruolo |
|---|---|
| **Headquarter** | launcher/cockpit, accesso rapido a tutte le app e gestione dei run |
| **SPACES** (Model Area Editor) | digitalizzazione raster del model area; geometria del modello (x,y,z, risoluzioni, splitting, telescoping); "Model Inspector" per scegliere la griglia verticale |
| **Monde** (World Editor) | editor **vettoriale** del mondo prima dell'export a INX raster; import **OpenStreetMap** e **OpenTopography**, import **shapefile** con CRS/UTM, classificazione attributi → layer ENVI-met |
| **ENVI-guide** (Simulation Settings) | crea/edita i file `.SIMX`; impostazioni mandatory + advanced; scelta della meteorologia |
| **ENVI-core** (Start Simulation) | il solutore, "the true workhorse" |
| **Leonardo** | analisi e visualizzazione, mappe 2D/3D e animazioni; ospita **DataStudio** (Python) |
| **BIO-met / Thermal Comfort** | post-processing degli indici di comfort (§2.8) |

**Applicazioni di supporto** `[P]`:
| Componente | Ruolo |
|---|---|
| **Database Manager** | editor di materiali, suoli, e altri parametri fisici; DB di sistema / progetto / utente |
| **Albero** | libreria alberi; vegetazione 3D e QSM; L-system procedurali |
| **Workspaces/Projects** | organizzazione di workspace, progetti, impostazioni per progetto |
| **Forcing Manager** | costruzione dei file Full Forcing (`.FOX`) da CSV/EPW/TRY |

### 3.3 Formati file

**Input**
- **`.INX`** — Area Input File (model area). **Reference ufficiale vuota** (§2.11). `[P]` (pagina esiste, contenuto assente)
- **`.SIMX`** — Simulation Settings, creato da ENVI-guide. `[P]`
- **`.FOX`** — Meteo Data / Full Forcing file. `[P]` — https://envi-met.info/doku.php?id=apps:enviguide
- Database materiali/suoli/piante in formato EML.

**Formato EML (ENVI-met Markup Language)** `[P]` — https://envi-met.info/doku.php?id=filereference:fileformat
- introdotto in V4, **XML-like ma non XML**: sottoinsieme limitato di XML più estensioni non standard (es. tag matrice); il root node è `<ENVI-MET_Datafile>`, **non** l'header XML classico; **niente nesting di elementi**; niente shortcut per tag vuoti.
- Header con `filetype`, `version`, `revisiondate`, `remark`, `encryptionlevel`.
- Avvertenza del vendor: si possono generare EML con un editor XML esterno **ma non è garantito che ENVI-met li accetti**.
- **Migrazione in corso**: *"ENVI-met EML-based files will be, step-by-step, replaced by JSON formatted ASCII files"*, con conversione on-the-fly e backup `.old`.

**Output**
- **`.EDX` + `.EDT`** — coppia metadati ASCII (EML) + **binario raw Intel float single precision**. `[P]` — https://envi-met.info/doku.php?id=filereference:edx_edi
  - `data_type`: `ft2DRaster=1`, `ft3DRaster=2`, `ft3DFacade=3` (3 dati per cella: facce x, y, z);
  - `data_content`: 14 valori enum (`fcAtmosphere=1`, `fcSurface=2`, `fcSoil=3`, `fcPollutants=4`, `fcBiomet=5`, `fcVegetation=6`, `fcFacade=7`, `fcSolarAccess=8`, `fcFacadeStatic=9`, `fcFacadeSolarAccess=10`, `fcRadiation=11`, `fcViewScape=12`, `fcPhotocat=13`);
  - `data_health_status`: `fsNormal=0`, `fsCheck=1`, `fsInitialisation=2`, `fsPanicDump=3` — Leonardo ignora tutto ciò che non è Normal;
  - dimensioni e spacing per asse, ridondanti **di proposito** per permettere a lettori esterni di decodificare senza capire la semantica.
  - Le enum sono pubblicate **come dichiarazioni di tipo Object Pascal** nella doc — ulteriore conferma dello stack.
- **CSV** — dalla V5 **tutti i file di testo sono CSV**, pensati per Pandas. `[P]`
- **NetCDF** — export disponibile dal Winter Release 2019; dalla **V5.8.0 le librerie NetCDF sono bundled e i file NetCDF sono generati automaticamente durante la simulazione**. `[P]` — modelconcept + https://envi-met.info/doku.php?id=apps:updates

**Organizzazione output a 3 livelli** `[P]` — https://envi-met.info/doku.php?id=filereference:output:start
1. cartella (`InputData`, `Atmosphere`, `Buildings/STATIC|DYNAMIC`, `Inflow`, `Log`, `Pollutants`, `Radiation`, `Receptors`, `Soil`, `Solar Access`, `Surface`, `Vegetation`, + `Biomet`);
2. nome file `<BaseName>_<TYPE>_<YYYY-MM-DD_hh.mm.ss>` con identificatori `_AT_`, `_FX_`, `_SO_`, `_VEG_`, `_BLDG_`, `_POLU_`, `_RD_`, `_BIO_`, `_SA_`, `_SAFAC_`;
3. tag `<data_content>` nell'EDX.
- Da V5.0.3 la cartella `InputData` contiene **una copia di INX, SIMX, DB e forcing usati**, per riprodurre il run — una forma primitiva ma reale di riproducibilità. `[P]`

### 3.4 Forcing meteorologico

Due modalità `[P]` — https://envi-met.info/doku.php?id=apps:enviguide , https://envi-met.info/doku.php?id=apps:forcingmanager :
- **Basic weather** (ex "simple forcing"): curve di T e umidità su 24 h definite a mano, ora di max/min, vento, copertura nuvolosa su 3 quote in **octas** (0–8).
- **Detailed Weather / Full Forcing** (raccomandata): import **`.EPW`**, **`.TRY`**, **`.CSV`**. È una **condizione al contorno laterale**: i valori del modello 1D o del file di forcing vengono copiati sul bordo. Forza temperatura, umidità, velocità e direzione del vento, radiazione **oppure** copertura nuvolosa, precipitazione.
- **Risoluzione richiesta: intervalli di 30 minuti.** `[P]`
- Precipitazione: forzarla è raccomandata solo per periodi molto lunghi (mesi). `[P]`
- **Controlli di sanità raccomandati dal vendor**: vento non troppo debole (**< 0.8 m/s**) né troppo forte (**> 5 m/s**); direzione del vento non deve cambiare bruscamente (es. da 0° a 180° in un'ora); umidità relativa coerente con la temperatura. `[P]`
- Dalla **V5.8.0**: **download automatico di dati ERA5** e ricerca di giorni tipici, con import diretto nel SIMX. `[P]` — https://envi-met.info/doku.php?id=apps:updates

### 3.5 Workflow end-to-end tipico

Ricostruito da fonti primarie `[P]`:
1. Creare/selezionare **workspace e progetto** (Workspaces).
2. Costruire il model area: **Monde** (vettoriale, import OSM/OpenTopography/shapefile) → export a INX; **oppure** **SPACES** direttamente in raster; **oppure** plugin QGIS/Rhino/SketchUp (§3.7).
3. In SPACES: impostare **location geografica** (serve per la radiazione), geometria (nx, ny, nz, risoluzioni), splitting/telescoping, verificare con **Model Inspector**; lasciare 4–8 celle libere ai bordi.
4. Assegnare materiali/suoli/piante dal **Database Manager**; alberi da **Albero**.
5. Preparare la meteorologia: **Forcing Manager** → file `.FOX` da EPW/TRY/CSV (o ERA5 automatico da 5.8.0).
6. **ENVI-guide**: creare il `.SIMX` (scenario, data/ora di start, durata, nome simulazione, cartella output, file INX, cosa forzare, radiazione IVS/AVF, time step, intervalli di update).
7. Eseguire con **ENVI-core**, da GUI o da CLI (§3.6).
8. **BIO-met** per gli indici di comfort → cartella `Biomet`.
9. **Leonardo** per mappe 2D/3D e animazioni; **DataStudio** per Python custom; export NetCDF/CSV per GIS o pipeline esterne.
10. (business, da 5.8.0) upload nel **project cloud** e generazione report per stakeholder.

### 3.6 Headless / batch — smentita di un luogo comune

**Esiste una versione da riga di comando.** `[P]` — https://envi-met.info/doku.php?id=apps:envimet_core

```
\win64\envicore_console.exe {WorkspaceFolder} {ProjectName} {SIMXName}
```

Senza parametri apre un dialogo per workspace/progetto/SIMX. L'output a terminale è ridotto rispetto alla GUI.

> Questa è una premessa che va corretta rispetto alla narrativa comune ("ENVI-met non ha headless"): **l'headless c'è**, resta Windows-only e senza API strutturata, ma il batch scripting è possibile. Esistono infatti runner di terze parti costruiti sopra (§3.7).

**Python**: dalla V5 Python è integrato nel sistema. **DataStudio** (dentro Leonardo e progressivamente in altre app) permette di lanciare script Python per analisi e visualizzazione, con script di esempio; il vendor dichiara che "all modules can be accessed via Python scripts" e che DataStudio permetterà all'utente di controllare gran parte della logica applicativa. `[P]` — modelconcept, https://envi-met.info/doku.php?id=filereference:output:start , https://envi-met.com/envi-met-v5-python-and-datastudio/ `[S]`
> `[I]` **Non ho trovato una reference API Python pubblicata.** Le pagine tutorial referenziate (`envi-met.com/tutorials/python-2/`) restituiscono 404. Assumo che DataStudio sia scripting embedded per il post-processing, **non** una API programmabile documentata e stabile.

### 3.7 Integrazioni

**Ufficiali** `[P]` — https://envi-met.info/doku.php?id=apps:start :
- **QGIS** — plugin "Geodata to ENVI-met": genera INX da geodata. Configurabili: dimensioni e risoluzione del dominio, edifici (altezza, sottopassi, materiali muro/tetto, greening di tetto e facciata), tipi di superficie, vegetazione semplice e complessa, DEM/terreno, sorgenti di gas/particolato, receptor. Testato su **Windows, Ubuntu e macOS**. Richiede layer nella stessa proiezione (preferibilmente UTM) e con i campi ID del DB ENVI-met. — https://envi-met.info/doku.php?id=apps:gis4envi-met
  - Sorgente: **GitHub `One-Click-LCA/geodata2ENVI-met`, Python, GPL-2.0, 5 star, ultimo push 2026-07-02** `[P]` (via GitHub API)
- **Rhino/Grasshopper — Morpho** (anche Dynamo): crea INX 2.5D e 3D, scrive SIMX con >15 impostazioni avanzate (simple e full forcing), **lancia la simulazione**, e legge i binari EDT (Atmosphere, Soil, Surface, Buildings, Vegetation, Solar access, Radiation). Installer su GitHub `AntonelloDN/Morpho` v1.11.0. — https://envi-met.info/doku.php?id=plugins:grasshopper
- **SketchUp** — plugin per creare model area. Sorgente: `AntonelloDN/Envimet-inx` e `One-Click-LCA/Envimet-INX`, Ruby, GPL-3.0. `[P]`
- **Blender** — plugin per visualizzare model area e risultati.

**Terze parti rilevanti** `[P]` (dati GitHub API, 2026-08-31):
- `kunifujiwara/VoxCity` — Python, MIT, **540 star**, push 2026-08-30. Framework di integrazione geospaziale/3D city model che **esporta verso INX** (altezza edifici e terreno → INX; canopy height e land cover → vegetation ID e material ID). Paper: arXiv 2504.13934. **Il progetto più vivo dell'ecosistema open attorno a ENVI-met.**
- `mothlight/Envimet.py` — librerie Python per leggere i file ENVI-met (2 star).
- `JOHNDST/ENVImet_batch` — runner multi-scenario sequenziale (2 star, 2025).
- `ufz-vislab/EnvimetReader` — plugin reader ParaView, C++, MIT (2019, fermo).
- `Natasja1992/ifc-citygml-2-envi` — IFC e CityGML → INX (2021, fermo).
- `aetherrootr/envi-met-converter` — CLI Go per convertire export (2024).

> `[I]` Nota importante per CLIMESH: la maggior parte di questi progetti è **micro-scala e semi-abbandonata** (0–5 star, ultimo commit anni fa). L'unico con massa critica è VoxCity, che però tratta ENVI-met come *un* target di export fra tanti, non come oggetto da sostituire.

### 3.8 Hardware e tempi di simulazione

**Requisiti hardware** — la pagina FAQ ufficiale è **stale** (parla ancora di 32 bit e single core), quindi i numeri sotto sono `[S]` da riassunti di terze parti sui documenti ENVI-met e vanno verificati:
- CPU raccomandata: 16 core o più (i9-7960, Threadripper 2990WX); configurazione modesta: 6–8 core (i5-8400, Ryzen 5 1600X);
- RAM: 64–128 GB raccomandati, 32 GB minimo consigliato;
- >500 GB di disco libero;
- nessun requisito specifico di GPU.

Confermato `[P]`: minimo **Windows 10 64 bit**; ~**3 GB di memoria per istanza** (dato però riferito alla vecchia versione 32 bit); si possono lanciare più istanze se ci sono memoria e core liberi.

**Tempi di simulazione — numeri concreti** `[P]` — https://envi-met.info/doku.php?id=kb:compute :
- *"running a day cycle (24 h) for a 250×250×30 grid cell model can easily take up to **one week or more** processing time"*;
- complessità: il vendor la descrive come **N²** — un modello 30×30×20 = 18.000 celle ≈ 324 milioni di relazioni; un 60×60×20 = 72.000 celle ≈ 5.184 milioni di relazioni;
- 250×250×30 è considerato "un modello grande";
- consiglio ufficiale: iniziare con **30×30**;
- consiglio ufficiale: **avere un PC dedicato**, accessibile via Remote Desktop, e controllare periodicamente che il run non sia morto.

> `[I]` Attenzione: la pagina `kb:compute` è pre-parallelizzazione. Con le ottimizzazioni dichiarate in 5.8.0 (−20/50%) e 5.9.0 (fino a −50% ulteriore) e il multi-core, il numero reale su hardware 2026 sarà sensibilmente più basso, ma **il vendor non ha aggiornato la stima**. La settimana per 250×250×30 va presa come limite superiore storico, non come dato corrente.

**Cloud**: nessun servizio cloud di calcolo ufficiale. La doc suggerisce di farsi da soli macchine Windows su **Azure** (preferito perché Windows) o EC2, installare ENVI-met a mano via RDP, e riportarsi indietro i risultati. `[P]` — https://envi-met.info/doku.php?id=kb:compute. Dalla 5.8.0 esiste un **cloud di reporting** (upload risultati, visualizzazione interattiva, report) ma **non** di calcolo. `[P]` — https://envi-met.info/doku.php?id=apps:updates

---

## 4. Punti deboli e spazio per un sostituto

### 4.1 Debolezze confermate da fonte primaria

| # | Debolezza | Fonte |
|---|---|---|
| 1 | **Windows-only, 64 bit.** Niente Linux, niente macOS. Non gira su supercomputer non-Windows. | `[P]` kb:parallel, files:download, kb:faq |
| 2 | **Codice chiuso**, Object Pascal/Delphi. Nessun accesso al core. | `[P]` modelconcept |
| 3 | **Nessun checkpoint/restart.** Un run interrotto riparte da zero. | `[P]` kb:faq |
| 4 | **Nessuna distribuzione su più macchine.** Parallelismo solo shared-memory, e nemmeno su tutto il modello. | `[P]` kb:faq, kb:parallel |
| 5 | **Parallelismo a pagamento.** LITE/BASIC/STUDENT girano single-core. | `[P]` kb:parallel |
| 6 | **Geometria vincolata a griglia ortogonale.** Niente strade oblique, niente superfici curve se non a gradini. | `[P]` modelconcept ; `[S]` letteratura |
| 7 | **Formato INX non documentato pubblicamente** (pagina reference vuota). EDX/EDT ed EML invece lo sono. | `[P]` filereference:inx |
| 8 | **Nessuna API pubblica documentata.** Python via DataStudio è scripting embedded per il post-processing. | `[P]`/`[I]` |
| 9 | **Documentazione stale a macchia di leopardo** (pagine "outdated", topic inesistenti, pagine pre-2018 accanto a changelog 2025). | `[P]` §2.11 |
| 10 | **Instabilità numerica come modalità di fallimento normale**, senza diagnostica strutturata: il rimedio suggerito è provare cose. | `[P]` kb:faq, kb:turbulence |
| 11 | **Dipendenza cloud obbligatoria dal 2025**: account One Click LCA e SSO richiesti per usare un software desktop. | `[P]` files:start, files:download |
| 12 | **Prezzi non pubblici**, solo "contattaci". | `[P]` envi-met.com/pricing |
| 13 | **Lentezza**: fino a una settimana per 24 h su 250×250×30 (stima vendor storica). | `[P]` kb:compute |
| 14 | **Bug di conservazione dell'energia corretto in V5.9.0**, con impatto dichiarato sui risultati → i risultati storici non sono confrontabili fra versioni. | `[P]` apps:updates |
| 15 | **Nessun servizio di calcolo cloud**: fai-da-te su Azure. | `[P]` kb:compute |

### 4.2 Cosa manca — opportunità

`[I]` salvo dove indicato; questa è la sezione più speculativa e va letta come tale.

- **Riproducibilità e versionamento**: esiste solo `InputData/` come copia degli input. Nessun concetto di run ID immutabile, hash degli input, lineage, diffing fra run. Un sostituto con input dichiarativi versionabili in git avrebbe un vantaggio immediato su un pubblico che oggi versiona a mano cartelle `.INX`.
- **Headless-first e CI**: l'headless esiste (`envicore_console.exe`) ma è un'appendice della GUI, Windows-only. Un core Linux containerizzabile che gira in CI/cloud è lo spazio più netto.
- **Restart/checkpoint**: assenza confermata. Su run di giorni è un difetto grave e concettualmente facile da risolvere.
- **Scaling orizzontale**: nessuna distribuzione multi-nodo. PALM (§5) dimostra che è fattibile su questa fisica — ha scalato fino a 32.000 core `[P]`.
- **GPU**: nessuna traccia in ENVI-met. SOLWEIG-GPU dimostra che almeno la parte radiativa/geometrica è portabile su GPU `[P]` (https://pypi.org/project/solweig-gpu/).
- **API Python di primo livello** (definizione dominio, materiali, run, lettura risultati come xarray/pandas), invece di scripting dentro la GUI.
- **Interoperabilità formati**: INX chiuso vs. un formato aperto e leggibile (JSON/YAML + NetCDF/Zarr). Nota: **ENVI-met stesso sta migrando EML → JSON** `[P]`, il che indica che il vendor riconosce il problema.
- **Geometria non-ortogonale** o almeno cut-cell / immersed boundary per strade oblique e facciate curve. Questo è però il punto dove il costo implementativo esplode.
- **Trasparenza dei bias**: pubblicare validazioni riproducibili contro dataset di riferimento, invece di lasciare che siano 52 paper indipendenti a stimare l'MAE.
- **Prezzo/accesso**: la LITE è CC BY-NC-SA con dominio 50×50 — inutilizzabile per lavoro reale e per uso commerciale. Il segmento "studio di progettazione piccolo, budget zero, dominio 200×200" è **scoperto**.

---

## 5. Panorama delle alternative

### 5.1 Confronto sintetico

| Strumento | Copertura | Licenza | Stato | Più forte di ENVI-met su | Più debole su |
|---|---|---|---|---|---|
| **PALM / PALM-4U** | LES + RANS, edifici, radiazione con riflessioni multiple, chimica+aerosol, indoor/energia, biometeorologia (PET/PT/UTCI), multi-agent | **GPL v3** `[P]` | Vivo: **v25.10.1**, v25.10 del 16 feb 2026 `[P]` | Scaling (32.000 core), LES vera, GPU (dynamic core, CUDA-aware MPI), NetCDF nativo, Linux/HPC, chimica avanzata, forcing da COSMO/ICON/WRF | Barriera d'ingresso enorme (Fortran, HPC), **GUI non pubblica** ("will not be available to the general public until further notice"), vegetazione 3D meno fisiologica |
| **UMEP / SOLWEIG** | Tmrt, UTCI, PET, SVF, solar access; plugin QGIS | **GPL-3.0** `[P]` | Molto vivo: push 2026-08-27, 98 star (UMEP) `[P]` | Facilità d'uso, dentro QGIS, gratis, veloce, coprono grandi aree, versione **GPU** in Rust in test | **Nessun CFD**: niente campo di vento, niente advezione, niente bilancio dell'aria. Non sostituisce ENVI-met, sostituisce solo la parte radiativa/comfort |
| **SUEWS** | Bilancio energetico e idrico urbano, scala di quartiere | **MPL-2.0** `[P]` | Molto vivo: push 2026-08-31, 31 star `[P]` | Bilanci a lungo termine, leggerezza | Non risolve la geometria 3D né il microclima intra-canyon |
| **Ladybug Tools / Dragonfly / Honeybee** | Analisi ambientale, energia edifici (EnergyPlus), UWG per UHI, morphing EPW | **AGPL-3.0** `[P]` | Molto vivo: push 2026-08-28, 226 star (ladybug) `[P]` | Ecosistema, integrazione Rhino/Grasshopper/Revit, community, energia edifici | Non è un microclima CFD; UWG **morfa** un EPW, non risolve il campo. Sensibilità limitata alla vegetazione, specie alberi `[S]` |
| **UWG (Urban Weather Generator)** | Effetto isola di calore su file EPW | AGPL (via ladybug-tools) `[P]` | Vivo dentro ladybug-tools | Velocissimo, ottimo per input a simulazioni energetiche | Zero risoluzione spaziale intra-urbana; limitata sensibilità agli alberi `[S]` |
| **OpenFOAM (+ setup urbano)** | CFD generico | GPL | Maturo | Flessibilità totale su geometria e turbolenza, mesh non strutturate | **Niente** vegetazione fisiologica, radiazione urbana, suolo, comfort out-of-the-box: va tutto costruito. Richiede training pesante `[S]` |
| **SOLENE-microclimat** | Termo-radiativo + CFD (Code_Saturne, EDF) + modello termico edificio; vegetazione, suolo, green wall/roof, umidificazione stradale | Non distribuito pubblicamente `[I]` | Ricerca (CRENAU, Nantes) | Accoppiamento con un CFD industriale open (Code_Saturne); prestazioni comparabili a ENVI-met secondo utenti `[S]` | Non è un prodotto: strumento di ricerca, poco accessibile |
| **CitySim** | Energia urbana + radiazione, scala quartiere | Ricerca (EPFL) | Poco attivo `[I]` | Energia a scala di quartiere | Flusso d'aria non risolto (accoppiamenti CFD sono ad hoc) `[S]` |
| **TEB / SURFEX** | Schema di canyon urbano, accoppiabile a Meso-NH | Open (Météo-France) | Mantenuto | Scala meso-urbana, uso operativo | **Non** risolve il singolo edificio o il singolo albero |
| **MITRAS** | Microscala ostacolo-risolvente | Ricerca (Amburgo) | Poco visibile `[I]` | Microscala, obstacle-resolving | Ecosistema e documentazione scarsi |
| **TUF-3D** | Temperature delle facce urbane in 3D, radiazione | Ricerca | Legacy `[I]` | Radiazione/temperature di superficie | Niente vegetazione, niente atmosfera piena |
| **VoxCity** | Integrazione geodata → 3D city model → simulazioni; esporta a ENVI-met | **MIT** `[P]` | Molto vivo: 540 star, push 2026-08-30 `[P]` | Ingestione dati aperti, generazione dominio | Non è un solutore microclimatico completo; è a monte |

Fonti puntuali: `[P]` https://palm.muk.uni-hannover.de/trac (GPL v3, 32.000 core, GPU, feature list), https://palm.muk.uni-hannover.de/trac/wiki/palm4u (moduli PALM-4U; nota: **ultima modifica 9 set 2019**, quindi la descrizione dei moduli è datata anche se il codice no), https://gitlab.palm-model.org/releases/palm_model_system/-/tags (versioni e date), GitHub API per stelle/licenze/push di UMEP, SUEWS, ladybug-tools, VoxCity (rilevati il 2026-08-31).

### 5.2 Che spazio resta libero

`[I]` — questa è la mia lettura, non un fatto.

Il panorama si divide nettamente in due:

- **Sopra**: PALM-4U copre e supera ENVI-met sulla fisica e sul calcolo, è GPL, gira su HPC Linux, scala. Il suo tallone è l'**usabilità**: Fortran, namelist, HPC, e una GUI che il consorzio dichiara esplicitamente non disponibile al pubblico. Un urbanista non lo userà mai direttamente.
- **Sotto**: UMEP/SOLWEIG e Ladybug sono usabili, gratuiti, vivi — ma **non risolvono il campo di flusso**. Danno MRT, ombre, comfort su geometria statica; non danno "cosa succede all'aria in questa piazza".

**Il buco è nel mezzo**: uno strumento che
1. risolva davvero l'accoppiamento aria-suolo-pianta-edificio (cioè faccia quello che fa ENVI-met, non quello che fa SOLWEIG),
2. sia open, Linux, containerizzabile, headless-first, con API Python e formati aperti,
3. sia usabile da un progettista senza background in fisica dell'atmosfera,
4. e giri su hardware ragionevole o su cloud effimero, con checkpoint/restart.

Due strategie plausibili, alternative:
- **A — Wrapper su PALM-4U**: non riscrivere la fisica, costruire il layer mancante (definizione dominio da geodata, orchestrazione, restart, post-processing, UI). Costo basso, rischio scientifico basso, ma si eredita il modello di calcolo di PALM (serve HPC per domini seri).
- **B — Solutore proprio più leggero**: replicare l'architettura ENVI-met (RANS + k-ε, radiazione tipo IVS, A-gs, Darcy, muri multi-nodo) su stack moderno. Costo altissimo — sono ~30 anni di fisica accumulata — e il rischio di validazione è il vero ostacolo, non il codice.

`[I]` La differenza fra le due non è tecnica ma di posizionamento: A compete sull'usabilità, B compete sulla fisica. Il panorama suggerisce che il gap sfruttabile sia sull'usabilità, non sulla fisica.

---

## 6. Fonti

**Primarie — wiki tecnica ufficiale ENVI-met** (`envi-met.info`)
- Model Architecture: https://envi-met.info/doku.php?id=intro:modelconcept
- Applications: https://envi-met.info/doku.php?id=apps:start
- ENVI-core / CLI: https://envi-met.info/doku.php?id=apps:envimet_core
- SPACES: https://envi-met.info/doku.php?id=apps:spaces
- Monde: https://envi-met.info/doku.php?id=apps:monde
- ENVI-guide: https://envi-met.info/doku.php?id=apps:enviguide
- BIO-met: https://envi-met.info/doku.php?id=apps:biomet
- Forcing Manager: https://envi-met.info/doku.php?id=apps:forcingmanager
- QGIS plugin: https://envi-met.info/doku.php?id=apps:gis4envi-met
- Grasshopper/Morpho: https://envi-met.info/doku.php?id=plugins:grasshopper
- IVS vs AVF: https://envi-met.info/doku.php?id=kb:ivs
- Turbolenza (marcata outdated): https://envi-met.info/doku.php?id=kb:turbulence
- Nesting: https://envi-met.info/doku.php?id=kb:nesting
- Model layout: https://envi-met.info/doku.php?id=kb:modellayout
- LAD: https://envi-met.info/doku.php?id=kb:lad
- Parallel computing: https://envi-met.info/doku.php?id=kb:parallel
- Running/compute (stale): https://envi-met.info/doku.php?id=kb:compute
- FAQ: https://envi-met.info/doku.php?id=kb:faq
- PLANTMODEL: https://envi-met.info/doku.php?id=plantmodel
- A-gs: https://envi-met.info/documents/new_a_gs.pdf
- TIMING: https://envi-met.info/doku.php?id=timing
- TIMESTEPS: https://envi-met.info/doku.php?id=timesteps
- EDX/EDT: https://envi-met.info/doku.php?id=filereference:edx_edi
- EML: https://envi-met.info/doku.php?id=filereference:fileformat
- Output files: https://envi-met.info/doku.php?id=filereference:output:start
- INX (pagina vuota): https://envi-met.info/doku.php?id=filereference:inx
- Downloads: https://envi-met.info/doku.php?id=files:start , https://envi-met.info/doku.php?id=files:download
- Changelog: https://envi-met.info/doku.php?id=apps:updates

**Primarie — sito commerciale**
- https://envi-met.com/pricing/
- https://envi-met.com/news/envi-met-acquired-by-one-click-lca/
- https://envi-met.com/microclimate-simulation-software/

**Primarie — alternative**
- PALM: https://palm.muk.uni-hannover.de/trac ; https://palm.muk.uni-hannover.de/trac/wiki/palm4u ; https://gitlab.palm-model.org/releases/palm_model_system/-/tags
- UMEP: https://github.com/UMEP-dev/UMEP ; https://umep-docs.readthedocs.io/en/latest/Introduction.html
- Ladybug Tools: https://github.com/ladybug-tools
- VoxCity: https://github.com/kunifujiwara/VoxCity ; https://arxiv.org/pdf/2504.13934
- geodata2ENVI-met: https://github.com/One-Click-LCA/geodata2ENVI-met

**Secondarie — letteratura**
- Bruse & Fleer 1998: https://www.sciencedirect.com/science/article/abs/pii/S1364815298000425
- Tsoka et al. 2018 (review, MAE/RMSE): https://www.sciencedirect.com/science/article/abs/pii/S2210670718307649
- Liu et al. 2021 (green/blue infrastructure in ENVI-met V4): https://www.sciencedirect.com/science/article/pii/S0360132321003437
- Simon et al. 2021 (IVS/radiative transfer): https://doi.org/10.3390/app11125449
- Sinsel/Simon (MRT schemes): https://www.sciencedirect.com/science/article/abs/pii/S2212095522001973
- Crank et al. (suitability, grid independence): https://www.sciencedirect.com/science/article/abs/pii/S2212095518301007
- Fischereit & Schlünzen 2018 (scelta indice comfort): https://link.springer.com/article/10.1007/s00484-018-1591-6
- Validazione 5.6 vs 5.9 (2026, **non letto, 403**): https://www.sciencedirect.com/science/article/pii/S2590162126000675

**Secondarie — pricing/mercato**
- https://www.capterra.com/p/10008080/ENVI-met/
- https://www.getapp.com/operations-management-software/a/envi-met/
- https://alternativeto.net/software/envi-met/about
- https://en.wikipedia.org/wiki/ENVI-met
- https://tech.eu/2024/09/17/one-click-lca-acquires-envi-met/

---

## 7. Caveat

1. **Prezzi**: nessun listino ufficiale. I €290/anno di Capterra e i $242/mese di AlternativeTo sono incompatibili fra loro. Non usare questi numeri per una business case senza un preventivo diretto.
2. **Requisiti hardware**: la fonte primaria (FAQ) è pre-2018. I numeri 16 core/64-128 GB sono `[S]`.
3. **Tempi di simulazione**: la stima "una settimana per 250×250×30 su 24 h" è primaria ma antecedente al multi-core e alle ottimizzazioni 5.8/5.9. Limite superiore storico.
4. **Documentazione ENVI-met parzialmente stale**: coesistono pagine 2025 e pagine pre-2018 senza distinzione visibile. Ogni dato preso dalla wiki va incrociato con il changelog.
5. **PALM-4U**: la pagina descrittiva dei moduli è ferma al **9 settembre 2019**, mentre il codice è al 25.10.1 (2026). Le capacità reali attuali sono probabilmente superiori a quanto descritto lì.
6. **Formato INX**: non documentato. Ogni piano di interoperabilità deve prevedere reverse engineering o riuso dei plugin GPL esistenti (con le conseguenze di licenza del caso).
7. **Non ho letto** il paper di validazione 5.6 vs 5.9 (403 ScienceDirect) né il PDF Bruse 2004 "ENVI-met 3.0 Updated Model Overview" (scaricato, ma nessun estrattore PDF disponibile nell'ambiente). Entrambi sono letture consigliate per completare il quadro della fisica e dell'accuratezza corrente.
8. **Nota metodologica**: questo file è un rapporto di ricerca, non contiene raccomandazioni operative approvate. La §5.2 (strategie A/B) è **una raccomandazione, non una decisione**.
