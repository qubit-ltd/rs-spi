# rs-spi Direct Error Matching Design

**Status:** Approved in conversation on 2026-07-16
**Scope:** `qubit-spi` and direct workspace consumer `qubit-mime`

## Goal

Simplify the unpublished error API so downstream code can inspect correlated
error context without pairing a separate kind enum with optional accessors or
restating `rs-spi` invariants through `expect`. Preserve forward-compatible
matching through non-exhaustive enums. Record, but do not implement, the
requirement to benchmark selector lookup before attempting allocation
optimization.

## Selected Design

`ResolutionError` and `AttemptFailure` remain public `#[non_exhaustive]` enums.
Downstream callers match their variants directly and include a wildcard arm.
The following parallel APIs are removed as an intentional breaking change:

- `ResolutionErrorKind` and `ResolutionError::kind`;
- `AttemptFailureKind` and `AttemptFailure::kind`;
- variant-specific optional accessors on `ResolutionError`;
- variant-specific optional accessors on `AttemptFailure`.

Direct matching keeps each discriminator and its fields in one pattern. It
also removes duplicate public concepts without introducing borrowed view types
that merely reproduce the existing enums.

General queries that express cross-variant semantics remain methods. In
particular, `ResolutionError::is_absence` remains available for callers such as
`qubit-fs` that do not need structural details.

## Decisive Attempt

`ResolutionError` gains:

```rust
pub fn decisive_attempt(&self) -> Option<&AttemptFailure>;
```

It returns the terminal attempt when resolution stopped by policy, or the only
attempt when an exhausted resolution contains exactly one attempt. It returns
`None` for multi-attempt exhaustion and non-aggregate errors. This captures the
shared rule currently duplicated by the MIME detector and classifier adapters.

The existing aggregate `attempts`, `termination`, and `terminal_attempt`
queries remain because they describe aggregate resolution independently of a
specific variant field layout and are useful for diagnostics.

## Downstream Migration

`qubit-mime` matches `ResolutionError` directly. Invalid-selector and
unknown-provider branches take their required context from the matched fields.
`NoProviderSucceeded` uses `decisive_attempt`; its helper matches
`AttemptFailure` directly and never calls `expect` for correlated fields.

`qubit-fs` continues to use `ResolutionError::is_absence` and requires no
semantic change. `qubit-magika` does not inspect either affected error type and
requires no source change unless verification reveals a compile dependency.

## Selector Benchmark TODO

No selector allocation optimization is implemented. Rustdoc on
`ProviderSelector::parse` and `ProviderResolver::create_named` records a TODO
requiring a representative benchmark before adding a no-allocation lookup fast
path. The TODO notes the repeated canonical URI-scheme lookup used by
`qubit-fs` as the representative scenario.

## Testing and Verification

Implementation proceeds test-first:

1. Add a failing `decisive_attempt` contract test covering policy stop,
   singleton exhaustion, multi-attempt exhaustion, and non-aggregate errors.
2. Implement the minimal method and make the focused test pass.
3. Rewrite `qubit-mime` tests or compile-time usages to require direct enum
   matching, then remove the redundant kind enums and accessors.
4. Migrate both MIME error adapters and confirm no correlated-field `expect`
   remains.
5. Add the Rustdoc TODO without changing lookup behavior.
6. Run each affected repository's `align-ci.sh`, then `ci-check.sh`; run
   `coverage.sh json` only if CI reports coverage below its threshold.

No compatibility aliases, borrowed error-view enums, selector benchmarks, or
selector lookup optimizations are added in this change.
