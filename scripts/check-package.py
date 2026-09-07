#!/usr/bin/env python3
"""Build a real Cargo release archive and verify it without the checkout."""
import argparse
import json
import os
from pathlib import Path, PurePosixPath
import subprocess
import shutil
import sys
import tarfile
import tempfile


def validate_member(member: tarfile.TarInfo, crate: str) -> None:
    """Accept only regular files/directories inside the expected crate root."""
    path = PurePosixPath(member.name)
    if (path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != crate
            or '\\' in member.name or not (member.isfile() or member.isdir())):
        raise ValueError(f'unsafe archive member: {member.name}')


def extract_archive(archive: tarfile.TarFile, destination: Path, crate: str) -> None:
    """Validate all members before extracting regular data on Python 3.11+."""
    members = archive.getmembers()
    for member in members:
        validate_member(member, crate)
    for member in members:
        target = destination / member.name
        if member.isdir():
            target.mkdir(parents=True, exist_ok=True)
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, target.open('xb') as output:
                shutil.copyfileobj(source, output)


def run(command: list[str], root: Path, env: dict) -> None:
    """Run a bounded subprocess, retaining its diagnostics and exit status."""
    print(f'RUN {command} in {root}', flush=True)
    subprocess.run(command, cwd=root, env=env, timeout=600, check=True)


def check_package(root: Path, allow_dirty: bool) -> None:
    """Package, safely extract, and test the actual publishable files."""
    env = os.environ.copy()
    for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS',
                'LLVM_PROFILE_FILE', 'CARGO_LLVM_COV_TARGET_DIR'):
        env.pop(key, None)
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1'],
                                                 cwd=root, env=env, text=True, timeout=60))
    package = next(package for package in metadata['packages'] if Path(package['manifest_path']).parent == root)
    crate = f'{package["name"]}-{package["version"]}'
    command = ['cargo', 'package', '--locked']
    if allow_dirty:
        command.append('--allow-dirty')
    run(command, root, env)
    archive_file = Path(metadata['target_directory']) / 'package' / f'{crate}.crate'
    with tempfile.TemporaryDirectory(prefix='spi-package-') as temporary:
        destination = Path(temporary)
        with tarfile.open(archive_file) as archive:
            extract_archive(archive, destination, crate)
        extracted = destination / crate
        env['CARGO_TARGET_DIR'] = str(destination / 'target')
        run(['cargo', 'test', '--locked', '--all-targets', '--all-features', '--no-run'], extracted, env)
        run(['cargo', 'test', '--locked', '--all-features'], extracted, env)
        run([sys.executable, 'scripts/check-documentation.py'], extracted, env)
    print(f'PASS release archive {crate}; allow_dirty={allow_dirty}', flush=True)


def main() -> None:
    """Expose explicit dirty-tree opt-in without changing Cargo's default policy."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--allow-dirty', action='store_true')
    args = parser.parse_args()
    try:
        check_package(Path(__file__).resolve().parents[1], args.allow_dirty)
    except (ValueError, OSError, subprocess.SubprocessError, tarfile.TarError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
