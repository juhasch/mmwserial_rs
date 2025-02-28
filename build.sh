#!/bin/bash
# Build and install the mmwserial package
set -e  # Exit on error

# Check for patchelf and install if missing
if ! command -v patchelf &> /dev/null; then
    echo "Installing patchelf..."
    pip install 'maturin[patchelf]'
fi

# Create Python package directory if it doesn't exist
mkdir -p python/mmwserial

# Build and install the package using maturin
echo "Building package with maturin..."
maturin develop

# Clean up any build artifacts
echo "Cleaning up build artifacts..."
find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
find . -type f -name "*.pyc" -delete 2>/dev/null || true

echo "Build completed successfully!"

# Install the Python package in development mode using system Python
python3 -m pip install -e python/ 