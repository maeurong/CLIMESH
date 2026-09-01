# La catena radiativa si prende modulo per modulo, e per il passo fuso si chiede a monte

[ADR 0002](0002-vendoring-del-motore.md) ha deciso che il nucleo radiativo si vendora.
Funzionava per le ombre. Per la catena completa — sky view factor, radiazione, temperatura
media radiante — **non funziona allo stesso modo**, e questo ADR dice perché e cosa si fa.

## Il fatto

Il sorgente di `UMEP-dev/solweig` al commit pinnato è di 12.823 righe in 23 moduli. Letto,
non descritto:

**`shadowing.rs` e `utci.rs` hanno una porta d'ingresso nativa.** `calculate_shadows_rust`
prende `ArrayView2<f32>` e restituisce una struct Rust; `utci_single` prende quattro `f32`.
Python compare solo ai bordi, e toglierlo è ciò che `estrai.py` sa fare. Sono infatti i due
moduli già presi.

**Il passo fuso non ce l'ha.** Il calcolo di un'ora sta dentro `compute_timestep`, che è un
`#[pyfunction]`. Le sue righe 502-707 legano array numpy; le 708-1954 sono Rust puro dentro
una chiusura `allow_threads_unchecked(py, || …)` che restituisce `TimestepResultRaw`; dalla
1955 in poi si riconverte per Python. La fisica è quindi già separata — ma è una **chiusura
che cattura una quarantina di viste**, non una funzione, e nessuno script la trasforma in una
funzione in modo che regga il prossimo aggiornamento.

**I moduli intermedi hanno un ostacolo minore ma reale.** `skyview.rs`, `gvf.rs`, `ground.rs`
e `sky.rs` usano `PyResult` nelle firme del percorso Rust: `calculate_svf_inner` restituisce
`PyResult<SvfIntermediate>` perché `create_patches` fallisce con un `PyValueError` su
un'opzione sconosciuta. In `skyview.rs` sono quattro firme e due costruzioni di errore.

Dichiarare `pyo3` fra le dipendenze della copia vendorata non è una via d'uscita: senza la
feature `extension-module` il crate si lega a `libpython`, e il **binario singolo senza
dipendenze native** è un vincolo di progetto, non una preferenza.

## Cosa si è scartato

**Scrivere la nostra orchestrazione** sopra le funzioni di fisica vendorate. Sono le ~1250
righe che accoppiano ombre, SVF, radiazione e inerzia termica del suolo, ed è esattamente il
posto dove stanno le costanti e le euristiche che [ADR 0002](0002-vendoring-del-motore.md)
dice non stare nei paper. Riscriverle contraddirebbe la ragione per cui la reimplementazione
era stata scartata, e in più farebbe perdere i test di parità contro l'implementazione di
riferimento, che sono il motivo principale per cui quel codice merita fiducia.

## Cosa si fa

**Si prende modulo per modulo, dal basso e dall'alto verso il centro.** Ogni modulo che ha una
porta nativa entra subito; ogni modulo che chiede una trasformazione entra solo se la
trasformazione è **dichiarata, meccanica e capace di fermarsi con errore** quando a monte
cambia — la stessa disciplina che `estrai.py` già applica alle visibilità e agli attributi,
e che dal `2026-09-01` la CI verifica rieseguendo l'estrazione e confrontandola byte per byte.

Il prossimo modulo è `skyview.rs`, e chiede una classe di trasformazione nuova: `PyResult<T>`
diventa `Result<T, String>`. Due siti di errore, dichiarabili uno per uno in `PROVENIENZA.toml`.

**Per il passo fuso si chiede a monte.** Serve `compute_timestep_rust`: la chiusura di
`compute_timestep` estratta in una funzione che prende viste e restituisce
`TimestepResultRaw`, con il `#[pyfunction]` che le si appoggia sopra. È il rifattorizzamento
che a monte hanno già fatto per `calculate_shadows_rust`, quindi è nella loro forma e non nella
nostra, e non toglie niente a nessuno: Python continua a vedere la stessa API.

**Chi lo chiede è Mario, non un agente.** [ADR 0002](0002-vendoring-del-motore.md) l'ha già
stabilito — «aprire una richiesta a nome di Mario su un progetto altrui è un atto pubblico» —
e vale identico qui.

**Il ripiego, se a monte non risponde o dice di no**, è estendere la trasformazione dichiarata
anche a `compute_timestep`: `estrai.py` sostituisce il blocco dei parametri e cancella le due
regioni di conversione, delimitate da marcatori su cui lo script si ferma se non li trova più.
È più fragile della porta nativa, e per questo è il ripiego e non il piano.

## Cosa comporta

**Fino ad allora la catena è tronca**, e il Giornale lo dice campo per campo come già fa per
tutto il resto. Ciò che c'è oggi — ombre di edifici e chiome, con la trasmissività — è una
grandezza vera e verificabile; non è ancora la temperatura media radiante, e nessun documento
del progetto deve lasciar credere il contrario.

**UTCI è già in casa e non serve a niente da solo.** Non è uno spreco: è il pezzo che chiude la
catena dall'alto, ed è preso adesso perché è aritmetica pura e perché la promessa che un numero
di CLIMESH sia confrontabile con un numero di SOLWEIG andava resa vera almeno per una grandezza.

**Se a monte accettasse la porta nativa**, questo ADR si chiude e ADR 0002 va riletto: il
vendoring resterebbe la forma, ma con una porta d'ingresso costruita apposta la distanza dal
fork si accorcia, e la scelta andrebbe rimotivata invece che ereditata.
