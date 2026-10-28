#!/bin/bash

# Check if xdelta3 exists
if ! command -v xdelta3 &> /dev/null; then
    echo "xdelta3 not found. Installing via Homebrew..."

    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        echo "Error: Homebrew is not installed. Please install Homebrew first:"
        echo "Visit: https://brew.sh"
        exit 1
    fi

    # Install xdelta3
    brew install xdelta
else
    echo "xdelta3 found."
fi

# Create Hebrew directory if it doesn't exist
mkdir -p Hebrew

# Apply patches
echo "Applying patch.000..."
xdelta3 -d -s MISE.exe PATCHES/patch.000 Hebrew/MONKEY.000

echo "Applying patch.001..."
xdelta3 -d -s MISE.exe PATCHES/patch.001 Hebrew/MONKEY.001

echo "Done!"
