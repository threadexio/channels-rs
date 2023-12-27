#!/usr/bin/env bash

optimize_svg() {
  local in="${1:-expected input svg}"
  local out="${2:-expected output svg}"

  scour "$in" "$out" \
    --create-groups \
    --strip-xml-prolog \
    --remove-titles \
    --remove-descriptions \
    --remove-descriptive-elements \
    --enable-comment-stripping \
    --enable-viewboxing \
    --indent=none \
    --no-line-breaks \
    --strip-xml-space \
    --enable-id-stripping \
    --shorten-ids
}

export_inkscape() {
  local in="${1:-expected input file}"

  local actions="$(mktemp)"
  cat << EOA > "$actions"
export-area-page;
export-overwrite;
export-png-use-dithering:true;
export-text-to-path:true;
EOA

  while read -r line; do
    echo "$line" >> "$actions"
  done
  unset line

  inkscape --actions-file="$actions" "$in"
  rm "$actions"
}

INPUT="$1"
INPUT="${INPUT:=logo.inkscape.svg}"
INPUT="$(realpath -e --no-symlinks "$INPUT")"

OUT="$(pwd)"

mkdir -p out
cd out
ln -rsf "$INPUT" ./src.svg

cat << EOF | export_inkscape ./src.svg
export-width:1280;
export-height:640;

export-filename:repo.banner.png;
export-do;

select-by-id:background_layer;
selection-hide;

export-filename:logo.transparent.svg;
export-plain-svg;
export-do;
EOF

cp -v \
	repo.banner.png \
	"$OUT"

optimize_svg logo.transparent.svg "$OUT/logo.transparent.svg"
