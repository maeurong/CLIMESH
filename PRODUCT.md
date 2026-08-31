# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Rust per tutto — nucleo di calcolo, riga di comando e server web — distribuito
come **binario singolo senza installazione**. La scelta è di Mario, presa contro
la raccomandazione iniziale di un guscio Python: il ragionamento è che l'utente
primario non ha Python né alcun gestore di pacchetti, e non installerà nessuno
dei due.

Il nucleo radiativo non si riscrive: si riusa quello di
[`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig), GPL-3, consumato come
dipendenza git **pinnata a un tag**, dopo una richiesta a monte che esponga
un'API Rust nativa. Il vendoring del sorgente è il ripiego; il prodotto non
dipende dall'esito di quella richiesta.

**Aperto:** il modo in cui è costruita l'interfaccia nel browser. Il binario
serve la pagina, ma se questa sia HTML e CSS scritti a mano o passi da uno
strumento di costruzione non è deciso.

## Users

**Utente primario: il tesista o dottorando di architettura o ingegneria edile.**
Ha un caso studio, un portatile, e una scadenza. Non ha accesso a un cluster di
calcolo. Non ha Python installato e non ha intenzione di installarlo. Conosce il
proprio sito meglio di chiunque altro e la fisica del microclima meno di quanto
gli servirebbe. Deve produrre una relazione in cui i risultati siano difendibili
davanti a un relatore.

Il suo vincolo di velocità non è una preferenza: **se una corsa dura più di una
pausa caffè smette di iterare sul progetto e comincia ad aspettare**, e a quel
punto lo strumento ha smesso di essere uno strumento di progetto.

**Utente secondario: lo studente dentro un'esercitazione.** Un laboratorio dura
tre ore e in quelle tre ore deve arrivare a un risultato. È il caso che fissa il
tetto di tempo più severo, e non ha mai visto il programma prima.

**Utenti successivi: chiunque, ed è l'obiettivo dichiarato.** L'adozione fuori
dall'ateneo di origine non è un effetto collaterale sperato ma il fine del
progetto — è il senso di "gratuito, usato nelle università". Ne discendono
obblighi di prodotto veri: documentazione d'ingresso, un caso d'esempio che non
sia quello dell'autore, stati vuoti che spiegano, messaggi d'errore che dicono
cosa fare, e la promessa di non rompere i formati.

> **Tensione registrata, non risolta.** Durante il tracciamento della mappa il
> progetto è stato inquadrato come *esperimento senza scadenza*. Il presente
> documento dichiara invece l'adozione da parte di terzi come obiettivo. Le due
> cose non si contraddicono — un esperimento può avere un fine dichiarato — ma
> gli obblighi verso utenti sconosciuti costano lavoro reale fin dal primo
> giorno, e quel costo va speso con gli occhi aperti.

**Pubblico di sola osservazione:** relatori e commissioni, che vedono i risultati
in una relazione senza mai aprire il programma.

## Product Purpose

CLIMESH calcola il **comfort termico all'aperto** in uno spazio urbano — dove
cade l'ombra, quanta radiazione riceve una persona, quanto caldo sente — e lo fa
in modo **riproducibile e citabile**.

Sostituisce ENVI-met come strumento di lavoro per il caso d'uso più comune in un
corso o in una tesi. ENVI-met è closed source, solo Windows, senza ripresa dopo
un'interruzione, senza API, e da febbraio 2025 richiede un account con
autenticazione centralizzata per essere aperto — dopo l'acquisizione dello
sviluppatore da parte di un'azienda terza nel settembre 2024. I prezzi non sono
pubblici.

**Il successo ha due metà.** La prima: uno strumento che un tesista installa in
un doppio clic e con cui ottiene un risultato difendibile nel tempo di una
lezione. La seconda: la dimostrazione che la grandezza giusta da guardare non è
quella che si guarda di solito.

## Positioning

Il panorama libero ha già tutti i pezzi e nessuno che li tenga insieme in modo
usabile. PALM-4U supera la fisica di ENVI-met ma dichiara di non avere
un'interfaccia pubblica e vuole un cluster. UMEP e SOLWEIG sono vivi, validati e
citati, ma vivono dentro QGIS e pretendono che l'utente arrivi con i propri
raster già pronti. CLIMESH sta nel mezzo: **la fisica di UMEP, presa così com'è,
dentro uno strumento che si apre.**

Ma la posizione difendibile non è l'usabilità, che chiunque può copiare. È
questa: **CLIMESH misura la grandezza che il modello risolve, e dichiara quanto
di ciò che mostra è stimato.**

Il caso studio di riferimento è la prova. La relazione da cui nasce il progetto
concludeva che l'intervento di mitigazione funzionava, sulla base di una
differenza di temperatura dell'aria di **0,21 °C** — mentre l'errore validato di
ENVI-met sulla stessa grandezza è di **1,34 °C** di errore medio assoluto. La
conclusione stava per intero dentro la barra d'errore del modello che l'aveva
prodotta. La differenza di temperatura media radiante fra sole e ombra nella
stessa corte vale **20-35 °C**, contro un errore del modello di 2,4-7,3 °C: da
tre a dieci volte il rumore.

Nessun prodotto vicino può copiare questa posizione senza ammettere che la
grandezza che mostra di solito non regge.

## Operating Context

Il flusso reale, ricostruito dal caso studio e dai materiali del corso.

**Prima.** L'utente ha un sito, un rilievo o un indirizzo, e un file meteo. Il
rilievo può essere un insieme di tavole; l'indirizzo può essere solo un punto su
una mappa. Le altezze degli edifici raramente esistono già: sui 168 edifici
mappati in OpenStreetMap attorno al caso studio, **zero** portano l'altezza e
tre il numero di piani.

**Durante.** Costruisce un Progetto, che contiene una Griglia, uno o più Scenari
— il luogo in un certo assetto — e uno o più Periodi — file meteo più intervallo
di date. Una Corsa è uno Scenario calcolato per un Periodo. Il caso di
riferimento sono due Scenari per due Periodi, cioè quattro Corse. Il vocabolario
completo è in [`CONTEXT.md`](CONTEXT.md).

**Dopo.** Porta via mappe raster georeferenziate, serie temporali nei punti che
ha scelto, figure pronte da mettere in una relazione, e il **Giornale della
Corsa**: quali parametri, da dove viene ogni cosa, e le metriche di verifica del
risultato.

**Il contesto d'uso è avverso in due modi che il prodotto deve assorbire.** Il
portatile non ha una scheda grafica dedicata utilizzabile, quindi il percorso su
sola CPU deve rispettare i tempi; e il dato pubblico è incompleto, quindi lo
strumento deve funzionare con quello che c'è invece di pretendere quello che
servirebbe.

## Capabilities and Constraints

**Ambito fisico.** Radiazione a onda corta e lunga, ombreggiamento, temperatura
media radiante, indici di comfort. Temperatura dell'aria, umidità e vento sono
**ingressi** letti dal file meteo, non risultati calcolati, e le uscite lo devono
dire esplicitamente.

L'indice primario è **UTCI**, che ha un solo polinomio autoritativo e quindi dà
lo stesso numero in strumenti diversi. PET è secondario perché ne esistono almeno
quattro varianti incompatibili. PMV è escluso.

**Vincolo di velocità, di primo livello.** Il caso di riferimento completo —
50 × 50 celle a 1 m, 48 ore, due Scenari per due Periodi — **sotto i 60 secondi
su sola CPU**. Tetto di progetto: 200 × 200 m a 2 m per 24 ore in meno di 10
minuti. Un'opzione che non rientra in questi numeri è fuori, indipendentemente da
ogni altro merito.

**Fuori scope, deciso.** Dispersione di inquinanti e particolato; modello indoor
e bilancio energetico degli edifici; simulazione a grandi vortici e fluidodinamica
risolta; import IFC e BIM; scrittura di file `.INX`; plugin QGIS; qualunque
componente solo per Windows.

**Vincoli noti da dichiarare, non da nascondere.**

- Il vento entra semplificato e uniforme. Un errore di 1 m/s sposta UTCI di circa
  1 °C d'estate e **3 °C d'inverno**: accettabile in estate, debolezza reale in
  inverno.
- La cucitura verso valori meteorologici calcolati regge per campi **prescritti**
  — misurati, interpolati, presi da un modello a scala maggiore — e **non regge
  per un solutore accoppiato**. Una temperatura dell'aria calcolata a partire
  dalle temperature superficiali sarebbe una retroazione, cioè un'altra
  architettura.
- La trasmissività fogliare è uno scalare per Corsa. La differenza fra specie si
  esprime nella Derivazione, escludendo le latifoglie dal raster delle chiome nel
  Periodo invernale, non variando la trasmissività.

**Licenza.** GPL-3 o AGPL-3. È una scelta di posizionamento oltre che etica: il
copyleft è la promessa credibile che a CLIMESH non accadrà mai ciò che è appena
accaduto a ENVI-met.

**Lingua dell'interfaccia: italiano e inglese, entrambe dal primo giorno.**
Nessuna stringa annidata nel codice. I termini tecnici del dominio — Tmrt, UTCI,
DSM, sky view factor — restano in inglese in entrambe le lingue, perché è così
che l'utente li conosce.

**Aperto.** Come si riempie la Provenienza quando l'altezza di un edificio manca
davvero, cioè quale valore predefinito e con quale avviso. Il protocollo di
confronto contro i risultati pubblicati del caso studio, dato che CLIMESH produce
comfort radiativo e la relazione riporta temperatura dell'aria.

## Brand Commitments

Il nome **CLIMESH** è fissato. La lingua di lavoro del progetto è l'italiano per
i documenti e l'inglese per codice, nomi di API e termini tecnici.

La voce è quella di uno strumento scientifico: dichiara cosa sa e cosa ha
stimato, non promette accuratezza che non ha, e non usa il linguaggio del
marketing. **Un'altezza stimata non è un errore** e non va segnalata come tale.

## Evidence on Hand

**Il caso di riferimento è reale e completo.** Casa Evolutiva, progetto di Renzo
Piano a Bastia Umbra (PG), 43.07 N 12.56 E. Dominio 50 × 50 × 25 celle a 1 m,
rotazione 21°, 616 istanze di piante di cinque specie, edifici a 6 m i duplex e
3 m il simplex. Modello estratto e valori pubblicati della relazione originale in
[`casi/bastia/`](casi/bastia/); struttura del formato di partenza in
[`docs/formato-inx.md`](docs/formato-inx.md).

**Ricerca svolta, sei rapporti in [`research/`](research/):** cosa è ENVI-met e
come funziona; la disponibilità di una sua versione gratuita; da dove arriva la
geometria urbana in Italia; l'ecosistema Rust; il riuso di SOLWEIG, con misure
di tempo prese sul caso reale; formulazioni di temperatura media radiante e
indici di comfort, con sensibilità misurate.

**Dati di validazione disponibili ma non nostri.** Il repository di
`UMEP-dev/solweig` contiene 7 giorni e 85 ore di misure radiative reali a tre
siti di Göteborg, con tutti i dati di ingresso, e una verifica che gira in
integrazione continua. Sono riusabili, ma **non vanno copiati**: i file di misura
non portano una dichiarazione di licenza propria, quindi si recuperano dal loro
repository a un tag pinnato.

**Assenze che il lavoro futuro non deve colmare inventando.**

- **Non esistono misure di campo sul caso italiano.** Il forcing viene da un anno
  tipo costruito su rilievi 1951-1970, non dal 2021 reale. Nessun confronto con
  "la realtà di quel giorno" è possibile.
- **Lo Scenario degli interventi non esiste.** Il file del caso contiene solo lo
  stato di fatto, verificato con tre indizi concordi. Mancano posizioni e specie
  della vegetazione aggiunta e l'estensione dello specchio d'acqua.
- **Le altezze delle chiome non sono nel file di partenza**: stanno nel database
  piante di ENVI-met, che non abbiamo.
- Non esistono utenti reali oltre l'autore, né riscontri d'uso, né confronti con
  altri strumenti eseguiti da noi.

**Materiale del corso escluso dal repository.** Rilievi, tavole e relazione
originale restano fuori: non sono ridistribuibili. I numeri che ne derivano sì.

## Product Principles

1. **Misura ciò che il modello risolve.** Una grandezza il cui segnale sta dentro
   la barra d'errore non va mostrata come se fosse un risultato. È il principio
   da cui nasce il progetto, e vale contro CLIMESH stesso quanto contro ENVI-met.

2. **La provenienza viaggia con l'oggetto, non con la corsa.** Ogni Edificio,
   ogni Albero, ogni Superficie sa da dove viene e quali dei suoi attributi sono
   rilevati anziché stimati. Sopravvive a mille Corse, perché la domanda che
   l'utente si sentirà fare è "quell'edificio lì, l'altezza da dove viene".

3. **Nessun risultato senza il suo giornale.** Un numero che non sa dire come è
   stato ottenuto non è citabile, e la citabilità è il prodotto. Senza il
   Giornale, CLIMESH è soltanto un ENVI-met gratuito.

4. **Funziona con il dato che c'è.** Il dato pubblico è incompleto e lo resterà.
   Uno strumento che pretende ingressi perfetti non viene usato; uno che accetta
   ingressi imperfetti **e lo dichiara** viene usato e resta onesto.

5. **La velocità è una proprietà di prodotto, non un'ottimizzazione.** Sotto il
   minuto l'utente itera sul progetto; sopra, aspetta. Il numero viene prima
   dell'architettura, e un'opzione che non lo rispetta è fuori a prescindere.

## Accessibility & Inclusion

**Obiettivo dichiarato: WCAG 2.2 livello AA.** L'adozione da parte di enti
pubblici è l'obiettivo del progetto, e un ateneo italiano ricade sotto la
normativa AgID e la EN 301 549 per i servizi che pubblica.

La conseguenza pesante è specifica di questo prodotto: **una mappa termica è
un'immagine di dati.** Renderla accessibile davvero significa che ogni campo
calcolato ha una vista tabellare alternativa dei propri valori, ogni figura una
descrizione testuale del proprio contenuto, e che **nessuna informazione è
affidata al solo colore** — il confronto fra Scenari si legge anche con
etichette dirette, non solo dalla tinta della linea.

Il sistema di design descritto in [`DESIGN.md`](DESIGN.md) aiuta invece di
ostacolare: l'interfaccia è monocroma, il confronto fra Scenari è già codificato
come base contro segnale con etichetta diretta, e la scala della rampa termica
porta sempre i propri valori numerici accanto.
