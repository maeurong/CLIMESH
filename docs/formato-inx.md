# Il formato `.INX` di ENVI-met, per quanto serve a leggerlo

CLIMESH legge i file `.INX` prodotti da ENVI-met, e non li scrive: rientrare in ENVI-met non è
un caso d'uso. Del formato non esiste documentazione pubblica. Quello che segue è ricavato
leggendo un solo file reale, `LAB1.INX` del caso di riferimento — 210 042 byte, 5405 righe,
scritto il 15/03/2023 da SPACES 5.1.1, versione di formato 440 — più il codice del lettore che
ne è nato. Ogni affermazione è marcata **verificato** quando l'ho letta nel file, e **inferito**
quando è una congettura che il file non basta a confermare.

## Che tipo di file è

Un file di testo che assomiglia a XML: un albero di tag annidati, con l'intestazione in cima e
le sezioni una dopo l'altra.

- **Verificato**: nessuna dichiarazione XML in testa, nessun namespace, nessuna entità. Il file
  del caso è UTF-8 valido, senza BOM, con terminatori di riga CRLF.
- **Verificato**: *non è XML valido*. Le 616 istanze di vegetazione stanno in elementi
  `<3Dplants>`, e un nome XML non può cominciare con una cifra. Un parser XML conforme rifiuta
  l'intero file, non solo quel tag.
- **Verificato**: i valori scalari sono scritti con uno spazio attorno, `<grids-I> 50 </grids-I>`,
  e le righe delle matrici sono rientrate. Ogni lettura va ripulita degli spazi.

La conseguenza pratica per CLIMESH: il lettore non costruisce un albero e non valida la sintassi.
Cerca i tag per nome sul testo intero, perché ogni tag che serve compare una volta sola nel file
(le eccezioni sono i campi interni ai blocchi `<3Dplants>`, che si cercano dentro il blocco). Un
lettore ad albero avrebbe richiesto un parser tollerante, e in cambio non avrebbe dato niente:
la gerarchia del file non porta informazione che i nomi dei tag non portino già.

## L'intestazione

```
<ENVI-MET_Datafile>
<Header>
  <filetype>INPX ENVI-met Area Input File</filetype>
  <version>440</version>
  <revisiondate>15/03/2023 17:32:25</revisiondate>
  <remark>Created with SPACES 5.1.1</remark>
  <checksum>0</checksum>
  <encryptionlevel>0</encryptionlevel>
</Header>
```

- **Verificato**: nell'intestazione, e solo lì, il contenuto dei tag è scritto senza spazi attorno.
- **Inferito**: `encryptionlevel` diverso da zero indica un file cifrato o offuscato, il cui
  contenuto non sarebbe leggibile come testo. Non ho un file simile per verificarlo. Il lettore
  di CLIMESH rifiuta con un errore esplicito qualunque valore diverso da `0`, perché l'alternativa
  — proseguire e leggere byte cifrati come se fossero numeri — è peggiore di fermarsi.
- **Inferito**: `checksum` a zero significa "nessuna somma di controllo calcolata". Il lettore lo
  ignora.
- **Inferito**: `version` è la versione del formato, non del programma che l'ha scritto; il nome
  del programma sta in `remark`. Un lettore che ignorasse questo numero leggerebbe in silenzio un
  file di una versione futura, quindi CLIMESH lo riporta al chiamante anche se oggi non lo usa.

## Le sezioni

Nell'ordine in cui compaiono in `LAB1.INX` (**verificato**):

| Sezione | Contenuto |
|---|---|
| `<baseData>` | descrizione, autore e nota di copyright del modello, tutti al valore predefinito |
| `<modelGeometry>` | dimensioni della griglia e passo |
| `<nestingArea>` | numero di griglie di annidamento e due profili di suolo per il bordo |
| `<locationData>` | posizione geografica, rotazione, fuso orario |
| `<defaultSettings>` | materiale predefinito di parete e tetto |
| `<buildings2D>` | quattro matrici: `zTop`, `zBottom`, `buildingNr`, `fixedheight` |
| `<simpleplants2D>` | una matrice: `ID_plants1D` |
| `<3Dplants>` × 616 | un elemento per ogni istanza di vegetazione |
| `<soils2D>` | una matrice: `ID_soilprofile` |
| `<dem>` | `DEMReference` più la matrice `terrainheight` |
| `<sources2D>` | una matrice: `ID_sources` |

Gli elementi `<3Dplants>` sono fratelli fra loro e figli diretti della radice: **non** stanno
dentro un contenitore che li raccolga. Sono interposti fra `<simpleplants2D>` e `<soils2D>`.

La geometria e la posizione sono le uniche parti che CLIMESH considera obbligatorie:

```
<modelGeometry>          <locationData>
  <grids-I> 50 </grids-I>       <modelRotation> 21.00000 </modelRotation>
  <grids-J> 50 </grids-J>       <location_Longitude> 12.56000 </location_Longitude>
  <grids-Z> 25 </grids-Z>       <location_Latitude> 43.07000 </location_Latitude>
  <dx> 1.00000 </dx>            <locationName> bergamo </locationName>
  <dy> 1.00000 </dy>            <locationTimeZone_Name> CET/ UTC+1 </locationTimeZone_Name>
  <dz-base> 1.00000 </dz-base>  <locationTimeZone_Longitude> 15.00000 </locationTimeZone_Longitude>
```

- **Verificato**: `grids-I`, `grids-J` e `grids-Z` sono numeri di celle; `dx`, `dy` e `dz-base`
  sono il passo in metri. Nessuna dimensione va cablata nel lettore.
- **Inferito**: `modelRotation` sono gradi in senso orario da nord, come la rotazione di un
  disegno importato. Il file non lo dice e il caso ha un solo valore, 21°.
- **Inferito**: quando `useTelescoping_grid` o `verticalStretch` sono diversi da zero, il passo
  verticale non è più uniforme e `dz-base` da solo non basta a ricostruire le quote. Nel caso di
  riferimento valgono zero e la questione non si pone; un lettore che vada oltre la lettura 2D
  dovrà affrontarla.

## I blocchi `matrix-data`

Sono sette, tutti con la stessa forma:

```
<zTop type="matrix-data" dataI="50" dataJ="50">
0,0,0,0,0,0, ... 0,0,0
... 50 righe ...
</zTop>
```

- **Verificato**: il contenuto è una riga di testo per riga della matrice, e in ogni riga i valori
  sono separati da virgole, senza virgolette e senza spazi obbligatori.
- **Verificato**: le celle possono essere **vuote**. In `ID_plants1D` sono 898 su 2500, e in
  `ID_sources` lo sono tutte e 2500: la riga è `,,,,` ripetuto. Una cella vuota vuol dire "qui non
  c'è niente", che non è lo stesso di zero e non è lo stesso di una sezione assente. Il lettore di
  CLIMESH tiene distinte le tre cose.
- **Inferito**: `dataI` è il numero di valori per riga e `dataJ` il numero di righe. Nel caso di
  riferimento la griglia è quadrata, 50 × 50, quindi il file non permette di distinguerlo; è una
  deduzione dai nomi e dalla corrispondenza con `grids-I` e `grids-J`. Un `.INX` non quadrato
  chiuderebbe la questione, e il lettore di CLIMESH è scritto per accorgersene: se il conto delle
  righe o dei valori non torna, si ferma dicendo cosa aspettava e cosa ha trovato.

### L'orientamento delle righe, che è la parte che si sbaglia

**Verificato**, ed è la scoperta che è costata di più: *la prima riga scritta è quella più a nord*.
Cioè la prima riga del blocco è `j = grids-J` e l'ultima è `j = 1`, mentre dentro la riga l'indice
`i` cresce da sinistra a destra, da 1 a `grids-I`.

La verifica non è un'opinione. Nel file, le celle vuote di `ID_plants1D` sono l'unione delle celle
occupate da edifici (280, dove l'erba non viene messa) e delle celle dove è radicato un albero
(618). Confrontando queste ultime con le coordinate `rootcell_i` e `rootcell_j` delle 616 istanze
`<3Dplants>`, delle sei corrispondenze possibili fra indici di riga o colonna e indici `i`, `j`,
una sola regge:

| ipotesi | istanze che cadono su una cella vuota |
|---|---|
| `i` = colonna+1, `j` = riga+1 | 178 / 616 |
| **`i` = colonna+1, `j` = grids-J − riga** | **614 / 616** |
| `i` = riga+1, `j` = colonna+1 | 172 / 616 |
| `i` = grids-J − riga, `j` = colonna+1 | 180 / 616 |
| `i` = riga+1, `j` = grids-J − colonna | 207 / 616 |
| `i` = grids-J − riga, `j` = grids-J − colonna | 176 / 616 |

Le due istanze che restano fuori sono posizioni dove l'erba non è stata tolta: rumore del disegno,
non del formato. Un lettore che sbagli questo verso produce un modello specchiato rispetto al nord,
e siccome un modello specchiato è ancora plausibile a vederlo, l'errore sopravvive a lungo.

`Matrix::at(i, j)` del lettore prende indici ENVI-met e fa il ribaltamento; `rows_as_written()`
restituisce le righe nell'ordine del file, per chi deve riscriverle o confrontarle.

### Che cosa contiene ciascuna matrice

| tag | sezione | tipo | valori in `LAB1.INX` |
|---|---|---|---|
| `zTop` | `buildings2D` | numerico | 0, 3, 6 — quota superiore del volume costruito, in metri |
| `zBottom` | `buildings2D` | numerico | 0 ovunque — quota inferiore, per edifici sospesi |
| `buildingNr` | `buildings2D` | numerico | 0, 3, 5, 6, 7, 8, 9, 10 — identificativo dell'edificio |
| `fixedheight` | `buildings2D` | numerico | 0 ovunque |
| `ID_plants1D` | `simpleplants2D` | identificativi | `0100XX` o vuoto — vegetazione bassa |
| `ID_soilprofile` | `soils2D` | identificativi | `000000` ovunque |
| `terrainheight` | `dem` | numerico | 0 ovunque — quota del terreno sopra `DEMReference` |
| `ID_sources` | `sources2D` | identificativi | tutte vuote — sorgenti di emissione |

- **Verificato**: gli identificativi sono stringhe di sei caratteri alfanumerici, non numeri.
  `0100XX` perde il significato se lo si legge come intero. Il lettore li tiene come testo.
- **Inferito**: `fixedheight` distingue gli edifici la cui altezza è fissata da quelli che seguono
  il terreno. Vale zero ovunque nel caso, quindi non è verificabile, e CLIMESH non lo legge.
- **Verificato**, e vale come avvertenza: `buildingNr` **non** identifica un edificio per come lo
  intenderebbe un architetto. Nel caso di riferimento i tre blocchi costruiti sono spezzati in
  sette numeri — il blocco a 6 m di destra porta i numeri 5 e 6, quello a 3 m i numeri 7, 8, 9 e
  10 — con i numeri accessori che coprono una riga o una singola cella al bordo. Chi ne ricava gli
  Edifici di CLIMESH deve raggruppare per contiguità, non per numero.

## Le istanze di vegetazione

```
<3Dplants>
  <rootcell_i> 1 </rootcell_i>
  <rootcell_j> 45 </rootcell_j>
  <rootcell_k> 0 </rootcell_k>
  <plantID> 020027 </plantID>
  <name> .Pine Tree (middle) </name>
  <observe> 0 </observe>
</3Dplants>
```

- **Verificato**: `rootcell_i` e `rootcell_j` sono indici di cella **1-based**: nel caso stanno fra
  1 e 50 su una griglia di 50 celle. `rootcell_k` vale 0 in tutte e 616 le istanze.
- **Inferito**, e va detto perché è una trappola: `rootcell_k` sembra essere **0-based**, cioè lo 0
  è il livello del suolo, mentre `i` e `j` partono da 1. Le due convenzioni convivono nello stesso
  elemento. Il file, con tutti gli alberi a terra, non permette di verificarlo; il lettore accetta
  `k` fra 0 e `grids-Z` − 1 e rifiuta il resto nominando il valore e il limite.
- **Inferito**: `plantID` è la chiave di una voce nella banca dati delle piante di ENVI-met, che
  sta fuori dal file. Senza quella banca dati, altezza, chioma e trasmissività fogliare della
  specie non sono ricavabili dal `.INX`. Per CLIMESH è la parte di Provenienza che il formato non
  porta con sé.
- **Inferito**: `observe` a 1 marca l'istanza come osservata nei risultati. Vale 0 in tutte le 616.
- **Verificato**: `name` è il nome di catalogo della specie, con un punto iniziale e spazi attorno.

## Quello che questo lettore non fa

Non legge `<baseData>`, `<nestingArea>`, `<defaultSettings>`, `fixedheight`, `DEMReference`,
`useTelescoping_grid` né gli altri interruttori di `<modelGeometry>`. Non conosce i file di
configurazione della simulazione (`.SIMX`), i database di materiali e piante, né i risultati.
Non scrive `.INX`.

Nessuna di queste è una lacuna da colmare per principio: si aggiungono quando un ticket ne ha
bisogno. Il lettore sta in `src/inx.rs` e i suoi casi di prova in `tests/inx.rs`.
