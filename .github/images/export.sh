#!/usr/bin/env bash

export_svg() {
	local in="${1:-expected input file}"
	shift

	local out="${1:-expected output file}"
	local type="${out##*.}"
	shift

	local res="${1:-expected WIDTHxHEIGHT}"
	local width="${res%x*}"
	local height="${res#*x}"
	shift

	inkscape "$in" \
	        --export-filename "$out" \
		--export-overwrite \
		--export-type "$type" \
		--export-area-page \
		-w "$width" -h "$height" \
		"$@"
}

export_svg logo.inkscape.svg logo.png 1280x640 
