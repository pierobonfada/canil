#!/bin/bash
set -e
cd "$(dirname "$0")"

# Create venv if not exists
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi

# Activate and install requirements
source venv/bin/activate
pip install -r requirements.txt

# Run the python script
python create_demonstration.py
