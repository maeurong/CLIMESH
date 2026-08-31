# Il nucleo radiativo si vendora, non si dipende

CLIMESH riusa il nucleo radiativo di [`UMEP-dev/solweig`](https://github.com/UMEP-dev/solweig),
GPL-3. La [risoluzione del ticket sul motore](https://github.com/maeurong/CLIMESH/issues/9)
prevedeva di consumarlo come **dipendenza git pinnata a un tag**, dopo una richiesta a monte
che ne esponesse un'API Rust nativa, tenendo il vendoring come ripiego.

Si fa il contrario: **il sorgente entra in `vendor/solweig/`**, e non si apre nessuna richiesta
a monte.

## Perché

**Il fork era più grande di quanto sembrasse.** Il dossier `research/motore-api.md`, letto sul
sorgente e non su una descrizione, ha trovato tre ostacoli invece di uno. Il tipo di crate è
`cdylib` e va esteso; ma tutti i moduli in `lib.rs` sono privati, quindi rendere pubblica una
funzione non basta; e soprattutto `pyo3` è dichiarato con la feature `extension-module` **non
opzionale**, che secondo la guida di PyO3 impedisce ai binari di compilare. Il vendoring aggira
tutti e tre in un colpo solo, perché non consumiamo il loro crate: prendiamo i file che servono.

**A monte non esiste nessuna richiesta simile**, e l'autore ha dichiarato in una discussione
pubblica di volere tutto dentro il binario Rust. Aprire una richiesta e aspettarla avrebbe legato
il nostro calendario a quello di un manutentore singolo su un progetto in beta.

**Aprire una richiesta a nome di Mario su un progetto altrui è un atto pubblico**, e non è una
cosa che si delega a un agente che lavora di notte. Resta possibile più avanti, con calma, e a
quel punto sarà una scelta di contribuzione e non una dipendenza del piano.

## Cosa comporta

**Obblighi.** Le intestazioni di licenza dei file presi non si toccano. `vendor/solweig/LICENSE`
porta il testo GPL-3 a monte. `vendor/solweig/PROVENIENZA.toml` dichiara in forma leggibile da un
programma il repository, il commit, la data, la licenza, il percorso sorgente, i file presi e
**l'elenco esatto delle modifiche fatte**. Adattare per far compilare è lecito; migliorare non lo
è, perché ogni divergenza è debito da riconciliare al prossimo aggiornamento.

**Il codice vendorato marcisce in silenzio**, e un promemoria in un file è un promemoria che
nessuno rilegge. Il lavoro pianificato `.github/workflows/vendor-check.yml` chiede a monte ogni
quattro mesi se si è mosso e apre una issue quando è successo, con l'elenco dei commit in mezzo.
Dice esplicitamente di **non aggiornare per abitudine**: il commit vendorato è pinnato anche
perché il Giornale cita per versione la validazione del nucleo, e cambiarlo cambia ciò che quella
citazione afferma. I nomi delle chiavi di `PROVENIENZA.toml` sono un contratto con quel lavoro.

**Licenza.** GPL-3 da entrambe le parti, quindi la combinazione è ammessa senza condizioni oltre
al mantenere le note di copyright e distribuire il sorgente. Se CLIMESH finisse sotto AGPL-3, la
§ 13 della GPLv3 lo permette esplicitamente.

**Il ripiego è deciso e non si improvvisa.** Se il sorgente vendorato non compilasse comunque, si
riscrivono le ombre da zero: la marcia del raggio su un campo di altezze è poche centinaia di
righe, e il test geometrico — torre di 10 m, sole a sud a 45 gradi, ombra lunga 10 m verso nord —
la verifica indipendentemente da chi l'ha scritta. In quel caso `vendor/` sparisce, il lavoro
pianificato si toglie, e serve un ADR nuovo, perché quella scelta contraddirebbe la ragione per
cui la reimplementazione era stata scartata: costanti ed euristiche che servono davvero non stanno
nei paper, stanno solo nel codice.

## Cosa si perde

Il fork avrebbe reso il nucleo consumabile da chiunque, ed era il tipo di contributo che fa
esistere un progetto agli occhi di quello a monte. Il vendoring non restituisce niente a nessuno,
e ci lascia una copia da tenere allineata a mano. È il prezzo di non dipendere dai tempi di un
manutentore singolo, ed è consapevole.
