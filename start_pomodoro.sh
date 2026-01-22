#!/bin/bash
# Startup script for Charlie Kirk Pomodoro
# This script should be added to autostart

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Activate virtual environment if it exists
if [ -d "$SCRIPT_DIR/venv" ]; then
    source "$SCRIPT_DIR/venv/bin/activate"
fi

# Run the Charlie Kirk Pomodoro
python3 "$SCRIPT_DIR/src/charlie_kirk_pomodoro.py"
