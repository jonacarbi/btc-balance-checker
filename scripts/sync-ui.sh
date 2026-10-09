#!/bin/sh
# The desktop app (src/) owns the shared design system; the site gets a copy. CI fails if they drift.
set -e
cd "$(dirname "$0")/.."
cp src/ledger.css site/ledger.css
cp src/fonts/*.woff2 site/fonts/
cp src/logo.svg site/logo.svg
