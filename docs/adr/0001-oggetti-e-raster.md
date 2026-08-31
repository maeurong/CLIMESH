# Si disegna in oggetti, si calcola in raster

Il motore radiativo che CLIMESH riusa consuma esclusivamente raster co-registrati: altezze di terreno ed edifici, altezze delle chiome, classi di superficie. La strada ovvia sarebbe quindi tenere quei raster come rappresentazione canonica del modello urbano, e infatti è così che lavorano gli strumenti dell'ecosistema UMEP. CLIMESH fa il contrario: la verità sono gli **oggetti** — Edifici, Alberi, Superfici, ciascuno con i propri attributi e la propria Provenienza — e i raster sono un **derivato rigenerabile**.

## Perché

Tre ragioni, tutte pratiche.

**L'informazione persa non torna.** Con i soli raster non si può cambiare la risoluzione della Griglia, correggere l'altezza di un albero, o sapere perché un pixel è alto sei metri. Ogni operazione di modifica diventerebbe un ritocco su un'immagine.

**La provenienza è un attributo di un oggetto, non di un pixel.** Una ricerca condotta durante il tracciamento della mappa ha misurato che su 168 edifici mappati in OpenStreetMap attorno al caso di riferimento, zero portano l'altezza e tre portano il numero di piani. Molte altezze saranno quindi stimate o inserite a mano, e il prodotto di CLIMESH è un risultato citabile: deve poter rispondere alla domanda "quell'edificio lì, l'altezza da dove viene". Un raster non può.

**Il limite dell'interfaccia del motore non deve diventare un limite del modello.** Il motore accetta una sola trasmissività fogliare per esecuzione, mentre il caso di riferimento ha cinque specie diverse; d'inverno un pino ombreggia e una latifoglia no. Poiché la specie vive sull'oggetto Albero, la Derivazione può escludere le latifoglie dal raster delle chiome nel Periodo invernale, lavorando per pixel — cosa che il motore permette. Con i raster come verità, quella distinzione non sarebbe stata neppure esprimibile.

## Conseguenze

La Derivazione diventa un passo esplicito del programma, con i propri parametri e le proprie scelte di modellazione, e per questo va registrata nel Giornale della Corsa: "la derivazione invernale ha escluso *n* latifoglie" è una scelta di modellazione, non un dettaglio di implementazione.

Scartata anche l'ipotesi di tenere entrambe le rappresentazioni come verità sincronizzate: due verità che si sincronizzano sono due verità che prima o poi divergono.
