# Qubit SPI 用户手册

本手册完整介绍 `qubit-spi` 0.8 的公共使用模型。

## 概述

Qubit SPI 为同一服务支持多种实现的应用提供类型安全的基础设施。应用定义服务族，
在启动阶段注册 Provider 工厂，构建不可变 Registry，然后通过明确的选择规则解析
服务。

本 crate 不提供全局 Registry，也不会产生自动发现的副作用。应用自行决定链接哪些
Provider、何时注册，以及向各子系统共享哪个 Registry 或 Resolver。这使启动失败
保持可见，也让测试免受进程级全局状态干扰。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-spi = "0.8"
```

0.8 版本要求 Rust 1.94 或更高版本。本 crate 没有 feature flag，唯一的运行时依赖是
`thiserror`。

大多数应用从 `qubit_spi` 导入核心类型，从 `qubit_spi::error` 导入错误类型：

```rust
use qubit_spi::error::{ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};
```

## 核心模型

主要类型构成一条从启动到运行时的处理链：

| 阶段 | 类型 | 职责 |
| --- | --- | --- |
| 服务定义 | `ServiceSpec` | 将配置类型绑定到同一服务族中所有 Provider 返回的完整输出类型。 |
| Provider 实现 | `ServiceProvider<S>` | 根据 `&S::Config` 创建 `S::Output`，并对创建失败分类。 |
| 注册元数据 | `ProviderDescriptor` | 保存 canonical ID、alias 和自动选择 priority。 |
| 启动装配 | `ProviderRegistryBuilder<S>` | 注册工厂，并在构建 Registry 前拒绝 selector 冲突。 |
| 运行时目录 | `ProviderRegistry<S>` | 提供不可变查询和确定性的自动选择顺序。 |
| 选择 | `ProviderSelection` | 表示经过校验的自动、具名或链式请求。 |
| 创建 | `ProviderResolver<S>` | 应用选择与 fallback policy 来创建服务。 |
| 成功结果 | `CreatedService<S::Output>` | 返回输出和胜出 Provider 的 canonical ID。 |

Provider 身份属于注册过程，而不是工厂对象。同一个工厂类型可以在不同 Registry 中
以不同方式注册。SPI 核心会原样返回 `ServiceSpec` 选定的输出类型，不会添加或移除
`Box`、`Arc` 或 `Rc` 包装。

## 定义服务

为每个独立服务族定义一个实现 `ServiceSpec` 的标记类型：

```rust
use std::sync::Arc;

use qubit_spi::ServiceSpec;

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

struct GreeterConfig {
    prefix: String,
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}
```

`Config` 可以是 unsized 类型，因此 Provider 可以接收 `str` 或 trait object 等视图。
`Output` 是返回给调用方的完整值。应根据服务真实的所有权与并发需求选择拥有值、
`Box<dyn Trait>`、`Arc<dyn Trait>` 或其他句柄。

## 实现 Provider

每个 Provider 都实现 `ServiceProvider<S>`。Provider 实现必须满足
`Send + Sync + 'static`，因为 Registry 会保留并可能共享它。配置以借用方式传入，
每次调用都创建一个输出。

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::ServiceProvider;

# trait Greeter: Send + Sync {
#     fn greet(&self, name: &str) -> String;
# }
# struct GreeterConfig { prefix: String }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }
struct LocalGreeter {
    prefix: String,
}

impl Greeter for LocalGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{} {name}", self.prefix)
    }
}

struct LocalProvider;

impl ServiceProvider<GreeterSpec> for LocalProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            ));
        }
        Ok(Arc::new(LocalGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}
```

应选用语义最准确的 `ProviderError` 构造函数；错误分类会直接决定 Resolver 是否可以
继续尝试下一个 Provider。

## Provider 身份与元数据

`ProviderId` 是严格的 canonical identity。它必须是小写 ASCII，以 ASCII 字母或
数字开头和结尾，中间只能包含字母、数字、`-`、`_`、`.` 和 `+`。输入不会被裁剪或
归一化：

```rust
use qubit_spi::ProviderId;

let id = ProviderId::new("local-v2")?;
assert_eq!("local-v2", id.as_str());
# Ok::<(), qubit_spi::error::ProviderIdError>(())
```

`ProviderSelector` 面向配置和用户输入。解析过程会裁剪首尾空白并把 ASCII 字母转为
小写，然后应用相同的 token 语法。因此 `" LOCAL-V2 "` 可以选择 canonical ID
`local-v2`。

`ProviderDescriptor` 把 canonical ID、alias 和 priority 组合在一起：

```rust
use qubit_spi::{ProviderDescriptor, ProviderId};

let descriptor = ProviderDescriptor::new(ProviderId::new("local")?)
    .with_aliases(["builtin", "default"])?
    .with_priority(50);

assert_eq!("local", descriptor.id().as_str());
assert_eq!(50, descriptor.priority());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Alias 按 selector 规则解析。Descriptor 会拒绝非法 alias、与 canonical ID 相同的
alias，以及归一化后重复的 alias。Priority 只影响自动选择；具名和链式选择仍使用
调用方指定的目标或顺序。

## 构建 Registry

应在应用启动阶段构建 Registry：

```rust
# use std::sync::Arc;
# use qubit_spi::error::ProviderError;
use qubit_spi::{ProviderDescriptor, ProviderId, ProviderRegistry};
# trait Greeter: Send + Sync { fn greet(&self, name: &str) -> String; }
# struct GreeterConfig { prefix: String }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }
# struct LocalProvider;
# impl qubit_spi::ServiceProvider<GreeterSpec> for LocalProvider {
#     fn create(&self, _: &GreeterConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
#         Err(ProviderError::unavailable("example provider"))
#     }
# }

let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("local")?)
        .with_aliases(["builtin"])?
        .with_priority(50),
    LocalProvider,
)?;
let registry = builder.build();

assert_eq!(1, registry.len());
assert!(!registry.is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

`register` 接收拥有所有权的具体 Provider，并将其存入共享的 Registry 存储；如果
工厂已经保存在 `Arc<dyn ServiceProvider<S>>` 中，则使用 `register_shared`。

Registry 中每个 canonical ID 和 alias 都必须唯一。注册操作会先检查所有 selector
声明，再修改 Builder，因此被拒绝的注册不会占用部分 alias。`RegistrationError`
会报告冲突 selector、现有所有者以及试图声明它的 Provider。

调用 `build` 后 Registry 不可变。`clone` 只克隆内部 `Arc`，因此 Registry 句柄的
共享成本很低。`descriptors()` 和 `provider_ids()` 按注册顺序迭代。`find` 对非法和
未知输入都返回 `None`；如果调用方需要通过结构化 `ResolutionError` 区分两者，应
使用 `resolve`。

## 选择 Provider

Qubit SPI 支持三种选择模式：

- `ProviderSelection::auto()` 使用 Registry 的确定性自动顺序：priority 降序，
  canonical Provider ID 升序。
- `ProviderSelection::named(value)` 校验一个 selector，并且只尝试该 Provider。
- `ProviderSelection::chain(values)` 校验一个非空有序 selector 列表，并按输入顺序
  尝试候选项。

```rust
use qubit_spi::ProviderSelection;

let automatic = ProviderSelection::auto();
let named = ProviderSelection::named(" local ")?;
let chain = ProviderSelection::chain(["cloud", "local"])?;

assert!(named.selector().is_some());
assert_eq!(2, chain.selectors().len());
# Ok::<(), qubit_spi::error::ProviderSelectionError>(())
```

链式解析会把未知 selector 记录为失败尝试并继续。如果同一个 ID 及其 alias 同时出现
在一条 chain 中，底层 Provider 只会被调用一次。具名选择从不回退，即使 Resolver
使用 `OnAnyError` 也是如此。

## 回退策略

在自动或链式选择中，Provider 工厂返回 `ProviderError` 后会应用
`FallbackPolicy`：

| 策略 | `Unsupported` | `Unavailable` | `InvalidConfiguration` | `InitializationFailed` |
| --- | --- | --- | --- | --- |
| `OnAbsence` | 继续 | 继续 | 停止 | 停止 |
| `OnAnyError` | 继续 | 继续 | 继续 | 继续 |

`OnAbsence` 是默认策略，适用于“当这个实现无法支持请求或当前环境时换用另一个实现”
的语义。它会在配置无效和意外初始化失败时停止，避免隐藏真实问题。

`OnAnyError` 明确表示尽力而为。只有在配置无效或意外初始化失败后尝试其他 Provider
仍然正确时才应使用。

Chain 中的未知 selector 不是 Provider error；它们会被记录并跳过，不受 fallback
policy 影响。具名选择没有下一个候选项，因此仍会立即报告未知 Provider。

## 解析并创建服务

通过 Registry 和策略构造 Resolver：

```rust
# use std::sync::Arc;
use qubit_spi::{FallbackPolicy, ProviderRegistry, ProviderResolver};
# struct GreeterConfig { prefix: String }
# trait Greeter: Send + Sync { fn greet(&self, name: &str) -> String; }
# struct GreeterSpec;
# impl qubit_spi::ServiceSpec for GreeterSpec {
#     type Config = GreeterConfig;
#     type Output = Arc<dyn Greeter>;
# }

let registry = ProviderRegistry::<GreeterSpec>::default();
let resolver = ProviderResolver::new(registry, FallbackPolicy::OnAbsence);

assert!(resolver.registry().is_empty());
assert_eq!(FallbackPolicy::OnAbsence, resolver.fallback_policy());
```

在运行时输入边界使用 `create_auto`、`create_named` 或 `create_chain`。这些方法解析
原始 selector，并把校验失败转换为 `ResolutionError`：

```rust,ignore
let service = resolver.create_auto(&config)?;
let service = resolver.create_named(configured_name, &config)?;
let service = resolver.create_chain(configured_chain, &config)?;
```

如果同一配置选择会重复使用，应只校验一次，然后重复调用 `create`：

```rust,ignore
let selection = ProviderSelection::chain(["cloud", "local"])?;
let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

这样可以避免重复分配和归一化 selector。Resolver 与 Registry 的克隆仍指向同一份
不可变目录。

## 检查成功结果

Resolver 方法返回 `CreatedService<S::Output>`。即使使用 alias 选择，它也会保留
胜出 Provider 的 canonical ID：

```rust,ignore
let created = resolver.create_named("builtin", &config)?;
tracing::info!(provider = %created.provider_id(), "created greeter");
let greeting = created.service().greet("Ada");
```

使用 `service()` 借用输出；使用 `into_service()` 丢弃 Provider 身份并取得输出；
使用 `into_parts()` 同时取得两个拥有所有权的值。

如果需要不带回退的直接查询，`ProviderRegistry::resolve` 会返回借用的
`ResolvedProvider`。其 `descriptor()` 暴露注册元数据，`create()` 调用该单一
Provider。当代码需要在创建前检查元数据时可使用此方式；普通创建流程应使用
Resolver，以保持诊断和策略处理一致。

## 错误处理与诊断

错误按生命周期拆分：

| 错误 | 含义 |
| --- | --- |
| `ProviderIdError` | Canonical Provider ID 为空或不符合 canonical 语法。 |
| `ProviderSelectorError` | 原始 selector 归一化后为空或不符合 token 语法。 |
| `ProviderDescriptorError` | Alias 非法、重复或与 canonical ID 相同。 |
| `ProviderSelectionError` | Named/chain selector 非法或 chain 为空。 |
| `RegistrationError` | Builder 中的 canonical ID 或 alias 已被声明。 |
| `ProviderError` | 单个 Provider 对服务创建失败进行了分类。 |
| `ResolutionError` | 选择解析、查询、遍历或创建未能产生服务。 |

`ProviderErrorKind` 有四种分类：`Unsupported`、`Unavailable`、
`InvalidConfiguration` 和 `InitializationFailed`。名称以 `_with_source` 结尾的
构造函数会保留底层 `Error + Send + Sync + 'static`，供标准 error source chain
使用。

`ResolutionError` 区分非法原始 selector、空原始 chain、未知具名 Provider、
在空 Registry 上自动选择，以及聚合的 `NoProviderSucceeded` 结果。对于聚合错误：

- `attempts()` 按遇到顺序返回失败。
- `terminal_attempt()` 返回最后一次已记录的尝试。
- `termination()` 返回 `Exhausted` 或 `StoppedByPolicy`。
- `decisive_attempt()` 返回导致策略停止的尝试，或只有一次尝试时的 exhausted 结果；
  对含多个尝试且原因不唯一的 exhausted 结果返回 `None`。
- `is_absence()` 对未知具名 Provider，或只包含未知、不支持和不可用尝试的聚合结果
  返回 `true`。

每个 `AttemptFailure` 会区分未知 selector 和已调用 Provider 返回的错误。Provider
尝试会保留显式请求的 selector（如有）、canonical Provider ID、原始
`ProviderError` 及其 source。`ResolutionError` 的显示文本包含按顺序排列的尝试
诊断。

公共错误枚举标记为 `#[non_exhaustive]`。匹配已知 variant 时必须保留通配分支：

```rust
use qubit_spi::error::ResolutionError;
use qubit_spi::ResolutionTermination;

fn describe(error: &ResolutionError) -> &'static str {
    match error {
        ResolutionError::InvalidSelector { .. } => "invalid selector",
        ResolutionError::EmptySelection => "empty chain",
        ResolutionError::UnknownProvider { .. } => "unknown provider",
        ResolutionError::EmptyRegistry => "empty registry",
        ResolutionError::NoProviderSucceeded {
            termination: ResolutionTermination::StoppedByPolicy,
            ..
        } => "fallback stopped",
        ResolutionError::NoProviderSucceeded { .. } => "candidates exhausted",
        _ => "future resolution error",
    }
}
```

## 共享与性能

Registry 只构建一次，并由共享不可变存储支持。克隆 Registry 或 Resolver 只会增加
`Arc` 引用计数，不会复制 Provider 或索引。查询使用 selector 索引，自动候选顺序
在 `build` 阶段计算，而不是每次解析时重新计算。

成功解析 `ProviderSelector` 会分配并持有归一化文本。如果同一配置会重复使用，应缓存
`ProviderSelector` 或 `ProviderSelection`。在请求或配置边界，原始 Resolver 方法
更合适，因为它们会在 `ResolutionError` 中保留非法输入及其解析 source。

Provider 工厂满足 `Send + Sync`，不可变 Registry/Resolver 可以共享用于并发查询和
创建。已创建服务输出的线程安全性、生命周期和分配行为仍由 `ServiceSpec::Output`
及 Provider 实现决定。

## 推荐实践

- 在启动阶段显式装配 Registry，并让 descriptor 或 registration error 导致启动失败。
- 在持久化配置中使用稳定的小写 canonical ID，将 alias 保留给兼容性或运维便利性。
- 有意识地分配 priority，并记住 canonical ID 是稳定的同 priority 排序规则。
- 除非产品明确要求在配置或初始化错误后继续，否则使用 `OnAbsence`。
- 准确分类 `ProviderError`；回退行为是否正确依赖该分类。
- 在不可信输入边界使用原始 Resolver 方法，为重复内部调用缓存已校验的
  `ProviderSelection`。
- 在日志或指标中记录 `CreatedService::provider_id()`，以便识别实际选择的实现。
- 检查有序 attempts 和 termination，不要解析 display 文本。
- 匹配公共错误枚举时保留通配分支。

## 常见问题

**ID 被拒绝，但类似 selector 可以使用。** `ProviderId` 要求输入本身已经 canonical，
不会裁剪或转小写；`ProviderSelector` 则会归一化配置输入。应把 canonical 形式保存为
ID。

**添加 alias 后注册失败。** Canonical ID 和 alias 共享同一个 selector 命名空间。
检查 `RegistrationError::DuplicateSelector` 的字段以找到现有所有者和冲突注册。

**自动解析返回 `EmptyRegistry`。** 在调用 `build` 前没有注册 Provider，或者向
Resolver 传入了错误类型的 Registry。

**Chain 在解析前就拒绝所有候选项。** `ProviderSelection::chain` 和 `create_chain`
会拒绝空 chain，并在第一个非法 selector 处停止解析。语法有效但未知的 selector
不同：它们会在解析期间成为有序 attempt failure。

**回退比预期更早停止。** 在 `OnAbsence` 下，`InvalidConfiguration` 和
`InitializationFailed` 会停止遍历。检查 `termination()` 和
`decisive_attempt()` 以识别导致策略停止的 Provider error。

**`decisive_attempt()` 返回 `None`。** 多个候选项已经耗尽，没有一次失败能够单独
解释聚合结果。应检查 `attempts()` 返回的每一项。

## 完整示例

以下示例注册一个优先的 Cloud Provider 和一个本地回退 Provider。Cloud Provider
报告不可用，因此使用 `OnAbsence` 的自动解析会继续尝试本地 Provider。

```rust
use std::sync::Arc;

use qubit_spi::error::{ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ResolutionTermination,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

struct GreeterConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}

struct TextGreeter {
    prefix: String,
}

impl Greeter for TextGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{} {name}", self.prefix)
    }
}

struct CloudProvider;

impl ServiceProvider<GreeterSpec> for CloudProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if !config.cloud_available {
            return Err(ProviderError::unavailable(
                "the cloud greeting service is offline",
            ));
        }
        Ok(Arc::new(TextGreeter {
            prefix: format!("{} from the cloud,", config.prefix),
        }))
    }
}

struct LocalProvider;

impl ServiceProvider<GreeterSpec> for LocalProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreeterSpec>::builder();
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

    let resolver = ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    );
    let config = GreeterConfig {
        prefix: "Hello,".to_owned(),
        cloud_available: false,
    };

    match resolver.create_auto(&config) {
        Ok(created) => {
            assert_eq!("local", created.provider_id().as_str());
            assert_eq!("Hello, Ada", created.service().greet("Ada"));
        }
        Err(error) => report_resolution_error(&error),
    }

    let named = resolver.create_named("builtin", &config)?;
    assert_eq!("local", named.provider_id().as_str());
    Ok(())
}

fn report_resolution_error(error: &ResolutionError) {
    match error.termination() {
        Some(ResolutionTermination::StoppedByPolicy) => {
            eprintln!("resolution stopped: {error}");
        }
        Some(ResolutionTermination::Exhausted) => {
            eprintln!("all candidates failed: {error}");
        }
        None => eprintln!("selection failed: {error}"),
        _ => eprintln!("resolution failed: {error}"),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("startup failed: {error}");
    }
}
```

## API 参考

完整生成的 API 文档位于 [docs.rs](https://docs.rs/qubit-spi)。主要入口如下：

| 领域 | 类型 |
| --- | --- |
| 服务契约 | `ServiceSpec`、`ServiceProvider` |
| 身份与元数据 | `ProviderId`、`ProviderSelector`、`ProviderDescriptor` |
| Registry | `ProviderRegistryBuilder`、`ProviderRegistry`、`ResolvedProvider` |
| 选择与解析 | `ProviderSelection`、`ProviderSelectionKind`、`FallbackPolicy`、`ProviderResolver` |
| 结果 | `CreatedService`、`ResolutionTermination` |
| 错误 | `ProviderIdError`、`ProviderSelectorError`、`ProviderDescriptorError`、`ProviderSelectionError`、`RegistrationError`、`ProviderError`、`ProviderErrorKind`、`AttemptFailure`、`ResolutionError` |
