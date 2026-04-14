#!/usr/bin/env bash
# Regenerate data/gen1_en.json from PokéAPI (English names, national dex order 1–151).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/data/gen1_en.json"
mkdir -p "$ROOT/data"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
for i in $(seq 1 151); do
  curl -sS "https://pokeapi.co/api/v2/pokemon-species/$i/" \
    | jq -r '.names[] | select(.language.name=="en") | .name'
  sleep 0.03
done > "$tmp"
jq -R -s -c 'split("\n") | map(select(length>0))' "$tmp" > "$OUT"
echo "Wrote $OUT ($(jq 'length' "$OUT") names)"
