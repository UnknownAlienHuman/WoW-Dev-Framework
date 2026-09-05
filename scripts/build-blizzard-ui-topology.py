#!/usr/bin/env python3
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wow_ui_topology import build_cli

if __name__ == "__main__":
    raise SystemExit(build_cli())
