#!/bin/bash

# Build the release version
cargo build --release

# Install to /usr/local/bin
sudo cp target/release/snix /usr/local/bin/

echo "snix installed successfully!" 