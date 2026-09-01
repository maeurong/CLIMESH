# CLIMESH

Calcola il comfort termico all'aperto in uno spazio urbano — dove cade l'ombra,
quanta radiazione riceve una persona, quanto caldo sente — in modo riproducibile
e citabile.

Un binario singolo: si scarica, si apre, si lavora nel browser. Nessuna
installazione, nessun account, nessuna licenza da chiedere.

## Perché

Sostituisce [ENVI-met](https://envi-met.com) per il caso d'uso più comune in una
tesi o in un corso. ENVI-met è closed source e solo per Windows, non riprende una
simulazione interrotta, non ha un'API, e dal febbraio 2025 richiede un account con
autenticazione centralizzata per essere aperto — dopo che il suo sviluppatore è
stato acquisito da un'azienda terza nel settembre 2024. I prezzi non sono
pubblici.

Ma il motivo per cui il progetto esiste è un altro, ed è più scomodo.

La relazione di laboratorio da cui nasce CLIMESH concludeva che un intervento di
mitigazione funzionava, sulla base di una differenza di temperatura dell'aria di
**0,21 °C** fra i due scenari. L'errore validato di ENVI-met su quella stessa
grandezza è di **1,34 °C** di errore medio assoluto: la conclusione stava per
intero dentro la barra d'errore del modello che l'aveva prodotta.

Nella stessa corte, la differenza di **temperatura media radiante** fra sole e
ombra vale fra 20 e 35 °C, contro un errore del modello di 2,4-7,3 °C. Da tre a
dieci volte il rumore, e per giunta è la grandezza che descrive cosa sente
davvero una persona ferma lì.

CLIMESH misura quella. E dichiara, oggetto per oggetto, quanto di ciò che mostra
è stato rilevato e quanto stimato.

## Dove guardare

- **[`docs/spec.md`](docs/spec.md)** — cosa si costruisce e in che forma.
- **[`PRODUCT.md`](PRODUCT.md)** — a chi serve, cosa deve fare, cosa è fuori.
- **[`CONTEXT.md`](CONTEXT.md)** — come si chiamano le cose. Progetto, Griglia,
  Scenario, Periodo, Corsa, Giornale.
- **[`DESIGN.md`](DESIGN.md)** — che faccia ha, e perché il colore è un'unità di
  misura invece che una decorazione.
- **[`research/`](research/)** — i sei rapporti che hanno istruito le decisioni:
  come funziona ENVI-met, cosa offre il panorama libero, quanto costa davvero il
  calcolo, quale indice di comfort ha senso.
- **[Mappa del progetto](https://github.com/maeurong/CLIMESH/issues/1)** — ogni
  decisione presa, con la sua motivazione, sotto forma di issue chiuse.

## Stato

**In costruzione, ma il nucleo calcola.** Il progetto è stato progettato per
intero prima di essere scritto, e il piano del nucleo di calcolo è chiuso.

Esiste: il lettore dei file `.INX` di ENVI-met, con la
[documentazione del formato](docs/formato-inx.md) che non era mai stata pubblica;
il [caso di riferimento](casi/bastia/) estratto e versionato; il Progetto su
disco con il suo modello a oggetti; la Derivazione da oggetti a raster; il
[motore radiativo vendorato](vendor/solweig/) e collegato; l'ombra degli
Edifici e quella degli Alberi, con le chiome che attenuano invece di spegnere;
il Giornale della Corsa.

**Il caso di riferimento completo gira in un quarto di secondo**, contro un
budget di sessanta: due Scenari per due Periodi, 50 × 50 celle a un metro,
48 ore, su sola CPU.

Non esiste ancora: la catena radiativa completa fino alla temperatura media
radiante e agli indici di comfort, la riga di comando, la pagina nel browser, e
la validazione contro le misure di campo. L'ordine in cui arrivano è nella
[spec](docs/spec.md).

Il caso di riferimento è la Casa Evolutiva di Renzo Piano a Bastia Umbra: 50 × 50
celle a un metro, 616 alberi di cinque specie, due scenari, due stagioni. Il
materiale del corso da cui deriva resta fuori dal repository perché non è
ridistribuibile; i numeri che ne discendono sono qui.

## Verifica

```bash
export PATH=$HOME/.cargo/bin:$PATH
cargo test
```

Su una macchina senza linker C, come quella su cui il progetto è nato:

```bash
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
  cargo test --target x86_64-unknown-linux-musl
```

## Licenza

GPL-3. Non è solo una scelta etica: è la promessa credibile che a CLIMESH non
accadrà mai ciò che è appena accaduto a ENVI-met.

Il nucleo radiativo è riusato da [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig),
anch'esso GPL-3, e riscriverlo sarebbe stato uno spreco: la fisica è già scritta,
validata contro misure di campo e citata in letteratura. Quello che mancava, e
che CLIMESH costruisce, è tutto il resto.
