# rs-spi Rustdoc Completion Design

## Objective

Complete the Rust documentation for every item under `src/` without changing
runtime behavior, public signatures, module structure, or error semantics.
The result must explain how each API participates in provider registration,
selection, resolution, and service creation.

## Documentation Scope

The review covers all 15 Rust source files under `src/` and includes:

- every `struct`, `enum`, and `trait`;
- every enum variant and associated type;
- every named and tuple field, including private and crate-visible fields;
- every free function, inherent method, trait method, and trait implementation
  method, regardless of visibility;
- every module-level responsibility statement.

Type documentation describes the type's purpose, its place in the SPI
workflow, and the situations in which callers or implementers use it. Field
documentation explains the stored value and its semantic role rather than
restating the field name.

## Function and Method Documentation

Each function or method documents its operation and all meaningful inputs.
Documentation also states the return semantics when a value is returned.
Methods returning `Option` distinguish `Some` from `None`, and methods
returning `Result` use a `# Errors` section that identifies the concrete
conditions producing each error category.

Documentation calls out relevant normalization, ownership transfer,
borrowing, fallback, ordering, deduplication, error chaining, formatting, and
validation behavior. Private helpers receive concise documentation describing
their inputs, outputs, and invariants. Standard trait implementations may be
brief, but they are not left undocumented.

## Examples

Examples are added only when they materially clarify correct use. The crate
level example remains the primary end-to-end demonstration of defining a
service specification, registering a provider, resolving a selection, and
using the created service. Individual APIs receive examples only when their
usage cannot be understood as clearly from that workflow and their prose.

All examples must compile as rustdoc doctests. Examples avoid external
services, network access, nondeterministic state, and unnecessary panics.

## Change Boundaries

The implementation changes documentation comments only. It does not add
dependencies, lint policy, test files, public APIs, attributes, optimizations,
or refactoring. Existing English terminology and rustdoc link style are
preserved. Comments must accurately describe the current implementation,
including edge cases visible in tests and internal helper behavior.

## Existing Work and Isolation

The pre-existing documentation changes on `dev-starfish` are retained as one
coherent English `docs(spi)` commit. The repository-local `.worktrees/`
directory is ignored in a separate English Git-maintenance commit. Remaining
work is performed on branch `docs/complete-rustdoc` in the isolated worktree
at `.worktrees/complete-rustdoc`.

## Verification

The completed documentation is verified with:

- `cargo fmt --check` for formatting stability;
- `RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc --no-deps` for public
  rustdoc coverage, links, and warning-free rendering;
- `cargo clippy --all-targets --all-features -- -D warnings` for lint safety;
- `cargo test --all-features` for integration tests and doctests;
- a source-level audit of every type, variant, associated type, field, free
  function, inherent method, trait method, and trait implementation method to
  cover private items that `missing_docs` does not enforce;
- `git diff --check` and final diff inspection to confirm that no behavior or
  signatures changed.

Success means all commands pass, the source audit finds no undocumented item
in scope, and the final diff contains rustdoc changes only.
