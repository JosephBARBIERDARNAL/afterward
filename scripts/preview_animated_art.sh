#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
preview_dir="$repo_root/art/previews"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

command -v typst >/dev/null || { echo "typst is required" >&2; exit 1; }
command -v magick >/dev/null || { echo "ImageMagick (magick) is required" >&2; exit 1; }

mkdir -p "$preview_dir"

# Render each source sheet, split it into frames, and assign GIF delays in
# centiseconds. Frame sizes and timings match the comments and playback code.
make_preview() {
    local source="$1"
    local name="$2"
    local frame_width="$3"
    local frame_height="$4"
    shift 4
    local delays=("$@")
    local sheet="$temporary_dir/$name.png"

    typst compile --root "$repo_root/art" --format png --ppi 72 \
        "$repo_root/art/$source" "$sheet"

    local -a frames=()
    local -a gif_args=(-dispose Background)
    local frame
    for frame in "${!delays[@]}"; do
        local frame_path="$temporary_dir/$name-$frame.png"
        magick "$sheet" -crop "${frame_width}x${frame_height}+$((frame * frame_width))+0" \
            +repage -filter point -resize 400% "$frame_path"
        frames+=("$frame_path")
        gif_args+=(-delay "${delays[$frame]}" "$frame_path")
    done

    magick "${gif_args[@]}" -loop 0 "$preview_dir/$name.gif"
    echo "art/$source -> art/previews/$name.gif (${#frames[@]} frames)"
}

# Door states hold for a moment; the twelve opening frames play at 30 fps.
make_preview doors/exit_door.typ exit_door 48 96 80 80 3 3 3 3 3 3 3 3 3 3 3 80

# The shield stays intact, then plays its five-frame break at 30 fps.
make_preview ui/enemy_shield.typ enemy_shield 21 16 100 3 3 3 3 100

# The hero's sheets, every frame of each at 10 fps. A clip of
# player/animations.yaml may play only some of a sheet's frames, at its own pace.
for source in "$repo_root"/art/player/sheets/*.typ; do
    name="$(basename "$source" .typ)"
    delays=()
    for ((frame = 0; frame < $(grep -c '```,' "$source"); frame++)); do
        delays+=(10)
    done
    make_preview "player/sheets/$name.typ" "player_$name" 128 128 "${delays[@]}"
done
