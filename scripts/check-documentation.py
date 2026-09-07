#!/usr/bin/env python3
"""Compile and run the exact Rust blocks shipped in both document languages."""
import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tempfile


@dataclass(frozen=True)
class ExampleBlock:
    """A source block with its original diagnostic location."""
    scenario: str
    file: PurePosixPath
    code: str
    document: str
    first_code_line: int


def validate_relative_file(value: str) -> PurePosixPath:
    """Reject ambiguous or escaping paths before any temporary file is written."""
    path = PurePosixPath(value)
    if not value or '\\' in value or path.is_absolute() or '..' in path.parts or not path.parts:
        raise ValueError(f'invalid scenario file: {value!r}')
    return path


def extract_blocks(text: str, document: str) -> list[ExampleBlock]:
    """Extract marked Rust fences, rejecting incomplete or exempted examples."""
    blocks = []
    marker = None
    fence = None
    code = []
    rust = False
    start = 0
    for number, line in enumerate(text.splitlines(keepends=True), 1):
        stripped = line.strip()
        if fence is not None:
            if re.fullmatch(re.escape(fence[0]) + '{' + str(len(fence)) + ',}', stripped):
                if rust:
                    scenario, file = marker
                    blocks.append(ExampleBlock(scenario, validate_relative_file(file), ''.join(code), document, start))
                fence, marker = None, None
            else:
                code.append(line)
            continue
        match = re.fullmatch(r'<!-- spi-example: ([\w-]+); file: ([^;]+) -->', stripped)
        if match:
            if marker:
                raise ValueError(f'{document}:{number}: unused example marker')
            marker = match.groups()
            continue
        opening = re.fullmatch(r'(`{3,}|~{3,})(.*)', stripped)
        if opening:
            fence, language = opening.groups()
            language = language.strip()
            rust = re.match(r'rust(?:\s|,|$)', language) is not None
            if rust and (marker is None or language != 'rust'):
                raise ValueError(f'{document}:{number}: Rust fence requires a marker and no exemptions')
            if marker and not rust:
                raise ValueError(f'{document}:{number}: example marker requires a Rust fence')
            code, start = [], number + 1
        elif marker:
            raise ValueError(f'{document}:{number}: marker must immediately precede its Rust fence')
    if fence is not None or marker is not None:
        raise ValueError(f'{document}: unclosed fence or unused marker')
    return blocks


def validate_blocks(blocks: list[ExampleBlock], expected: dict, document: str) -> None:
    """Require every declared example file exactly once, including whole scenarios."""
    actual = [(block.scenario, str(block.file)) for block in blocks]
    required = {(scenario, str(validate_relative_file(file))) for scenario, files in expected.items() for file in files}
    if len(actual) != len(set(actual)) or set(actual) != required:
        raise ValueError(f'{document}: example inventory mismatch; expected {sorted(required)}, got {actual}')


def cargo_environment(target: Path) -> dict:
    """Isolate example builds from parent coverage instrumentation and target locks."""
    env = os.environ.copy()
    for key in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTDOCFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS',
                'LLVM_PROFILE_FILE', 'CARGO_LLVM_COV_TARGET_DIR'):
        env.pop(key, None)
    env['CARGO_TARGET_DIR'] = str(target)
    return env


def run_scenario(root: Path, blocks: list[ExampleBlock], scenario: dict, workspace: Path) -> None:
    """Materialize one document's example and check its real Cargo output."""
    workspace.mkdir(parents=True)
    for name, content in scenario['manifests'].items():
        file = workspace / validate_relative_file(name)
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(content.replace('"$SPI_ROOT"', json.dumps(str(root))), encoding='utf-8')
    for block in blocks:
        file = workspace / block.file
        if file.exists():
            raise ValueError(f'{block.document}: source overwrites manifest: {block.file}')
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(block.code, encoding='utf-8')
    command = ['cargo', 'run', '--quiet', '-p', 'app']
    result = subprocess.run(command, cwd=workspace, env=cargo_environment(workspace / 'target'),
                            timeout=180, capture_output=True, text=True)
    locations = ', '.join(f'{b.document}:{b.first_code_line}' for b in blocks)
    if result.returncode or result.stdout != scenario['stdout']:
        raise ValueError(f'{locations}: {blocks[0].scenario}: {command} exited {result.returncode}\n'
                         f'stdout={result.stdout!r}; expected={scenario["stdout"]!r}\n{result.stderr}')
    print(f'PASS {locations} ({blocks[0].scenario})', flush=True)


def check_documentation(root: Path) -> None:
    """Verify the complete bilingual inventory before running its scenarios."""
    manifest = json.loads((root / 'tests/fixtures/documentation_examples/scenarios.json').read_text())
    with tempfile.TemporaryDirectory(prefix='spi-documentation-') as temporary:
        for index, (document, expected) in enumerate(manifest['documents'].items()):
            file = root / validate_relative_file(document)
            blocks = extract_blocks(file.read_text(encoding='utf-8'), document)
            validate_blocks(blocks, expected, document)
            for scenario in expected:
                if scenario not in manifest['scenarios']:
                    raise ValueError(f'{document}: unknown scenario {scenario}')
                selected = [block for block in blocks if block.scenario == scenario]
                run_scenario(root, selected, manifest['scenarios'][scenario], Path(temporary) / str(index) / scenario)


def main() -> None:
    """Report validation failures with a nonzero process exit status."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-root', type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        check_documentation(args.source_root.resolve())
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
