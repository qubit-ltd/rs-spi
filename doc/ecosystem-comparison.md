# Comparing rs-spi with Related Rust Ecosystem Approaches

English

## Scope and version

This document uses `qubit-spi` 0.13.0 as its baseline and compares its responsibilities and trade-offs with several related Rust patterns and libraries. The comparison concerns capability scope and integration models; it is not a benchmark and does not imply that the approaches are interchangeable. External APIs and support may change over time; consult each project's current documentation.

## Project positioning

`qubit-spi` provides a typed service-provider registry and resolution mechanism for statically linked Rust applications. An application registers candidate providers, selects candidates by name, an explicit chain, or automatic ordering, and then invokes factories with configuration to create a service. It separates candidate selection from creation-failure fallback policy and provides corresponding APIs for synchronous and asynchronous services.

It is intended for cases where an application configures multiple optional backends while dependent libraries use a service abstraction. When there is only one implementation and no runtime selection is needed, direct construction or trait injection is usually simpler.

This crate does not load or unload dynamic libraries, build a general-purpose dependency-injection graph, cache created services, monitor their health, or handle their subsequent runtime failures. The optional `inventory` feature can discover static provider entries that are linked into the final binary; this is still link-time discovery, not runtime binary plugin loading.

## Operating model

Resolution can be summarized in three stages:

1. **Registration or discovery**: the application explicitly registers `ProviderDefinition` values, or enables the `inventory` feature to collect linked static providers.
2. **Selection**: `ProviderSelection` determines candidates by name, chain, or automatic ordering. Automatic ordering sorts by descending priority, then ascending canonical ID.
3. **Creation and fallback**: the resolver invokes factories in candidate order and uses `Never`, `OnAbsence`, or `OnAnyError` to decide whether to continue after a creation failure.

`ServiceSpec` binds configuration and domain error types to a service family; a synchronous or asynchronous capability trait then binds its output type. Cloned catalogs share registration state. A resolved provider holds a candidate snapshot, so later catalog mutations do not change the candidates of an existing resolver. The registry lock is not held while provider factories run or asynchronous futures are polled.

Failure categories include `Unsupported`, `Unavailable`, `InvalidConfiguration`, and `InitializationFailed`. Fallback behavior depends on providers assigning failure categories accurately; the crate does not infer whether a domain error is suitable for fallback. If asynchronous creation is cancelled, the current future is dropped; the next provider is not attempted, and external side effects that have already occurred are not guaranteed to be rolled back.

## Comparing the approaches

| Approach | Strengths | Trade-offs | Suitable when |
| --- | --- | --- | --- |
| Direct trait, constructor, or parameter injection | Dependencies remain visible; no registry or discovery mechanism is needed; the abstraction and runtime overhead are usually minimal. | The caller must organize multi-implementation selection, fallback, and error aggregation. | There is one implementation, or the caller already knows which implementation to use. |
| `qubit-spi` | Provides a typed provider catalog, candidate selection, creation policy, and diagnostics; synchronous and asynchronous APIs follow corresponding models. | Requires a service-family contract and registration flow; configuration and error models are shared within a service family; caching, general dependency graphs, and dynamic-library lifecycle management are out of scope. | An application needs runtime selection among multiple statically linked backends with explicit creation-failure semantics. |
| [`inventory`](https://docs.rs/inventory/latest/inventory/) | Allows static entries to be submitted from different crates and collected by the final program; useful for declarations distributed across crates. | Primarily collects entries; it does not define provider selection, creation, fallback, or error protocols. Iteration order is not guaranteed. A dependency crate must also be present in the final linked artifact. | Static distributed registration is needed, while the application or another layer owns subsequent processing. |
| [`linkme`](https://docs.rs/linkme/latest/linkme/) | Gathers static elements through distributed slices, which consumers read as a slice; suitable for tables fixed at build time. | Elements must satisfy constraints such as static initialization; the library itself does not define service construction, selection policy, or failure handling. | Command tables, handler tables, or other static collections fixed at build time. |
| [`shaku`](https://docs.rs/shaku/latest/shaku/) | Focuses on dependency injection and component composition, and can describe components and dependencies in application modules. | It addresses object dependency graphs and injection; its abstraction scope may be broader when the requirement is only backend selection and creation fallback. | An application wants to organize interdependent components through dependency injection. |
| [`abi_stable`](https://docs.rs/abi_stable/latest/abi_stable/) | Focuses on a stable ABI and runtime loading between Rust libraries, and can support extension systems that cross dynamic-library boundaries. | Requires FFI-safe types and stable-ABI constraints, along with handling loading, compatibility, and resource lifetimes; it is not equivalent to registering ordinary Rust traits. | Plugins are delivered as separate dynamic libraries that the host must load at runtime. |

These approaches can be combined. For example, `inventory` can provide static provider discovery, after which `qubit-spi` performs catalog validation, selection, and creation. Choose an approach based on deployment boundaries and required semantics, rather than API count or assumptions about performance. This document makes no cross-approach performance claims; performance-sensitive workloads should be measured on their target platforms.

## Adoption guidance and limitations

- If only one implementation is needed, or a call site can pass the implementation explicitly, consider direct dependency injection first.
- If multiple statically linked backends must be selected by configuration with an explicit creation-failure policy, consider `qubit-spi`.
- If providers are distributed across multiple static extension crates, `inventory` can serve as a discovery layer; the relevant crates must still be linked and the application must build the catalog.
- If independently shipped binary plugins must be loaded, choose and design a stable-ABI plugin boundary for that purpose. `qubit-spi` may serve as the in-process backend selection layer, but it does not provide that boundary.

When adopting `qubit-spi`, account for the fact that a service family shares one configuration and domain error type; the asynchronous interface uses boxed `Send` futures and is not suited to `!Send` futures; correct fallback depends on accurate provider error classification; and the application chooses the catalog scope and when to seal it. The project includes benchmarks, but this document does not run cross-approach comparisons and therefore makes no performance conclusions.

## Further reading

- [English README](../README.md)
- [English user guide](user_guide.md)
- [English design notes](design.md)
- [Rust API documentation](https://docs.rs/qubit-spi/0.13.0/qubit_spi/)
