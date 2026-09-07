# Fuzz contracts

`provider_input` bounds raw input at 4096 bytes before UTF-8 parsing and checks
identifier, selector, descriptor and selection input invariants.

`registry_model` uses a linear `Vec<ModelEntry>` oracle, independent of the
production catalog's indexes. It checks all four selection modes after each
registration, full canonical invocation order, classified attempts, termination,
and resolver snapshots retained across later registration/default changes.
Unknown selector occurrences are preserved; only matched provider identities
are deduplicated. The oracle does not call the production fallback predicate.

Each five-byte operation encodes canonical ID, alias, priority, outcome and
policy. IDs range over 16 names; alias byte 255 means none, and its high bit
chooses the alias namespace. Priorities 0/255 mean i32::MIN/MAX. Outcomes modulo
5 select success, unsupported, unavailable, invalid configuration, initialization
failure. Policy modulo 3 selects Never, OnAbsence, OnAnyError. At most 32
operations/registrations and 16 selectors are materialized before model work.

The named seeds are long-lived boundary inputs, not a random corpus dump:

| Seed | Stable integration contract |
| --- | --- |
| extreme-priority | default snapshot during concurrent updates (MIN/MAX ranking) |
| alias-dedup | lenient chain missing entries and candidate deduplication |
| strict-missing | ordered duplicate unknown selectors |
| last-failure | named failure identity; concurrent snapshot exhaustion vs policy stop |
| policy-stop | OnAbsence rejects configuration/initialization failure |
| historical-snapshot | resolved candidates ignore later registration |

Replay or extend with the project's installed nightly toolchain:

```sh
cargo +nightly fuzz run registry_model fuzz/corpus/registry_model -- -runs=100
cargo +nightly fuzz run registry_model -- -max_total_time=60
cargo +nightly fuzz run provider_input -- -max_total_time=60
```

Record the actual nightly version and run output when reporting validation.
Async Pending/cancellation is covered by deterministic ordinary tests; the model
has no external I/O or uncontrolled task scheduling.
