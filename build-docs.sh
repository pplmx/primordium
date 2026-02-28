#!/bin/bash
# Build documentation for Primordium (mdBook)
set -e

echo "Building Primordium documentation..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Sync docs to src before mdbook build
echo -e "${BLUE}[1/1]${NC} Preparing and building mdBook..."
cp docs/wiki/BRAIN.md docs/book/src/wiki-brain.md 2>/dev/null || true
cp docs/wiki/GENETICS.md docs/book/src/wiki-genetics.md 2>/dev/null || true
cp docs/wiki/ECOSYSTEM.md docs/book/src/wiki-ecosystem.md 2>/dev/null || true
cp docs/wiki/HISTORY.md docs/book/src/wiki-history.md 2>/dev/null || true
cp docs/MANUAL.md docs/book/src/manual.md 2>/dev/null || true
cp docs/MANUAL_zh.md docs/book/src/manual_zh.md 2>/dev/null || true
cp README.md docs/book/src/readme.md 2>/dev/null || true
cp CHANGELOG.md docs/book/src/changelog.md 2>/dev/null || true
cp docs/CHANGELOG_zh.md docs/book/src/changelog_zh.md 2>/dev/null || true
cp ROADMAP.md docs/book/src/roadmap.md 2>/dev/null || true
cp ARCHITECTURE.md docs/book/src/architecture.md 2>/dev/null || true

cd docs/book
mdbook build
cd ../..

echo ""
echo -e "${GREEN}Documentation build complete!${NC}"
echo "Output: docs/book/book/"
echo ""
echo "To serve locally:"
echo " cd docs/book && mdbook serve --port 3000"