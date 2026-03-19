#!/bin/bash
# Setup script for TTS worker — creates a Python virtual environment
# and installs all required dependencies.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"

echo "=== TTS Worker Setup ==="

# Create venv if it doesn't exist
if [ ! -d "$VENV_DIR" ]; then
    echo "Creation de l'environnement virtuel..."
    python3 -m venv "$VENV_DIR"
fi

echo "Activation de l'environnement virtuel..."
source "$VENV_DIR/bin/activate"

echo "Installation des dependances..."
pip install --upgrade pip
pip install -r "$SCRIPT_DIR/requirements.txt"

echo ""
echo "=== Setup termine ==="
echo "Les moteurs TTS suivants sont installes :"
echo "  - Kokoro (leger, CPU)"
echo "  - Chatterbox (qualite, GPU recommande)"
echo "  - Qwen3-TTS (multilingue, GPU recommande)"
echo ""
echo "Les modeles seront telecharges automatiquement au premier usage."
