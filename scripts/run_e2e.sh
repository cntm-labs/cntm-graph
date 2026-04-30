#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Step 1: Building and Running Rust E2E Writer...${NC}"
cargo run --example e2e_writer

echo -e "\n${GREEN}Step 2: Building C Helper Library...${NC}"
gcc -shared -o src/libhelper.so -fPIC src/helper.c

echo -e "\n${GREEN}Step 3: Building and Running Mojo E2E Reader...${NC}"
# Use absolute paths for linking to ensure success in various environments
REPO_ROOT=$(pwd)
pixi run mojo build src/main.mojo \
    -Xlinker -L"$REPO_ROOT/src" \
    -Xlinker -lhelper \
    -Xlinker -rpath -Xlinker "$REPO_ROOT/src" \
    -o src/reader

./src/reader

echo -e "\n${GREEN}Step 4: Cleaning up...${NC}"
rm e2e_test_graph.bin e2e_test_graph.bin.meta src/reader

echo -e "\n${GREEN}[SUCCESS] End-to-End Zero-copy Validation Complete!${NC}"
