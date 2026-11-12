# Downstream Compatibility Validation

This command checks the local consumer repositories against the selected
`qubit-spi` source tree. It runs independently from the normal `rs-spi` CI so
that ordinary checks do not require sibling repositories.

## Prerequisites

- Python 3.11 or later, `rustup`, and the already installed Rust/Cargo 1.94.0
  toolchain. The validator does not install or update toolchains. Check the
  native Cargo path first with `rustup which --toolchain 1.94.0 cargo`.
- The `rs-fs-registry`, `rs-mime`, `rs-magika` and `rs-event-bus` repositories
  under the workspace root, plus all local path dependencies they reference.
- Network access when Cargo must download crates. `rs-magika` enables its
  bundled ONNX Runtime by default; its build may download the runtime binaries.
- Enough temporary disk space for source snapshots, Cargo logs and build
  artifacts. Build artifacts share a temporary target directory.
- The validation tool requires POSIX process-group support and `/proc` or `ps`
  to verify that command groups have stopped. This is a limit of the external
  validation tool; it does not change the platforms supported by the
  `qubit-spi` crate.

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

The validator copies the native Cargo 1.94.0 executable to its temporary
directory as `rs-spi-cargo-driver`; it does not rename or modify the toolchain's
Cargo. A wrapper launches that copy directly. This relies on Cargo 1.94.0's
executable-name and `CARGO` selection behavior, so a toolchain upgrade requires
rechecking the implementation and passing the startup probe and nested-entry
regression. The production probe builds a standard-library-only test crate with
no Cargo environment patches, then starts `cargo --version` through `PATH`, the
compile-time `CARGO` path, and runtime `CARGO`. All three must be recorded by the
wrapper before consumers start. Probe or driver preparation failure stops the
run without falling back to another Cargo.

The wrapper records nested fixture Cargo commands and checks their resolved SPI
source. This does not intercept programs that use another hard-coded Cargo
path. Each top-level metadata command has a 180-second budget, the startup probe
has a 180-second budget, and each consumer test command has a 1,800-second
budget. On completion, failure or timeout, the tool terminates and reaps
remaining members of that command's process group; cleanup may take up to a
further two seconds before escalation. A cleanup failure makes validation fail
and preserves the diagnostic report and snapshot.

The tool copies each consumer, `rs-spi`, and the recursively discovered local
path dependency closure to a temporary directory outside every input repository.
It includes existing dirty and untracked files in those snapshots and records
the input revisions, dirty states and content digests. It does not edit source
repositories or their lockfiles. If any input is dirty, the result describes a
validation of that recorded snapshot rather than a clean revision alone.

## Read the report

After the native driver and its copy both pass their version checks, the JSON
report records the original Cargo path, version and SHA-256, the temporary
driver path and SHA-256, the startup probe result, input snapshots, Cargo
override, per-consumer feature mode, resolved `qubit-spi` manifest and source,
command arguments and working directories, exit status, duration, timeout
status and log paths. If driver preparation fails before those checks complete,
inspect the report error for the failing path and reason; the digest metadata is
written only after both binaries run successfully. The validator only writes its
temporary copy. For probe failures, inspect
`cargo_entry_probe.command.log_path` and `cargo_entry_probe.records` to see which
entry was missing or failed. A non-zero exit retains the report and temporary
snapshot/log directory for diagnosis. On success, the report is written before
the tool's temporary directory is removed.

When a command fails, inspect its recorded log and verify the reported
`qubit-spi` manifest points inside the SPI snapshot with `source: null`. The
source repositories and consumer helpers are not rewritten; their normal
`PATH`, compile-time `CARGO`, and runtime `CARGO` launches are observed by the
probe and nested regression. Resolve missing local repositories, Cargo/network
errors or native runtime prerequisites before treating the consumers as
validated. This command is a reproducible local compatibility check; the
standard `rs-spi` CI does not run these downstream repositories.
