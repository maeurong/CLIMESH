# ENVI-met gratuito e caso Bastia — esiste una via percorribile?

**Scopo**: rispondere al ticket #6 (`maeurong/CLIMESH`) — se esista oggi una versione di ENVI-met usabile gratis che regga il caso di riferimento (50×50×25 celle, 48 h), quali limiti imponga, se esistano licenze accademiche, e cosa implichi il fix di conservazione dell'energia della 5.9.0 per la comparabilità fra versioni.
**Data ricerca**: 2026-08-31. **Repo**: CLIMESH, branch `main`, HEAD `7f23d42`.
**Rapporto precedente**: `research/envi-met.md` (non tracciato da git) copre già il quadro generale su ENVI-met; questo file approfondisce solo il punto della disponibilità gratuita, aggiungendo verifiche fatte oggi direttamente sulle pagine ufficiali.

**Convenzione**: `[P]` fonte primaria (envi-met.info, envi-met.com, oneclicklca.com, file del repo), `[S]` fonte secondaria, `[I]` inferenza mia non confermata.

---

## 1. Quali versioni sono disponibili senza pagare, oggi

- La wiki tecnica descrive **ENVI-met LITE** come tuttora gratuita, licenza **Creative Commons BY-NC-SA**, non commerciale, dominio limitato a **50×50 celle**. `[P]` — https://envi-met.info/doku.php?id=intro:modelconcept (fetched 2026-08-31)
- Ma la pagina Download ufficiale (https://envi-met.info/doku.php?id=files:download, fetched 2026-08-31) oggi elenca **un solo file scaricabile**: `Setup.exe V5.9.0` (5 dicembre 2025), con la nota *"Recent Version V5.9.0, Winter 25 Setup I Licensing changed. Contact support@envi-met.com for support"*. Non esiste più un link o un'edizione LITE separata da scaricare in self-service. `[P]`
- La pagina files:start (https://envi-met.info/doku.php?id=files:start) dichiara, senza eccezioni esplicite per LITE: *"Beginning with the new version 5.7.2, a One Click LCA account is required to use ENVI-met."* Procedura descritta: creare un account su oneclicklca.com → verificare l'email → completare il profilo → poi **"contact support@envi-met.com so that everything can be set up for you"**. `[P]`
- Stessa pagina: chi aveva una licenza rilasciata prima del 21/02/2025 poteva continuare a usare la 5.7.1 o precedente **solo fino al 31/12/2025**; il vecchio sistema di licenza non è più supportato dopo quella data. Oggi (31/08/2026) quella finestra è già chiusa. `[P]`
- Le versioni precedenti alla 5.9.0 sono nell'archivio (https://envi-met.info/doku.php?id=files:downloadv4), etichettato esplicitamente **"License required"** — quindi non sono scaricabili liberamente nemmeno in forma storica. `[P]`
- La pagina commerciale attuale https://envi-met.com/pricing/ (fetched 2026-08-31) elenca solo tre tipi di licenza — **Business, Universities, Students** — e **non menziona affatto LITE**. `[P]`

**Conclusione su questo punto**: la documentazione tecnica non è stata aggiornata per rimuovere LITE, ma il percorso operativo reale oggi è cambiato — un solo installer unico, obbligo di account One Click LCA con SSO dal 21/02/2025, e un passaggio manuale ("contact support so everything can be set up for you") anche per ottenere l'edizione gratuita. **Non ho trovato, in questa sessione, alcuna conferma pubblica post-acquisizione (2025-2026) che qualcuno abbia effettivamente ottenuto e usato LITE tramite questo nuovo flusso.** Le uniche discussioni trovate su come scaricare LITE gratis (es. ResearchGate) sono precedenti al cambio di sistema o non accessibili (403). Questo è un vero e proprio **buco di evidenza**, non un fatto negativo accertato: non posso dire con certezza né che LITE sia ancora ottenibile gratis, né che sia stata discontinuata — solo che il self-service descritto dalla vecchia documentazione non esiste più e va sostituito da un contatto diretto col supporto, il cui esito non è documentato pubblicamente. `[I]`

## 2. Limiti noti della versione gratuita (LITE)

| Limite | Stato | Fonte |
|---|---|---|
| Dominio orizzontale massimo 50×50 celle | confermato, nessun limite verticale esplicito dichiarato | `[P]` intro:modelconcept, files:download |
| Nessun calcolo parallelo | dichiarato esplicitamente solo per le edizioni **BASIC e STUDENT** ("The BASIC and STUDENT editions do not support parallel computing"); per LITE non ho trovato una frase equivalente esplicita — è ragionevole assumere la stessa restrizione dato che LITE è l'edizione più limitata, ma non è una citazione diretta | `[P]`/`[I]` — https://envi-met.info/doku.php?id=kb:parallel |
| Cartella output `Buildings/DYNAMIC` (dati dinamici di facciata) assente | confermato esplicitamente "not in ENVI-met LITE" | `[P]` — https://envi-met.info/doku.php?id=filereference:output:start |
| Licenza CC BY-NC-SA | uso non commerciale, condivisione allo stesso modo, attribuzione | `[P]` intro:modelconcept |
| Durata massima di simulazione | **nessun limite specifico per LITE trovato** in nessuna delle pagine consultate | — |

Non ho trovato una dichiarazione ufficiale su un tetto orario di simulazione per LITE: il vincolo noto resta solo dimensionale (50×50 in pianta).

## 3. Il caso di riferimento (50×50×25 celle, 48 h) regge su LITE?

- Verificato nel repo: `materiale università/LAB1.INX:4` contiene `<version>440</version>` (creato con SPACES 5.1.1), e il dominio dichiarato nel brief del ticket è 50×50×25 celle a 1 m.
- **Orizzontalmente il caso è esattamente al limite dichiarato (50×50)**: nessuna fonte ufficiale chiarisce se il vincolo sia "≤ 50×50" o "< 50×50". Questa ambiguità testuale non è risolvibile da documentazione pubblica.
- **Verticalmente (25 celle)**: nessun vincolo LITE è documentato, quindi sulla carta non sarebbe un problema.
- **48 ore di simulazione**: nessun limite di durata dichiarato per LITE.
- Non ho scaricato né installato ENVI-met (per istruzione esplicita), quindi non posso confermare empiricamente se il software accetti 50×50 esatto o lo rifiuti. Sulla sola base dei testi ufficiali, il caso sembra rientrare nel limite dichiarato, ma il margine è zero e la formulazione vaga non consente una risposta certa.
- Il vincolo pratico più probabile non è il dominio ma il **tempo di calcolo**: se LITE gira single-core (come le edizioni BASIC/STUDENT), un dominio 50×50×25 su 48 h potrebbe richiedere tempi lunghi; le uniche stime vendor su tempi di calcolo (kb:compute) sono pre-parallelizzazione e già segnalate come stale nel report precedente, quindi non utilizzabili per una stima affidabile.
- **Sottoinsieme più piccolo eseguibile "con certezza" secondo la lettera della documentazione**: qualsiasi dominio orizzontale strettamente minore di 50×50 (es. 49×49) rientrerebbe senza ambiguità nel limite LITE; ridurre nz sotto 25 non è necessario perché non risulta vincolato.

## 4. Licenze accademiche/didattiche

Esistono due percorsi accademici, **entrambi a pagamento**, distinti dalla LITE gratuita:

- **Students**: https://envi-met.com/pricing/ → "Students" → **GET IN TOUCH** (nessun prezzo pubblicato). Condizioni dichiarate: durata massima **1 anno**, **prova di iscrizione obbligatoria**, **nessun calcolo parallelo**. `[P]`
- **Universities** (Science): fino a **50 dispositivi per dipartimento**, uso esclusivo ricerca/didattica, abbonamento o termine fisso. `[P]` stessa pagina.
- Una fonte secondaria non riverificata su pagina primaria in questa sessione indica che per lo Student va inviata una "proof of enrolment" a `license@envi-met.com` e che l'installazione sarebbe permessa su un numero limitato di dispositivi; non ho ri-letto il documento (PDF di terze parti) per confermarlo in questa sessione. `[S]`
- **In sintesi**: non esiste una licenza accademica gratuita. Student e Universities/Science sono sconti/percorsi dedicati al mondo accademico, ma richiedono comunque un contatto commerciale diretto e — presumibilmente — un pagamento non pubblicato. L'unica via realmente gratuita resta LITE, con l'incertezza descritta al punto 1.

## 5. Il confronto è possibile per altra via?

- Non ho trovato risultati pubblicati sullo stesso identico caso (Casa Evolutiva di Renzo Piano, Bastia Umbra) al di fuori del materiale già presente nel repository.
- **Il materiale già nel repo è esso stesso un run ENVI-met completo su questo caso**: `materiale università/LAB1.INX` (dominio 50×50×25, versione core 5.1.1) e `materiale università/RELAZIONI/LA01.pdf` (relazione dei risultati). Non sono riuscito a rileggere il PDF in questa sessione (l'ambiente non ha `pdftoppm`/poppler-utils installato per il rendering pagina); mi affido qui alla verifica già fatta dal thread dispacciante sul contenuto di pagina 2. Questo output pre-esistente è probabilmente la via di confronto più concreta e meno rischiosa: usare i risultati già prodotti dalla tesi come termine di paragone, invece di rincorrere una nuova licenza gratuita di esito incerto.
- Non ho trovato dataset di output ENVI-met condivisi da terzi per questo caso specifico.

## 6. Il fix della 5.9.0 sulla conservazione dell'energia e la comparabilità fra versioni

- Confermato da fonte primaria (changelog ufficiale, fetched oggi): la **V5.9.0** (5 dicembre 2025) dichiara *"ENVI-core: Fixed a major bug in conservation of energy - this leads to noticeable changes in the results"*, oltre a *"Changed the heat exchange method between building walls and air to a more accurate approach"*. `[P]` — https://envi-met.info/doku.php?id=apps:updates
- Il caso di riferimento è stato creato con **SPACES 5.1.1** (`materiale università/LAB1.INX:4`, `<version>440</version>`; il file è datato marzo 2023, coerente col fatto che il changelog colloca la 5.1.1 nel "Winter 22/23"). Tra la 5.1.1 e la 5.9.0 il changelog ufficiale documenta **molteplici cambiamenti nel core fisico con impatto dichiarato sui risultati**, non solo il fix del 5.9.0: ad esempio la 5.5 dichiara un miglioramento del calcolo della radiazione a onda lunga "mainly affecting nocturnal urban heat island and improving MRT accuracy", e la 5.6 dichiara un bugfix per cui "air temperature increase is now larger than before for impervious areas". `[P]` — stessa pagina changelog.
- **Implicazione diretta per il ticket**: un run gratuito fresco oggi sarebbe necessariamente in **5.9.0** (l'unico installer attualmente distribuito). Confrontarlo numericamente contro i risultati del 2023 prodotti in 5.1.1 mescolerebbe due fonti di differenza indistinguibili: (a) le differenze dovute al modello CLIMESH rispetto a ENVI-met, e (b) le differenze dovute ai cambiamenti del motore ENVI-met stesso fra 5.1.1 e 5.9.0, di entità non quantificata dal vendor. Un confronto onesto dovrebbe dichiarare esplicitamente questo confondimento, oppure evitare di trattare il risultato del 2023 come riferimento "ENVI-met" tout court.
- Ho trovato l'esistenza di un paper del 2026, *"Comparative validation of ENVI-met versions 5.6 and 5.9 against high-resolution in-situ air temperature measurements"* (ScienceDirect, S2590162126000675), che sembra affrontare proprio la comparabilità fra versioni. **Non sono riuscito a leggerne il testo**: sia `defuddle` sia `WebFetch` hanno ricevuto 403 Forbidden dal sito. Ho solo la sintesi restituita dal motore di ricerca (fonte secondaria, non verificata direttamente da me), secondo cui lo studio userebbe un dataset di temperatura dell'aria ad alta risoluzione raccolto per 7 giorni intorno a un edificio a Aalborg (Danimarca), e riporterebbe per entrambe le versioni un R² > 0.94 sul ciclo diurno di temperatura. **Questo dato, se confermato, riguarderebbe comunque un solo edificio, un solo clima, diverso da Bastia Umbra, e non è chiaro se copra il fix di conservazione dell'energia della 5.9.0 rispetto a versioni precedenti alla 5.6** (il paper compara 5.6 e 5.9, non 5.1.1 e 5.9). Non è quindi utilizzabile per affermare che i risultati del caso Bastia (prodotti in 5.1.1) siano comparabili a un run 5.9.0. `[S]`, non verificato su fonte primaria.

## 7. Risposta sintetica alla domanda del ticket

Non esiste, ad oggi, una conferma pubblica affidabile che una versione gratuita di ENVI-met sia effettivamente ottenibile in self-service dopo l'acquisizione da parte di One Click LCA e l'obbligo di account SSO (dal 21/02/2025): la wiki tecnica descrive ancora LITE come gratuita e limitata a 50×50 celle, ma il flusso di download reale offre un solo installer (5.9.0) e richiede di "farsi impostare tutto" dal supporto dopo aver creato un account One Click LCA — un processo il cui esito per un utente non licenziato non è documentato da nessuna parte trovata. Anche se si ottenesse LITE, il caso di riferimento (50×50×25, 48 h) sta esattamente al limite dichiarato (50×50 in pianta) senza che il vendor chiarisca se il limite sia inclusivo, e senza vincoli verticali o di durata documentati — quindi tecnicamente plausibile ma non verificabile senza un test diretto. Le licenze accademiche (Students, Universities/Science) esistono ma **non sono gratuite**: richiedono contatto commerciale e nessun prezzo è pubblico. La via di confronto più concreta e a rischio più basso resta l'uso dei risultati già prodotti dalla tesi (`materiale università/RELAZIONI/LA01.pdf`, run in ENVI-met 5.1.1), non un nuovo run gratuito — cosa che risolve anche il secondo problema: qualunque run fresco sarebbe oggi obbligatoriamente in 5.9.0, la cui correzione dichiarata di un bug maggiore di conservazione dell'energia introduce un cambiamento "percepibile" nei risultati rispetto a tutte le versioni precedenti, inclusa la 5.1.1 usata per il caso Bastia — un confronto diretto fra le due versioni del motore mescolerebbe quindi differenze del modello CLIMESH con differenze del motore ENVI-met stesso, di entità non quantificabile dalle fonti pubbliche disponibili.

---

## Fonti consultate in questa sessione (2026-08-31)

**Primarie**
- https://envi-met.info/doku.php?id=intro:modelconcept
- https://envi-met.info/doku.php?id=files:download
- https://envi-met.info/doku.php?id=files:start
- https://envi-met.info/doku.php?id=files:downloadv4
- https://envi-met.info/doku.php?id=apps:updates
- https://envi-met.info/doku.php?id=kb:faq
- https://envi-met.info/doku.php?id=kb:parallel
- https://envi-met.info/doku.php?id=filereference:output:start
- https://envi-met.com/pricing/
- https://oneclicklca.com/software/design-construction/envi-met
- `materiale università/LAB1.INX` (riga 4, `<version>440</version>`)

**Secondarie / non verificate direttamente (403 o non riletto in questa sessione)**
- https://www.sciencedirect.com/science/article/pii/S2590162126000675 (paper 2026, validazione 5.6 vs 5.9 — letto solo tramite snippet di ricerca)
- https://www.researchgate.net/post/HOW_can_i_download_envi_met_lite_software_for_free... (403)
- https://help.envi-met.com/en/ (nessun contenuto estraibile, SPA)
- `materiale università/RELAZIONI/LA01.pdf` (non rirenderizzato in questa sessione: `pdftoppm`/poppler-utils non installato nell'ambiente)

## Caveat

1. Il punto più importante di questo report — se LITE sia ancora attivabile gratis dopo l'acquisizione — **non è accertabile da fonte pubblica**: ho trovato solo indizi indiretti (installer unico, obbligo di contatto col supporto) che suggeriscono un attrito nuovo, non una conferma né una smentita. Se questo dato è determinante per il gate di validazione del ticket #1, l'unico modo per saperlo con certezza è tentare la registrazione (fuori dallo scope di questa ricerca, che non doveva scaricare/installare nulla).
2. La lettura del paper 2026 su ScienceDirect è di seconda mano (snippet del motore di ricerca), non un abstract letto direttamente: trattarla con cautela.
3. Non ho riletto `RELAZIONI/LA01.pdf` in questa sessione per mancanza di poppler-utils nell'ambiente; i dettagli su di esso citati nel brief del ticket (applicazioni usate, giorni simulati) sono presi per validi perché il thread dispacciante dichiara di averli già verificati.
