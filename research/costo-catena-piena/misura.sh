#!/usr/bin/env sh
# Rifà la misura del ticket wayfinder #24. Dalla radice del repository:
#
#     PY=<venv di monte>/bin/python sh research/costo-catena-piena/misura.sh
#
# Il primo argomento di ogni riga di uscita è la Corsa, il secondo il cielo.
# `RAYON_NUM_THREADS=1` mette il pool globale di monte nella condizione in cui
# CLIMESH tiene il proprio: senza, i numeri salgono di due o tre ordini di
# grandezza e non parlano più del caso di riferimento.
set -e
PY=${PY:?serve PY=<venv>/bin/python}
export PATH="$HOME/.cargo/bin:$PATH"

echo "== CLIMESH: quello che il Motore fa oggi =="
cargo run --quiet --release --manifest-path research/costo-catena-piena/Cargo.toml -- casi/bastia/progetto

echo
echo "== monte: il passo fuso, un passo per ora =="
for corsa in stato-di-fatto-estate stato-di-fatto-inverno interventi-estate interventi-inverno; do
  for cielo in iso aniso; do
    log=$(mktemp)
    RAYON_NUM_THREADS=1 SOLWEIG_TIMING=1 "$PY" research/costo-catena-piena/passo_fuso.py "$corsa" "$cielo" 2>"$log" | tail -1
    printf '   '
    for passo in shadow ground gvf delay radiation tmrt total; do
      grep -o "$passo=[0-9.]*ms" "$log" | sed "s/$passo=//;s/ms//" \
        | awk -v k="$passo" '{s+=$1} END {printf "%s=%.1f ", k, s}'
    done
    echo "(somma su 48 passi, ms)"
    rm -f "$log"
  done
done
