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

# I file di monte che CLIMESH riusa, e nient'altro.
FILE = ["shadowing.rs", "utci.rs"]

# Elementi che a monte sono `pub(crate)` e che devono diventare `pub`, perche'
# il crate vendorato viene consumato da fuori. E' l'unica modifica al codice
# rimasto, ed e' "adattare per far compilare": senza, la copia non serve a
# niente. Va dichiarata in PROVENIENZA.toml, e sta qui perche' una patch fatta
# a mano si perderebbe alla prossima estrazione senza che nessuno se ne accorga.
DA_ESPORRE = {
    "shadowing.rs": [
        "fn calculate_shadows_rust",
        "struct ShadowingResultRust",
    ],
    "utci.rs": [],
}

# Elementi annotati `#[pyfunction]` che non sono involucri: il corpo e' fisica,
# e Python compare solo nell'attributo. Di questi si toglie l'attributo e si
# tiene il resto, che e' la trasformazione piu' conservativa possibile — una
# riga in meno, zero righe cambiate.
#
# Perche' non riscriverli da noi: il corpo di `utci_single` e' la conversione
# documentata da umidita' relativa a pressione di vapore piu' la chiamata al
# polinomio. Copiarlo in CLIMESH significherebbe tenere fisica vendorata fuori
# da `vendor/`, dove nessuno la riconoscerebbe come tale al prossimo
# aggiornamento.
DA_TENERE = {
    "shadowing.rs": [],
    "utci.rs": ["pub fn utci_single"],
}


def dichiarazione(righe: list[str], i: int) -> str:
    """La dichiarazione dell'elemento che comincia a `i`, saltando attributi e commenti."""
    while i < len(righe):
        spoglia = righe[i].strip()
        if spoglia.startswith("#[") or spoglia.startswith("///") or not spoglia:
            i += 1
            continue
        return " ".join(spoglia.split()[:3]).rstrip("(")
    return ""


def estrai(testo: str, da_tenere: list[str]) -> tuple[str, int, int]:
    """Il sorgente senza gli involucri Python, quanti ne ha tolti e quanti ne ha tenuti."""
    righe = testo.split("\n")
    fuori: list[str] = []
    i = tolti = tenuti = 0
    while i < len(righe):
        riga = righe[i]
        if riga.startswith("use numpy::") or riga.startswith("use pyo3::"):
            i += 1
            continue
        if riga.strip() in ("#[pyfunction]", "#[pyclass]"):
            # Un elemento dichiarato in DA_TENERE perde l'attributo e resta.
            if any(dichiarazione(righe, i + 1).startswith(e) for e in da_tenere):
                tenuti += 1
                i += 1
                continue
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
    return "\n".join(fuori), tolti, tenuti


def esponi(testo: str, da_esporre: list[str]) -> tuple[str, int]:
    """Promuove a `pub` gli elementi indicati. Restituisce quanti ne ha promossi."""
    righe = testo.split("\n")
    fatti = 0
    for k, riga in enumerate(righe):
        for elemento in da_esporre:
            if riga.startswith(f"pub(crate) {elemento}"):
                righe[k] = riga.replace("pub(crate) ", "pub ", 1)
                fatti += 1
    return "\n".join(righe), fatti


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
        da_esporre = DA_ESPORRE[nome]
        da_tenere = DA_TENERE[nome]
        testo = (sorgente / nome).read_text(encoding="utf-8")
        pulito, tolti, tenuti = estrai(testo, da_tenere)
        pulito, esposti = esponi(pulito, da_esporre)
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
        print(
            f"{nome}: {prima} -> {dopo} righe, {tolti} involucri rimossi, "
            f"{tenuti} tenuti senza attributo, {esposti} elementi esposti"
        )
        if esposti != len(da_esporre):
            print(
                f"{nome}: attesi {len(da_esporre)} elementi da esporre, promossi {esposti}. "
                "A monte le visibilita' sono cambiate: rileggi DA_ESPORRE prima di fidarti.",
                file=sys.stderr,
            )
            return 1
        if tenuti != len(da_tenere):
            print(
                f"{nome}: attesi {len(da_tenere)} elementi da tenere, tenuti {tenuti}. "
                "A monte quegli elementi non ci sono piu' o non sono piu' annotati: "
                "rileggi DA_TENERE prima di fidarti.",
                file=sys.stderr,
            )
            return 1

    # Anche `lib.rs` esce da qui, e non da una mano: cosi' l'intera cartella
    # `src` e' prodotto dell'estrazione, e il controllo in integrazione continua
    # puo' confrontarla per intero invece di sapere quali file ignorare.
    moduli = "\n".join(f"pub mod {nome.removesuffix('.rs')};" for nome in FILE)
    (destinazione / "lib.rs").write_text(
        "// Vendorato da UMEP-dev/solweig, GPL-3. Vedi PROVENIENZA.toml.\n"
        "// Generato da estrai.py: non si modifica a mano.\n"
        f"\n{moduli}\n",
        encoding="utf-8",
    )

    return 0


if __name__ == "__main__":
    sys.exit(main())
