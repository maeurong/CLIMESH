# Design — CLIMESH

Sistema di design bloccato. Ogni esecuzione futura di Hallmark legge questo file
per primo, e le pagine gli si conformano invece di differenziarsi. Si emenda con
intenzione: il file è la regola.

La fonte di verità dei valori è [`assets/tokens.css`](assets/tokens.css). Qui c'è
il perché; lì ci sono i numeri.

## Sistema

- **Genere** · editorial
- **Macrostruttura** · Map / Diagram — un unico grande diagramma spaziale organizza
  la pagina. È la forma naturale di CLIMESH: il campo calcolato *è* il contenuto.
- **Tema** · Grid — scuola svizzera dei sistemi, griglia a 12 colonne esposta,
  un solo grottesco, un solo inchiostro segnale speso in geometria.
- **Assi** · carta chiara / display grotesk-heavy minuscolo / accento ultramarina

## La regola che tiene insieme tutto

**Il colore è un'unità di misura, non una decorazione.**

L'interfaccia è monocroma: carta, inchiostro, filetti, e un solo inchiostro
segnale. Di conseguenza ogni pixel colorato della pagina è un dato. La rampa
termica di una mappa non compete con la grafica attorno, perché attorno non c'è
grafica colorata.

Questa non è una preferenza estetica: è la stessa disciplina che il progetto
applica ai numeri. Un'interfaccia che mette colore attorno ai dati mette rumore
attorno al segnale.

## Token

Valori canonici in [`assets/tokens.css`](assets/tokens.css). In sintesi:

```css
:root {
  --color-paper:      oklch(99%  0.003 255);  /* foglio, freddo, mai #fff     */
  --color-paper-2:    oklch(97.2% 0.005 255); /* riga in hover, cella quieta  */
  --color-paper-3:    oklch(94.5% 0.006 255); /* fondo di un campo vuoto      */
  --color-ink:        oklch(16%  0.010 255);  /* testo, righi, serie di base  */
  --color-ink-2:      oklch(52%  0.012 255);  /* etichette, assi, secondario  */
  --color-rule:       oklch(88%  0.004 255);  /* il filetto da 1px            */
  --color-rule-2:     oklch(94%  0.003 255);  /* filetto interno, più quieto  */
  --color-accent:     oklch(45%  0.190 264);  /* ultramarina: l'unico segnale */
  --color-accent-ink: oklch(99%  0.003 255);  /* testo sopra la lastra        */
  --color-focus:      oklch(45%  0.190 264);

  --font-display: "Archivo", system-ui, sans-serif;  /* 800, minuscolo        */
  --font-body:    "Archivo", system-ui, sans-serif;  /* 400/500/600/700       */
  /* Nessuna seconda famiglia. Nessun serif. Nessun monospazio.               */

  --ease-out: cubic-bezier(.22,.61,.36,1);
  --dur-fast: 180ms;
}
```

Spaziatura su scala da 4 pt, `--space-3xs` … `--space-4xl`. Scala tipografica a
rapporto 1.25. Raggi e ombre: **zero, sempre**. La profondità la fanno i filetti,
i righi d'inchiostro e l'unica lastra.

## Il colore dei dati

Il livello che sta sopra il sistema, e l'unico autorizzato a essere policromo.

- **Rampa sequenziale** — dal viola profondo al giallo pallido, monotona in
  chiarezza, nessuna inversione di tono. Serve ogni campo scalare: temperatura
  media radiante, indice di comfort, radiazione. Mai un arcobaleno.
- **Ombra** — scala di grigi, nessun tono. L'ombra non è una temperatura.
- **Confronto fra Scenari** — non due tinte categoriche ma **base contro segnale**:
  lo Scenario di riferimento è inchiostro, quello confrontato è ultramarina,
  entrambi con etichetta diretta. Due tinte per due Scenari moltiplicherebbero
  gli inchiostri e romperebbero la regola sopra.
- **Provenienza** — un attributo rilevato porta il segnale, uno stimato porta il
  grigio secondario. Mai il rosso: un'altezza stimata non è un errore.

## Voce dei controlli

- **Primario** · fondo ultramarina, testo carta, raggio zero, ritmo 14/20 px.
- **Secondario** · bordo d'inchiostro da 1px, fondo trasparente, stesso ritmo.
- **Campi** · nessun riquadro. Filetto inferiore da 2px in inchiostro, che passa
  a ultramarina in hover.
- **Focus** · anello ultramarina da 2px con scarto di 2px, **mai animato**.

## Movimento

Trattenuto, geometrico, agganciato alla griglia. Nessun reveal allo scroll,
nessuna parallasse, nessun autoplay. Ammessi solo micro-stati in hover — sfondo
di riga, filetto che vira al segnale, trasformazioni che atterrano su incrementi
di griglia — a 180 ms. `prefers-reduced-motion: reduce` azzera tutto.

## La stampa

**La relazione non è una seconda interfaccia: è questa pagina stampata.** Non
esiste una vista "documento" accanto alla vista "strumento", e non deve
esistere: due superfici che mostrano lo stesso risultato divergono, e quella
stampata diverge per prima perché la si guarda di meno.

Il foglio è un `@media print` sulla stessa pagina, e traduce invece di
reimpaginare:

- **I controlli spariscono, i loro valori restano.** Periodo, ora, grandezza e
  scala diventano una riga di testo sotto la testata. Su carta non si può
  interagire, ma sapere con quali parametri è stata prodotta la figura è
  esattamente ciò che rende citabile il foglio.
- **Le figure prendono una didascalia.** A schermo sarebbe rumore, perché i
  pannelli sono già etichettati e l'ora è nel cursore. Su carta è l'unica cosa
  che spiega cosa si sta guardando.
- **La lastra non si allaga.** L'inchiostro pieno a piena larghezza in stampa
  spesso non arriva — molte impostazioni scartano i fondi — e quando arriva
  costa. Diventa un blocco chiuso fra due righi, che dice la stessa cosa.
- **I filetti a 12 colonne non si stampano.** Sono un dispositivo da schermo; sul
  foglio la griglia la fanno le celle rigate, che sono già lì.
- **Il Giornale comincia in una pagina nuova.** È l'appendice metodologica, e va
  potuta staccare.

## Note

**Mondo singolo chiaro, deliberato.** Il tema vieta la pagina scura, quindi non
c'è un tema scuro: ogni colore è dipinto esplicitamente e la pagina regge su
qualunque fondo dell'ospite. È un vincolo accettato con gli occhi aperti, e il
punto in cui potrebbe stringere è noto — uno strumento che si usa per ore davanti
a mappe potrebbe volere un fondo scuro. Se un giorno servirà davvero, si emenda
questo file con una sezione `## Varianti`, non con un'eccezione locale.

**La griglia non è impalcatura.** I filetti a 12 colonne restano visibili, anche
attraverso la lastra. Cancellarli perché "sembrano una guida" toglie il tema.

**Ogni banda porta un oggetto.** Una sezione di sola prosa non appartiene a
questo sistema: una lastra, una figura, un numerale tagliato, una matrice di
celle rigate, barre a gradini accanto a numeri veri.

**Una lastra sola per pagina.** La banda a piena larghezza in ultramarina è il
momento poster. Due lastre sono un template a righe.

## Export

`assets/tokens.css` è la fonte di verità. Per Tailwind v4 `@theme`, DTCG
`tokens.json` o variabili shadcn/ui, chiedere *"estendi DESIGN.md con gli export
Tailwind"* (o il formato voluto).
