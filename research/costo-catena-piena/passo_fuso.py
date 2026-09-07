"""Cronometra il passo fuso di monte sulla geometria di Bastia.

Ticket wayfinder #24, «Quanto costa la catena piena». Legge i CSV che il
programma Rust accanto a questo file scrive in `raster/`, li dà a
`solweig.calculate` di UMEP-dev/solweig al commit pinnato in
`vendor/solweig/PROVENIENZA.toml`, e riporta quanto costa ogni passo temporale.

Uso, dalla radice del repository, con l'ambiente costruito a parte:

    RAYON_NUM_THREADS=1 SOLWEIG_TIMING=1 <venv>/bin/python \
        research/costo-catena-piena/passo_fuso.py stato-di-fatto-estate iso 2> timing.txt

La riga `[TIMING]` per passo la stampa il nucleo su stderr quando
`SOLWEIG_TIMING=1`: `rust/src/pipeline.rs`, `timing_enabled`.

Il meteo è sintetico e dichiarato tale: il file EPW del caso sta sotto
`materiale università/`, che è in `.gitignore`. Global radiation da cielo
sereno, `900 * sin(altezza solare)`, con l'altezza calcolata dal codice solare
di monte stesso in una prima passata. Il costo di un passo dipende dal fatto
che il sole sia sopra l'orizzonte, non dal valore esatto dell'irraggiamento.
"""

import math
import os
import sys
import tempfile
import time
from datetime import datetime, timedelta
from pathlib import Path

import numpy as np
import solweig
from solweig import Location, SurfaceData, Weather

RASTER = Path(__file__).parent / "raster"
# Bastia Umbra: `casi/bastia/progetto/progetto.toml`, chiave `griglia.origine`
# (longitudine, latitudine) e il fuso che `corsa::coordinate` deriva dalla
# longitudine quando il file meteo non si legge.
LUOGO = Location(latitude=43.07, longitude=12.56, utc_offset=1)
INIZIO = {"estate": datetime(2021, 7, 15), "inverno": datetime(2021, 1, 15)}
ORE = 48


def leggi(nome):
    percorso = RASTER / nome
    if not percorso.exists():
        return None
    return np.loadtxt(percorso, delimiter=",", dtype=np.float32)


def superficie(base):
    dsm = leggi(f"{base}-dsm.csv")
    if dsm is None:
        sys.exit(f"manca {RASTER / (base + '-dsm.csv')}: esegui prima il programma Rust")
    # CLIMESH scrive NaN dove uno strato non ha chioma; monte vuole zero, ed è
    # la stessa convenzione perché il terreno di Bastia sta tutto a quota zero.
    strati = []
    i = 0
    while (chiome := leggi(f"{base}-cdsm-{i}.csv")) is not None:
        tronco = leggi(f"{base}-tdsm-{i}.csv")
        strati.append((np.nan_to_num(chiome), np.nan_to_num(tronco)))
        i += 1
    # Il passo fuso ha un solo `psi` per passo temporale, quindi un solo strato:
    # gli strati si fondono prendendo la chioma più alta cella per cella. È la
    # semplificazione che il ticket #27 dovrà comunque affrontare, e qui serve
    # solo perché il costo non dipende da quale chioma vince.
    if strati:
        cdsm = np.maximum.reduce([c for c, _ in strati])
        tdsm = np.maximum.reduce([t for _, t in strati])
    else:
        cdsm = tdsm = None
    s = SurfaceData(dsm=dsm, cdsm=cdsm, tdsm=tdsm, pixel_size=1.0,
                    cdsm_relative=False, tdsm_relative=False)
    t0 = time.perf_counter()
    s.compute_svf()
    return s, time.perf_counter() - t0


def meteo(stagione):
    base = INIZIO[stagione]
    serie = []
    for ora in range(ORE):
        quando = base + timedelta(hours=ora)
        sonda = Weather(datetime=quando, ta=20.0, rh=50.0, global_rad=0.0, ws=2.0)
        sonda.compute_derived(LUOGO)
        seno = math.sin(math.radians(max(sonda.sun_altitude, 0.0)))
        serie.append(Weather(datetime=quando,
                             ta=27.0 if stagione == "estate" else 5.0,
                             rh=50.0, global_rad=900.0 * seno, ws=2.0))
    return serie


def main():
    base, cielo = sys.argv[1], sys.argv[2]
    stagione = "inverno" if "inverno" in base else "estate"
    s, t_svf = superficie(base)
    serie = meteo(stagione)
    di_giorno = sum(1 for w in serie if (w.compute_derived(LUOGO) or w.sun_altitude) > 0.0)
    with tempfile.TemporaryDirectory(prefix="climesh-24-") as uscita:
        t0 = time.perf_counter()
        solweig.calculate(surface=s, weather=serie, location=LUOGO,
                          use_anisotropic_sky=(cielo == "aniso"),
                          output_dir=uscita)
        t_catena = time.perf_counter() - t0
    print(f"{base} {cielo} rayon={os.environ.get('RAYON_NUM_THREADS', 'predefinito')} "
          f"celle={s.dsm.size} ore={ORE} di_giorno={di_giorno} "
          f"svf={t_svf * 1e3:.1f}ms catena={t_catena:.3f}s "
          f"per_ora={t_catena / ORE * 1e3:.1f}ms")


if __name__ == "__main__":
    main()
