# rs-spi Resolution Outcome Redesign

**Status:** Approved in conversation on 2026-07-16
**Scope:** `qubit-spi` and direct workspace consumers `qubit-fs`, `qubit-mime`,
`qubit-magika`

## Goal

Make failed resolution preserve why candidate traversal ended, replace
downstream structural matching with stable query APIs, reuse one validated
selection parser, and make MIME configuration retain validated selections.
Breaking changes are intentional. Selector lookup allocation optimization is
outside this change.

## Selected Design

`ResolutionError::NoProviderSucceeded` gains a `termination` field of the new
public `ResolutionTermination` enum:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionTermination {
    Exhausted,
    StoppedByPolicy,
}
```

`Exhausted` means every candidate admitted by the validated selection was
considered. `StoppedByPolicy` means a provider returned a failure for which the
resolver policy prohibited another attempt. Named selection has exactly one
candidate and therefore reports `Exhausted`; it does not consult fallback
policy.

All public error enums become `#[non_exhaustive]`. Downstream callers stop
depending on their field layout and use stable observation methods. The
minimal APIs required by current consumers are:

```rust
impl ResolutionError {
    pub const fn kind(&self) -> ResolutionErrorKind;
    pub fn invalid_selector_input(&self) -> Option<&str>;
    pub const fn invalid_selector_index(&self) -> Option<usize>;
    pub const fn selector_error(&self) -> Option<&ProviderSelectorError>;
    pub const fn unknown_selector(&self) -> Option<&ProviderSelector>;
    pub fn attempts(&self) -> &[AttemptFailure];
    pub const fn termination(&self) -> Option<ResolutionTermination>;
    pub fn terminal_attempt(&self) -> Option<&AttemptFailure>;
    pub fn is_absence(&self) -> bool;
}

impl AttemptFailure {
    pub const fn kind(&self) -> AttemptFailureKind;
    pub const fn requested_selector(&self) -> Option<&ProviderSelector>;
    pub const fn provider_id(&self) -> Option<&ProviderId>;
    pub const fn provider_error(&self) -> Option<&ProviderError>;
}
```

`ResolutionErrorKind` and `AttemptFailureKind` are non-exhaustive copyable enums
in their own files. `RegistrationError` exposes `selector()`,
`existing_provider()`, and `provider()`; `ProviderSelectorError` exposes
`input()`, `normalized()`, and `is_empty()`. Other validation errors become
non-exhaustive but do not gain speculative accessors.

`ProviderResolver` creates `NoProviderSucceeded` through separate internal
constructors for exhaustion and policy stop. Display text distinguishes the two
termination modes while retaining ordered attempt diagnostics.

## Selection Parsing

`ProviderSelection::named` and `ProviderSelection::chain` remain the only raw
selection parsers. `ProviderResolver::create_named` and `create_chain` construct
a `ProviderSelection`, convert `ProviderSelectionError` into
`ResolutionError`, and delegate to `create`. This removes the duplicated raw
parsing loops while keeping boundary convenience methods.

`MimeConfig` stores validated `ProviderSelection` values for detector and media
classifier selection. Configuration loading performs validation once. The old
raw default/fallback getters are replaced by:

```rust
pub const fn mime_detector_selection(&self) -> &ProviderSelection;
pub const fn media_stream_classifier_selection(&self) -> &ProviderSelection;
```

Detector and classifier registries call `ProviderResolver::create` with these
values. Invalid configured selectors therefore fail during `MimeConfig`
construction instead of on every service creation. Existing configuration
keys and textual formats remain unchanged.

## Downstream Error Mapping

`qubit-fs` maps unknown or wholly absence-like resolution through
`ResolutionError::is_absence()`.

`qubit-mime` uses `ResolutionErrorKind` and accessors. For
`StoppedByPolicy`, it inspects `terminal_attempt()` and preserves the terminal
provider's unavailable/backend classification even after earlier attempts.
For `Exhausted`, singleton failures retain the existing precise domain mapping;
multi-attempt exhaustion remains `NoAvailableDetector` or
`NoAvailableClassifier` with ordered diagnostics.

`qubit-magika` needs no semantic change beyond compiling against the revised
non-exhaustive error contract.

## Rust Organization

Every new enum has its own snake-case source file and matching external test
file. `ProviderDescriptor` inherent methods are ordered as constructors and
builder transformations first (`new`, `with_aliases`, `with_priority`), then
getters (`id`, `aliases`, `priority`). Existing public paths remain rooted at
`qubit_spi::error`.

## Testing

Work proceeds test-first:

1. Add failing `rs-spi` tests for termination modes and query APIs.
2. Implement the new error contract and make focused tests pass.
3. Add failing tests proving raw resolver methods delegate through the same
   validated selection semantics.
4. Add failing `rs-mime` configuration and multi-attempt terminal-error tests.
5. Migrate downstream implementations and tests.
6. Run each repository's `align-ci.sh`, then `ci-check.sh`; run
   `coverage.sh json` only if CI reports coverage below threshold.

No compatibility aliases are added, no async provider API is introduced, and
no selector allocation fast path is implemented.
