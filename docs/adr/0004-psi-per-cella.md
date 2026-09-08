# La trasmissività delle chiome è per cella, non per griglia

Il passo fuso di [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig) riceve la
trasmissività delle chiome come **uno scalare per passo orario**, `psi`, valido per l'intera
griglia. CLIMESH ha due trasmissività nella stessa griglia, i sempreverdi e le chiome spoglie
d'inverno, e la [risoluzione del ticket](https://github.com/maeurong/CLIMESH/issues/18) le ha
confermate come vincolo. Un solo scalare sbaglia per costruzione proprio a Bastia d'inverno,
dove il sole sotto le chiome è la domanda per cui il caso esiste.

Si decide: **nella copia estratta il nucleo riceve `psi` come raster, per cella**, e ne riceve
due. La [trasmissività lungo il raggio](../../CONTEXT.md) per l'ombra del sole, calcolata ogni
ora, e la [trasmissività verso il cielo](../../CONTEXT.md) per la volta celeste, calcolata una
volta per Scenario. La sostituzione la fa `estrai.py` ed è dichiarata in `PROVENIENZA.toml`
con il conteggio dei punti toccati: uno per il raggio, sette per il cielo.

## Perché non le alternative

**Un solo `psi` con una regola di scelta** (per stagione, o pesato sull'area delle chiome) costa
zero e sbaglia in una delle due direzioni per ogni cella mista: o i sempreverdi lasciano passare
il 50%, o le chiome spoglie fanno ombra piena.

**Due Corse del Motore, una per Strato, ricomposte a valle**, non si ricompongono: il passo fuso
porta lo stato termico del suolo da un'ora alla successiva, Tmrt è una radice quarta e non è
lineare nelle ombre, e la cella sotto entrambe le chiome verrebbe contata due volte.

**Chiedere a monte un `psi` per cella** resta possibile, ma la mappa non ne dipende e la
decisione, presa l'8 settembre 2026, è di non chiederlo: la modifica resta nostra.

## Cosa comporta

**È la prima modifica di significato nella copia vendorata.** L'[ADR 0002](0002-vendoring-del-motore.md)
ammette di adattare per far compilare e vieta di migliorare. Questa decisione apre una terza
classe, la trasformazione additiva dichiarata, a una condizione che non si negozia: **con un
raster costante il nucleo deve riprodurre a monte byte per byte.** È ciò che tiene validi i test
di parità, e un test lo verifica.

**Nel cielo anisotropo resta un'approssimazione**, e il Giornale la dichiara: una cella che vede
un sempreverde in una patch e una chioma spoglia in un'altra riceve una sola trasmissività verso
il cielo. Renderla esatta chiederebbe una matrice d'ombra per Strato, che è una modifica più
profonda di quella ammessa qui.

**`conifer` non entra nel ragionamento.** Nel nucleo Rust al commit pinnato è un campo mai letto:
vive nella struct degli scalari e nei metadati Python. Non si sostituisce e non si dichiara.
