#!/usr/bin/env python3
"""Resolve, safely update, and inspect the current EmmyLua analyzer source."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping, Sequence
from urllib.parse import urlsplit, urlunsplit

SCHEMA = "wow-dev-framework/emmylua-compatibility-report"
SCHEMA_VERSION = 1
PRODUCER_VERSION = 1
DEFAULT_REMOTE = "https://github.com/EmmyLuaLs/emmylua-analyzer-rust.git"
DEFAULT_BRANCH = "main"
MANAGED_MARKER = "wow-dev-framework-managed-upstream.json"
MAX_MANIFEST_BYTES = 4 * 1024 * 1024
MAX_SOURCE_FILE_BYTES = 16 * 1024 * 1024
PUBLIC_SYMBOL = re.compile(
    r"(?m)^\s*pub(?:\([^\n)]*\))?\s+"
    r"(?:async\s+|unsafe\s+|const\s+|extern\s+(?:\"[^\"]+\"\s+)?)?"
    r"(struct|enum|trait|type|fn|mod|const|static|union)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


class EmmySourceError(Exception):
    """Bounded upstream-management error with a stable code."""

    def __init__(self, code: str, message: str, *, details: Mapping[str, Any] | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = dict(details or {})

    def record(self) -> dict[str, Any]:
        return {"code": self.code, "message": self.message, "details": self.details}


def default_source_dir() -> Path:
    configured = os.environ.get("WOW_EMMY_SOURCE_DIR")
    if configured:
        return Path(configured).expanduser()
    if os.name == "nt":
        base = Path(os.environ.get("LOCALAPPDATA", Path.home() / "AppData" / "Local"))
    elif sys.platform == "darwin":
        base = Path.home() / "Library" / "Caches"
    else:
        base = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return base / "wow-dev-framework" / "upstreams" / "emmylua-analyzer-rust"


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")


def sha256_id(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _safe_remote(remote: str) -> str:
    value = remote.strip()
    if not value or "\x00" in value or "\n" in value or "\r" in value:
        raise EmmySourceError("remote_invalid", "upstream remote is empty or contains control characters")
    if re.match(r"^[^/@\s:]+@[^\s:]+:.+$", value):
        return value
    parsed = urlsplit(value)
    if parsed.scheme not in {"https", "ssh", "file"}:
        raise EmmySourceError("remote_scheme", "upstream remote must use https, ssh, file, or SCP-style SSH")
    if parsed.username or parsed.password:
        raise EmmySourceError("remote_credentials", "credentials must not be embedded in the upstream URL")
    if parsed.scheme != "file" and not parsed.hostname:
        raise EmmySourceError("remote_host", "upstream remote has no host")
    return value


def _normalize_remote(remote: str) -> str:
    value = _safe_remote(remote)
    if re.match(r"^[^/@\s:]+@[^\s:]+:.+$", value):
        user_host, path = value.split(":", 1)
        return f"{user_host.lower()}:{path.rstrip('/').removesuffix('.git')}"
    parsed = urlsplit(value)
    hostname = (parsed.hostname or "").lower()
    port = f":{parsed.port}" if parsed.port is not None else ""
    netloc = hostname + port
    path = parsed.path.rstrip("/").removesuffix(".git")
    return urlunsplit((parsed.scheme.lower(), netloc, path, "", ""))


def _run_git(
    source: Path | None,
    arguments: Sequence[str],
    *,
    check: bool = True,
    text: bool = True,
    hooks_dir: Path | None = None,
) -> subprocess.CompletedProcess[Any]:
    command = ["git"]
    if hooks_dir is not None:
        command.extend(["-c", f"core.hooksPath={hooks_dir}"])
    if source is not None:
        command.extend(["-C", os.fspath(source)])
    command.extend(arguments)
    try:
        result = subprocess.run(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            text=text,
        )
    except OSError as error:
        raise EmmySourceError("git_unavailable", "Git is not available") from error
    if check and result.returncode != 0:
        stderr = result.stderr if text else result.stderr.decode("utf-8", errors="replace")
        message = stderr.strip().splitlines()
        raise EmmySourceError(
            "git_failed",
            message[-1] if message else "Git command failed",
            details={"arguments": list(arguments), "returncode": result.returncode},
        )
    return result


def _git_text(source: Path, *arguments: str) -> str:
    return _run_git(source, arguments).stdout.strip()


def _git_bytes(source: Path, *arguments: str) -> bytes:
    return _run_git(source, arguments, text=False).stdout


def _is_repository(source: Path) -> bool:
    return source.is_dir() and _run_git(
        source,
        ["rev-parse", "--is-inside-work-tree"],
        check=False,
    ).returncode == 0


def _marker_path(source: Path) -> Path:
    git_dir = Path(_git_text(source, "rev-parse", "--git-dir"))
    if not git_dir.is_absolute():
        git_dir = source / git_dir
    return git_dir / MANAGED_MARKER


def _write_marker(source: Path, remote: str, branch: str) -> None:
    marker = _marker_path(source)
    marker.write_text(
        json.dumps(
            {"schema_version": 1, "remote": _normalize_remote(remote), "branch": branch},
            sort_keys=True,
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )


def _read_marker(source: Path) -> dict[str, Any] | None:
    marker = _marker_path(source)
    try:
        value = json.loads(marker.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return None
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EmmySourceError("marker_invalid", "managed-checkout marker is invalid") from error
    return value if isinstance(value, dict) else None


def _remote_head(remote: str, branch: str) -> str:
    result = _run_git(None, ["ls-remote", "--heads", remote, f"refs/heads/{branch}"])
    lines = [line for line in result.stdout.splitlines() if line.strip()]
    if len(lines) != 1:
        raise EmmySourceError("remote_branch_missing", f"remote branch {branch!r} was not resolved exactly")
    revision, ref = lines[0].split("\t", 1)
    if ref != f"refs/heads/{branch}" or not re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", revision):
        raise EmmySourceError("remote_response", "remote branch response is invalid")
    return revision.lower()


def _local_state(source: Path, remote: str, branch: str) -> dict[str, Any]:
    if not _is_repository(source):
        return {
            "exists": source.exists(),
            "repository": False,
            "state": "missing" if not source.exists() else "not_repository",
        }
    head = _git_text(source, "rev-parse", "HEAD").lower()
    tree = _git_text(source, "rev-parse", "HEAD^{tree}").lower()
    current_branch_result = _run_git(source, ["symbolic-ref", "--quiet", "--short", "HEAD"], check=False)
    current_branch = current_branch_result.stdout.strip() if current_branch_result.returncode == 0 else None
    origin_result = _run_git(source, ["remote", "get-url", "origin"], check=False)
    origin = origin_result.stdout.strip() if origin_result.returncode == 0 else None
    dirty = bool(_git_text(source, "status", "--porcelain=v1", "--untracked-files=normal"))
    marker = _read_marker(source)
    return {
        "exists": True,
        "repository": True,
        "state": "local",
        "head": head,
        "tree": tree,
        "branch": current_branch,
        "origin_matches": origin is not None and _normalize_remote(origin) == _normalize_remote(remote),
        "dirty": dirty,
        "managed": bool(
            marker
            and marker.get("remote") == _normalize_remote(remote)
            and marker.get("branch") == branch
        ),
    }


def status(source: Path, remote: str, branch: str, *, network: bool = True) -> dict[str, Any]:
    remote = _safe_remote(remote)
    local = _local_state(source, remote, branch)
    report: dict[str, Any] = {
        "source": os.fspath(source),
        "remote": remote,
        "branch": branch,
        "local": local,
        "network_checked": False,
        "remote_head": None,
        "relation": local["state"],
        "update_available": False,
        "safe_to_fast_forward": False,
    }
    if not network:
        if local.get("repository"):
            report["relation"] = "unverified_current"
        return report
    try:
        remote_head = _remote_head(remote, branch)
    except EmmySourceError as error:
        if local.get("repository"):
            report["relation"] = "unverified_current"
            report["network_error"] = error.record()
            return report
        raise
    report["network_checked"] = True
    report["remote_head"] = remote_head
    if not local.get("repository"):
        report["relation"] = "missing"
        report["update_available"] = True
        report["safe_to_fast_forward"] = not source.exists()
        return report
    if not local["origin_matches"]:
        report["relation"] = "wrong_origin"
        return report
    if local["branch"] != branch:
        report["relation"] = "wrong_branch"
        return report
    if local["dirty"]:
        report["relation"] = "dirty"
        return report
    local_head = local["head"]
    if local_head == remote_head:
        report["relation"] = "current"
        return report
    _run_git(source, ["fetch", "--no-tags", "origin", f"refs/heads/{branch}:refs/remotes/origin/{branch}"])
    local_is_ancestor = _run_git(
        source,
        ["merge-base", "--is-ancestor", local_head, remote_head],
        check=False,
    ).returncode == 0
    remote_is_ancestor = _run_git(
        source,
        ["merge-base", "--is-ancestor", remote_head, local_head],
        check=False,
    ).returncode == 0
    if local_is_ancestor:
        report["relation"] = "behind"
        report["update_available"] = True
        report["safe_to_fast_forward"] = True
    elif remote_is_ancestor:
        report["relation"] = "ahead"
    else:
        report["relation"] = "diverged"
    return report


def _clone(source: Path, remote: str, branch: str) -> None:
    if source.exists():
        raise EmmySourceError("clone_target_exists", "clone target already exists")
    source.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="wow-emmy-hooks-") as hooks:
        _run_git(
            None,
            [
                "clone",
                "--filter=blob:none",
                "--no-tags",
                "--single-branch",
                "--branch",
                branch,
                remote,
                os.fspath(source),
            ],
            hooks_dir=Path(hooks),
        )
    _write_marker(source, remote, branch)


def _fast_forward(source: Path, branch: str) -> None:
    with tempfile.TemporaryDirectory(prefix="wow-emmy-hooks-") as hooks:
        _run_git(
            source,
            ["merge", "--ff-only", f"refs/remotes/origin/{branch}"],
            hooks_dir=Path(hooks),
        )


def ensure(
    source: Path,
    remote: str,
    branch: str,
    *,
    update_policy: str,
    network: bool = True,
    interactive: bool | None = None,
) -> dict[str, Any]:
    if update_policy not in {"auto", "prompt", "never"}:
        raise EmmySourceError("update_policy", "update policy must be auto, prompt, or never")
    report = status(source, remote, branch, network=network)
    relation = report["relation"]
    should_update = False
    if relation == "missing":
        should_update = update_policy == "auto"
    elif relation == "behind" and report["safe_to_fast_forward"]:
        should_update = update_policy == "auto"
    if update_policy == "prompt" and relation in {"missing", "behind"}:
        if interactive is None:
            interactive = sys.stdin.isatty() and sys.stdout.isatty()
        if interactive:
            action = "clone" if relation == "missing" else "fast-forward"
            answer = input(f"{action} current EmmyLua upstream at {source}? [y/N] ").strip().casefold()
            should_update = answer in {"y", "yes"}
        else:
            report["prompt_required"] = True
    if should_update:
        if relation == "missing":
            _clone(source, remote, branch)
        else:
            _fast_forward(source, branch)
        report = status(source, remote, branch, network=network)
        report["updated"] = True
    else:
        report["updated"] = False
    if report["relation"] in {"not_repository", "wrong_origin", "wrong_branch", "dirty", "ahead", "diverged"}:
        report["blocked_reason"] = report["relation"]
    return report


def _read_exact(source: Path, revision: str, path: str, max_bytes: int) -> bytes:
    if (
        not path
        or "\\" in path
        or "\x00" in path
        or PurePosixPath(path).is_absolute()
        or any(part in {"", ".", ".."} for part in PurePosixPath(path).parts)
    ):
        raise EmmySourceError("source_path", "upstream source path is not canonical")
    header = _run_git(source, ["cat-file", "-s", f"{revision}:{path}"]).stdout.strip()
    try:
        size = int(header)
    except ValueError as error:
        raise EmmySourceError("source_size", "upstream source size is invalid") from error
    if size > max_bytes:
        raise EmmySourceError("source_size_limit", f"upstream source file {path!r} exceeds size limit")
    return _git_bytes(source, "show", f"{revision}:{path}")


def _parse_toml(data: bytes, path: str) -> dict[str, Any]:
    if len(data) > MAX_MANIFEST_BYTES:
        raise EmmySourceError("manifest_size_limit", f"upstream manifest {path!r} exceeds size limit")
    try:
        value = tomllib.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise EmmySourceError("manifest_invalid", f"upstream manifest {path!r} is invalid") from error
    if not isinstance(value, dict):
        raise EmmySourceError("manifest_shape", f"upstream manifest {path!r} is not a table")
    return value


def _manifest_paths(source: Path, revision: str) -> list[str]:
    output = _git_text(source, "ls-tree", "-r", "--name-only", revision)
    paths = [line for line in output.splitlines() if line == "Cargo.toml" or line.endswith("/Cargo.toml")]
    return sorted(paths, key=str.encode)


def _find_analysis_crate(source: Path, revision: str) -> tuple[str, dict[str, Any]]:
    matches: list[tuple[str, dict[str, Any]]] = []
    for path in _manifest_paths(source, revision):
        manifest = _parse_toml(_read_exact(source, revision, path, MAX_MANIFEST_BYTES), path)
        package = manifest.get("package")
        if isinstance(package, dict) and package.get("name") == "emmylua_code_analysis":
            matches.append((path, manifest))
    if len(matches) != 1:
        raise EmmySourceError(
            "analysis_crate",
            "expected exactly one emmylua_code_analysis package",
            details={"matches": [path for path, _manifest in matches]},
        )
    return matches[0]


def _public_surface(source: Path, revision: str, crate_root: PurePosixPath) -> dict[str, Any]:
    prefix = crate_root.as_posix().rstrip("/") + "/src/"
    paths = [
        path
        for path in _git_text(source, "ls-tree", "-r", "--name-only", revision).splitlines()
        if path.startswith(prefix) and path.endswith(".rs")
    ]
    symbols: list[dict[str, str]] = []
    files: list[dict[str, Any]] = []
    for path in sorted(paths, key=str.encode):
        data = _read_exact(source, revision, path, MAX_SOURCE_FILE_BYTES)
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise EmmySourceError("source_encoding", f"upstream Rust source {path!r} is not UTF-8") from error
        files.append({"path": path, "sha256": sha256_id(data), "bytes": len(data)})
        for kind, name in PUBLIC_SYMBOL.findall(text):
            symbols.append({"path": path, "kind": kind, "name": name})
    symbols.sort(key=lambda item: (item["name"].encode(), item["kind"], item["path"].encode()))
    return {
        "files": files,
        "symbols": symbols,
        "surface_sha256": sha256_id(canonical_json_bytes({"files": files, "symbols": symbols})),
    }


def build_report(
    source: Path,
    remote: str,
    branch: str,
    *,
    required_symbols: Sequence[str] = (),
    network: bool = True,
) -> dict[str, Any]:
    state = status(source, remote, branch, network=network)
    if not state["local"].get("repository"):
        raise EmmySourceError("source_missing", "EmmyLua upstream checkout is unavailable")
    revision = state["local"]["head"]
    tree = state["local"]["tree"]
    root_manifest = _parse_toml(
        _read_exact(source, revision, "Cargo.toml", MAX_MANIFEST_BYTES),
        "Cargo.toml",
    )
    crate_manifest_path, crate_manifest = _find_analysis_crate(source, revision)
    crate_root = PurePosixPath(crate_manifest_path).parent
    package = crate_manifest["package"]
    surface = _public_surface(source, revision, crate_root)
    available_symbols = {item["name"] for item in surface["symbols"]}
    missing_symbols = sorted(set(required_symbols) - available_symbols, key=str.encode)
    workspace = root_manifest.get("workspace") if isinstance(root_manifest.get("workspace"), dict) else {}
    workspace_package = (
        workspace.get("package") if isinstance(workspace.get("package"), dict) else {}
    )
    report: dict[str, Any] = {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "producer_version": PRODUCER_VERSION,
        "source": {
            "branch": branch,
            "revision": revision,
            "tree": tree,
            "relation": state["relation"],
            "remote_head": state.get("remote_head"),
            "network_checked": state["network_checked"],
        },
        "workspace": {
            "resolver": workspace.get("resolver"),
            "edition": workspace_package.get("edition"),
            "rust_version": workspace_package.get("rust-version"),
            "license": workspace_package.get("license"),
        },
        "analysis_crate": {
            "manifest_path": crate_manifest_path,
            "name": package.get("name"),
            "version": package.get("version"),
            "edition": package.get("edition"),
            "rust_version": package.get("rust-version"),
            "license": package.get("license"),
            "features": sorted(
                (crate_manifest.get("features") or {}).keys(), key=str.encode
            )
            if isinstance(crate_manifest.get("features"), dict)
            else [],
        },
        "surface": surface,
        "compatibility": {
            "required_symbols": sorted(set(required_symbols), key=str.encode),
            "missing_symbols": missing_symbols,
            "status": "compatible" if not missing_symbols else "incompatible",
        },
        "limitations": [
            "textual public-symbol inventory is a change detector, not a Rust semantic API proof",
            "the wow-emmy adapter must still compile and run its exact compatibility probes",
            "runtime analyzer behavior requires workspace fixtures and deterministic query tests",
        ],
    }
    report["report_sha256"] = sha256_id(canonical_json_bytes(report))
    return report


def verify_report(report: Mapping[str, Any], *, required_symbols: Sequence[str] = ()) -> None:
    if report.get("schema") != SCHEMA or report.get("schema_version") != SCHEMA_VERSION:
        raise EmmySourceError("report_schema", "unsupported EmmyLua compatibility report schema")
    supplied = report.get("report_sha256")
    if not isinstance(supplied, str):
        raise EmmySourceError("report_digest", "compatibility report has no digest")
    projection = dict(report)
    projection.pop("report_sha256", None)
    if supplied != sha256_id(canonical_json_bytes(projection)):
        raise EmmySourceError("report_digest", "compatibility report digest does not match content")
    source = report.get("source")
    if not isinstance(source, Mapping):
        raise EmmySourceError("report_source", "compatibility report has no source identity")
    for field in ("revision", "tree"):
        value = source.get(field)
        if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value):
            raise EmmySourceError("report_source", f"compatibility report {field} is invalid")
    compatibility = report.get("compatibility")
    if not isinstance(compatibility, Mapping):
        raise EmmySourceError("report_compatibility", "compatibility report has no result")
    report_required = compatibility.get("required_symbols")
    missing = compatibility.get("missing_symbols")
    if not isinstance(report_required, list) or not isinstance(missing, list):
        raise EmmySourceError("report_compatibility", "compatibility symbol lists are invalid")
    expected_required = sorted(set(required_symbols), key=str.encode)
    if required_symbols and report_required != expected_required:
        raise EmmySourceError("report_requirements", "compatibility report was built for different requirements")
    expected_status = "compatible" if not missing else "incompatible"
    if compatibility.get("status") != expected_status:
        raise EmmySourceError("report_compatibility", "compatibility status does not match missing symbols")


def write_json_atomic(path: Path, value: Mapping[str, Any]) -> None:
    data = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False).encode("utf-8") + b"\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", suffix=".tmp", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _print(value: Mapping[str, Any], json_output: bool) -> None:
    if json_output:
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
    else:
        relation = value.get("relation") or value.get("source", {}).get("relation")
        revision = value.get("local", {}).get("head") or value.get("source", {}).get("revision")
        print(f"{relation}: {revision or 'no local revision'}")


def cli(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Manage the current EmmyLua analyzer source")
    parser.add_argument("command", choices=("status", "ensure", "probe", "verify"))
    parser.add_argument("--source", type=Path, default=default_source_dir())
    parser.add_argument("--remote", default=DEFAULT_REMOTE)
    parser.add_argument("--branch", default=DEFAULT_BRANCH)
    parser.add_argument("--update", choices=("auto", "prompt", "never"), default="prompt")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--required-symbol", action="append", default=[])
    parser.add_argument("--json", action="store_true", dest="json_output")
    arguments = parser.parse_args(argv)
    try:
        remote = _safe_remote(arguments.remote)
        if arguments.command == "status":
            result = status(
                arguments.source,
                remote,
                arguments.branch,
                network=not arguments.offline,
            )
            _print(result, arguments.json_output)
            return 3 if result.get("update_available") else 0
        if arguments.command == "ensure":
            result = ensure(
                arguments.source,
                remote,
                arguments.branch,
                update_policy=arguments.update,
                network=not arguments.offline,
            )
            _print(result, arguments.json_output)
            return 0 if result["local"].get("repository") else 2
        if arguments.command == "probe":
            ensured = ensure(
                arguments.source,
                remote,
                arguments.branch,
                update_policy=arguments.update,
                network=not arguments.offline,
            )
            if not ensured["local"].get("repository"):
                raise EmmySourceError("source_missing", "EmmyLua checkout is unavailable")
            report = build_report(
                arguments.source,
                remote,
                arguments.branch,
                required_symbols=arguments.required_symbol,
                network=not arguments.offline,
            )
            verify_report(report, required_symbols=arguments.required_symbol)
            if arguments.output is not None:
                write_json_atomic(arguments.output, report)
            _print(report, arguments.json_output)
            return 0 if report["compatibility"]["status"] == "compatible" else 2
        if arguments.report is None:
            parser.error("verify requires --report")
        report, _raw = _load_report(arguments.report)
        verify_report(report, required_symbols=arguments.required_symbol)
        _print(report, arguments.json_output)
        return 0 if report["compatibility"]["status"] == "compatible" else 2
    except EmmySourceError as error:
        output = {"status": "error", "error": error.record()}
        print(
            json.dumps(output, ensure_ascii=False, sort_keys=True)
            if arguments.json_output
            else f"{error.code}: {error.message}",
            file=sys.stderr,
        )
        return 2


def _load_report(path: Path) -> tuple[dict[str, Any], bytes]:
    try:
        raw = path.read_bytes()
        value = json.loads(raw.decode("utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise EmmySourceError("report_json", "compatibility report is not valid UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise EmmySourceError("report_json", "compatibility report root must be an object")
    return value, raw


__all__ = [
    "EmmySourceError",
    "build_report",
    "canonical_json_bytes",
    "default_source_dir",
    "ensure",
    "sha256_id",
    "status",
    "verify_report",
    "write_json_atomic",
    "cli",
]
