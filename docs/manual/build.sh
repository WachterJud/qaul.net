#!/bin/sh

# crate the structural dependencies SVG image
dot -Tsvg src/assets/dependencies.dot -o src/assets/dependencies.svg

# build this mdbook
mdbook build
