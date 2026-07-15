# rs-spi Rustdoc Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete accurate rustdoc for every source type, variant, associated type, field, function, and method in `rs-spi`.

**Architecture:** Preserve the existing SPI implementation and improve only documentation comments. Complete the two error modules first, document the remaining standard-trait items, then audit every source declaration and run strict documentation, lint, and test verification.

**Tech Stack:** Rust 2024, rustdoc, Cargo, Clippy

## Global Constraints

- Work only in `/home/starfish/working/qubit/rust-common/rs-spi/.worktrees/complete-rustdoc` on branch `docs/complete-rustdoc`.
- Follow `~/.codex/specs/general.mdc`, `git.mdc`, `rust-coding.mdc`, `rust-comment.mdc`, and `rust-test.mdc`.
- Modify documentation comments only; do not change logic, signatures, attributes, dependencies, tests, or module structure.
- Keep documentation in English and preserve the crate's existing terminology and rustdoc link style.
- Document private and crate-visible items as thoroughly as public items.
- Do not commit implementation or plan changes unless the user explicitly authorizes those commits.

---

### Task 1: Complete registration-error documentation

**Files:**

- Modify: `src/registration_error.rs:12`
- Verify: `tests/registration_error_tests.rs`

**Interfaces:**

- Consumes: `RegistrationErrorKind`, the four stored diagnostic fields, and the existing constructors/accessors.
- Produces: complete rustdoc for registration error classification, construction, inspection, and display formatting; no Rust interface changes.

- [ ] **Step 1: Record the documentation gaps**

Run:

```bash
rg -n -B 5 '^\s*(pub(\([^)]*\))?\s+)?(struct|enum|trait)\s+|^\s*(pub(\([^)]*\))?\s+)?(const\s+)?fn\s+' src/registration_error.rs
rg -n '^\s*[a-z_][a-z0-9_]*:\s*' src/registration_error.rs
```

Expected: the four fields and `Display::fmt` have no rustdoc, while the type and inherent-method comments do not yet explain use cases, parameters, return semantics, or `Option` meanings completely.

- [ ] **Step 2: Expand type and field rustdoc**

Replace the existing type and field declarations with the same declarations
and these exact rustdoc comments:

```rust
/// Classification of a failure detected while validating registry metadata.
///
/// Inspect this value when callers need to distinguish malformed identifiers
/// from conflicts with registrations already accepted by a registry builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RegistrationErrorKind {
    /// An identifier was empty after applying its input rules.
    EmptyIdentifier,
    /// An identifier used unsupported characters or structure.
    InvalidIdentifier,
    /// A canonical identifier or alias was already owned by another provider.
    DuplicateSelector,
}

/// Error raised while validating provider identifiers, aliases, or ownership.
///
/// Registry construction returns this error before mutating the builder when
/// an identifier is invalid or a selector is already claimed. The optional
/// diagnostic fields expose the values relevant to the specific [`Self::kind`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistrationError {
    /// Classification identifying the registration rule that failed.
    kind: RegistrationErrorKind,
    /// Invalid or conflicting identifier, when the error concerns one.
    identifier: Option<Box<str>>,
    /// Canonical ID of the provider that already owns a conflicting selector.
    existing_provider: Option<Box<str>>,
    /// Canonical ID of the provider that attempted to claim the selector.
    provider: Option<Box<str>>,
}
```

Retain the existing enum-variant comments because each already states the represented condition.

- [ ] **Step 3: Complete constructor and accessor rustdoc**

Replace the terse method comments with documentation carrying these exact semantics:

```rust
/// Creates an error for an identifier that is empty after input processing.
///
/// Returns a [`RegistrationErrorKind::EmptyIdentifier`] error without an
/// identifier or provider ownership details.
pub fn empty_identifier() -> Self

/// Creates an error for an identifier that violates the canonical grammar.
///
/// `identifier` is retained verbatim for diagnostics. Returns an
/// [`RegistrationErrorKind::InvalidIdentifier`] error containing that value.
pub fn invalid_identifier(identifier: impl AsRef<str>) -> Self

/// Creates an error for a selector already claimed by another provider.
///
/// `identifier` is the conflicting canonical ID or alias,
/// `existing_provider` is its current canonical owner, and `provider` is the
/// canonical ID attempting the new claim. Returns a
/// [`RegistrationErrorKind::DuplicateSelector`] error retaining all three.
pub fn duplicate_selector(
    identifier: impl AsRef<str>,
    existing_provider: impl AsRef<str>,
    provider: impl AsRef<str>,
) -> Self

/// Returns the registration rule that failed.
pub const fn kind(&self) -> RegistrationErrorKind

/// Returns the invalid or conflicting identifier, when one was recorded.
///
/// Returns `Some` for invalid-identifier and duplicate-selector errors, and
/// `None` for an empty-identifier error.
pub fn identifier(&self) -> Option<&str>

/// Returns the canonical provider that already owns a conflicting selector.
///
/// Returns `Some` for a duplicate-selector error and `None` for other kinds.
pub fn existing_provider(&self) -> Option<&str>

/// Returns the canonical provider that attempted to claim a selector.
///
/// Returns `Some` for a duplicate-selector error and `None` for other kinds.
pub fn provider(&self) -> Option<&str>
```

Only replace comments; preserve every existing signature and body byte-for-byte.

- [ ] **Step 4: Document the formatting method**

Add this rustdoc immediately before `Display::fmt`:

```rust
/// Formats the failed registration rule and its available diagnostic values.
///
/// `formatter` receives a human-readable message selected from [`Self::kind`].
/// Returns a formatting error if the formatter cannot accept the message.
```

- [ ] **Step 5: Verify this task**

Run:

```bash
cargo fmt --check
cargo test --test registration_error_tests
git diff --check
git diff -- src/registration_error.rs
```

Expected: formatting and the focused test pass; the diff contains rustdoc lines only and every registration-error field and method has a meaningful comment.

---

### Task 2: Complete resolution-error documentation

**Files:**

- Modify: `src/resolution_error.rs:14`
- Verify: `tests/resolution_error_tests.rs`

**Interfaces:**

- Consumes: `ResolutionErrorKind`, `AttemptFailure`, `ResolutionError`, their stored diagnostics, constructors, accessors, and display implementation.
- Produces: complete rustdoc for individual provider attempts and aggregate resolution failures; no Rust interface changes.

- [ ] **Step 1: Record the documentation gaps**

Run:

```bash
rg -n -B 5 '^\s*(pub(\([^)]*\))?\s+)?(struct|enum|trait)\s+|^\s*(pub(\([^)]*\))?\s+)?(const\s+)?fn\s+' src/resolution_error.rs
rg -n '^\s*[a-z_][a-z0-9_]*:\s*' src/resolution_error.rs
```

Expected: all eight stored fields and `Display::fmt` have no rustdoc, and the existing public method comments omit important input, return, and `Option` semantics.

- [ ] **Step 2: Expand resolution types and fields**

Replace the existing type and field declarations with the same declarations
and these exact rustdoc comments:

```rust
/// Classification of a failed provider-selection resolution.
///
/// Inspect this value to distinguish a failed direct lookup from an exhausted
/// automatic or explicit fallback sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResolutionErrorKind {
    /// A named selector did not resolve to any registered provider.
    UnknownProvider,
    /// Every candidate failed or was skipped.
    NoProviderSucceeded,
}

/// Diagnostic record for one candidate that could not produce a service.
///
/// Resolvers collect these values in attempt order so callers can determine
/// which selector was requested, which canonical provider was reached, and why
/// lookup or service creation failed.
#[derive(Clone, Debug)]
pub struct AttemptFailure {
    /// Selector that caused this attempt, or `None` for automatic selection.
    requested_selector: Option<ProviderSelector>,
    /// Canonical provider reached by lookup, or `None` when lookup failed.
    provider_id: Option<ProviderId>,
    /// Provider-reported classification, or `None` when creation was not run.
    provider_error_kind: Option<ProviderErrorKind>,
    /// Human-readable explanation of the lookup or creation failure.
    reason: Box<str>,
    /// Optional underlying provider cause retained for error chaining.
    source: Option<Arc<dyn Error + Send + Sync>>,
}

/// Aggregate error returned when a provider selection cannot create a service.
///
/// Direct lookup failures retain the requested selector when it can be parsed.
/// Automatic and chained failures retain ordered [`AttemptFailure`] records so
/// callers can diagnose every candidate considered before resolution stopped.
#[derive(Clone, Debug)]
pub struct ResolutionError {
    /// Classification of the overall resolution failure.
    kind: ResolutionErrorKind,
    /// Normalized direct-lookup selector, when one could be retained.
    requested_selector: Option<ProviderSelector>,
    /// Ordered failures recorded while candidates were considered.
    attempts: Box<[AttemptFailure]>,
}
```

Retain the existing variant comments because they already describe both error categories.

- [ ] **Step 3: Complete `AttemptFailure` method rustdoc**

Document each method with these exact behavioral facts:

```rust
/// Creates a failed attempt for a selector that matched no provider.
///
/// `selector` is retained as the requested selector and used to construct the
/// reason. Returns an attempt without a provider ID, provider error kind, or
/// source because service creation was not reached.
pub fn unknown_provider(selector: ProviderSelector) -> Self

/// Creates a failed attempt from an error returned by a provider factory.
///
/// `requested_selector` identifies the explicit selector, or is `None` for
/// automatic selection; `provider_id` identifies the reached provider; and
/// `error` supplies the classification, reason, and optional source. Returns a
/// diagnostic record retaining those values.
pub fn provider_error(
    requested_selector: Option<ProviderSelector>,
    provider_id: ProviderId,
    error: &ProviderError,
) -> Self

/// Returns the selector that requested this attempt.
///
/// Returns `Some` for named and chained selection attempts and `None` for an
/// automatically selected candidate.
pub fn requested_selector(&self) -> Option<&ProviderSelector>

/// Returns the canonical provider reached by selector lookup.
///
/// Returns `Some` when service creation was attempted and `None` when lookup
/// failed before a provider was identified.
pub fn provider_id(&self) -> Option<&ProviderId>

/// Returns the provider-reported creation failure classification.
///
/// Returns `Some` when a provider factory returned an error and `None` when
/// service creation was not attempted.
pub const fn provider_error_kind(&self) -> Option<ProviderErrorKind>

/// Returns the human-readable explanation recorded for this failed attempt.
pub fn reason(&self) -> &str

/// Returns the underlying cause retained from the provider error.
///
/// Returns `Some` when the provider supplied a source error and `None` for an
/// unknown selector or a provider error without a source.
pub fn source(&self) -> Option<&(dyn Error + 'static)>
```

- [ ] **Step 4: Complete `ResolutionError` and formatting rustdoc**

Document the aggregate methods and formatter as follows:

```rust
/// Creates an error for a direct selector that matched no provider.
///
/// `selector` is normalized and retained when it is valid. Returns an
/// [`ResolutionErrorKind::UnknownProvider`] error with no attempt records; an
/// invalid selector is represented by `None` in [`Self::requested_selector`].
pub fn unknown_provider(selector: impl AsRef<str>) -> Self

/// Creates an aggregate error after all permitted candidates fail or are skipped.
///
/// `attempts` is retained in resolver attempt order. Returns a
/// [`ResolutionErrorKind::NoProviderSucceeded`] error without a direct
/// requested selector.
pub fn no_provider_succeeded(attempts: impl Into<Box<[AttemptFailure]>>) -> Self

/// Returns the overall resolution failure classification.
pub const fn kind(&self) -> ResolutionErrorKind

/// Returns the normalized selector retained for a direct lookup failure.
///
/// Returns `Some` for a valid unknown selector and `None` for invalid input or
/// an aggregate candidate failure.
pub fn requested_selector(&self) -> Option<&ProviderSelector>

/// Returns failed attempts in the order candidates were considered.
///
/// The slice is empty for a direct unknown-provider error.
pub fn attempts(&self) -> &[AttemptFailure]

/// Formats the overall resolution failure for human-readable diagnostics.
///
/// `formatter` receives either the unknown selector or the number of recorded
/// attempts. Returns a formatting error if the formatter rejects the message.
fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
```

- [ ] **Step 5: Verify this task**

Run:

```bash
cargo fmt --check
cargo test --test resolution_error_tests
git diff --check
git diff -- src/resolution_error.rs
```

Expected: formatting and the focused tests pass; only rustdoc changes appear and every field, constructor, accessor, and formatter is documented.

---

### Task 3: Document remaining standard-trait items and panic behavior

**Files:**

- Modify: `src/provider_id.rs:69`
- Modify: `src/provider_selector.rs:69`
- Modify: `src/provider_registry.rs:140`
- Verify: `tests/provider_id_tests.rs`
- Verify: `tests/provider_selector_tests.rs`
- Verify: `tests/provider_registry_tests.rs`

**Interfaces:**

- Consumes: the two `FromStr` implementations and the internal indexed registry lookup.
- Produces: documented parse inputs, return/error semantics, associated error types, and a standard `# Panics` section; no signature or behavior changes.

- [ ] **Step 1: Document `ProviderId` parsing**

Add rustdoc without changing the implementation:

```rust
impl FromStr for ProviderId {
    /// Error returned when the input is empty or violates canonical ID syntax.
    type Err = RegistrationError;

    /// Parses an already canonical provider identifier.
    ///
    /// `value` is validated without trimming or case normalization. Returns the
    /// canonical provider ID when validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when `value` is empty or noncanonical.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
```

- [ ] **Step 2: Document `ProviderSelector` parsing**

Add rustdoc without changing the implementation:

```rust
impl FromStr for ProviderSelector {
    /// Error returned when the normalized input is empty or invalid.
    type Err = RegistrationError;

    /// Parses a provider selector from configuration-style input.
    ///
    /// `value` is trimmed and ASCII-lowercased before validation. Returns the
    /// normalized selector used for registry lookup.
    ///
    /// # Errors
    ///
    /// Returns [`RegistrationError`] when the normalized input is empty or
    /// violates selector syntax.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}
```

- [ ] **Step 3: Standardize the internal lookup panic documentation**

Replace only the `resolved_at` rustdoc with:

```rust
/// Borrows the resolved provider at an internal entry position.
///
/// `index` identifies an entry in this registry. Returns a lookup wrapper
/// borrowing that entry.
///
/// # Panics
///
/// Panics when `index` is outside the registry's entry array. Registry-owned
/// indexes satisfy this invariant.
```

- [ ] **Step 4: Verify this task**

Run:

```bash
cargo fmt --check
cargo test --test provider_id_tests --test provider_selector_tests --test provider_registry_tests
RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc --no-deps
git diff --check
git diff -- src/provider_id.rs src/provider_selector.rs src/provider_registry.rs
```

Expected: focused tests and strict rustdoc pass; the diff contains comments only.

---

### Task 4: Audit all source declarations and run final verification

**Files:**

- Audit: `src/lib.rs`
- Audit: `src/created_service.rs`
- Audit: `src/provider_descriptor.rs`
- Audit: `src/provider_error.rs`
- Audit: `src/provider_id.rs`
- Audit: `src/provider_registration.rs`
- Audit: `src/provider_registry.rs`
- Audit: `src/provider_registry_builder.rs`
- Audit: `src/provider_resolver.rs`
- Audit: `src/provider_selection.rs`
- Audit: `src/provider_selector.rs`
- Audit: `src/registration_error.rs`
- Audit: `src/resolution_error.rs`
- Audit: `src/service_provider.rs`
- Audit: `src/service_spec.rs`

**Interfaces:**

- Consumes: all source declarations and the documentation added in Tasks 1–3.
- Produces: a complete private-and-public rustdoc audit with warning-free docs, lint, tests, and doctests.

- [ ] **Step 1: Enumerate every type, function, and associated type with context**

Run:

```bash
rg -n -B 5 '^\s*(pub(\([^)]*\))?\s+)?(struct|enum|trait)\s+|^\s*(pub(\([^)]*\))?\s+)?(const\s+)?(async\s+)?(unsafe\s+)?fn\s+|^\s*type\s+' src
```

Expected: every declaration is preceded by rustdoc explaining purpose and use; every callable item covers meaningful parameters, return values, errors, optional values, constraints, and side effects. Add only missing or inaccurate rustdoc discovered by this audit.

- [ ] **Step 2: Enumerate fields and enum variants with context**

Run:

```bash
rg -n -B 2 '^\s*(pub(\([^)]*\))?\s+)?[a-z_][a-z0-9_]*\s*:\s*' src
rg -n -B 2 '^\s*[A-Z][A-Za-z0-9_]*(\([^;]*\)|\s*\{)?[,]?$' src
```

Expected: actual stored fields and variants, after excluding function parameters, generic bounds, expressions, and constructors, are each preceded by rustdoc explaining their meaning. Add only missing comments.

- [ ] **Step 3: Run formatting and strict rustdoc**

Run:

```bash
cargo fmt --check
RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc --no-deps
```

Expected: both commands exit successfully; rustdoc reports no broken links, invalid rustdoc syntax, warnings, or missing public documentation.

- [ ] **Step 4: Run lint and the full test suite**

Run:

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Expected: Clippy passes; every integration test and the crate-level doctest passes.

- [ ] **Step 5: Prove the final source diff is documentation-only**

Run:

```bash
git diff --check
git status --short --branch
git diff --stat
git diff -- src
```

Expected: only the implementation-plan file and rustdoc comments are uncommitted; no source signature, body, attribute, import, test, dependency, or configuration line changed.

- [ ] **Step 6: Stop before committing**

Report the changed files, audit result, verification commands, and exact outcomes. Do not stage or commit the plan or implementation changes without explicit user authorization.
