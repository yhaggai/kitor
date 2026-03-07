#!/bin/bash

# Configuration
MISE_EXE="${1:-./MISE.exe}"
PATCH_DIR="."

# 0. Verify and auto-install xdelta3
if ! command -v xdelta3 &> /dev/null
then
    echo "xdelta3 not found. Attempting to install via Homebrew..."
    if command -v brew &> /dev/null; then
        brew install xdelta
    else
        echo "Error: xdelta3 is missing and Homebrew is not installed."
        echo "Please install Homebrew (https://brew.sh) or install xdelta3 manually."
        exit 1
    fi
fi

# 1. Verify source MISE.exe exists
if [ ! -f "$MISE_EXE" ]; then
    echo "Error: Standard Edition MISE.exe not found at: $MISE_EXE"
    echo "Please provide the path to MISE.exe as the first argument, e.g. ./patch.sh /path/to/MISE.exe"
    exit 1
fi

# 2. Create Hebrew directory if it doesn't exist
mkdir -p "$PATCH_DIR/Hebrew"

# 3. Apply patches using xdelta3
echo "Found xdelta3. Applying patch.000..."
xdelta3 -d -f -s "$MISE_EXE" "$PATCH_DIR/PATCHES/patch.000" "$PATCH_DIR/Hebrew/MONKEY.000"

echo "Applying patch.001..."
xdelta3 -d -f -s "$MISE_EXE" "$PATCH_DIR/PATCHES/patch.001" "$PATCH_DIR/Hebrew/MONKEY.001"

echo "--------------------------------------------------------"
echo "Done! The Hebrew files (MONKEY.000 and MONKEY.001) are in the 'Hebrew' folder."
echo "You can now add the 'Hebrew' folder to ScummVM to play."
