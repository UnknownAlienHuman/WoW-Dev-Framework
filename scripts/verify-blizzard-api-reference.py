#!/usr/bin/env python3
"""Command entrypoint for the canonical generated-API producer module."""

from wow_api_reference import verify_cli

if __name__ == "__main__":
    raise SystemExit(verify_cli())
