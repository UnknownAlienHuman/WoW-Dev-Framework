#!/usr/bin/env python3
"""Synchronize platform discovery copies from the canonical wow-dev skill."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tempfile
from pathlib import Path
from typing import NoReturn, Sequence

CANONICAL_PATH = Path(".agents/skills/wow-dev/SKILL.md")
TARGET_PATHS = (
    Path(".claude/skills/wow-dev/SKILL.md"),
    Path(".opencode/skills/wow-dev/SKILL.md"),
    Path(".agent/skills/wow-dev/SKILL.md"),
)
EXIT_ERROR = 2
EXIT_DRIFT = 3


class SkillSyncError(RuntimeError):
    """The canonical skill or a target path is unsafe or unreadable."""


def fail(message: str) -> NoReturn:
    raise SkillSyncError(message)


def validate_relative(path: Path) -> Path:
    if path.is_absolute() or not path.parts or any(part in {"", ".", ".."} for part in path.parts):
        fail(f"skill path must be canonical and repository-relative: {path}")
    return path


def read_canonical(root: Path) -> bytes:
    path = root / validate_relative(CANONICAL_PATH)
    try:
        content = path.read_bytes()
    except OSError as error:
        fail(f"cannot read canonical skill {CANONICAL_PATH}: {error}")
    if not content or len(content) > 1024 * 1024:
        fail("canonical skill must be non-empty and no larger than 1 MiB")
    try:
        text = content.decode("utf-8")
    except UnicodeDecodeError as error:
        fail(f"canonical skill is not UTF-8: {error}")
    if "wow-dev" not in text.lower() and "world of warcraft" not in text.lower():
        fail("canonical skill does not identify the WoW development workflow")
    return content


def atomic_write(path: Path, content: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
    )
    try:
        with os.fdopen(descriptor, "wb") as temporary:
            temporary.write(content)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_name, path)
    except BaseException:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def inspect(root: Path, canonical: bytes) -> list[dict[str, object]]:
    canonical_digest = hashlib.sha256(canonical).hexdigest()
    result: list[dict[str, object]] = []
    for relative in TARGET_PATHS:
        relative = validate_relative(relative)
        path = root / relative
        try:
            content = path.read_bytes()
        except FileNotFoundError:
            result.append(
                {
                    "path": relative.as_posix(),
                    "status": "missing",
                    "sha256": None,
                    "canonical_sha256": canonical_digest,
                }
            )
            continue
        except OSError as error:
            fail(f"cannot read skill adapter {relative}: {error}")
        digest = hashlib.sha256(content).hexdigest()
        result.append(
            {
                "path": relative.as_posix(),
                "status": "current" if content == canonical else "stale",
                "sha256": digest,
                "canonical_sha256": canonical_digest,
            }
        )
    return result


def synchronize(root: Path, canonical: bytes, status: list[dict[str, object]]) -> None:
    by_path = {entry["path"]: entry for entry in status}
    for relative in TARGET_PATHS:
        entry = by_path[relative.as_posix()]
        if entry["status"] != "current":
            atomic_write(root / relative, canonical)


def parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="repository root (default: inferred from this script)",
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="report drift without changing files")
    mode.add_argument("--write", action="store_true", help="atomically synchronize stale copies")
    parser.add_argument("--json", action="store_true", dest="json_output")
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    options = parse_arguments(arguments if arguments is not None else sys.argv[1:])
    try:
        root = options.root.expanduser().resolve()
        canonical = read_canonical(root)
        status = inspect(root, canonical)
        if options.write:
            synchronize(root, canonical, status)
            status = inspect(root, canonical)
        current = all(entry["status"] == "current" for entry in status)
        result = {
            "current": current,
            "canonical_path": CANONICAL_PATH.as_posix(),
            "canonical_sha256": hashlib.sha256(canonical).hexdigest(),
            "targets": status,
        }
        if options.json_output:
            print(json.dumps(result, sort_keys=True, separators=(",", ":")))
        else:
            for entry in status:
                print(f"{entry['status']}: {entry['path']}")
            if not current:
                print(
                    "wow-dev skill adapters are stale; run "
                    "python scripts/sync-wow-dev-skill.py --write",
                    file=sys.stderr,
                )
        return 0 if current else EXIT_DRIFT
    except (SkillSyncError, OSError, ValueError) as error:
        if options.json_output:
            print(json.dumps({"current": False, "error": str(error)}, sort_keys=True))
        else:
            print(f"error: {error}", file=sys.stderr)
        return EXIT_ERROR


if __name__ == "__main__":
    raise SystemExit(main())
