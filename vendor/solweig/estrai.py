#!/usr/bin/env python3
"""Estrae da UMEP-dev/solweig il percorso CPU delle ombre, senza Python.

Il vendoring non è una copia fatta a mano: è questo script più il commit
dichiarato in PROVENIENZA.toml. Chi vuole aggiornare al commit successivo non
deve indovinare cosa era stato tolto — riesegue e confronta.

Cosa toglie, e perché:

- `use numpy::…` e `use pyo3::…`. Il nucleo di calcolo prende `ArrayView2` e
  restituisce una struct Rust: Python compare solo negli involucri.
- Ogni elemento annotato `#[pyfunction]` o `#[pyclass]`. Sono gli involucri e
  la struct specchio del risultato per Python.

**La trappola.** Quegli elementi sono spesso preceduti da `#[cfg(feature =
"gpu")]` o da doc-comment. Togliendo solo la funzione, l'attributo resta
orfano e si attacca all'elemento seguente: nel caso reale questo cancellava in
silenzio `ShadowingResultRust`, e il compilatore riportava "cannot find type",
non "hai lasciato un attributo per aria". Per questo lo script risale sugli
attributi e sui commenti già emessi prima di rimuovere.

Il percorso GPU resta nel sorgente ma è dietro `#[cfg(feature = "gpu")]`, e la
feature non è dichiarata nel manifesto vendorato: si compila fuori da sé.

Uso:
    python3 vendor/solweig/estrai.py <clone-di-solweig> [destinazione]
"""

import sys
from pathlib import Path

# I file che servono al percorso CPU delle ombre, e nient'altro.
FILE = ["shadowing.rs"]


def estrai(testo: str) -> tuple[str, int]:
    """Restituisce il sorgente senza gli involucri Python, e quanti ne ha tolti."""
    righe = testo.split("\n")
    fuori: list[str] = []
    i = tolti = 0
    while i < len(righe):
        riga = righe[i]
        if riga.startswith("use numpy::") or riga.startswith("use pyo3::"):
            i += 1
            continue
        if riga.strip() in ("#[pyfunction]", "#[pyclass]"):
            # Risali sugli attributi e i doc-comment già emessi: un `#[cfg]`
            # lasciato orfano cancellerebbe l'elemento successivo.
            while fuori and (
                fuori[-1].lstrip().startswith("#[")
                or fuori[-1].lstrip().startswith("///")
                or fuori[-1].strip() == ""
            ):
                fuori.pop()
            # In codice formattato un elemento di primo livello chiude con una
            # graffa in colonna zero.
            i += 1
            while i < len(righe) and righe[i] != "}":
                i += 1
            i += 1
            tolti += 1
            continue
        fuori.append(riga)
        i += 1
    return "\n".join(fuori), tolti


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__.strip().split("Uso:")[1].strip(), file=sys.stderr)
        return 2

    sorgente = Path(sys.argv[1]) / "rust" / "src"
    destinazione = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(__file__).parent / "src"
    if not sorgente.is_dir():
        print(f"{sorgente}: non è la cartella rust/src di un clone di solweig", file=sys.stderr)
        return 1
    destinazione.mkdir(parents=True, exist_ok=True)

    for nome in FILE:
        testo = (sorgente / nome).read_text(encoding="utf-8")
        pulito, tolti = estrai(testo)
        residui = sum(
            1
            for r in pulito.split("\n")
            if "pyo3" in r or "numpy" in r or "PyArray" in r or "Py<" in r
        )
        if residui:
            print(f"{nome}: restano {residui} righe che nominano Python", file=sys.stderr)
            return 1
        (destinazione / nome).write_text(pulito, encoding="utf-8")
        prima = len(testo.split("\n"))
        dopo = len(pulito.split("\n"))
        print(f"{nome}: {prima} -> {dopo} righe, {tolti} involucri rimossi")

    return 0


if __name__ == "__main__":
    sys.exit(main())
