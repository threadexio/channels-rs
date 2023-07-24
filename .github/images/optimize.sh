#!/bin/sh

scour \
	--create-groups \
	--strip-xml-prolog \
	--remove-titles \
	--remove-descriptions \
	--remove-metadata \
	--remove-descriptive-elements \
	--enable-comment-stripping \
	--enable-viewboxing \
	--indent=none \
	--no-line-breaks \
	--strip-xml-space \
	--enable-id-stripping \
	--shorten-ids \
	logo.inkscape.svg logo.svg
