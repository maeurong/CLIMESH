# CLAUDE.md

CLIMESH calcola il comfort termico all'aperto in uno spazio urbano, in modo
riproducibile e citabile. Alternativa libera a ENVI-met.

## Dove stanno le decisioni

**Le decisioni di progetto non sono nel codice.** Vivono nella mappa wayfinder,
issue **#1** di `maeurong/CLIMESH`, con i ticket come sub-issue. Prima di
rispondere a una domanda di progetto, leggila: `gh issue view 1`. La risposta a
un ticket chiuso sta nel suo commento di risoluzione, non nella mappa, che ne
tiene solo il sunto.

Quattro documenti, quattro mestieri distinti. Non scriverne uno al posto di un
altro:

| File | Dice | Chi lo scrive |
|---|---|---|
| [`PRODUCT.md`](PRODUCT.md) | a chi serve, cosa deve fare, cosa è fuori | la skill `impeccable`, verbo `init` |
| [`DESIGN.md`](DESIGN.md) | che faccia ha, e perché | la skill `hallmark`, blocco del sistema |
| [`CONTEXT.md`](CONTEXT.md) | come si chiamano le cose | la skill `domain-modeling` |
| [`docs/adr/`](docs/adr/) | le decisioni costose da invertire | `domain-modeling`, con parsimonia |

## Il vocabolario è vincolante

Un **Progetto** contiene una **Griglia**, uno o più **Scenari** e uno o più
**Periodi**. Una **Corsa** è uno Scenario calcolato per un Periodo, e porta un
**Giornale**. Dentro uno Scenario stanno **Edifici**, **Alberi**, **Superfici**,
**Punti di osservazione**, ciascuno con la propria **Provenienza**. La
**Derivazione** trasforma gli oggetti nei raster che il **Motore** consuma.

Definizioni e termini da evitare in [`CONTEXT.md`](CONTEXT.md). Usa quelle parole
nel codice, nei commenti e nell'interfaccia: un sinonimo introdotto per comodità
costa più di quanto risparmia.

**La regola che genera il resto** ([ADR 0001](docs/adr/0001-oggetti-e-raster.md)):
si disegna in oggetti, si calcola in raster. Gli oggetti sono la verità, i raster
sono derivati rigenerabili. Il Motore parla raster, quindi l'istinto sarà di
tenere quelli come verità: è l'errore che l'ADR esiste per prevenire.

## Ambiente

Rust non è nel PATH e **manca un linker C** su questa macchina. Non usare
`cargo test` liscio: non collega.

```bash
export PATH=$HOME/.cargo/bin:$PATH
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld \
  cargo test --target x86_64-unknown-linux-musl
```

È un fatto della macchina, non del progetto, quindi non sta in un file di
configurazione committato. Nota però che musl con collegamento statico è
esattamente ciò che serve al binario singolo.

## Vincoli che non si negoziano

**Velocità.** Il caso di riferimento completo — 50 × 50 celle a 1 m, 48 ore, due
Scenari per due Periodi — sotto i **60 secondi su sola CPU**. Tetto di progetto
200 × 200 m a 2 m per 24 ore in 10 minuti. Un'opzione che non ci rientra è fuori,
qualunque altro merito abbia.

**Binario singolo, nessuna dipendenza nativa.** L'utente scarica un file e lo
apre. Niente GDAL, niente PROJ, niente NetCDF, niente toolchain da installare.
Prima di aggiungere una dipendenza, controlla su crates.io se tira dietro una
libreria C.

**Il colore è un'unità di misura.** L'interfaccia è carta, inchiostro, filetti e
un solo inchiostro segnale. Non aggiungere colore alla grafica: ogni pixel
colorato deve essere un dato. Regole in [`DESIGN.md`](DESIGN.md), valori in
[`assets/tokens.css`](assets/tokens.css).

**Niente informazione affidata al solo colore.** Obiettivo dichiarato WCAG 2.2
AA, e una mappa termica è un'immagine di dati: ogni campo calcolato deve avere
una vista tabellare alternativa.

**Le stringhe non si annidano nel codice.** Interfaccia in italiano e inglese dal
primo giorno. I termini tecnici del dominio restano in inglese in entrambe le
lingue: si dice *sky view factor*, non "fattore di vista del cielo".

## Il caso di riferimento

Casa Evolutiva, Bastia Umbra. Modello estratto e valori pubblicati in
[`casi/bastia/`](casi/bastia/), formato di partenza documentato in
[`docs/formato-inx.md`](docs/formato-inx.md).

**Trappola da conoscere prima di toccare una matrice:** le righe corrono da
**nord**. La prima riga è `j = grids-J`, l'ultima è `j = 1`. Sbagliare il verso
produce un modello specchiato che sembra ancora plausibile, e che nessuna
ispezione visiva smaschera.

**`materiale università/` è in `.gitignore` e ci resta.** Contiene rilievi,
tavole e la relazione originale di un corso: non è nostro da ridistribuire. I
numeri che ne derivano sì, i documenti no. Non copiarne il contenuto nel
repository in nessuna forma.

## Convenzioni

Codice, commenti, messaggi di commit e testi di issue in **inglese**.
Documentazione, file di dati e discussione in **italiano**.

Test prima dell'implementazione. Un test che fallisce se la logica si rompe,
niente framework e niente fixture elaborate. Enumera i casi degeneri prima di
scrivere: sono il grosso del lavoro di correzione successivo.
