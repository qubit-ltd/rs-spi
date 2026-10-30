# Qubit SPI Design

[中文版](design.zh_CN.md) · [User Guide](user_guide.md) · [README](../README.md)

This document describes the 0.14 API and its implementation contracts. It is for
maintainers and domain-crate authors. The user guide contains runnable integration
examples; this document records why registration, selection and creation remain
separate and what future changes must preserve.

## Responsibilities

`ServiceSpec` binds configuration and domain error types. Sync and async output
capabilities are separate; one family can support both without forcing the same
output type. `ProviderMetadata` and the leaf factory traits compose through blanket
`ProviderDefinition` implementations. A domain crate owns the business interface
and any global facade; an application owns registration and default policy.

A resolver is a composition of leaf providers with aggregate errors, not another
leaf provider. It returns the successful output directly. URI parsing, credentials,
output identity checks and client caching belong to domain adapters. For example,
filesystem registries can validate output identity in a wrapper, and a rooted local
provider can return a clone of its existing filesystem handle. SPI does not infer
those domain contracts from metadata.

## Optional Link-Time Provider Discovery

The default feature set is empty. Enabling `inventory` adds a static,
link-time discovery adapter; it does not change the explicit registry API or
make provider discovery a runtime default. A service-contract crate invokes
`declare_sync_provider_inventory!` or `declare_async_provider_inventory!` for a
concrete `ServiceSpec`. That declaration owns one inventory collection and its
`build_registry()` function. Provider crates invoke the corresponding
`submit_*_provider!` macro with a zero-argument factory. The factory runs only
when the application builds the registry, so providers can still construct
stateful values explicitly through the ordinary `register` API.

Collections are typed and isolated by service family. An entry submitted to the
collection declared for `GreeterSpec`, for example, cannot be read while
building a registry for a different spec. Building iterates discovered entries,
calls each zero-argument factory, and registers the resulting provider through
the ordinary atomic registration path. An ambiguous source or registration
conflict (canonical ID or alias ownership) is returned as a
`ProviderInventoryBuildError` with the submission source location; no partially
built registry is returned. Factories
do not return `Result`, and factory or `descriptor()` panics propagate unchanged
rather than being converted into inventory build errors.

When a domain adapter must validate or wrap each submitted provider before
registration, the contract can expose `build_registry_with(transform)` instead.
The transform runs after each factory and before registration; the ordinary
`build_registry()` uses the identity transform. This keeps domain-specific
output checks outside SPI while allowing the inventory path to preserve the
same adapter behavior as explicit registration.

Cargo dependency resolution is not linker reachability. A provider package in
`Cargo.toml` may be omitted from the final binary when no symbol anchors it.
Applications should keep such anchors together in a module such as
`linked_providers.rs`, using `use provider_friendly as _;`, before calling the
contract's `build_registry()`. The order in which crates are linked or entries
are discovered is not a selection policy: automatic selection remains priority
descending then canonical ID ascending. This facility discovers entries compiled
into the executable; it does not load dynamic libraries, scan directories, or
accept plugins after startup.

## Catalog and Snapshot Boundaries

```mermaid
flowchart LR
    SR[Sync registry] --> C[ProviderCatalog]
    AR[Async registry] --> C
    C --> I[RegistryInner under RwLock]
    I --> E[Entries ordered by priority and ID]
    I --> L[Selector ownership index]
    I --> R[Registration order]
    I --> D[Default selection]
    C --> S[Owned candidate snapshot]
    S --> F[FallbackState]
    F --> O[Output or aggregate error]
```

The diagram shows shared algorithms, not shared registration state between sync
and async registries. Each catalog instance owns an `Arc<RwLock<RegistryInner<P>>>`;
registry clones share that instance. Sync and async registries remain independent.
Entries share immutable descriptors and provider objects using `Arc`. Registration
order is separate from automatic order `(Reverse(priority), canonical ID)`.
`Reverse<i32>` handles minimum priority without integer-negation overflow.

Registration first invokes `descriptor()` outside the write lock. It then acquires
one write guard, checks canonical ID and every alias against existing ownership,
and inserts all indexes atomically. Reentrant metadata callbacks can register
other providers; conflict checks therefore happen after the callback. A conflict
inserts none of the attempted provider’s selectors, and the rejected provider
drops after the guard. A metadata panic also leaves the attempted registration
unapplied. Completed reentrant callback changes are not rolled back.

Explicit resolution holds one read guard. Default resolution clones the selection
and resolves candidates under the same guard. `resolve_default_snapshot()` returns
that pair even when resolution fails; callers must not combine a separate
`default_selection()` read with a later resolve when validating configuration.
`resolve()` is the convenience operation returning only the captured result.

Descriptor snapshots and Debug metadata use the same conversion helper. Debug
captures descriptors/default under one guard and formats the owned data after
unlocking. Provider factories and async future polling never run under the catalog
lock. Tests also cover reentrant metadata and destruction to protect this boundary.

A resolver owns a boxed candidate slice and fallback policy. Later registration or
default changes cannot alter that slice. Provider objects themselves remain shared;
a snapshot is not a deep copy of mutable backend state. Resolver cloning copies the
slice and clones its shared handles, so its cost is linear in the candidate count.

## Selection and Failure State Machine

IDs are already canonical ASCII tokens. Selectors normalize caller input; aliases
share the canonical ID lookup namespace. Alias validation parses lazily and stops
consuming the caller's iterator at the first invalid or conflicting item. Static
macro metadata is checked at compile time.

Named selection admits one provider. Strict chains reject all unknown occurrences,
preserving their diagnostic order; lenient chains skip unknowns but reject an empty
candidate set. Matched provider identities are deduplicated in first-encounter order.
Automatic selection uses descending priority and ascending canonical ID.

`FallbackState<E>` is the common sync/async diagnostic state machine:

1. Invoke the current candidate; success returns its output immediately.
2. Record a failure with the actual canonical provider ID and typed domain error.
3. If no candidate remains, return `Exhausted`.
4. Otherwise, if policy disallows the failure kind, return `StoppedByPolicy`.
5. Otherwise, try the next candidate.

The exhaustion check precedes policy rejection. `Never` stops at the first failure
when another candidate exists. `OnAbsence` admits Unsupported/Unavailable;
`OnAnyError` admits every current leaf classification. Panics are not failures for
this policy. Missing selectors never fabricate creation attempts. Aggregate errors
are nonempty; the decisive attempt is the last actual failure and `is_absence()`
requires every actual failure to be absence-related.

## Async Lifetime and Cancellation

Async registration/resolution are synchronous metadata operations. The resolver
awaits one sendable leaf future at a time without choosing an executor. Config is
borrowed and must be Sync; output is Send + 'static. Dropping an unpolled resolver
future invokes no factory. Cancellation after polling drops the active leaf future
and does not advance fallback, including after earlier candidates have failed.

Cancellation does not roll back external effects or stop independently spawned
tasks. A leaf panic before returning its future or during polling propagates.
Every create invokes a factory; providers may reuse an existing resource. SPI
neither caches output nor guarantees a fresh allocation. Errors in subsequent
service operations are outside construction fallback.

## Performance Decisions

The benchmark suite separates registration, named/chain/automatic/default
resolution, resolver clone, alias parsing, successful creation, fallback, complete
exhaustion, ready/pending async creation, and read/write contention. Registration
contention resets a bounded registry each batch and excludes thread startup/join.
Alias fixtures prepare source strings before timing; error construction remains
part of the corresponding creation workload.

The current candidate storage is a boxed slice, so clone cost grows with the
number of candidates. Historical performance comparisons are not currently
available with reproducible candidate revisions, environment details and
per-run results. The previous claims about three runs, a 5% threshold, and
specific shared-slice regressions therefore cannot be verified and are not
retained as quantitative conclusions. Future comparisons should measure
candidate cloning, named and automatic resolution, and creation paths, while
recording both revisions, the environment, commands and per-run results.

The current lazy alias parser does not collect the raw strings into a vector
and stops consuming input at the first error; these behaviors are verifiable
from the source and tests, but do not establish a measured performance gain.
Current implementation benchmarks can be rerun with:

```bash
rustc -Vv
cargo bench --locked --bench registry_resolution -- named_resolution
cargo bench --locked --bench registry_resolution -- auto_resolution
cargo bench --locked --bench registry_resolution -- resolver_clone
cargo bench --locked --bench registry_resolution -- alias_parsing
cargo bench --locked --bench registry_creation -- creation_success
```

These commands measure the current implementation and do not reproduce the
missing historical comparison. A future comparison needs identifiable
candidate revisions in a consistent environment.

The adapter approach used with `build_registry_with` is demonstrated in the
[link-time discovery guide](user_guide.md#link-time-discovery-for-the-same-greeter).

## Validation and Maintenance

- Public integration tests cover selection, failure classification, registration
  atomicity, reentrant callbacks/destruction, panic, resource sharing, concurrent
  default snapshots and all three cancellation boundaries.
- Doctests include runnable examples for all public named types and compile-fail
  checks for intentionally unavailable APIs/representations.
- The independent fuzz oracle uses a linear vector rather than production indexes.
  It checks full call/attempt histories, policy termination and historical snapshots.
  Named boundary seeds are retained; random exploration artifacts are temporary.
- Markdown checks extract marked Rust blocks from each language independently,
  require the declared inventory, and run real Cargo scenarios. Missing scenarios,
  ignored fences and stale methods fail validation. Rust examples must not be
  replaced by unrelated duplicate source fixtures.
- The executable `inventory-greeter` documentation workspace contains four
  crates: the service contract, consumer, provider and application. It verifies
  the linked-provider build, default selection and service creation shown in
  the guide. Runtime augmentation and sealing are covered by
  [registry integration tests](../tests/registry/) and
  [inventory integration tests](../tests/inventory/), not by this scenario.
- Inventory entries are sorted by source location, which must be unique within
  each service-family collection. Duplicate locations fail before any factory
  runs. Multiple submissions produced by one outer macro invocation may share
  its source location. Placing submissions on different lines inside the macro
  definition does not resolve the ambiguity. Submit directly on distinct
  invocation lines, or make each outer macro invocation emit one submission and
  invoke it on separate lines. Distinct named modules can distinguish
  `module_path`; sources must still be unique within each collection.
  `ProviderInventoryBuildError::registration_error()` returns
  `Some` for registration conflicts and `None` for ambiguous sources; callers
  must handle both cases. The return type is `Option` so this breaking API
  change can represent both inventory ambiguity and registration conflicts.
  Existing direct callers such as `rs-fs-registry`'s error formatter and
  `rs-event-bus`'s discovery-conflict tests must handle the optional result;
  this change belongs in `0.14.0`, not a `0.13.x` patch release.
- Package validation uses an actual Cargo archive and builds/tests its extracted
  contents without a checkout path patch. This prevents tests from depending on
  excluded fuzz sources or shared CI submodules.
- CI runs alignment-sensitive lint/test/documentation checks plus the project
  documentation and archive contracts. Changes must also be exercised through
  filesystem, MIME and Magika consumers with their actual dependency paths checked.

There is no unregister or unseal operation. Applications may call `seal()` after
startup registration and default-selection configuration; subsequent registry
mutations, including default-selection changes, are rejected. Do not add a
successful-output metadata wrapper, dependency-injection container, or dynamic
plugin loader without a concrete domain need.
Keep the catalog and failure state machine shared; keep domain-specific validation
in adapters. Any future public API change must update both languages, executable
examples and affected downstream crates together.
