# Downstream Compatibility Validation

This command checks the local consumer repositories against the selected
`qubit-spi` source tree. It runs independently from the normal `rs-spi` CI so
that ordinary checks do not require sibling repositories.

## Prerequisites

- Python 3.11 or later and the Rust/Cargo toolchain required by this checkout
  (currently Rust 1.94.0).
- The `rs-fs-registry`, `rs-mime`, `rs-magika` and `rs-event-bus` repositories
  under the workspace root, plus all local path dependencies they reference.
- Network access when Cargo must download crates. `rs-magika` enables its
  bundled ONNX Runtime by default; its build may download the runtime binaries.
- Enough temporary disk space for source snapshots, Cargo logs and build
  artifacts. Build artifacts share a temporary target directory.

## Run

From the `rs-spi` checkout, replace the paths if the repositories are elsewhere:

```bash
python3 scripts/check-downstream.py \
  --workspace-root /data/working/qubit/rust-common \
  --spi-root /data/working/qubit/rust-common/rs-spi \
  --report /tmp/rs-spi-downstream-report.json
```

The consumer set is fixed; there is no partial-success mode. For each consumer,
the tool runs dependency metadata resolution and tests with default features and
with all features. It injects a Cargo configuration override so `qubit-spi`
resolves to the supplied `--spi-root`, then checks the resolved package graph's
manifest path and source. Tests run with `--locked` against the snapshot lockfile.

The tool copies each consumer, `rs-spi`, and the recursively discovered local
path dependency closure to a temporary directory outside every input repository.
It includes existing dirty and untracked files in those snapshots and records
the input revisions, dirty states and content digests. It does not edit source
repositories or their lockfiles. If any input is dirty, the result describes a
validation of that recorded snapshot rather than a clean revision alone.

## Read the report

The JSON report records the toolchain versions, input snapshots, Cargo override,
per-consumer feature mode, resolved `qubit-spi` manifest and source, command
arguments and working directories, exit status, duration, timeout status and
log paths. A non-zero exit retains the report and temporary snapshot/log
directory for diagnosis. On success, the report is written before the tool's
temporary directory is removed.

When a command fails, inspect its recorded log and verify the reported
`qubit-spi` manifest points inside the SPI snapshot with `source: null`. Resolve
missing local repositories, Cargo/network errors or native runtime prerequisites
before treating the consumers as validated. This command is a reproducible local
compatibility check; the standard `rs-spi` CI does not run these downstream
repositories.
