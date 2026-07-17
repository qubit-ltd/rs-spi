# Qubit SPI User Guide

This guide first explains the problem Qubit SPI solves and the boundaries of
its model. It then builds one service from a small example, expands that example
into a realistic multi-Provider scenario, and explains the public API in detail.

## Why Qubit SPI Exists

An application usually depends on a capability rather than one concrete
implementation. A MIME subsystem needs to detect media types, but the best
implementation can vary by deployment: one environment may have a trained
model, another may expose a system command, and a restricted environment may
need a built-in fallback.

The application still needs one stable `MimeDetector` interface. What changes
is how that interface is constructed and which implementation is usable now.
That distinction is the reason Qubit SPI exists.

Without a shared model, each service family tends to grow its own configuration
parser, factory map, priority rules, fallback loop, and error format. The code
may start as one `match` expression, but it becomes difficult to answer basic
production questions:

- Which implementation was requested, and which one actually won?
- Was a missing implementation skipped intentionally or because of a defect?
- Which failures permit fallback, and which must stop resolution?
- Do aliases, priorities, and attempt ordering behave consistently across
  service families?

Qubit SPI puts those decisions in one typed and explicit lifecycle.

## The Problem It Solves

The crate separates concerns that handwritten selection code commonly mixes:

| Concern | Qubit SPI owner |
| --- | --- |
| Business operations | An application-defined Service trait such as `MimeDetector` |
| Construction input and output type | `ServiceSpec` |
| Constructing one implementation | `ServiceProvider::create` |
| Canonical name, aliases, and priority | `ProviderDescriptor` |
| Startup assembly and conflict checking | `ProviderRegistryBuilder` |
| Immutable lookup catalog | `ProviderRegistry` |
| Candidate order, fallback, and diagnostics | `ProviderResolver` |

The result is not automatic dependency injection. The application still
chooses which Providers to register, builds the Registry explicitly, supplies
construction configuration, and decides when to create a service. Rust checks
that every Provider in one service family accepts the same configuration type
and returns the same complete service type.

## When to Use It

Qubit SPI is useful when all of the following are true:

- one application capability has two or more interchangeable implementations;
- implementations are chosen by configuration, environment, preference order,
  or availability;
- construction can fail in ways that need different fallback behavior; and
- the caller needs deterministic selection and structured diagnostics.

Typical service families include MIME detectors, filesystems, serializers,
cryptographic engines, model backends, and platform-specific adapters.

Do not add this crate merely to wrap one implementation. It also does not load
dynamic libraries, discover code from the filesystem, manage arbitrary object
graphs, or cache created services. Provider discovery and registration remain
explicit application responsibilities.

## The Mental Model

Start with the business capability and work outward:

| Role | First-principles meaning |
| --- | --- |
| Service | A reusable capability whose methods handle business requests. |
| Provider | A factory that knows how to construct one Service implementation. |
| Config | Construction-time input such as paths, endpoints, credentials, or defaults. |
| Output | The complete Service value or handle returned by the factory. |
| Descriptor | Registration metadata used to identify and order the factory. |
| Registry | The immutable catalog of factories assembled during startup. |
| Resolver | The policy engine that chooses factories and calls `create`. |
| CreatedService | The usable Service plus the canonical ID of the factory that created it. |

The complete lifecycle is:

```text
define Service capability
  -> bind Config and Output with ServiceSpec
  -> implement one ServiceProvider factory per backend
  -> register factories and metadata during startup
  -> select candidates with named / auto / chain
  -> call Provider::create(config)
  -> retain the returned Service and call its business methods
```

The key boundary is that `create` constructs a Service. It does not perform a
single business operation. In the MIME example, database paths and default
types are construction configuration; a file name and its bytes belong to a
later `detect` call.

## A Five-Minute Example

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

struct MimeConfig {
    default_type: String,
}

struct MimeDetectorSpec;

impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}

struct ExtensionDetector {
    default_type: String,
}

impl MimeDetector for ExtensionDetector {
    fn detect(&self, file_name: &str, _content: &[u8]) -> &str {
        if file_name.ends_with(".png") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

struct ExtensionProvider;

impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(ExtensionDetector {
            default_type: config.default_type.clone(),
        }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("extension")?),
        ExtensionProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let config = MimeConfig {
        default_type: "application/octet-stream".to_owned(),
    };
    let created = resolver.create_named("extension", &config)?;

    assert_eq!("extension", created.provider_id().as_str());
    assert_eq!(
        "image/png",
        created.service().detect("photo.png", b"PNG contents"),
    );
    Ok(())
}
```

This guide describes `qubit-spi` 0.8, which requires Rust 1.94 or later. The
example registers one Provider factory, creates an extension-based detector,
and then uses that detector to classify a PNG file. The winning Provider ID is
returned beside the service handle.

Add the crate to your application before running the example:

```toml
[dependencies]
qubit-spi = "0.8"
```

## How the First Example Works

### 1. Define the Service Capability

`MimeDetector` is the interface business code needs. Its `detect` method accepts
one file request at a time. Neither the Resolver nor the Provider performs this
operation on the application's behalf.

### 2. Separate Construction Configuration

`MimeConfig` contains the default media type needed while constructing a
detector. It does not contain `file_name` or `content`, because those values vary
for every business call after the detector has been created.

### 3. Bind the Provider Contract

`MimeDetectorSpec` binds `MimeConfig` to `Arc<dyn MimeDetector>`. Rust therefore
requires every `ServiceProvider<MimeDetectorSpec>` to accept `&MimeConfig` and
return the same complete, shareable detector handle.

### 4. Implement a Factory

`ExtensionProvider::create` validates construction configuration and builds an
`ExtensionDetector`. A `ProviderError` describes why construction could not
complete; successful creation returns the Service itself, not a detection
result.

### 5. Register and Select the Factory

`ProviderDescriptor` gives the factory the canonical ID `extension`.
`ProviderRegistry::builder()` collects factories during startup, and `build()`
freezes the catalog. `create_named("extension", &config)` selects exactly that
factory and calls `create`.

### 6. Use the Created Service

The returned `CreatedService` exposes the detector through `service()` and the
winning canonical ID through `provider_id()`. Only now does business code call
`detect("photo.png", ...)`. The same detector can handle later files without
another resolution call.

## Complete Annotated Example

The next program models a realistic MIME detector family with two Provider
factories, aliases, priorities, three selection modes, fallback, and structured
diagnostics. It stays self-contained: a production Magic Provider would load a
real database, while this example keeps only enough behavior to demonstrate the
construction boundary. Read the comments in order; each explains why that part
exists and what changes at runtime.

```rust
use std::{
    path::PathBuf,
    sync::Arc,
};

use qubit_spi::error::{AttemptFailure, ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ResolutionTermination, ServiceProvider, ServiceSpec,
};

/*
 * This trait is the application-facing Service. detect() handles changing
 * business input after construction. Returning an Arc from the SPI lets the
 * application retain and share one selected detector without knowing its
 * concrete implementation.
 */
trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

/*
 * Config contains only values needed to construct a detector. The name and
 * bytes of an individual file do not belong here; they are supplied later to
 * detect().
 */
struct MimeConfig {
    default_type: String,
    magic_database: Option<PathBuf>,
}

struct MimeDetectorSpec;

/*
 * ServiceSpec is the compile-time contract shared by every Provider factory.
 * Both factories must accept MimeConfig and return the same complete Service
 * handle, so selection never changes the type seen by business code.
 */
impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}

struct MagicDatabaseDetector {
    _database: PathBuf,
    default_type: String,
}

/*
 * A created Service performs MIME detection repeatedly. This compact detector
 * recognizes one signature so the example needs no external database. A real
 * implementation would retain and query the database loaded during create().
 */
impl MimeDetector for MagicDatabaseDetector {
    fn detect(&self, _file_name: &str, content: &[u8]) -> &str {
        if content.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

/*
 * This fallback Service uses the file name instead of a content database. It
 * still implements the same MimeDetector contract, so business code is
 * independent of which Provider created the detector.
 */
struct ExtensionDetector {
    default_type: String,
}

impl MimeDetector for ExtensionDetector {
    fn detect(&self, file_name: &str, _content: &[u8]) -> &str {
        if file_name.to_ascii_lowercase().ends_with(".png") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

/*
 * Provider types are factories, not Services and not registration identities.
 * MagicDatabaseProvider is responsible only for constructing one usable
 * MagicDatabaseDetector from the shared initialization configuration.
 */
struct MagicDatabaseProvider;

impl ServiceProvider<MimeDetectorSpec> for MagicDatabaseProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        /*
         * No database means this backend cannot run in the current deployment.
         * Unavailable tells OnAbsence that another Provider may be tried.
         */
        let Some(database) = &config.magic_database else {
            return Err(ProviderError::unavailable(
                "no magic database is configured",
            ));
        };

        /*
         * A configured path with the wrong form is a caller configuration
         * error, not environmental absence. OnAbsence must stop instead of
         * hiding it behind another backend.
         */
        if database.extension().and_then(|value| value.to_str()) != Some("mgc") {
            return Err(ProviderError::invalid_configuration(
                "magic_database must point to an .mgc file",
            ));
        }
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(MagicDatabaseDetector {
            _database: database.clone(),
            default_type: config.default_type.clone(),
        }))
    }
}

struct ExtensionProvider;

impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        /*
         * This factory creates a complete fallback Service. It validates only
         * construction configuration; file-specific work remains in detect().
         */
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(ExtensionDetector {
            default_type: config.default_type.clone(),
        }))
    }
}

fn build_resolver() -> Result<ProviderResolver<MimeDetectorSpec>, Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();

    /*
     * Canonical IDs are stable observability identities; aliases are accepted
     * configuration names. Priority 100 makes magic the first automatic
     * candidate without changing caller-controlled named or chained order.
     */
    builder.register(
        ProviderDescriptor::new(ProviderId::new("magic")?)
            .with_aliases(["content", "libmagic"])?
            .with_priority(100),
        MagicDatabaseProvider,
    )?;
    builder.register(
        ProviderDescriptor::new(ProviderId::new("extension")?)
            .with_aliases(["filename", "suffix"])?
            .with_priority(10),
        ExtensionProvider,
    )?;

    /*
     * build() ends mutable startup assembly. The resolver shares the resulting
     * immutable Registry and applies one explicit fallback policy at runtime.
     * OnAbsence permits an unavailable backend to fall back while protecting
     * invalid configuration and unexpected initialization failures.
     */
    Ok(ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    ))
}

/*
 * ResolutionError exposes structured diagnostics. Branching on these values
 * is stable and testable; parsing its Display text would couple the program to
 * wording intended only for people.
 */
fn report_resolution_error(error: &ResolutionError) {
    match error.termination() {
        Some(ResolutionTermination::Exhausted) => {
            eprintln!("all admitted candidates were exhausted");
        }
        Some(ResolutionTermination::StoppedByPolicy) => {
            eprintln!("fallback policy stopped candidate traversal");
        }
        Some(_) => eprintln!("resolution ended for a newer reason"),
        None => eprintln!("resolution failed before candidate traversal"),
    }

    for (index, attempt) in error.attempts().iter().enumerate() {
        match attempt {
            AttemptFailure::UnknownProvider {
                requested_selector, ..
            } => eprintln!("attempt {index}: unknown selector {requested_selector}"),
            AttemptFailure::ProviderError {
                requested_selector,
                provider_id,
                error,
                ..
            } => eprintln!(
                "attempt {index}: selector {requested_selector:?} reached {provider_id}, \
                 which failed with {:?}: {}",
                error.kind(),
                error.reason(),
            ),
            _ => eprintln!("attempt {index}: newer failure kind"),
        }
    }

    match error.decisive_attempt() {
        Some(attempt) => eprintln!("decisive attempt: {attempt}"),
        None => eprintln!("no single attempt explains the whole outcome"),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = build_resolver()?;
    let config = MimeConfig {
        default_type: "application/octet-stream".to_owned(),
        magic_database: None,
    };
    let png_header = b"\x89PNG\r\n\x1a\n";

    /*
     * Automatic selection follows priority order. magic is tried first, but a
     * missing database produces Unavailable, so OnAbsence reaches extension.
     * The returned value is a reusable detector plus the winning canonical ID.
     */
    let automatic = resolver.create_auto(&config)?;
    assert_eq!("extension", automatic.provider_id().as_str());
    assert_eq!(
        "image/png",
        automatic.service().detect("photo.png", png_header),
    );

    /*
     * Named selection resolves exactly one canonical ID or alias. filename
     * maps to extension. create_named constructs that Service once and never
     * falls back to magic; detect() is a separate business operation.
     */
    let named = resolver.create_named("filename", &config)?;
    assert_eq!("extension", named.provider_id().as_str());
    assert_eq!(
        "application/octet-stream",
        named.service().detect("README", b"plain text"),
    );

    /*
     * Chained selection preserves caller order. missing is recorded as an
     * unknown selector, content reaches unavailable magic, and suffix finally
     * creates the extension Service. Aliases of one Provider are deduplicated.
     */
    let chained = resolver.create_chain(["missing", "content", "suffix"], &config)?;
    assert_eq!("extension", chained.provider_id().as_str());

    /*
     * This second construction request deliberately fails. magic is still
     * unavailable, then extension rejects the empty construction default.
     * OnAbsence stops because invalid configuration is not absence.
     */
    let invalid_config = MimeConfig {
        default_type: "  ".to_owned(),
        magic_database: None,
    };
    let failure = resolver
        .create_auto(&invalid_config)
        .err()
        .expect("the invalid configuration must fail");
    assert_eq!(
        Some(ResolutionTermination::StoppedByPolicy),
        failure.termination(),
    );
    report_resolution_error(&failure);

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fatal error: {error}");
        std::process::exit(1);
    }
}
```

With no magic database configured, automatic selection reaches `magic`,
continues after its `Unavailable` error, and returns `extension`. Named and
chained selection also create extension detectors. The final deliberately
invalid construction stops at `ExtensionProvider` and exercises structured
diagnostics. Each successful Resolver call creates a new detector; the example
then calls business methods on those returned Services.

## Service Contracts

Use this when introducing an independently configured family of Provider
implementations.

Focused excerpt from the complete example:

```rust,ignore
trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

struct MimeConfig {
    default_type: String,
    magic_database: Option<PathBuf>,
}

struct MimeDetectorSpec;

impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}
```

The observable result is one compile-time contract: every
`ServiceProvider<MimeDetectorSpec>` receives `&MimeConfig` and returns
`Arc<dyn MimeDetector>`.

`Config` may be unsized, so a service may use a view such as `str` or a trait
object. `Output` is the complete caller-owned value; choose a plain value,
`Box<dyn Trait>`, `Arc<dyn Trait>`, or another handle according to the
application's ownership and concurrency needs. For a Service Provider family,
`Output` should normally be the complete reusable Service or its handle, not the
result of one business method. The SPI never inserts or removes a wrapper.

**Common mistake:** defining one overly broad specification for unrelated
services. Use a separate marker type whenever configuration, output, Provider
set, or selection policy should evolve independently.

## What create Actually Does

`ServiceProvider::create` is the boundary between selecting a factory and using
the Service it constructs. It receives borrowed construction configuration and
must return one complete `S::Output` that is ready for business calls.

Focused excerpt from the complete example:

```rust,ignore
impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(ExtensionDetector {
            default_type: config.default_type.clone(),
        }))
    }
}
```

The factory may validate Provider-specific configuration, check whether a
required executable or model is available, initialize a client or engine, and
wrap the concrete implementation behind the output handle. It must not process
one file on behalf of `MimeDetector::detect`; file-specific values are not
construction configuration.

The Resolver invokes this method whenever traversal reaches the Provider. A
named call invokes at most one factory. Automatic and chained calls may invoke
several factories until one succeeds or policy stops traversal. Calling
`create_auto`, `create_named`, `create_chain`, or `create` again performs a new
resolution and may create another Service: Qubit SPI does not memoize outputs.

`create` is synchronous. A Provider that requires asynchronous network setup
should normally construct a lazy async-capable client, complete unavoidable
synchronous setup deliberately, or place asynchronous initialization outside
this interface. Hidden long-running I/O inside `create` can make resolution
block unexpectedly.

Provider implementations must be `Send + Sync + 'static`; the Registry retains
and may share the factories themselves. The configuration is borrowed, while a
new complete output is returned for every successful factory call.

Choose the error classification by meaning because it controls fallback:

| `ProviderError` constructor | Meaning | `OnAbsence` |
| --- | --- | --- |
| `unsupported` | This Provider cannot construct the requested capability or configuration. | Continues |
| `unavailable` | This Provider cannot run in the current environment. | Continues |
| `invalid_configuration` | The caller supplied invalid construction settings. | Stops |
| `initialization_failed` | Constructing this implementation failed unexpectedly. | Stops |

Each classification also has a `_with_source` constructor for retaining an
underlying `Error + Send + Sync + 'static`.

**Common mistake:** reporting invalid configuration as `Unavailable`. That can
hide a caller error by letting `OnAbsence` silently choose a different
Provider.

## Provider Identity and Ranking

Use this when assigning stable identity, accepted configuration names, and
automatic-selection order to a factory registration.

Focused excerpt from the complete example:

```rust,ignore
let magic = ProviderDescriptor::new(ProviderId::new("magic")?)
    .with_aliases(["content", "libmagic"])?
    .with_priority(100);
```

This descriptor makes `magic` the canonical ID, accepts `content` and
`libmagic` as aliases, and gives it priority 100 for automatic selection.

Canonical `ProviderId` values are strict lowercase ASCII tokens. They must
start and end with an ASCII alphanumeric character; interior characters may
also include `-`, `_`, `.`, and `+`. `ProviderId::new` neither trims nor
normalizes input. By contrast, runtime `ProviderSelector` input is trimmed and
ASCII-lowercased before validation, so `" LIBMAGIC "` resolves the alias
`libmagic`.

Aliases use the same selector namespace as canonical IDs. A descriptor rejects
an invalid alias, an alias equal to its own ID, or duplicate aliases. The
Builder rejects a selector already claimed by another registration. Priority
affects only `create_auto`; named and chained selection follow the caller's
selector or order.

Invalid canonical IDs return `ProviderIdError`; invalid or duplicate aliases
return `ProviderDescriptorError`.

**Common mistake:** treating an alias as the Provider's identity. Results and
diagnostics always report the canonical ID, even when an alias was requested.

## Building the Registry

Use this when assembling all available factories during application startup or
examining the immutable catalog later.

Focused excerpt using the complete-example types:

```rust,ignore
let shared_magic: Arc<dyn ServiceProvider<MimeDetectorSpec>> =
    Arc::new(MagicDatabaseProvider);
let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("extension")?),
    ExtensionProvider,
)?;
builder.register_shared(
    ProviderDescriptor::new(ProviderId::new("magic")?),
    shared_magic,
)?;

let registry = builder.build();
assert_eq!(2, registry.len());
assert!(!registry.is_empty());
for descriptor in registry.descriptors() {
    println!("{}", descriptor.id());
}
```

`register` moves an owned concrete Provider into Registry storage.
`register_shared` accepts an existing `Arc<dyn ServiceProvider<S>>`. A
registration is transactional: all canonical and alias conflicts are checked
before mutation, so a rejected registration reserves no partial selector set.
Conflicts are reported as `RegistrationError`.

`build()` consumes the mutable Builder and prepares immutable lookup and
automatic-order indexes. Runtime inspection is read-only:

- `len()` and `is_empty()` report catalog size;
- `descriptors()` and `provider_ids()` iterate in registration order;
- `find(raw)` returns `None` for both invalid and unknown input;
- `resolve(raw)` returns a `ResolvedProvider` or distinguishes invalid and
  unknown input with `ResolutionError`.

A `ResolvedProvider` borrows one Registry entry. Its `descriptor()` method
exposes registration metadata, and `create(config)` calls that one factory
directly. Direct creation returns `ProviderError` and deliberately bypasses
Resolver fallback and `CreatedService` winner tracking. The crate provides no
global Registry or implicit discovery.

**Common mistake:** keeping the Builder as mutable runtime state. Finish
registration during startup, build once, and share the Registry or Resolver.

## Selecting Providers

Use this when deciding whether configuration names one Provider, requests the
best available Provider, or supplies an ordered preference list.

Focused excerpt from the complete example:

```rust,ignore
let automatic = resolver.create_auto(&config)?;
let named = resolver.create_named("filename", &config)?;
let chained = resolver.create_chain(["missing", "content", "suffix"], &config)?;
```

| Need | Raw Resolver method | Candidate order |
| --- | --- | --- |
| Best available implementation | `create_auto` | Priority descending, then canonical ID ascending |
| Exactly one configured implementation | `create_named` | The one canonical ID or alias |
| Ordered preferences | `create_chain` | Caller-provided selector order |

Named selection never tries a second Provider. Automatic selection from an
empty Registry returns `ResolutionError::EmptyRegistry`. A chain must be
nonempty, and every selector is validated before any Provider is called.
Unknown valid selectors are recorded as ordered attempts. If two chain entries
are aliases for the same Provider, that Provider is invoked only once.

The observable winner is always available through
`CreatedService::provider_id()`.

**Common mistake:** expecting priority to reorder a chain. Priority applies
only to automatic selection; a chain preserves caller order.

### Reuse Validated Selections

Use a `ProviderSelection` when the same configured choice is applied to many
creation calls. This focused excerpt omits the Resolver and MIME configuration
already built in the complete example:

```rust,ignore
use qubit_spi::ProviderSelection;

let selection = ProviderSelection::chain(["content", "suffix"])?;

let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

`ProviderSelection::auto()` is infallible. `named(...)` normalizes and validates
one selector; `chain(...)` validates every selector, preserves order, and
rejects an empty chain. `ProviderResolver::create` reuses that validated value.
`Default` is automatic selection; `kind()` reports the mode, `selector()`
borrows the named selector when present, and `selectors()` returns the chain or
an empty slice for the other modes.

Raw `create_named` and `create_chain` are preferable at runtime input boundaries
because they convert parse failures into `ResolutionError` while preserving the
invalid input and chain index. They parse and allocate owned selector data on
every call. A reusable `ProviderSelection` moves that work to configuration
loading and reports validation as `ProviderSelectionError`.

Reusing a selection avoids reparsing names; it does not cache a created Service.
Both `first` and `second` perform resolution and may invoke Provider factories.

## Fallback and Error Classification

Use this when deciding which Provider failures may be hidden by trying a later
candidate.

Focused excerpt from the complete example:

```rust,ignore
let safe_resolver = ProviderResolver::new(registry.clone(), FallbackPolicy::OnAbsence);
let best_effort_resolver = ProviderResolver::new(registry, FallbackPolicy::OnAnyError);
```

| Policy | Continues after | Stops after |
| --- | --- | --- |
| `OnAbsence` | `Unsupported`, `Unavailable` | `InvalidConfiguration`, `InitializationFailed` |
| `OnAnyError` | Every `ProviderError` | No Provider error by classification |

`OnAbsence` is also the `FallbackPolicy::default()` value.

Fallback covers only failures returned while constructing a Service. Once a
Provider has returned `Arc<dyn MimeDetector>`, errors or results from later
`detect` calls belong to that Service's API; the Resolver does not revisit the
Provider chain for business-operation failures.

An unknown selector inside a chain is recorded and traversal continues because
no Provider was invoked. The policy matters only when another candidate
exists; named selection still has exactly one candidate. A policy stop produces
`ResolutionTermination::StoppedByPolicy`; visiting all admitted candidates
produces `ResolutionTermination::Exhausted`.

**Common mistake:** choosing `OnAnyError` merely to make a request succeed.
That policy can mask configuration and initialization defects, so use it only
for an explicitly best-effort workflow.

## Successful Results and Diagnostics

### Inspect Successful Results

Use this when consuming the output or recording which Provider actually won.

Focused excerpt from the complete example:

```rust,ignore
let created = resolver.create_named("filename", &config)?;
println!("winner: {}", created.provider_id());
let media_type = created.service().detect("photo.png", png_header);

let (provider_id, service) = created.into_parts();
```

`provider_id()` and `service()` borrow the two values. `into_service()` consumes
the wrapper and returns only the output. `into_parts()` consumes it and returns
`(ProviderId, Output)`. The ID is always canonical, so observability does not
depend on which alias a caller used.

**Common mistake:** discarding the winning ID before logging or metrics. It is
the simplest way to confirm fallback behavior in production.

### Diagnose Failures

Use this when resolution cannot create a service and the caller needs a stable,
structured explanation.

Focused excerpt from the complete example:

```rust,ignore
match error.termination() {
    Some(ResolutionTermination::Exhausted) => println!("all candidates failed"),
    Some(ResolutionTermination::StoppedByPolicy) => println!("policy stopped"),
    Some(_) => println!("newer termination reason"),
    None => println!("failure occurred before traversal"),
}

for attempt in error.attempts() {
    println!("{attempt}");
}
```

Direct `ResolutionError` variants distinguish these boundaries:

| Variant | Meaning |
| --- | --- |
| `InvalidSelector` | Raw input failed normalization or grammar validation; a chain index is included when applicable. |
| `EmptySelection` | A raw or validated chain contained no selectors. |
| `UnknownProvider` | A valid direct named selector matched no registration. |
| `EmptyRegistry` | Automatic selection was requested from an empty Registry. |
| `NoProviderSucceeded` | Candidate attempts failed or policy stopped traversal. |

For aggregate failure, `attempts()` returns ordered `AttemptFailure` values.
Each attempt is either an unknown selector or a Provider error retaining the
requested selector, canonical Provider ID, classification, reason, and optional
source. `termination()` distinguishes exhaustion from a policy stop, while
`terminal_attempt()` returns the last recorded attempt.

`decisive_attempt()` returns the last attempt after a policy stop and the only
attempt after single-attempt exhaustion. It returns `None` for multi-attempt
exhaustion because no one failure explains the whole result. `is_absence()` is
true only for unknown, unsupported, and unavailable outcomes.

The standard `Error::source()` chain exposes a selector parser error or an
unambiguous decisive attempt; a Provider attempt in turn exposes its
`ProviderError`, whose `_with_source` constructors retain an underlying cause.
All public error enums are non-exhaustive, so downstream matches need a
wildcard arm. Use fields and accessors for control flow; `Display` text is for
people, not parsing.

**Common mistake:** using the final attempt as the cause of every exhausted
chain. For multi-attempt exhaustion, inspect all ordered attempts instead.

## Lifetime, Sharing, and Performance

Both `ProviderRegistry` and `ProviderResolver` are cheaply cloneable. A Registry
stores immutable entries and indexes behind an internal `Arc`; cloning it does
not duplicate Providers. A Resolver clone shares that Registry and copies its
small fallback policy value. `registry()` and `fallback_policy()` provide
read-only access to the Resolver configuration.

This sharing applies to Provider factories and registration metadata, not to
the Services those factories create. Every Resolver creation call starts a new
traversal and may invoke `ServiceProvider::create` again. The crate has no
singleton scope, memoization, or output cache.

For an expensive detector, client, engine, or connection pool, resolve it once
during startup, retain the returned `Arc<dyn MimeDetector>`, and clone that
output handle for consumers. Repeated Resolver calls are appropriate only when
a new Service instance is actually desired or construction configuration has
changed.

Providers are `Send + Sync + 'static`, so the catalog can be shared. The SPI
places no `Send` or `Sync` bound on `ServiceSpec::Output`; choose a thread-safe
output such as `Arc<dyn Trait + Send + Sync>` when the created service itself
must cross threads. In this guide, `MimeDetector: Send + Sync` makes
`Arc<dyn MimeDetector>` suitable for that use.

Raw selector parsing normalizes and owns selector text so errors and selections
can retain it safely. Reuse `ProviderSelection` when that allocation matters.
Registry lookup and automatic order use indexes prepared once by `build()`.

## Recommended Practices

- Assemble Providers once during startup and fail startup on registration
  errors.
- Keep construction inputs in `Config`; pass per-operation requests to Service
  methods.
- Return a complete Service handle from `create`, then retain and share it when
  construction is expensive.
- Keep canonical IDs stable; use aliases for accepted legacy or friendly
  names.
- Set explicit priorities only when automatic preference has product meaning.
- Prefer `OnAbsence`; adopt `OnAnyError` only for documented best-effort
  behavior.
- Validate reusable configuration into `ProviderSelection` before the hot
  path.
- Log the winning canonical Provider ID on success.
- Match structured errors and retain their source chain instead of parsing
  messages.
- Test named, automatic, chain, policy-stop, and exhaustion behavior with small
  purpose-built registries.

## Troubleshooting

| Symptom | Cause | What to do |
| --- | --- | --- |
| `ProviderId::new` rejects a name | Canonical IDs are not trimmed or lowercased and must follow the lowercase ASCII token grammar. | Normalize configuration before choosing a canonical ID, then pass a stable valid token. |
| Registration reports a selector conflict | A canonical ID or alias is already owned by another Provider. | Inspect both descriptors; rename or remove the overlapping selector. The failed registration changed nothing. |
| `create_auto` returns `EmptyRegistry` | No Provider was registered before `build()`. | Register at least one Provider or treat the service family as optional before resolving. |
| A chain is empty or invalid | Empty chains are rejected, and every selector is validated before traversal. | Validate configuration at startup and use the reported selector index/input. |
| Traversal stops earlier than expected | `OnAbsence` encountered `InvalidConfiguration` or `InitializationFailed`. | Fix the input or initialization problem; use `OnAnyError` only if masking it is intentional. |
| Expensive initialization runs repeatedly | Each Resolver creation call invokes Provider factories again; outputs are not cached. | Resolve once during startup and retain or clone the returned Service handle. |
| A Service method fails but another Provider is not tried | Fallback ends when a Provider successfully creates a Service. | Handle business-operation errors in the Service API; perform another explicit resolution only if the application truly wants a new Service. |
| `decisive_attempt()` returns `None` | The error is non-aggregate or multiple exhausted attempts jointly explain the outcome. | Inspect `attempts()`, `termination()`, and `terminal_attempt()` instead of assuming one cause. |
| `find` returns `None` but the reason is unclear | `find` intentionally combines invalid and unknown input. | Use `resolve` when the caller needs a structured distinction. |
| An alias was requested but another ID is reported | Success and Provider failures report canonical identity. | Use the canonical ID for logs and treat aliases only as accepted input. |

## API Reference

| Role | API |
| --- | --- |
| Bind configuration and output types | [`ServiceSpec`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/trait.ServiceSpec.html) |
| Implement a factory | [`ServiceProvider`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/trait.ServiceProvider.html) |
| Represent canonical and runtime lookup names | [`ProviderId`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderId.html), [`ProviderSelector`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderSelector.html) |
| Define identity, aliases, and priority | [`ProviderDescriptor`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderDescriptor.html) |
| Assemble registrations | [`ProviderRegistryBuilder`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderRegistryBuilder.html) |
| Inspect and resolve the immutable catalog | [`ProviderRegistry`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderRegistry.html) |
| Use one directly resolved factory | [`ResolvedProvider`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ResolvedProvider.html) |
| Store reusable validated selection | [`ProviderSelection`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderSelection.html) |
| Apply selection and fallback | [`ProviderResolver`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderResolver.html) |
| Consume output and winning ID | [`CreatedService`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.CreatedService.html) |
| Choose fallback behavior | [`FallbackPolicy`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/enum.FallbackPolicy.html) |
| Interpret aggregate termination | [`ResolutionTermination`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/enum.ResolutionTermination.html) |
| Inspect attempts and factory classifications | [`AttemptFailure`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.AttemptFailure.html), [`ProviderErrorKind`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ProviderErrorKind.html) |
| Classify factory failure and diagnose resolution | [`ProviderError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/struct.ProviderError.html), [`ResolutionError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ResolutionError.html) |
| Handle validation, registration, Provider, and resolution errors | [`qubit_spi::error`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/index.html) |
