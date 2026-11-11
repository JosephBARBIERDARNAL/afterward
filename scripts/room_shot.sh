#!/usr/bin/env bash
# Takes a picture of whole rooms, as the game draws them, without opening a
# window: the fastest way to see what a room's YAML looks like.
#
#   scripts/room_shot.sh 3           room 3 -> target/room-shots/room_003.png
#   scripts/room_shot.sh --grid 3 4  rooms 3 and 4, ruled in H (..._grid.png)
#   scripts/room_shot.sh             every room under src/assets/rooms
#
# Prints the path of each picture, one per line, and nothing else.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

grid=()
rooms=()
for argument in "$@"; do
    case "$argument" in
        --grid) grid=(--grid) ;;
        *[!0-9]* | "") echo "usage: $0 [--grid] [room ID...]" >&2; exit 2 ;;
        *) rooms+=("$argument") ;;
    esac
done
if [ ${#rooms[@]} -eq 0 ]; then
    for file in src/assets/rooms/room_*.yaml; do
        id="${file##*room_}"
        rooms+=("$((10#${id%.yaml}))")
    done
fi

cargo build --quiet --locked --bin afterward

log="$(mktemp)"
trap 'rm -f "$log"' EXIT
for room in "${rooms[@]}"; do
    # The game's own log is only worth reading when the picture failed.
    if ! target/debug/afterward --room "$room" --shot ${grid[@]+"${grid[@]}"} 2>"$log"; then
        cat "$log" >&2
        exit 1
    fi
done
