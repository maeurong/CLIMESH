# CLIMESH

Simulazione del microclima urbano per il comfort termico all'aperto: dato un luogo e un file meteo, CLIMESH calcola dove cade l'ombra, quanta radiazione riceve una persona e quanto caldo sente, e lo fa in modo che il risultato sia rifacibile e citabile.

## Language

### Il progetto e le sue parti

**Progetto**:
Una cartella che contiene tutto ciò che serve a rispondere a una domanda progettuale su un luogo. Ne è la verità il manifesto di testo; tutto il resto è rigenerabile.
_Avoid_: caso studio, modello, workspace

**Griglia**:
Estensione, passo e sistema di coordinate condivisi da tutti i raster del Progetto. Vive sul Progetto e non sullo Scenario, perché due Scenari con griglie diverse non sarebbero confrontabili.
_Avoid_: dominio, mesh, area di calcolo

**Scenario**:
Il luogo in un certo assetto: geometria, vegetazione e materiali di superficie, cioè tutto ciò che non cambia col tempo. Ogni Scenario è autonomo e contiene la lista completa dei propri oggetti; il manifesto annota da quale altro Scenario è stato creato, ma l'annotazione non è un legame vivo.
_Avoid_: variante, alternativa, stato

**Periodo**:
Il file meteo più l'intervallo di date e i parametri di forcing. È un oggetto a sé perché è riusabile fra Scenari: nominarlo una volta sola è ciò che impedisce a più Corse di divergere per una svista.
_Avoid_: clima, meteo, condizioni

**Corsa**:
Uno Scenario calcolato per un Periodo. È l'unità che produce risultati e a cui è associato un Giornale.
_Avoid_: run, simulazione, esecuzione

**Giornale**:
Il registro di una Corsa: i parametri usati, la provenienza di ciò che è entrato, le metriche di verifica del risultato. È ciò che rende il risultato citabile.
_Avoid_: log, report, metadati

### Cosa c'è dentro uno Scenario

**Edificio**:
Un volume costruito, con impronta, altezza e provenienza dei propri attributi.
_Avoid_: building, volume, massa

**Albero**:
Un'istanza di vegetazione: posizione, specie, altezza, chioma e zona-tronco. La specie è un attributo dell'Albero anche dove il calcolo non la usa direttamente, perché guida la derivazione.
_Avoid_: pianta, vegetazione, canopy

**Superficie**:
Una porzione di suolo con un tipo preso da un elenco chiuso, per esempio pavimentato, erba, acqua. L'elenco è chiuso perché il motore accetta solo classi note, e un elenco chiuso permette di dirlo prima del calcolo invece che dopo.
_Avoid_: materiale, land cover, copertura

**Punto di osservazione**:
Un punto del Progetto in cui si vogliono le serie temporali dei risultati, invece della sola mappa.
_Avoid_: POI, sonda, ricettore

**Provenienza**:
Ciò che un oggetto sa della propria origine: da dove viene e quali dei suoi attributi sono rilevati anziché stimati. Sta sull'oggetto, non sulla Corsa, perché sopravvive a mille Corse.
_Avoid_: metadati, fonte, origine

### Il calcolo

**Derivazione**:
Il passo che trasforma gli oggetti di uno Scenario nei raster che il motore consuma. Ha i suoi parametri e le sue scelte di modellazione, e per questo finisce nel Giornale.
_Avoid_: rasterizzazione, preprocessing, conversione

**Strato di chioma**:
Un raster di chiome, con la trasmissività che le accomuna: quanta radiazione diretta lasciano passare. La Derivazione ne produce uno per stagione con foglie e due nella stagione senza — i sempreverdi, ancora opachi, e le chiome spoglie — perché il Motore risponde se una cella sta all'ombra di *una* chioma, mai di *quale*: due trasmissività chiedono due strati.
_Avoid_: layer, CDSM, canopy raster

**Motore**:
Il nucleo di calcolo radiativo riusato da `UMEP-dev/solweig`, che dai raster e dal meteo produce ombre, radiazione, temperatura media radiante e indici di comfort.
_Avoid_: solver, kernel, backend
