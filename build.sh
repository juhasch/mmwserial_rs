#!/bin/bash
# Build and install the mmwserial package

# Build the Rust package
cargo build --release

# Create Python package directory if it doesn't exist
mkdir -p python/mmwserial

# Copy the compiled library to the Python package
cp target/release/libmmwserial.so python/mmwserial/mmwserial.so

# Install the Python package in development mode using system Python
python3 -m pip install -e python/ 