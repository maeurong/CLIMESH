# Geometria urbana in Italia per CLIMESH — da dove arriva, e quanto vale sul caso Bastia Umbra

**Scopo**: rispondere al ticket [#5](https://github.com/maeurong/CLIMESH/issues/5) — da dove arriva la geometria urbana necessaria a CLIMESH in Italia, e quanto è buona sul caso di riferimento.
**Data ricerca**: 2026-08-31. **Repo**: CLIMESH, branch `main`, HEAD `7f23d421a2f26df31d374cd141c1137d3d8496d5`.
**Metodo**: verifica diretta via Overpass API sull'area del sito, più fonti primarie (OSM wiki, taginfo, documentazione UMEP, geoportali).

**Convenzione di marcatura** (stessa di `research/envi-met.md`):
- `[P]` = confermato da fonte primaria (query Overpass fatta io stesso, API ufficiali, wiki/doc del progetto citato).
- `[S]` = fonte secondaria (paper, articolo, listing di terzi).
- `[I]` = inferenza mia, non confermata da fonte diretta.

---

## 1. Il caso di riferimento — verifica delle premesse

Ho letto direttamente `materiale università/LAB1.INX:19-42` `[P]`: griglia `50×50×25` celle, `dx=dy=dz-base=1.0 m`, `modelRotation=21.0`, `location_Longitude=12.56`, `location_Latitude=43.07`, `locationName=bergamo`. Le coordinate sono coerenti con Bastia Umbra (PG), non con Bergamo (45.7°N) — è un refuso lasciato nel template SPACES, non un dato geografico: `[P]` diretto sul file, coerente con la premessa del brief.

Ho verificato indipendentemente l'identità del sito (il brief cita `RELAZIONI/LA01.pdf` p.2, che non sono riuscito a leggere: `pdftoppm`/poppler non è installato nell'ambiente e non ho permessi per installarlo, né `pip` è disponibile per un estrattore alternativo). Ho invece trovato online la scheda ufficiale del censimento nazionale delle architetture del Novecento (fonte: `www.architetturecontemporanee.beniculturali.it`, riportata su OICOS Riflessioni) `[S]`, che conferma punto per punto la descrizione del brief:

> "Casa Famiglia (Casa evolutiva – Centro di salute mentale)", Bastia Umbra, **Via Irlanda**, inserita nel **Sito Giontella** (ex Tabacchificio Giontella), progetto architettonico **Renzo Piano**, progetto strutturale **Peter Rice**, 1976-1978. "Il complesso è costituito da **due blocchi duplex e da un blocco simplex**, disposti a formare **una corte verde aperta** e collegati da una pensilina."
> — [OICOS Riflessioni, Renzo Piano e Peter Rice, Casa evolutiva – Bastia Umbra](https://www.oicosriflessioni.it/2018/12/14/renzo-piano-e-peter-rice-casa-evolutiva-bastia-umbra/)

Quindi: la premessa del brief su nome, autore e tipologia del sito è confermata `[S]` (fonte secondaria giornalistico/divulgativa che riporta però una scheda di censimento pubblico ufficiale). L'edificio oggi risulta **dismesso e in stato di abbandono** (infiltrazioni, minaccia di demolizione discussa nel 2009-2021), non un edificio residenziale attivo — un dettaglio non nel brief ma rilevante: il "caso reale" è un rudere, non un quartiere abitato.

## 2. Cosa c'è davvero su OpenStreetMap a Bastia Umbra, attorno a 43.07/12.56

Questo è il punto su cui ho speso più tempo, con query dirette a Overpass API (`https://overpass-api.de/api/interpreter`, endpoint pubblico ufficiale del progetto OSM) `[P]`.

### 2.1 Query sull'area del dominio di simulazione (raggio 400 m attorno a 43.07/12.56)

Query eseguita:
```
[out:json][timeout:25];
(way["building"](around:400,43.07,12.56);
 relation["building"](around:400,43.07,12.56););
out tags center;
```
Risultato (eseguito il 2026-08-31, `timestamp_osm_base: 2026-08-31T16:54:49Z`) `[P]`:

- **168 poligoni `building` totali** entro 400 m dal punto (il dominio 50×50 m del caso studio ci sta comodamente dentro, con ampio margine per un dominio più grande).
- **0 su 168** hanno il tag `height`.
- **3 su 168** hanno `building:levels`: "Ex Tabacchificio Giontella" (levels=2), "Scuola Calcio A.C. Bastia" (levels=1), un edificio anonimo (levels=3).
- Solo 4 edifici hanno un `name`: l'ex Tabacchificio Giontella, il Palazzetto dello Sport, la Scuola Pubblica Umberto Fifi, la Scuola Calcio A.C. Bastia. **Nessun edificio è taggato come "Casa Evolutiva", "Casa Famiglia" o simili.**

### 2.2 Query mirata sul Sito Giontella (raggio 200 m attorno a 43.0668846/12.5577628, il centroide dell'edificio "Ex Tabacchificio Giontella")

Ho usato l'edificio nominato come ancora per restringere la ricerca al complesso specifico dove si trova la Casa Evolutiva (via Irlanda, dentro il Sito Giontella). Risultato, filtrando per distanza dal centroide sui dati già scaricati `[P]`:

- **14 edifici** entro 200 m dal Tabacchificio.
- Fra questi, un gruppo di **5-6 poligoni piccoli** (117-240 m² di impronta ciascuno, ID Overpass sequenziali `1332703806-1332703812`, quindi verosimilmente digitalizzati nella stessa sessione di editing) compatibili per dimensione con i moduli duplex/simplex descritti dalla scheda di censimento. **Nessuno di questi porta un attributo di altezza, piani o nome**: sono poligoni `building=yes` nudi. Non ho un modo indipendente per confermare che uno specifico poligono sia esattamente la Casa Evolutiva piuttosto che un'altra struttura del sito industriale dismesso — l'impronta geometrica è presente, l'identità e l'altezza no.
- L'unico edificio con `building:levels` in tutta l'area è il capannone del Tabacchificio stesso (2 piani), non la Casa Evolutiva.

### 2.3 Interpretazione

**Ciò che è disponibile**: l'impronta a terra (footprint poligonale) dei fabbricati è presente e ragionevolmente densa — 168 edifici mappati in un raggio di 400 m è una copertura di digitalizzazione buona per un'area semi-periferica italiana. Per la parte 2D (dove sono gli edifici, che sagoma hanno in pianta) l'import OSM funzionerebbe.

**Ciò che manca**: qualunque informazione di altezza verticale. Zero edifici su 168 hanno `height`; solo 3 (l'1.8%) hanno `building:levels`, e nessuno di questi tre è il caso studio. Per ricostruire l'altezza reale (6 m per i blocchi duplex, 3 m per il simplex, secondo la scheda di censimento) **non c'è alcuna scorciatoia nei dati OSM di quel punto**: va misurata da un DSM, da un rilievo, o assegnata a mano da chi conosce l'edificio. Per un caso che l'utente-studente conosce già da un rilievo di laboratorio (i PDF `9a...9f_Rilievo CSM` presenti in `materiale università/Caso_Studio_CSM Bastia_studenti/`), questo non è un problema bloccante — ma smentisce l'idea che "importa da OSM e hai tutto": per **questo** sito specifico, l'automazione dà la pianta e nient'altro.

## 3. Quanto è rappresentativo questo caso — confronto con statistiche più ampie

### 3.1 Copertura nazionale height/building:levels (taginfo regionale Geofabrik)

Ho interrogato l'istanza taginfo di Geofabrik per l'estratto Italia (`https://taginfo.geofabrik.de/europe:italy/api/4/key/stats?key=...`) `[P]`, snapshot al 2026-08-30:

| chiave | conteggio su `way` | `building` totali (way) | quota approssimata |
|---|---|---|---|
| `building` | 16.284.109 | — | — |
| `height` | 1.394.461 | 16.284.109 | **~8,6%** |
| `building:levels` | 388.066 | 16.284.109 | **~2,4%** |

`[I]` la quota è una divisione mia fra due conteggi taginfo che non sono garantiti riferirsi esattamente allo stesso universo (height/levels possono comparire anche su way non-building), quindi va letta come ordine di grandezza, non come percentuale esatta. Il dato locale di Bastia Umbra (0% height, 1,8% levels su 168 edifici) è **coerente con la media nazionale o leggermente sotto**, non un'anomalia negativa isolata.

### 3.2 Conferma indipendente dalla documentazione ufficiale di UMEP

Il tutorial ufficiale del **DSM Generator** di UMEP (lo strumento che, dentro QGIS, scarica edifici OSM e li trasforma in un DSM assegnando l'altezza da `height`/`building:levels`) riporta lo stesso fenomeno su un caso reale (Göteborg, centro storico): `[P]` — https://umep-docs.readthedocs.io/projects/tutorial/en/latest/Tutorials/DSMGenerator.html

> "the result is not very good since only **8 out of 47 polygons** included useful height information [...] Usually height information in the OSM dataset is not very common but for some larger cities the information can be very close to full coverage."

Il rimedio che il tutorial stesso insegna è **assegnare un'altezza uniforme a mano** con la Field Calculator di QGIS per tutti gli edifici privi del dato — cioè un intervento manuale sistematico dopo l'import, non un'estensione dell'automazione. Nota rilevante anche per CLIMESH: lo stesso tutorial avverte che questa tecnica **assume tutti i tetti piani** ("roof structures are not included [...] all roofs are assumed flat"); per tetti inclinati serve LiDAR.

Questo conferma da fonte primaria indipendente (il progetto GPL-3.0 più vicino concettualmente a CLIMESH, non un blog) che il problema visto a Bastia Umbra non è un caso limite italiano ma **il comportamento normale di OSM per l'altezza degli edifici**, con forte variabilità: alcune grandi città sono vicine alla copertura piena, il resto no.

## 4. DEM/DSM pubblici per l'Italia

`[P]` salvo dove indicato.

- **TINITALY/1.1** (INGV) — DTM nazionale gratuito, griglia 10 m, GeoTIFF, UTM WGS84 zona 32, con servizio **WCS** dedicato. Non contiene edifici né vegetazione: è un modello del solo terreno. — https://tinitaly.pi.ingv.it/ , https://tinitaly.pi.ingv.it/wcs_service.html
- **Geoportale Nazionale (PCN, ora sotto MASE)** — DTM a 20 m via WCS/WMS/WMTS pubblico, oltre a cataloghi di prodotti LiDAR regionali (vedi sotto). — https://gn.mase.gov.it/portale/en/wcs
- **DTM/DSM LiDAR Regione Umbria**, risoluzione **1 m** (DTM, DSM First, DSM Last, Intensity), da rilievo aereo del **Piano Straordinario di Telerilevamento** del Ministero dell'Ambiente, licenza **CC BY 4.0**, distribuito via PCN/RNDT/INSPIRE. `[S]`(scheda di metadato, non ho scaricato il dato) — https://www.pcn.minambiente.it (scheda "DTM LiDAR con risoluzione a terra 1 metro - Regione Umbria")
  - **Caveat importante, non verificato in dettaglio**: la scheda di metadato descrive la copertura come relativa ai corsi d'acqua di I e II ordine ("survey covered I and II order river channels"), non necessariamente all'intero territorio regionale. Se questo è corretto, la copertura LiDAR a 1 m su Bastia Umbra dipenderebbe dalla vicinanza del sito ai corsi d'acqua locali (il torrente Chiona/Chiascio scorrono nell'area) — **non ho verificato se il poligono di volo copre esattamente il Sito Giontella**; sarebbe il passo naturale successivo se questo dataset diventa la scelta primaria per DSM ad alta risoluzione su Bastia Umbra.
- **Copernicus DEM GLO-30** — copertura globale, risoluzione 30 m, licenza gratuita per uso generale, distribuito via Copernicus Data Space Ecosystem / AWS Open Data / OpenTopography. `[P]` — https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM
  - **Nota di attualità, verificata il 2026-08-31**: a partire dal **28 luglio 2026** l'accesso al servizio di visualizzazione COP-DEM-GLO-30 è stato ristretto alle categorie di utenti autorizzate secondo la licenza CCM (registrazione obbligatoria su CDSE Identity Portal); la copertura pubblica "GLO-30 Public" resta comunque disponibile ma non è più identica a prima. `[P]` — https://dataspace.copernicus.eu/news/2026-7-17-copernicus-dem-30m-view-service-license-acceptance . Questo è un cambiamento recente (dopo il cutoff di conoscenza del modello che ha scritto questo report) e va riverificato al momento dell'implementazione, non dato per scontato sulla base di documentazione più vecchia.
  - 30 m è comunque una risoluzione grossolana per un dominio di 50×50 m a cella 1 m: utile solo come contesto/fallback per il terreno più ampio attorno al dominio, non come sorgente della quota di dettaglio.

`[I]` Per il caso Bastia Umbra specifico, nessuna di queste fonti contiene edifici o vegetazione: sono tutte DTM (solo terreno) tranne il LiDAR regionale, che se disponibile come DSM First/Last potrebbe includere edifici e vegetazione, ma la copertura non è confermata sul punto esatto.

## 5. Vegetazione — come la ricava chi lo fa già (UMEP)

`[P]` — https://umep-docs.readthedocs.io/en/latest/OtherManuals/SOLWEIG.html (manuale ufficiale SOLWEIG)

SOLWEIG/UMEP non prende l'altezza degli alberi da OSM. Richiede due raster dedicati:
- **CDSM** (Canopy DSM): quota della cima della chioma sopra il suolo, pixel a 0 dove non c'è vegetazione;
- **TDSM** (Trunk-zone DSM): quota della base della chioma (tronco nudo sotto la chioma).

Questi raster si producono normalmente da **classificazione di nuvole di punti LiDAR** (vedi tutorial dedicato "Generating UMEP input data from a LiDAR point cloud"), non da OSM. Per chi non ha LiDAR, UMEP offre un **"Tree Generator"**: un plugin che genera vegetazione sintetica **da un file vettoriale a punti inserito a mano** dall'utente, con attributi `id, ttype, trunk, totheight, diameter` — cioè uno strumento di inserimento manuale assistito, non un estrattore automatico da dati aperti.

`[I]` Conclusione per CLIMESH: OSM offre alberi puntuali (`natural=tree`) e aree verdi (`landuse=forest`, `leisure=park` ecc.), ma quasi mai altezza o diametro chioma nei tag; nessuno strumento esistente, UMEP compreso, ha risolto questo con l'automazione — tutti finiscono su LiDAR o inserimento manuale. Non ho verificato la copertura specifica del tag `natural=tree` a Bastia Umbra (non l'ho interrogata separatamente), ma dato il pattern generale è ragionevole aspettarsi copertura scarsa o nulla anche lì.

## 6. Formati e API — cosa si presta a un binario Rust senza dipendenze pesanti

`[P]` per l'esistenza e il funzionamento delle interfacce, verificate con query dirette in questa sessione:

- **Overpass API**: endpoint HTTP pubblico (`overpass-api.de`, mirror `overpass.kumi.systems`), richiesta POST con querystring nel linguaggio Overpass QL, risposta JSON (`out:json`) o XML. Nessuna libreria client ufficiale necessaria: un client HTTP generico (es. `reqwest`) più un parser JSON (`serde_json`) bastano; ho verificato in questa sessione che il servizio pubblico **può restituire errori di timeout/server-busy** anche per query piccole (mi è successo due volte su cinque tentativi) — un binario di produzione dovrebbe prevedere retry ed eventualmente un mirror alternativo o un endpoint self-hosted per uso intensivo.
- **DTM/DSM via WCS/WMS**: sia TINITALY sia il Geoportale Nazionale espongono WCS standard OGC — protocollo HTTP con XML/GetCoverage, anch'esso raggiungibile senza dipendenze GIS pesanti (una richiesta HTTP + parsing del GeoTIFF risultante, per cui serve comunque un lettore GeoTIFF, es. crate `tiff` o `gdal` se si accetta la dipendenza da libgdal).
- **Nominatim** (geocoding OSM): utile per risolvere un nome di località a coordinate, ma **non ha trovato nulla** cercando "Casa Evolutiva Bastia Umbra" o "Casa Evolutiva Renzo Piano" `[P]` (verificato in questa sessione) — conferma che il sito non è geocodificabile per nome tramite OSM, va sempre passato per coordinate esplicite.

## 7. Risposta alla domanda centrale del ticket

**Quanto è realistico ricostruire il caso Bastia Umbra dai soli dati pubblici?**

- **Geometria in pianta (impronta edifici)**: realistico. 168 edifici mappati entro 400 m dal punto, densità di digitalizzazione buona, footprint plausibilmente riconducibili anche al complesso specifico della Casa Evolutiva. `[P]`
- **Altezza degli edifici**: **non realistico dall'automazione**. Zero copertura del tag `height` nell'area, copertura `building:levels` marginale (1,8%) e comunque non sul caso studio. Serve altrove: rilievo esistente (già disponibile per questo caso nei PDF di laboratorio), DSM LiDAR se e quando se ne conferma la copertura sul punto esatto, o inserimento manuale — esattamente come fa oggi UMEP quando OSM non basta.
- **Vegetazione (616 istanze di piante nel caso originale)**: non realistico dall'automazione OSM in nessun caso generale verificato; nessuno strumento esistente lo risolve automaticamente, tutti richiedono LiDAR o inserimento manuale.
- **Identificazione del sito per nome**: non disponibile; il progetto va sempre referenziato per coordinate, non per geocoding.

`[I]` La misura concreta richiesta dal ticket ("quanto varrebbe l'import automatico") è quindi: l'import OSM+DTM copre bene la sagoma 2D e il terreno di sfondo, ma lascia interamente a carico dell'utente (o di un DSM LiDAR locale ancora da confermare) altezza edifici e vegetazione — che sono anche le due grandezze più radiativamente rilevanti (ombreggiamento, MRT). Per il caso di riferimento specifico l'informazione mancante esiste già altrove nel materiale di laboratorio (rilievi CSM), quindi il costo reale per *questo* caso è basso; ma non generalizza: un sito italiano scelto a caso ha una probabilità concreta (Bastia Umbra è vicina alla mediana nazionale, non un caso peggiore) di trovarsi nella stessa situazione.

---

## Riepilogo fonte primaria vs. inferenza

**Confermato da fonte primaria in questa sessione** `[P]`:
- Contenuto di `LAB1.INX` (coordinate, griglia, rotazione, refuso "bergamo").
- Query Overpass dirette sull'area (168 edifici, 0 height, 3 levels, nessun nome "Casa Evolutiva").
- Statistiche taginfo Italia per `height`/`building:levels`/`building`.
- Contenuto dei manuali/tutorial ufficiali UMEP (SOLWEIG, DSM Generator) su CDSM/TDSM, Tree Generator, e l'esempio Göteborg 8/47.
- Esistenza, risoluzione e licenza di TINITALY, Copernicus DEM GLO-30, incluso il cambio di accesso di luglio 2026.
- Nominatim non trova "Casa Evolutiva" per nome.

**Fonte secondaria, non verificata di prima mano da me** `[S]`:
- Descrizione architettonica del sito (via OICOS Riflessioni, che a sua volta cita il censimento pubblico ufficiale — non ho letto la fonte primissima).
- Scheda del LiDAR Umbria 1 m (solo metadato letto, non il dato).
- Paper su completezza OSM/ISTAT in Italia (accesso bloccato, 403 su MDPI).

**Non verificato / limite di questa sessione**:
- Impossibile leggere `RELAZIONI/LA01.pdf` (mancano `poppler-utils` e `pip` nell'ambiente).
- Non confermato se il poligono LiDAR Regione Umbria copra esattamente il Sito Giontella.
- Non ho interrogato la copertura del tag `natural=tree` su Bastia Umbra (inferenza per analogia, non dato diretto).
- Non è stato identificato con certezza quale singolo poligono OSM, fra i 5-6 candidati vicino al Tabacchificio Giontella, corrisponda esattamente alla Casa Evolutiva.
