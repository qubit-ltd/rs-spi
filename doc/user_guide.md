# Qubit SPI User Guide

This guide starts with a working program, expands it into a realistic example,
and then explains every public usage decision in detail.

## Start Here: A Five-Minute Example

```rust
use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = ();
    type Output = &'static str;
}

struct EnglishProvider;

impl ServiceProvider<GreetingSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<&'static str, ProviderError> {
        Ok("hello")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("english")?),
        EnglishProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let created = resolver.create_named("english", &())?;

    assert_eq!("english", created.provider_id().as_str());
    assert_eq!("hello", *created.service());
    Ok(())
}
```

This guide describes `qubit-spi` 0.8, which requires Rust 1.94 or later. The
example builds a Registry containing one Provider, selects `english`, and gets
the service value `"hello"` together with the winning Provider ID.

Add the crate to your application before running the example:

```toml
[dependencies]
qubit-spi = "0.8"
```

## How the Core Flow Works

### 1. Define the Input and Output

The `impl ServiceSpec for GreetingSpec` block says that every Provider in this
service family receives `&()` and must return `&'static str`. `GreetingSpec` is
only a marker that gives those two types one name.

### 2. Implement a Factory

`EnglishProvider::create` is a factory operation. It receives the configuration
selected by `GreetingSpec` and either creates the complete output or returns a
classified `ProviderError`.

### 3. Assign Provider Identity

`ProviderDescriptor::new(ProviderId::new("english")?)` gives the factory a
stable canonical ID. Identity belongs to the registration, so the Provider
type itself does not need to know its configured name, aliases, or priority.

### 4. Assemble the Registry

`ProviderRegistry::builder()` starts the mutable startup phase. Each
`register` call checks identity conflicts. `build()` consumes the
`ProviderRegistryBuilder` and returns an immutable `ProviderRegistry` for
runtime lookup and sharing.

### 5. Resolve and Create

`ProviderResolver::new` combines the Registry with a `FallbackPolicy`.
`create_named("english", &())` normalizes the selector, finds that one
Provider, invokes its factory, and returns the result.

### 6. Use the Result

The returned `CreatedService` exposes `service()` for the output and
`provider_id()` for the canonical ID that actually succeeded. Keeping the
winner is useful for logs, metrics, and support diagnostics.

The complete flow is therefore:

```text
ServiceSpec -> ServiceProvider -> ProviderDescriptor -> Registry Builder
            -> immutable Registry -> Resolver -> CreatedService
```

## Complete Annotated Example

The next program adds a real service trait, two Providers, aliases, priorities,
three selection modes, fallback, and structured diagnostics. Read the comments
in order; each one explains why that part exists and what changes at runtime.

```rust
use std::sync::Arc;

use qubit_spi::error::{AttemptFailure, ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ResolutionTermination, ServiceProvider, ServiceSpec,
};

/*
 * The application-facing trait is the useful service. Returning an Arc from
 * the SPI lets callers cheaply clone one thread-safe handle without knowing
 * which concrete provider created it.
 */
trait Greeter: Send + Sync {
    fn greet(&self) -> String;
}

struct TextGreeter {
    message: String,
}

impl Greeter for TextGreeter {
    fn greet(&self) -> String {
        self.message.clone()
    }
}

/*
 * The specification is the compile-time contract shared by every provider:
 * all factories receive the same configuration and return the same complete
 * caller-owned service handle.
 */
struct GreetingConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = GreetingConfig;
    type Output = Arc<dyn Greeter>;
}

/*
 * Providers are factories, not registration identities. Keeping names and
 * ranking out of these types lets startup code reuse a factory implementation
 * while choosing deployment-specific metadata independently.
 */
struct CloudProvider;

impl ServiceProvider<GreetingSpec> for CloudProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        /*
         * Unavailable means this provider is valid for the request but cannot
         * serve it now. OnAbsence may therefore continue to another provider.
         */
        if !config.cloud_available {
            return Err(ProviderError::unavailable(
                "the cloud greeting service is offline",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from cloud", config.prefix),
        }))
    }
}

struct LocalProvider;

impl ServiceProvider<GreetingSpec> for LocalProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        /*
         * InvalidConfiguration identifies a caller error. OnAbsence stops on
         * it instead of hiding the bad input by trying more providers.
         */
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from local", config.prefix),
        }))
    }
}

fn build_resolver() -> Result<ProviderResolver<GreetingSpec>, Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();

    /*
     * Canonical IDs are stable identities; aliases are accepted input names.
     * Priority 100 makes cloud the first automatic candidate, while explicit
     * named and chained selections still follow the caller's choice.
     */
    builder.register(
        ProviderDescriptor::new(ProviderId::new("cloud")?)
            .with_aliases(["remote"])?
            .with_priority(100),
        CloudProvider,
    )?;
    builder.register(
        ProviderDescriptor::new(ProviderId::new("local")?)
            .with_aliases(["builtin"])?
            .with_priority(10),
        LocalProvider,
    )?;

    /*
     * build() ends mutable startup assembly. The resolver shares the resulting
     * immutable registry and applies one explicit fallback policy at runtime.
     * OnAbsence protects caller errors; OnAnyError would also continue after
     * InvalidConfiguration and other non-absence provider failures.
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
    let config = GreetingConfig {
        prefix: "hello".to_owned(),
        cloud_available: false,
    };

    /*
     * Automatic selection tries priority order. Cloud is considered first,
     * but its Unavailable error permits OnAbsence to reach local. The result
     * retains local's canonical ID so logs never depend on the alias used.
     */
    let automatic = resolver.create_auto(&config)?;
    assert_eq!("local", automatic.provider_id().as_str());
    assert_eq!("hello from local", automatic.service().greet());

    /*
     * Named selection resolves exactly one canonical ID or alias. "builtin"
     * maps to local, and named selection never falls back to cloud.
     */
    let named = resolver.create_named("builtin", &config)?;
    assert_eq!("local", named.provider_id().as_str());
    assert_eq!("hello from local", named.service().greet());

    /*
     * Chained selection preserves caller order. The unknown name is recorded,
     * remote reaches unavailable cloud, and builtin finally succeeds locally.
     */
    let chained = resolver.create_chain(["missing", "remote", "builtin"], &config)?;
    assert_eq!("local", chained.provider_id().as_str());
    assert_eq!("hello from local", chained.service().greet());

    /*
     * This second request deliberately fails so the example also demonstrates
     * diagnostics: cloud is unavailable, then local rejects the empty prefix,
     * and OnAbsence stops because invalid configuration is not an absence.
     */
    let invalid_config = GreetingConfig {
        prefix: "  ".to_owned(),
        cloud_available: false,
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

With `cloud_available: false`, automatic selection first reaches `cloud`, then
continues after its `Unavailable` error and returns `local`. Named and chained
selection also return `local`. The final deliberately invalid request stops at
`LocalProvider` and exercises the diagnostic function.

## Define a Service

Use this when introducing an independently configured family of Provider
implementations.

Focused excerpt from the complete example:

```rust,ignore
struct GreetingConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = GreetingConfig;
    type Output = Arc<dyn Greeter>;
}
```

The observable result is one compile-time contract: every
`ServiceProvider<GreetingSpec>` receives `&GreetingConfig` and returns
`Arc<dyn Greeter>`.

`Config` may be unsized, so a service may use a view such as `str` or a trait
object. `Output` is the complete caller-owned value; choose a plain value,
`Box<dyn Trait>`, `Arc<dyn Trait>`, or another handle according to the
application's ownership and concurrency needs. The SPI never inserts or
removes a wrapper.

**Common mistake:** defining one overly broad specification for unrelated
services. Use a separate marker type whenever configuration, output, Provider
set, or selection policy should evolve independently.

## Implement a Provider

Use this when adding one factory capable of creating the output chosen by a
`ServiceSpec`.

Focused excerpt from the complete example:

```rust,ignore
impl ServiceProvider<GreetingSpec> for LocalProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from local", config.prefix),
        }))
    }
}
```

The result is a factory invoked whenever the Resolver reaches this Provider.
Provider implementations must be `Send + Sync + 'static`; the Registry retains
them and may be shared across threads. The configuration is borrowed, while a
new complete output is returned for each successful call.

Choose the error classification by meaning because it controls fallback:

| `ProviderError` constructor | Meaning | `OnAbsence` |
| --- | --- | --- |
| `unsupported` | This Provider cannot handle the request. | Continues |
| `unavailable` | It could handle the request but cannot do so now. | Continues |
| `invalid_configuration` | The caller supplied invalid settings. | Stops |
| `initialization_failed` | Creating this implementation failed unexpectedly. | Stops |

Each classification also has a `_with_source` constructor for retaining an
underlying `Error + Send + Sync + 'static`.

**Common mistake:** reporting invalid configuration as `Unavailable`. That can
hide a caller error by letting `OnAbsence` silently choose a different
Provider.

## Name and Rank Providers

Use this when assigning stable identity, accepted configuration names, and
automatic-selection order to a factory registration.

Focused excerpt from the complete example:

```rust,ignore
let cloud = ProviderDescriptor::new(ProviderId::new("cloud")?)
    .with_aliases(["remote"])?
    .with_priority(100);
```

This descriptor makes `cloud` the canonical ID, accepts `remote` as an alias,
and gives it priority 100 for automatic selection.

Canonical `ProviderId` values are strict lowercase ASCII tokens. They must
start and end with an ASCII alphanumeric character; interior characters may
also include `-`, `_`, `.`, and `+`. `ProviderId::new` neither trims nor
normalizes input. By contrast, runtime `ProviderSelector` input is trimmed and
ASCII-lowercased before validation, so `" REMOTE "` resolves the alias
`remote`.

Aliases use the same selector namespace as canonical IDs. A descriptor rejects
an invalid alias, an alias equal to its own ID, or duplicate aliases. The
Builder rejects a selector already claimed by another registration. Priority
affects only `create_auto`; named and chained selection follow the caller's
selector or order.

Invalid canonical IDs return `ProviderIdError`; invalid or duplicate aliases
return `ProviderDescriptorError`.

**Common mistake:** treating an alias as the Provider's identity. Results and
diagnostics always report the canonical ID, even when an alias was requested.

## Build and Inspect a Registry

Use this when assembling all available factories during application startup or
examining the immutable catalog later.

Focused excerpt using the complete-example types:

```rust,ignore
let shared_cloud: Arc<dyn ServiceProvider<GreetingSpec>> = Arc::new(CloudProvider);
let mut builder = ProviderRegistry::<GreetingSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("local")?),
    LocalProvider,
)?;
builder.register_shared(
    ProviderDescriptor::new(ProviderId::new("cloud")?),
    shared_cloud,
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

## Choose Providers

Use this when deciding whether configuration names one Provider, requests the
best available Provider, or supplies an ordered preference list.

Focused excerpt from the complete example:

```rust,ignore
let automatic = resolver.create_auto(&config)?;
let named = resolver.create_named("builtin", &config)?;
let chained = resolver.create_chain(["missing", "remote", "builtin"], &config)?;
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

## Choose a Fallback Policy

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

An unknown selector inside a chain is recorded and traversal continues because
no Provider was invoked. The policy matters only when another candidate
exists; named selection still has exactly one candidate. A policy stop produces
`ResolutionTermination::StoppedByPolicy`; visiting all admitted candidates
produces `ResolutionTermination::Exhausted`.

**Common mistake:** choosing `OnAnyError` merely to make a request succeed.
That policy can mask configuration and initialization defects, so use it only
for an explicitly best-effort workflow.

## Reuse Validated Selections

Use this when the same configured selection is applied across many creation
calls.

Focused excerpt using the complete-example types:

```rust,ignore
use qubit_spi::ProviderSelection;

let selection = ProviderSelection::chain(["remote", "builtin"])?;

let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

`ProviderSelection::auto()` is infallible. `named(...)` normalizes and validates
one selector; `chain(...)` validates every selector, preserves order, and
rejects an empty chain. `ProviderResolver::create` then reuses that validated
value. `Default` is automatic selection; `kind()` reports the mode,
`selector()` borrows the named selector when present, and `selectors()` returns
the chain or an empty slice for the other modes.

Raw `create_named` and `create_chain` are preferable at runtime input
boundaries because they convert parse failures into `ResolutionError` while
preserving the invalid input and chain index. They parse and allocate owned
selector data on every call. A reusable `ProviderSelection` moves that work to
configuration loading and reports validation as `ProviderSelectionError`.

**Common mistake:** reparsing a constant configuration string on every hot-path
call. Validate it once and retain the selection.

## Inspect Successful Results

Use this when consuming the output or recording which Provider actually won.

Focused excerpt from the complete example:

```rust,ignore
let created = resolver.create_named("builtin", &config)?;
println!("winner: {}", created.provider_id());
created.service().greet();

let (provider_id, service) = created.into_parts();
```

`provider_id()` and `service()` borrow the two values. `into_service()` consumes
the wrapper and returns only the output. `into_parts()` consumes it and returns
`(ProviderId, Output)`. The ID is always canonical, so observability does not
depend on which alias a caller used.

**Common mistake:** discarding the winning ID before logging or metrics. It is
the simplest way to confirm fallback behavior in production.

## Diagnose Failures

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

## Share Registries and Resolvers

Both `ProviderRegistry` and `ProviderResolver` are cheaply cloneable. A Registry
stores immutable entries and indexes behind an internal `Arc`; cloning it does
not duplicate Providers. A Resolver clone shares that Registry and copies its
small fallback policy value. `registry()` and `fallback_policy()` provide
read-only access to the Resolver configuration.

Providers are `Send + Sync + 'static`, so the catalog can be shared. The SPI
places no `Send` or `Sync` bound on `ServiceSpec::Output`; choose a thread-safe
output such as `Arc<dyn Trait + Send + Sync>` when the created service itself
must cross threads.

Raw selector parsing normalizes and owns selector text so errors and selections
can retain it safely. Reuse `ProviderSelection` when that allocation matters.
Registry lookup and automatic order use indexes prepared once by `build()`.

## Recommended Practices

- Assemble Providers once during startup and fail startup on registration
  errors.
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
| Classify factory failure and diagnose resolution | [`ProviderError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/struct.ProviderError.html), [`ResolutionError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ResolutionError.html) |
| Handle validation, registration, Provider, and resolution errors | [`qubit_spi::error`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/index.html) |
