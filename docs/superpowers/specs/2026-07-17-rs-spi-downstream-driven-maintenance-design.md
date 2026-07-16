# rs-spi Downstream-Driven Maintenance Design

## Context

`qubit-spi` 0.8 already has a stable core model: typed provider descriptors,
explicit selection, deterministic resolution, structured attempt failures, and
separate registry construction and lookup. A review of the crate and its `rs-*`
consumers found no architectural defect that warrants another API redesign.
The remaining work is compatibility-preserving maintenance around selector
lookup cost, API guidance, documentation consistency, and test organization.

## Goals

- Preserve all existing public behavior and downstream source compatibility.
- Measure the canonical selector lookup path used repeatedly by downstream
  registries and remove avoidable allocation only when the benchmark supports
  doing so.
- Make result-bearing APIs harder to ignore accidentally.
- Align inlining, rustdoc, README content, and external test layout with the
  repository conventions.
- Keep validation deterministic and suitable for CI.

## Non-goals

- No async provider abstraction.
- No global registry, custom policy framework, or output-only API.
- No redesign of the current error or resolution models.
- No change to the existing `2025 - 2026` file-header policy in this work.
- No breaking typed-selector API solely to optimize lookup.

## Design

### 1. Benchmark-gated selector lookup

Add a focused benchmark for the canonical selector path exercised by
downstream crates, including repeated lookup of a known provider name. The
benchmark must make the allocation behavior observable rather than reporting
wall-clock timing alone.

The preferred implementation is a borrowed lookup against the selector cache,
so a canonical known selector does not need a fresh owned key. It must remain an
internal optimization: existing constructors, conversions, equality,
serialization-facing behavior, and error values stay unchanged. If the
benchmark cannot demonstrate the expected allocation reduction without a
meaningful throughput regression, retain the benchmark and correct the cache
documentation without adding the fast path.

### 2. `must_use` guidance

Mark values whose contents encode a completed operation or resolution result as
`#[must_use]`, including `CreatedService`, `ResolvedProvider`, and consuming
accessors such as `into_service` and `into_parts`. The attributes provide
compiler guidance only and do not alter runtime behavior or signatures.

### 3. Inlining consistency

Use `#[inline(always)]` only for trivial forwarding functions and remove it
from functions that iterate or build structured errors. In particular, align
the resolver forwarding entry point and resolution-error constructors with the
actual amount of work each performs. This is a mechanical annotation change;
no control flow or error semantics change.

### 4. Rustdoc consistency

Replace source-level `# Arguments` sections with the repository-standard
`# Parameters` heading and document public tuple-struct fields. Preserve the
meaning and examples of existing documentation. Do not broaden this into API
renaming or prose rewrites unrelated to the findings.

### 5. README completeness

Keep the English and Chinese READMEs structurally aligned. Add the coverage
badge and the expected Testing, License, Contributing, and Author sections;
remove badge drift where needed. Links and commands must refer to the actual
repository scripts and Apache-2.0 license.

### 6. External test mapping and fixtures

Maintain the single explicit integration-test entry point, but give each source
module a corresponding external test module where meaningful. Move reusable
provider/service doubles out of unrelated test files into clearly named fixture
modules. Existing assertions remain intact while missing modules receive
focused public-behavior tests. Avoid tests of private implementation details or
one assertion split across many tiny files.

## Verification

Behavioral changes follow test-first development: add a failing allocation or
lookup regression test before implementing a selector fast path. Compile-time
guidance, documentation, annotations, and README structure are verified with
the repository's static checks because they do not have runtime behavior to
drive through a failing unit test.

Run, at minimum:

1. Focused selector and resolver tests.
2. The complete integration and doctest suite.
3. Formatting, Clippy, rustdoc, and repository alignment checks.
4. The selector benchmark before and after any optimization.
5. Downstream checks for the reviewed `rs-*` consumers when dependency wiring
   permits doing so without modifying their lockfiles.

## Compatibility and risk

The work is source-compatible. The main implementation risk is adding
complexity for an unproven selector optimization; benchmark gating controls
that risk. The main maintenance risk is mechanical documentation or test-file
movement losing coverage; preserving assertions first and running the complete
suite controls that risk.
