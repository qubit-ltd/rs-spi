#!/usr/bin/env python3
"""Validate downstream consumers against an external snapshot of qubit-spi."""
from __future__ import annotations

import argparse
import hashlib
import glob
import json
import os
import signal
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path

CONSUMERS = ("rs-fs-registry", "rs-mime", "rs-magika", "rs-event-bus")
SKIP_DIRS = {".git", "target", "__pycache__", ".pytest_cache"}


def _toml_files(root: Path):
    for current, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        if "Cargo.toml" in files:
            yield Path(current) / "Cargo.toml"


def _local_paths(data):
    def dependencies(table):
        if isinstance(table, dict):
            for value in table.values():
                if isinstance(value, dict) and isinstance(value.get("path"), str):
                    yield value["path"]
    for key in ("dependencies", "dev-dependencies", "build-dependencies"):
        yield from dependencies(data.get(key, {}))
    for target in data.get("target", {}).values():
        if isinstance(target, dict):
            for key in ("dependencies", "dev-dependencies", "build-dependencies"):
                yield from dependencies(target.get(key, {}))
    yield from dependencies(data.get("workspace", {}).get("dependencies", {}))
    for patch_table in data.get("patch", {}).values():
        yield from dependencies(patch_table)
    for replacement in data.get("replace", {}).values():
        if isinstance(replacement, dict) and isinstance(replacement.get("path"), str):
            yield replacement["path"]
    # Workspace members are paths by definition, even though Cargo TOML stores
    # them as strings rather than `{ path = ... }` dependency tables.
    workspace = data.get("workspace", {})
    for member in workspace.get("members", []):
        if isinstance(member, str):
            yield member


def _toml_inline_table_compat(text: str) -> str:
    """Fold TOML 1.1 multiline inline tables for stdlib tomllib parsers."""
    out = []
    depth = 0
    mode = None
    i = 0
    while i < len(text):
        char = text[i]
        if mode is not None:
            if mode in ("basic", "literal"):
                quote = '"' if mode == "basic" else "'"
                out.append(char)
                if mode == "basic" and char == "\\" and i + 1 < len(text):
                    i += 1
                    out.append(text[i])
                elif char == quote:
                    mode = None
                i += 1
                continue
            quote = '"' if mode == "multiline-basic" else "'"
            if mode == "multiline-basic" and char == "\\" and i + 1 < len(text):
                out.append(char)
                i += 1
                out.append(text[i])
                i += 1
                continue
            if text.startswith(quote * 3, i):
                out.append(quote * 3)
                i += 3
                mode = None
                continue
            out.append(char)
            i += 1
            continue

        if char == "#":
            # A comment inside an inline table ends at the physical newline. Drop
            # its text when folding that newline so it cannot comment out fields.
            if depth:
                while i < len(text) and text[i] not in "\r\n":
                    i += 1
                continue
            while i < len(text) and text[i] not in "\r\n":
                out.append(text[i])
                i += 1
            continue
        if char in ('"', "'"):
            if text.startswith(char * 3, i):
                out.append(char * 3)
                mode = "multiline-basic" if char == '"' else "multiline-literal"
                i += 3
            else:
                out.append(char)
                mode = "basic" if char == '"' else "literal"
                i += 1
            continue
        if char == "{":
            depth += 1
            out.append(char)
            i += 1
            continue
        if char == "}" and depth:
            # Python's stdlib TOML parser still implements TOML 1.0 and rejects
            # TOML 1.1 trailing commas in inline tables. Remove only the comma
            # immediately preceding this table's closing brace.
            previous = len(out) - 1
            while previous >= 0 and out[previous].isspace():
                previous -= 1
            if previous >= 0 and out[previous] == ",":
                del out[previous]
            depth -= 1
            out.append(char)
            i += 1
            continue
        if char in "\r\n" and depth:
            # Treat CRLF as one folded separator.
            if char == "\r" and i + 1 < len(text) and text[i + 1] == "\n":
                i += 1
            out.append(" ")
            i += 1
            continue
        out.append(char)
        i += 1
    return "".join(out)


def _parse_cargo_toml(text: str) -> dict:
    return tomllib.loads(_toml_inline_table_compat(text))


def discover_inputs(workspace_root: Path, consumer_roots: list[Path], spi_root: Path) -> list[Path]:
    """Return deduplicated Cargo package/workspace roots in the local path closure."""
    workspace_root = workspace_root.resolve(strict=True)
    queue = [p.resolve(strict=True) for p in [*consumer_roots, spi_root]]
    seen: set[Path] = set()
    while queue:
        root = queue.pop()
        if not root.is_relative_to(workspace_root):
            raise ValueError(f"input path escapes workspace root: {root}")
        if root in seen:
            continue
        if not root.is_dir():
            raise ValueError(f"missing input directory: {root}")
        seen.add(root)
        for manifest in _toml_files(root):
            try:
                data = _parse_cargo_toml(manifest.read_text(encoding="utf-8"))
            except (OSError, tomllib.TOMLDecodeError) as exc:
                raise ValueError(f"cannot parse {manifest}: {exc}") from exc
            for rel in _local_paths(data):
                candidate = manifest.parent / rel
                matches = [Path(x) for x in glob.glob(str(candidate))] if any(c in rel for c in "*?[") else [candidate]
                if not matches:
                    raise ValueError(f"missing Cargo path dependency or workspace member {rel!r} in {manifest}")
                for match in matches:
                    if not match.exists():
                        raise ValueError(f"missing Cargo path dependency {rel!r} in {manifest}")
                    cursor = match.absolute()
                    while cursor != cursor.parent:
                        if cursor.is_symlink():
                            raise ValueError(f"Cargo path dependency traverses symlink: {cursor} -> {cursor.resolve(strict=False)}")
                        if cursor == workspace_root:
                            break
                        cursor = cursor.parent
                    resolved = match.resolve(strict=True)
                    if not resolved.is_relative_to(workspace_root):
                        raise ValueError(f"Cargo path dependency escapes workspace root: {match} -> {resolved}")
                    queue.append(resolved if resolved.is_dir() else resolved.parent)
    return sorted(seen)


def copy_tree_safe(source: Path, destination: Path, input_root: Path) -> None:
    source = source.resolve(strict=True)
    input_root = input_root.resolve(strict=True)
    for current, dirs, files in os.walk(source, followlinks=False):
        current_path = Path(current)
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in list(dirs) + files:
            path = current_path / name
            if path.is_symlink():
                target = path.resolve(strict=False)
                if target == input_root or target.is_relative_to(input_root):
                    raise ValueError(f"source-tree symlink rejected: {path} -> {target}")
                raise ValueError(f"symlink rejected for safe snapshot: {path} -> {target}")
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns(*SKIP_DIRS), symlinks=True)


def assert_spi_origin(metadata: dict, expected_manifest: Path) -> None:
    packages = {p["id"]: p for p in metadata.get("packages", [])}
    resolve = metadata.get("resolve") or {}
    nodes = {n["id"]: n for n in resolve.get("nodes", [])}
    root_id = resolve.get("root")
    if not root_id or root_id not in nodes:
        raise ValueError("Cargo metadata has no resolved root graph")
    reached, pending = set(), [root_id]
    while pending:
        node_id = pending.pop()
        if node_id in reached:
            continue
        reached.add(node_id)
        node = nodes.get(node_id)
        if node is None:
            raise ValueError(f"resolve graph references missing node {node_id}")
        for dep in node.get("dependencies", []):
            pending.append(dep if isinstance(dep, str) else dep.get("pkg"))
    spi = [packages[i] for i in reached if i in packages and packages[i].get("name") == "qubit-spi"]
    expected = str(expected_manifest.resolve())
    if len(spi) != 1:
        raise ValueError(f"expected exactly one reachable qubit-spi package; found {len(spi)}")
    actual = str(Path(spi[0]["manifest_path"]).resolve())
    if actual != expected or spi[0].get("source") is not None:
        raise ValueError(f"qubit-spi source mismatch: manifest_path={actual!r}, source={spi[0].get('source')!r}; expected {expected!r}, source=null")


@dataclass
class CommandResult:
    args: list[str]
    cwd: str
    returncode: int | None
    timed_out: bool
    elapsed: float
    log: str
    log_path: str = ""
    stdout: str = ""
    process_group: int | None = None
    cleanup_status: str = "not-started"
    cleanup_error: str | None = None


def _require_posix_process_groups():
    if os.name != "posix" or not hasattr(os, "killpg") or not hasattr(signal, "SIGKILL"):
        raise RuntimeError("downstream validation requires POSIX process-group support")
    if not Path("/proc").is_dir():
        try:
            _process_group_states_from_ps()
        except (OSError, subprocess.SubprocessError, RuntimeError) as exc:
            raise RuntimeError(
                "downstream validation requires /proc or a working POSIX ps process listing"
            ) from exc


def _require_command_cleanup(result, phase):
    status = getattr(result, "cleanup_status", "complete")
    # Injected runners used by tests and callers do not own subprocess groups.
    if status == "not-started" and getattr(result, "process_group", None) is None:
        status = "complete"
    if status != "complete":
        raise ValueError(
            f"{phase} process cleanup failed for {getattr(result, 'args', '<unknown command>')} "
            f"(see {getattr(result, 'log_path', '<unknown log>')}): "
            f"{getattr(result, 'cleanup_error', None)}"
        )


def _process_group_states_from_ps():
    """Return (process-group ID, state) rows from the POSIX ps utility."""
    ps = shutil.which("ps")
    if not ps:
        raise RuntimeError("cannot verify process-group cleanup: /proc and ps are unavailable")
    completed = subprocess.run([ps, "-e", "-o", "pgid=,state="], capture_output=True,
                               text=True, timeout=5, check=True)
    rows = []
    for line in completed.stdout.splitlines():
        fields = line.split(maxsplit=1)
        if len(fields) != 2:
            continue
        try:
            group = int(fields[0])
        except ValueError:
            continue
        rows.append((group, fields[1].lstrip()))
    if not rows:
        raise RuntimeError("ps returned no parseable process-group states")
    return rows


def _group_has_live_members_from_ps(pgid):
    # Other POSIX systems may also retain zombie-only groups. Use the system
    # process listing to distinguish those from members that can still run.
    return any(group == pgid and not state.startswith("Z")
               for group, state in _process_group_states_from_ps())


def _group_exists(pgid):
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        raise
    # Linux can retain a process group containing only zombies. They cannot
    # execute, so treat that state as clean after the direct child is reaped.
    proc_root = Path("/proc")
    if proc_root.is_dir():
        for entry in proc_root.iterdir():
            if not entry.name.isdigit():
                continue
            try:
                stat = (entry / "stat").read_text()
                fields = stat[stat.rfind(")") + 2:].split()
                state, group = fields[0], int(fields[2])
                if group == pgid and state != "Z":
                    return True
            except (OSError, ValueError, IndexError):
                continue
        return False
    return _group_has_live_members_from_ps(pgid)


def _cleanup_process_group(proc, pgid):
    """Stop all remaining members, reap the root, and confirm no live member remains."""
    try:
        if _group_exists(pgid):
            try:
                os.killpg(pgid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            deadline = time.monotonic() + 2.0
            while time.monotonic() < deadline and _group_exists(pgid):
                time.sleep(min(0.05, max(0, deadline - time.monotonic())))
            if _group_exists(pgid):
                try:
                    os.killpg(pgid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
        proc.wait(timeout=2.0)
        deadline = time.monotonic() + 2.0
        while _group_exists(pgid) and time.monotonic() < deadline:
            time.sleep(0.05)
        if _group_exists(pgid):
            raise RuntimeError(f"process group {pgid} still has running members")
        return "complete", None
    except (OSError, subprocess.SubprocessError, RuntimeError) as exc:
        return "failed", str(exc)


def _run(argv, cwd, env, timeout, log_path):
    _require_posix_process_groups()
    start = time.monotonic()
    stdout = stderr = ""
    code = None
    timed_out = False
    pgid = None
    cleanup_status, cleanup_error = "not-started", None
    proc = None
    execution_error = None
    log_path = Path(log_path)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryFile(mode="w+b") as stdout_file, tempfile.TemporaryFile(mode="w+b") as stderr_file:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=stdout_file, stderr=stderr_file,
                                start_new_session=True)
        pgid = proc.pid
        try:
            code = proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
        except BaseException as exc:
            execution_error = exc
        finally:
            cleanup_status, cleanup_error = _cleanup_process_group(proc, pgid)
        stdout_file.seek(0); stderr_file.seek(0)
        stdout = stdout_file.read().decode("utf-8", "replace")
        stderr = stderr_file.read().decode("utf-8", "replace")
    output = stdout + ("\n" if stdout and stderr else "") + stderr
    log_path.write_text(output, encoding="utf-8", errors="replace")
    if execution_error is not None:
        raise execution_error
    return CommandResult(list(argv), str(cwd), None if timed_out else code, timed_out,
                         time.monotonic() - start, output, str(log_path), stdout,
                         pgid, cleanup_status, cleanup_error)


def _head_dirty(root: Path):
    try:
        head = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True,
                              text=True, timeout=10, check=True).stdout.strip()
        status = subprocess.run(["git", "-C", str(root), "status", "--porcelain"], capture_output=True,
                                text=True, timeout=10, check=True).stdout
        return head, bool(status.strip())
    except (subprocess.SubprocessError, OSError):
        return None, None


def _tree_hash(root: Path):
    h = hashlib.sha256()
    file_records = []
    for current, dirs, names in os.walk(root, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
        symlink_dirs = [name for name in dirs if (Path(current) / name).is_symlink()]
        dirs[:] = [name for name in dirs if name not in symlink_dirs]
        for name in sorted([*names, *symlink_dirs]):
            p = Path(current) / name
            rel = p.relative_to(root).as_posix()
            h.update(rel.encode()); h.update(b"\0")
            if p.is_symlink():
                content = b"link:" + os.readlink(p).encode()
            else:
                content = p.read_bytes()
            digest = hashlib.sha256(content).hexdigest()
            file_records.append({"path": rel, "sha256": digest})
            h.update(content)
            h.update(b"\0")
    return h.hexdigest(), file_records


def run(workspace_root: Path, spi_root: Path, report: Path, command_runner=None) -> int:
    _require_posix_process_groups()
    workspace_root, spi_root = workspace_root.resolve(strict=True), spi_root.resolve(strict=False)
    consumers = [workspace_root / name for name in CONSUMERS]
    report = report.resolve(strict=False)
    inputs = [workspace_root, spi_root, *consumers]
    if any(report == p or report.is_relative_to(p) for p in inputs):
        raise ValueError("report must be outside every input repository")
    report.parent.mkdir(parents=True, exist_ok=True)
    result = {"version": 1, "success": False, "inputs": {}, "commands": [], "consumers": [], "errors": []}
    try:
        for path in [spi_root, *consumers]:
            if not path.is_dir():
                raise ValueError(f"missing required consumer or SPI path: {path}")
        closure = discover_inputs(workspace_root, consumers, spi_root)
        result["inputs"] = {}
        for p in closure:
            head, dirty = _head_dirty(p)
            digest, files = _tree_hash(p)
            result["inputs"][str(p)] = {"head": head, "dirty": dirty, "content_sha256": digest, "files": files}
        tmp = Path(tempfile.mkdtemp(prefix="rs-spi-downstream-"))
        if any(tmp == p or tmp.is_relative_to(p) or p.is_relative_to(tmp) for p in inputs):
            shutil.rmtree(tmp)
            raise ValueError(f"temporary root overlaps an input repository: {tmp}")
        result["temporary_directory"] = str(tmp)
        snap = tmp / "snapshot"
        copied = {}
        try:
            copy_roots = [p for p in closure if not any(p != q and p.is_relative_to(q) for q in closure)]
            for source in copy_roots:
                destination = snap / source.relative_to(workspace_root)
                destination.parent.mkdir(parents=True, exist_ok=True)
                copy_tree_safe(source, destination, workspace_root)
            copied = {source: snap / source.relative_to(workspace_root) for source in closure}
            spi_copy = copied[spi_root]
            cargo_home, target_dir = tmp / "cargo-home", tmp / "target"
            cargo_home.mkdir()
            (cargo_home / "config.toml").write_text(
                "[patch.crates-io]\nqubit-spi = { path = " + json.dumps(str(spi_copy)) + " }\n", encoding="utf-8")
            env = {k: v for k, v in os.environ.items() if k not in {"CARGO_HOME", "CARGO_TARGET_DIR"}}
            env.update({"CARGO_HOME": str(cargo_home), "CARGO_TARGET_DIR": str(target_dir)})
            versions = {}
            for tool, args in (("cargo", ["cargo", "--version"]), ("rustc", ["rustc", "--version"])):
                try:
                    version = subprocess.run(args, cwd=snap, env=env, text=True, stdout=subprocess.PIPE,
                                             stderr=subprocess.STDOUT, timeout=30, check=True).stdout.strip()
                except (OSError, subprocess.SubprocessError) as exc:
                    raise ValueError(f"cannot record {tool} version: {exc}") from exc
                versions[tool] = version
            result["toolchain"] = versions
            real_cargo = shutil.which("cargo", path=env.get("PATH"))
            if not real_cargo:
                raise ValueError("cargo executable not found in PATH")
            wrapper_dir = tmp / "cargo-wrapper"
            wrapper_dir.mkdir()
            wrapper = wrapper_dir / "cargo"
            wrapper.write_text("""#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys, time, hashlib
real = os.environ['RS_SPI_REAL_CARGO']
args = sys.argv[1:]
manifests = json.loads(os.environ['RS_SPI_ROOT_MANIFESTS'])
manifest = pathlib.Path(args[args.index('--manifest-path') + 1]).resolve() if '--manifest-path' in args else (pathlib.Path.cwd() / 'Cargo.toml').resolve()
nested = manifest.exists() and str(manifest) not in manifests and not os.environ.get('RS_SPI_BYPASS')
record_path = os.environ['RS_SPI_FIXTURE_RECORD']
log_dir = os.environ['RS_SPI_FIXTURE_LOGDIR']
def emit(record):
    with open(record_path, 'a', encoding='utf-8') as f:
        f.write(json.dumps(record) + '\\n')
def log_file(tag):
    return os.path.join(log_dir, tag + '-' + str(time.time_ns()) + '.log')
if nested:
    metadata_cmd = [real, '--config', os.environ['RS_SPI_PATCH_CONFIG'], 'metadata', '--format-version', '1', '--manifest-path', str(manifest)]
    feature_flags = {'--all-features', '--no-default-features'}
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in feature_flags:
            metadata_cmd.append(arg)
        elif arg in ('--features', '-F') and i + 1 < len(args):
            metadata_cmd.extend([arg, args[i + 1]])
            i += 1
        elif arg.startswith('--features=') or (arg.startswith('-F') and len(arg) > 2):
            metadata_cmd.append(arg)
        i += 1
    start = time.monotonic(); timed_out = False
    try:
        meta = subprocess.run(metadata_cmd, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        meta = type('TimedOut', (), {'returncode': 124, 'stdout': exc.stdout or '', 'stderr': str(exc)})()
    if isinstance(meta.stdout, bytes): meta.stdout = meta.stdout.decode('utf-8', 'replace')
    metadata_log = log_file('nested-metadata')
    with open(metadata_log, 'w', encoding='utf-8') as f: f.write(meta.stdout + meta.stderr)
    emit({'kind':'metadata-preflight','manifest_path':str(manifest),'argv':metadata_cmd,'cwd':os.getcwd(),'returncode':meta.returncode,'timed_out':timed_out,'elapsed_seconds':time.monotonic()-start,'log_path':metadata_log,'metadata':meta.stdout})
    if meta.returncode: sys.exit(meta.returncode)
start = time.monotonic(); timed_out = False
try:
    proc = subprocess.run([real, *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=1800)
except subprocess.TimeoutExpired as exc:
    timed_out = True
    proc = type('TimedOut', (), {'returncode':124,'stdout':exc.stdout or '', 'stderr':str(exc)})()
if isinstance(proc.stdout, bytes): proc.stdout = proc.stdout.decode('utf-8', 'replace')
if nested:
    cargo_log = log_file('nested-cargo')
    with open(cargo_log, 'w', encoding='utf-8') as f: f.write(proc.stdout + proc.stderr)
    emit({'kind':'cargo','manifest_path':str(manifest),'argv':[real,*args],'cwd':os.getcwd(),'returncode':proc.returncode,'timed_out':timed_out,'elapsed_seconds':time.monotonic()-start,'log_path':cargo_log})
sys.stdout.write(proc.stdout); sys.stderr.write(proc.stderr)
sys.exit(proc.returncode)
""", encoding="utf-8")
            wrapper.chmod(0o755)
            original_path = env.get("PATH", os.defpath)
            env["PATH"] = str(wrapper_dir) + os.pathsep + original_path
            env["CARGO"] = str(wrapper.resolve())
            env["RS_SPI_CARGO_WRAPPER"] = str(wrapper.resolve())
            env["RS_SPI_REAL_CARGO"] = real_cargo
            env["RS_SPI_PATCH_CONFIG"] = f"patch.crates-io.qubit-spi.path={json.dumps(str(spi_copy))}"
            env["RS_SPI_EXPECTED_MANIFEST"] = str((spi_copy / "Cargo.toml").resolve())
            runner = command_runner or _run
            for name, source in zip(CONSUMERS, consumers):
                root = copied[source]
                entry = {"name": name, "manifest_path": str(root / "Cargo.toml"), "modes": []}
                result["consumers"].append(entry)
                for mode, feature_args in (("default", []), ("all-features", ["--all-features"])):
                    mode_report = {"mode": mode, "metadata": None, "spi": None, "test": None, "executed_fixtures": []}
                    entry["modes"].append(mode_report)
                    override = ["--config", f"patch.crates-io.qubit-spi.path={json.dumps(str(spi_copy))}"]
                    actual_fixture_record = tmp / "logs" / name / mode / "actual-fixtures.jsonl"
                    log = tmp / "logs" / name / mode
                    log.mkdir(parents=True, exist_ok=True)
                    env["RS_SPI_FIXTURE_RECORD"] = str(actual_fixture_record)
                    env["RS_SPI_FIXTURE_LOGDIR"] = str(log)
                    env["RS_SPI_ROOT_MANIFESTS"] = json.dumps([str((copied[consumer_source] / "Cargo.toml").resolve()) for consumer_source in consumers])
                    common = ["cargo"]
                    meta_argv = common + override + ["metadata", "--format-version", "1", "--manifest-path", str(root / "Cargo.toml"), *feature_args]
                    mr = runner(meta_argv, root, env, 180, log / "metadata.log")
                    item = {"argv": mr.args, "cwd": mr.cwd, "returncode": mr.returncode, "timed_out": mr.timed_out, "elapsed_seconds": mr.elapsed, "log_path": mr.log_path,
                            "process_group": getattr(mr, "process_group", None),
                            "cleanup_status": getattr(mr, "cleanup_status", "complete"),
                            "cleanup_error": getattr(mr, "cleanup_error", None)}
                    result["commands"].append(item); mode_report["metadata"] = item
                    _require_command_cleanup(mr, f"metadata {name}/{mode}")
                    if mr.returncode != 0 or mr.timed_out:
                        raise ValueError(f"metadata failed for {name}/{mode} (see {mr.log_path})")
                    try:
                        md = json.loads(mr.stdout or mr.log)
                        assert_spi_origin(md, spi_copy / "Cargo.toml")
                        spi_pkg = next(p for p in md["packages"] if p["name"] == "qubit-spi" and p["manifest_path"] == str((spi_copy / "Cargo.toml").resolve()))
                        mode_report["spi"] = {"manifest_path": spi_pkg["manifest_path"], "source": spi_pkg["source"]}
                    except (ValueError, json.JSONDecodeError, StopIteration) as exc:
                        raise ValueError(f"SPI origin check failed for {name}/{mode}: {exc}") from exc
                    lockfile = root / "Cargo.lock"
                    lock_hash = hashlib.sha256(lockfile.read_bytes()).hexdigest() if lockfile.exists() else None
                    test_argv = common + override + ["test", "--locked", "--manifest-path", str(root / "Cargo.toml"), *feature_args]
                    tr = runner(test_argv, root, env, 1800, log / "test.log")
                    item = {"argv": tr.args, "cwd": tr.cwd, "returncode": tr.returncode, "timed_out": tr.timed_out, "elapsed_seconds": tr.elapsed, "log_path": tr.log_path,
                            "process_group": getattr(tr, "process_group", None),
                            "cleanup_status": getattr(tr, "cleanup_status", "complete"),
                            "cleanup_error": getattr(tr, "cleanup_error", None)}
                    result["commands"].append(item); mode_report["test"] = item
                    _require_command_cleanup(tr, f"test {name}/{mode}")
                    if actual_fixture_record.exists():
                        for line in actual_fixture_record.read_text(encoding="utf-8").splitlines():
                            record = json.loads(line)
                            item = {key: record[key] for key in ("argv", "cwd", "returncode", "timed_out", "elapsed_seconds", "log_path")}
                            result["commands"].append(item)
                            if record["kind"] == "metadata-preflight":
                                if record["returncode"] != 0 or record["timed_out"]:
                                    raise ValueError(f"executed nested Cargo metadata failed for {record['manifest_path']} (see {record['log_path']})")
                                try:
                                    fixture_md = json.loads(record["metadata"])
                                    assert_spi_origin(fixture_md, spi_copy / "Cargo.toml")
                                    fixture_spi = next(p for p in fixture_md["packages"] if p["name"] == "qubit-spi" and p["manifest_path"] == str((spi_copy / "Cargo.toml").resolve()))
                                except (ValueError, json.JSONDecodeError) as exc:
                                    raise ValueError(f"executed nested Cargo SPI origin check failed for {record['manifest_path']}: {exc}") from exc
                                except StopIteration as exc:
                                    raise ValueError(f"executed nested Cargo metadata omitted snapshot qubit-spi for {record['manifest_path']}") from exc
                                mode_report.setdefault("executed_fixtures", []).append({"manifest_path": record["manifest_path"], "spi_manifest_path": fixture_spi["manifest_path"], "source": fixture_spi["source"]})
                    if tr.returncode != 0 or tr.timed_out:
                        raise ValueError(f"cargo test failed for {name}/{mode} (see {tr.log_path})")
                    current_lock_hash = hashlib.sha256(lockfile.read_bytes()).hexdigest() if lockfile.exists() else None
                    if current_lock_hash != lock_hash:
                        raise ValueError(f"snapshot lockfile changed during locked test for {name}/{mode}: {lockfile}")
            result["success"] = True
        finally:
            if result["success"]:
                result["cleanup"] = {"status": "pending", "temporary_directory": str(tmp)}
                report.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
                try:
                    shutil.rmtree(tmp)
                    result["cleanup"] = {"status": "removed", "temporary_directory": str(tmp)}
                except OSError as exc:
                    result["success"] = False
                    result["errors"].append(f"could not clean successful snapshot {tmp}: {exc}")
                    result["cleanup"] = {"status": "failed", "temporary_directory": str(tmp)}
                    result["temporary_directory"] = str(tmp)
            else:
                result["temporary_directory"] = str(tmp)
    except Exception as exc:
        result["errors"].append(str(exc))
    report.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if result["success"] else 1


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace-root", required=True, type=Path)
    parser.add_argument("--spi-root", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args(argv)
    try:
        code = run(args.workspace_root, args.spi_root, args.report)
    except Exception as exc:
        print(f"check-downstream: {exc}", file=sys.stderr)
        return 1
    if code:
        try:
            doc = json.loads(args.report.read_text())
            for error in doc.get("errors", []):
                print(f"check-downstream: {error}", file=sys.stderr)
        except OSError:
            pass
    return code


if __name__ == "__main__":
    raise SystemExit(main())
