#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "${GREEN}Step 1: Building and Running Rust E2E Writer...${NC}"
cargo run --example e2e_writer

echo -e "\n${GREEN}Step 2: Running Mojo E2E Reader...${NC}"
pixi run mojo src/main.mojo

echo -e "\n${GREEN}Step 3: Cleaning up...${NC}"
rm e2e_test_graph.bin

echo -e "\n${GREEN}[SUCCESS] End-to-End Zero-copy Validation Complete!${NC}"
