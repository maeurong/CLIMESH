# Spec — CLIMESH

Questo documento dice **cosa si costruisce e in che forma**. Non ripete il
perché: quello sta altrove e va letto prima.

| Per | Leggi |
|---|---|
| a chi serve, cosa deve fare, cosa è fuori | [`PRODUCT.md`](../PRODUCT.md) |
| come si chiamano le cose | [`CONTEXT.md`](../CONTEXT.md) |
| che faccia ha, e la regola della stampa | [`DESIGN.md`](../DESIGN.md) |
| oggetti contro raster, e perché | [`adr/0001-oggetti-e-raster.md`](adr/0001-oggetti-e-raster.md) |
| ogni decisione, con la sua motivazione | mappa wayfinder, issue **#1** |

Le decisioni qui sotto sono già state prese e istruite. Questo file le mette in
una forma che si può implementare.

---

## 1. Cosa si costruisce

Un **binario Rust singolo** che, dato un modello urbano e un file meteo, calcola
ombreggiamento, radiazione, temperatura media radiante e indici di comfort su una
griglia, e ne produce mappe, serie temporali e un Giornale.

Due superfici sullo stesso nucleo: una **riga di comando** headless e una
**pagina servita nel browser**. La riga di comando è la base, non un ripiego: gli
studi parametrici e l'integrazione continua passano da lì.

Il calcolo radiativo **non si scrive**: si riusa il nucleo Rust di
[`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig), GPL-3, come dipendenza
git pinnata a un tag.

## 2. Ambito

**Dentro.** Radiazione a onda corta e lunga, ombreggiamento, sky view factor,
temperatura media radiante, UTCI come indice primario e PET come secondario.
Import da OpenStreetMap più modelli di elevazione, da shapefile e GeoPackage, e
lettura sola dei file `.INX` di ENVI-met. Mappe raster georeferenziate, serie
temporali nei Punti di osservazione, figure, Giornale.

**Fuori, deciso.** Inquinanti e particolato; modello indoor ed energetico degli
edifici; simulazione a grandi vortici e fluidodinamica risolta; import IFC e BIM;
scrittura di `.INX`; plugin QGIS; qualunque componente solo per Windows.

**Temperatura dell'aria, umidità e vento sono ingressi**, letti dal file meteo e
uniformi sulla Griglia. Le uscite che ne dipendono lo devono dichiarare.

## 3. I pezzi e i loro confini

Sei responsabilità distinte. Non sei moduli con interfacce astratte: sei cose che
sanno fare una cosa, di cui almeno tre stanno bene in un file solo finché non
crescono.

```
ingresso            dominio              calcolo              uscita
─────────           ────────             ────────             ────────
lettore .INX  ─┐                     ┌─ Derivazione ─┐
import OSM    ─┼─→  Progetto  ───────┤               ├─→  Motore  ─→  campi
import GIS    ─┘   (oggetti)         └─ cache SVF ───┘                  │
                        │                                               ▼
                        └──────────────→  Giornale  ←───────────────  verifiche
                                              │
                                              ▼
                                   pagina · CLI · foglio di stampa
```

**Lettori di ingresso** — producono oggetti, mai raster. Il lettore `.INX`
esiste già ([`src/inx.rs`](../src/inx.rs), formato in
[`formato-inx.md`](formato-inx.md)).

**Il Progetto** — cartella su disco, manifesto di testo, oggetti. È la verità.

**La Derivazione** — trasforma oggetti in raster co-registrati. Ha parametri e
scelte di modellazione, e per questo finisce nel Giornale.

**Il Motore** — il nucleo riusato. Consuma raster e meteo, produce campi.

**Le verifiche** — leggono i campi e producono numeri per il Giornale.

**Le superfici** — CLI, pagina, foglio di stampa. Non contengono logica di
dominio: rendono ciò che le altre cinque producono.

## 4. Il Progetto su disco

Una cartella. Il manifesto è la verità; tutto il resto è rigenerabile o è un
risultato.

```
progetto/
├── progetto.toml           il manifesto: Griglia, Scenari, Periodi, Punti
├── scenari/
│   ├── stato-di-fatto.toml oggetti: Edifici, Alberi, Superfici, Provenienza
│   └── interventi.toml
├── periodi/
│   └── luglio-2021.toml    riferimento all'EPW, intervallo, forcing
├── derivati/               cache: raster e SVF. Cancellabile senza perdite.
└── corse/
    └── <impronta>/         campi, figure, e giornale.toml
```

**La Griglia sta sul Progetto**, non sullo Scenario: due Scenari con griglie
diverse non sarebbero confrontabili, e confrontarli è l'unica cosa che l'utente
vuole fare.

**Ogni Scenario è autonomo.** Contiene la lista completa dei propri oggetti; il
manifesto annota da quale altro è stato creato, e l'annotazione resta
un'annotazione. Nessuna ereditarietà viva: una modifica al padre che cambia in
silenzio i figli farebbe cambiare un risultato già pubblicato.

**Ogni oggetto porta la propria Provenienza**: da dove viene e quali attributi
sono rilevati anziché stimati.

## 5. La Derivazione

Da oggetti a raster co-registrati sulla Griglia, tutti con stessa estensione e
stesso passo, che è ciò che il Motore pretende.

| Raster | Da |
|---|---|
| modello di superficie | terreno più Edifici |
| modello di terreno | terreno |
| chiome | Alberi, altezza per specie |
| zona-tronco | frazione dichiarata dell'altezza di chioma |
| classi di superficie | Superfici, elenco chiuso di tipi |

**Altezze mancanti: catena di ripiego dichiarata.** Differenza fra modello di
superficie e modello di terreno dove un DSM copre il sito; altrimenti numero di
piani per altezza di interpiano; altrimenti valore predefinito dichiarato. La
modifica manuale vince su ogni anello. **Ogni Edificio registra l'anello da cui
viene la sua altezza**, e il Giornale ne riporta il conteggio.

**Il Periodo invernale esclude le latifoglie dal raster delle chiome.** Il Motore
accetta una sola trasmissività fogliare per Corsa, ma la specie vive
sull'oggetto Albero e la Derivazione lavora per pixel. È la ragione concreta per
cui la regola dell'ADR 0001 esiste, e il Giornale registra quante piante sono
state escluse.

**Lo sky view factor dipende solo dalla geometria**, non dall'ora: si calcola una
volta per Corsa e vale per tutte le sue ore. Questa spec prevedeva di metterlo in
cache per Scenario, e diceva che era la differenza fra rispettare il budget dei
60 secondi e sforarlo. **Misurato, non lo è**: 32-63 ms per Corsa sul caso di
riferimento, contro i 29-36 s che avevano motivato la cache e che venivano dal
percorso Python con il pool rayon predefinito. La cache non è stata costruita, e
non si costruisce finché un dominio più grande non la chiede — vedi
[ADR 0003](adr/0003-catena-radiativa-e-cucitura-a-monte.md).

## 6. Il Motore

Dipendenza git **pinnata a un tag**, dopo una richiesta a monte che aggiunga
`rlib` ai tipi di crate ed esponga un'API Rust nativa. Il vendoring del sorgente
è il ripiego; **questa spec non dipende dall'esito della richiesta**.

Si prende la catena completa fino a temperatura media radiante e indici
compresi. Tagliare più in basso farebbe riscrivere i pezzi dove il rischio è alto
e il guadagno nullo, e farebbe perdere i test di parità contro l'implementazione
di riferimento, che sono la ragione principale per cui quel codice merita
fiducia.

**Il percorso GPU è un'accelerazione, mai un requisito.** L'utente di
riferimento è su un portatile.

## 7. Il Giornale

Un file TOML nella cartella della Corsa. La vista nella pagina e l'appendice
stampata sono **rese** di quel file, mai artefatti paralleli.

**Registra** gli ingressi con le loro somme di controllo; la versione del binario
e quella pinnata del Motore; la Griglia; Scenario e Periodo; le scelte della
Derivazione; il fornitore dei valori meteorologici, marcato come ingresso.

**Riporta le verifiche per Corsa**, che costano poco:

- ombra confrontata con la posizione solare calcolata analiticamente;
- conteggio della Provenienza per anello della catena;
- inviluppo di ogni campo — minimo, massimo, media — con una bandiera se esce da
  un intervallo fisicamente plausibile;
- frazione di celle senza dato.

**Cita le verifiche per rilascio**, che costano tanto: parità contro
l'implementazione di riferimento e confronto con le misure di Göteborg non si
rieseguono a ogni Corsa. Il Giornale dice *questo binario è la versione X, la cui
validazione riporta Y*. È la ragione tecnica per cui il Motore è pinnato: se la
versione cambiasse sotto i piedi, la citazione non varrebbe niente.

**Si apre all'inizio e si scrive man mano**, così una Corsa fallita lascia un
Giornale che dice fin dove è arrivata. **Nessun checkpoint del calcolo**: a 60
secondi il rimedio è premere di nuovo. Se il budget saltasse, questa decisione va
riaperta insieme a quello.

**Contratto di riproducibilità a due livelli.** Gli esiti **discreti** — quali
celle sono in ombra, i conteggi, l'ordine di qualunque cosa — sono identici su
qualunque macchina, e un esito discreto che dipende dalla piattaforma è un
difetto da correggere. Le grandezze **continue** rientrano entro una tolleranza
dichiarata. La promessa dell'identità bit a bit sarebbe falsa: il Motore
documenta già una deriva fra percorso GPU e CPU fino a 0,5 °C ai bordi delle
chiome.

**Ogni Corsa ha due nomi**: un'impronta calcolata dal contenuto — somme di
controllo, versioni, parametri — per cui due Corse con la stessa impronta *sono*
la stessa Corsa; e un'etichetta scelta dall'utente, perché nessuno chiama una
corsa `a3f9c1`. Il Giornale porta una riga di citazione già scritta.

## 8. Le superfici

**Riga di comando.** Costruire un Progetto da un `.INX` o da un'area; eseguire
una o più Corse; interrogare un Giornale. Headless, adatta a uno studio
parametrico e all'integrazione continua.

**Pagina nel browser**, servita dal binario. Una vista sola: le mappe degli
Scenari affiancate come celle dello stesso oggetto rigato, i Punti di
osservazione sovrapposti alle mappe, le serie temporali come figure, il Giornale
come indice. Comandi per Periodo, grandezza e ora.

**Foglio di stampa.** Non una seconda interfaccia: un `@media print` sulla stessa
pagina. Regole in [`DESIGN.md`](../DESIGN.md) sotto *La stampa*.

**Lingua**: italiano e inglese dal primo giorno, nessuna stringa annidata nel
codice. I termini tecnici del dominio restano inglesi in entrambe.

**Accessibilità**: obiettivo WCAG 2.2 AA. Una mappa termica è un'immagine di
dati, quindi ogni campo calcolato deve avere una vista tabellare alternativa dei
propri valori, e nessuna informazione può stare sul solo colore.

## 9. Criteri di accettazione

Numeri, non intenzioni. Un'opzione che non li rispetta è fuori.

| Criterio | Soglia |
|---|---|
| Caso di riferimento completo, sola CPU | **< 60 s** |
| Tetto di progetto: 200 × 200 m a 2 m, 24 h | < 10 min |
| Verifica dell'ombra contro posizione solare analitica | esatta sulle celle binarie |
| Parità contro l'implementazione di riferimento | entro tolleranza **misurata**, non assunta zero, fissata su versione **e** percorso di calcolo |
| Confronto con le misure di Göteborg | non peggiore dei numeri pubblicati dal Motore, riportato e non gated |
| Installazione | scaricare un file ed eseguirlo |

Il caso di riferimento completo sono due Scenari per due Periodi, cioè quattro
Corse, su 50 × 50 celle a 1 m per 48 ore. Materiale in
[`casi/bastia/`](../casi/bastia/).

## 10. Ordine di costruzione

Ogni passo produce qualcosa di verificabile, e nessuno richiede il successivo.

1. **Progetto su disco** — manifesto, Scenari, Periodi; scrittura e rilettura.
   Il lettore `.INX` diventa un costruttore di Progetto.
2. **Derivazione** — oggetti in raster, con la catena delle altezze e
   l'esclusione stagionale. Verificabile senza Motore: i raster si ispezionano.
3. **Motore collegato** — la richiesta a monte, oppure il vendoring. Prima Corsa
   che produce campi sul caso di riferimento. **Qui si misurano i 60 secondi**, e
   se non tornano l'architettura va rivista prima di costruirci sopra.
4. **Giornale** — verifiche per Corsa, impronta, contratto di riproducibilità.
5. **Riga di comando** — costruire, eseguire, interrogare.
6. **Pagina e foglio di stampa** — mappe, figure, Giornale, stampa.
7. **Validazione** — parità in integrazione continua, poi il confronto di
   Göteborg recuperato dal repository del Motore a un tag pinnato, **mai copiato**
   nel nostro.
8. **Import da OpenStreetMap** — l'ultimo, perché è quello che dipende di più da
   fatti esterni e meno dagli altri passi.

Il passo 3 è il vero cancello. Tutto ciò che lo precede è verificabile da solo;
tutto ciò che lo segue presuppone che il budget regga.

## 11. Questioni aperte

Registrate, non risolte. Nessuna blocca l'inizio.

- **Come è costruita la pagina** — HTML e CSS scritti a mano o uno strumento di
  costruzione. Il binario serve la pagina in ogni caso.
- **Il valore predefinito dell'altezza** quando ogni anello della catena fallisce,
  e con quale avviso.
- **Il protocollo di confronto** con i risultati pubblicati del caso di
  riferimento, dato che CLIMESH produce comfort radiativo e la relazione riporta
  temperatura dell'aria.
- **Packaging e distribuzione**: piattaforme, rilasci, versionamento.
- **La tolleranza numerica** del contratto di riproducibilità, che va misurata e
  non decisa a tavolino.
