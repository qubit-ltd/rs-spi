# Qubit SPI 用户手册

本手册介绍 Qubit SPI 的运行时 Provider 模型，覆盖从 App 启动注册到下游使用 Service
的完整生命周期，包括 selection、配置、fallback、错误诊断、并发和全局 facade。

## Qubit SPI 解决什么问题

假设一个可复用的 `lib-foo` 库需要 Greeter。`lib-foo` 不应该自己选择或构造具体实现，
因为最终 App 可能需要部署环境提供的 Provider。

预期的运行时关系是：

1. App 在启动时注册当前可用的 Provider。
2. App 可以设置进程级默认 Provider selection。
3. `lib-foo` 随后解析自己的显式 selection，或者使用这个默认值。
4. 解析出的 Provider 使用显式或默认 config 创建 Service。
5. `lib-foo` 使用返回的 Service，不需要了解其具体类型。

这是 Service Provider Registry，而不是通用依赖注入框架。它统一实现的注册、选择和
创建方式；Service 的业务接口仍然属于具体领域 crate。

## 第一性原理：三个独立阶段

最重要的设计规则是：注册、选择和创建回答的是三个不同问题，不能压缩成一次操作。

### 注册：当前有什么实现

注册把带有 `ProviderMetadata` 以及对应同步或异步创建能力的 Provider 安装到
Registry。同步与异步 Provider 位于不同 Registry 中。Registry 保存 Provider 身份和
查找元数据，而不是已经创建好的 Service。

canonical ID 或 alias 已被占用时，注册会失败。注册不会解析某次请求的 selection，
也不会创建 Service。

### 选择：本次允许尝试什么

`ProviderSelection` 描述 named Provider、调用方指定顺序的 chain，或者 Registry 自动
顺序。`ProviderRegistry::resolve_selected` 把显式 selection 解析为一个时间点上的
候选快照；`ProviderRegistry::resolve` 对 Registry 默认值执行同样操作。两者都返回
`ResolvingServiceProvider<S>`。

选择阶段不需要 `S::Config`，也不会调用 Provider 代码。请求的 Provider 或候选集合
不存在时，选择失败。

### 创建：候选能否构造服务

`ResolvingServiceProvider<S>` 是一个带有固有 `create` 方法的组合 resolver。它使用
`S::Config` 调用候选 Provider，执行 selection 中保存的 fallback policy，并在成功时
直接返回 `S::Output`。它的聚合失败返回 `ProviderCreationError`，因此并不实现
`ServiceProvider<S>`。异步 resolver 的行为相同，但会 await 每一次异步 Provider 调用。

Provider 不支持请求、运行环境不可用、配置非法或初始化失败都会导致创建错误。聚合错误
只保留真正调用过的 Provider。

```text
metadata + provider --register--> 同步或异步 Registry
                                      │
ProviderSelection ---------------- resolve
                                      │
                                      ▼
                         ResolvingServiceProvider
                                      │
S::Config ------------------------- create
                                      │
                                      ▼
                                 S::Output
```

## 核心类型

| 类型 | 职责 |
| --- | --- |
| `ServiceSpec` | 绑定一个服务族的 `Config` 类型 |
| `SyncServiceSpec` / `AsyncServiceSpec` | 分别绑定同步和异步输出类型 |
| `ServiceProvider<S>` / `AsyncServiceProvider<S>` | 根据 `S::Config` 创建对应输出 |
| `ProviderDefinition<S>` / `AsyncProviderDefinition<S>` | 组合元数据与对应同步或异步创建契约的 marker trait |
| `ProviderMetadata` | 提供 Provider 自有 descriptor |
| `ProviderFuture<'a, T>` | 异步 Provider 与 resolver 使用的、与运行时无关且可发送的 boxed future |
| `ProviderId` | 稳定 canonical 身份：非空小写 ASCII、首尾字母数字、分隔符仅限 `-`/`_`/`.`/`+`；不做规范化 |
| `ProviderDescriptor` | 保存 canonical ID、alias 和自动选择 priority |
| `ProviderRegistry<S>` / `AsyncProviderRegistry<S>` | 分别保存独立的同步或异步运行时注册状态和默认 selection |
| `ProviderSelection` | 描述候选目标和创建阶段 fallback policy |
| `ResolvingServiceProvider<S>` / `AsyncResolvingServiceProvider<S>` | 持有对应候选快照并创建同步输出或返回 future |

泛型参数 `S` 防止不同服务族的 Provider 被混用。MIME Provider 无法注册到文件系统
Registry，因为它们使用不同的 `ServiceSpec`。

## 定义服务族

首先定义业务能力，其中只放消费者在初始化完成后反复调用的操作。构造参数放在独立的
config 类型中。

```rust
use std::sync::Arc;

use qubit_spi::{ServiceSpec, SyncServiceSpec};

/// 所有 Greeter Service 都要实现的业务接口。
trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

/// Provider 创建 Greeter 时接收的配置。
#[derive(Clone)]
struct GreeterConfig {
    /// 每条问候语中放在名字前面的文本。
    prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

/// 向 Qubit SPI 绑定 Greeter 的配置类型和输出类型。
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Provider 创建 Greeter 时接收的输入类型。
    type Config = GreeterConfig;
}

impl SyncServiceSpec for GreeterSpec {
    // 创建成功后返回给消费者的 Service 类型。
    type Output = Arc<dyn Greeter>;
}
```

`SyncServiceSpec::Output` 或 `AsyncServiceSpec::Output` 是消费者需要的完整实体，常见形式包括 `Arc<dyn Trait>`、具体
client 或轻量 handle。Qubit SPI 不会用 Provider 元数据包装成功结果，也不会缓存它。

`ServiceSpec::Config` 可以是 unsized 类型。只有 config 实现 `Default` 时才能调用
`create()`；`create_configured(&config)` 始终可用。

## 实现自描述 Provider

可以注册的 Provider 实现两个契约：

1. `ServiceProvider<S>`：提供创建行为。
2. `ProviderMetadata`：提供稳定的注册元数据。

传入 `ProviderId::new` 的 canonical ID 必须已经是非空小写 ASCII token：首尾为字母或
数字，中间仅允许分隔符 `-`、`_`、`.`、`+`；构造时不会 trim 或转小写。

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::{
    ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider,
};

/// friendly Provider 创建的具体 Greeter 实现。
struct FriendlyGreeter {
    /// 从创建配置复制得到的问候语前缀。
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

/// 导出给 App，由 App 显式注册的自描述 Provider。
pub struct FriendlyGreeterProvider;

impl ServiceProvider<GreeterSpec> for FriendlyGreeterProvider {
    fn create_configured(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            ));
        }
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderMetadata for FriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_aliases(["default-greeter", "friendly-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}
```

## 异步 Provider 与异步 Registry

支持异步创建的服务族另外实现 `AsyncServiceSpec`。该 trait 要求 `Config: Sync`，并定义
独立的 `Output: Send + 'static`。异步 Provider 实现 `AsyncServiceProvider<S>`，返回
可发送的 boxed `ProviderFuture`，因此 Qubit SPI 不绑定 Tokio、async-std 或其他
executor。`ProviderMetadata` 提供注册身份；同时实现这两个 trait 后会自动满足
`AsyncProviderDefinition<S>` marker trait，无需显式实现。`AsyncProviderRegistry<S>`
解析得到 `AsyncResolvingServiceProvider<S>`，后者持有候选快照，并按照 selection 的
fallback policy 依次 await Provider。

同步与异步 Registry 复用相同的 `ProviderSelection`、`MissingProviderPolicy` 和
`FallbackPolicy` 类型。两种 Registry 的解析失败均为 `ProviderResolutionError`，两种
resolver 都会把创建失败聚合为 `ProviderCreationError`。主要区别在于同步创建直接返回
output，而异步创建返回需要 await 的 `ProviderFuture`。

```rust,ignore
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::{
    AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec,
    ProviderDescriptor, ProviderFuture, ProviderId, ProviderMetadata,
    ProviderSelection,
};

impl AsyncServiceSpec for GreeterSpec {
    type Output = Arc<dyn Greeter>;
}

/// 由 App 注册的异步 Greeter Provider。
pub struct AsyncFriendlyGreeterProvider;

impl AsyncServiceProvider<GreeterSpec> for AsyncFriendlyGreeterProvider {
    fn create_configured<'a>(
        &'a self,
        config: &'a GreeterConfig,
    ) -> ProviderFuture<'a, Result<Arc<dyn Greeter>, ProviderError>> {
        Box::pin(async move {
            if config.prefix.trim().is_empty() {
                return Err(ProviderError::invalid_configuration(
                    "the greeting prefix must not be empty",
                ));
            }
            Ok(Arc::new(FriendlyGreeter {
                prefix: config.prefix.clone(),
            }) as Arc<dyn Greeter>)
        })
    }
}

impl ProviderMetadata for AsyncFriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("async-friendly")
                .expect("static provider ID is valid"),
        )
        .with_aliases(["async-default-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}

let registry = AsyncProviderRegistry::<GreeterSpec>::default();
registry.register(AsyncFriendlyGreeterProvider)?;
let selection = ProviderSelection::named("async-friendly")?;
let resolver = registry.resolve_selected(&selection)?;
let greeter = resolver.create_configured(&config).await?;
```

注册、查询、默认选择修改和 resolve 都保持同步，因为它们只操作内存中的元数据与
快照。只有 Provider 创建是异步的。返回的 Future 被 poll 时不持有 Registry 锁，因此
pending 创建不会阻塞注册或查询。resolver 的候选快照不会看到 resolve 之后注册的
Provider。

异步默认 `create()` 仅在 `S::Config: Default + Send` 时可用；
`create_configured(&config)` 只需要服务族已有的 `Config: Sync` 约束。同步与异步
Registry 的注册状态并不共享。同时支持两种创建模式的 Provider 实现必须分别注册到
两个 Registry。

### 为什么 descriptor 属于 Provider

Provider 身份和创建实现共同构成一个注册单元。要求调用方分别传递两者，既可能造成
元数据与实现不匹配，也会让第三方安装过程变得繁琐。自描述 Provider 让 App 只需：

```rust,ignore
registry.register(FriendlyGreeterProvider)?;
```

注册会先调用 `descriptor()`，随后才获取 Registry 写锁，并保存 descriptor 快照。
Provider 后续状态变化无法修改已经注册的 ID、alias 或 priority。

### Canonical ID、alias 和 priority

`ProviderId` 必须已经是 canonical 形式；构造时不会去除空白，也不会转换大小写。
合法 ID 是非空的小写 ASCII token：首尾必须是字母或数字（`a`–`z`、`0`–`9`），
其余字符只能是字母、数字，或分隔符 `-`、`_`、`.`、`+`。首尾空白、大写字母以及
其他标点都会被拒绝。

`ProviderSelector` 用于输入边界。解析时会去除首尾空白、把 ASCII 字母转成小写，再按照
同一 token 语法校验。因此配置值 `" Friendly-Greeter "` 会解析成规范化 alias
`friendly-greeter`。

alias 规范化后不能与 canonical ID 或另一个 alias 重复。priority 只影响自动选择：
值越大越靠前；priority 相同时，canonical ID 按升序排列。

## 创建并共享运行时 Registry

最简单的 Registry 初始为空，并允许运行时修改：

```rust,ignore
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(FriendlyGreeterProvider)?;
```

隔离 Registry 直接通过运行时注册 API 装配：

```rust,ignore
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(FriendlyGreeterProvider)?;
registry.register(AnotherProvider)?;
```

`ProviderRegistry<S>` 与 `AsyncProviderRegistry<S>` 提供平行的同步 catalog API，但两种
Registry 的状态独立，不共享注册状态或默认 selection。同时支持两种创建模式的 Provider
必须分别注册。
Provider 已经保存在对应的 `Arc<dyn ProviderDefinition<S>>` 或
`Arc<dyn AsyncProviderDefinition<S>>` 中时使用 `register_shared`；其他情况优先使用
`register(provider)`。

### Clone 与同步语义

Registry clone 共享同一个 `Arc<RwLock<...>>` 状态：

```rust,ignore
let library_registry = registry.clone();
registry.register(FriendlyGreeterProvider)?;
assert_eq!(1, library_registry.len());
```

通过任何 clone 完成的注册和默认 selection 修改，都对其他 clone 可见。返回 descriptor、
ID、默认 selection 或解析结果的方法提供自有快照。执行第三方 Provider 代码时不会持有
Registry 锁。

对于 selector 冲突，注册是原子的。ID 或 alias 已被占用时，Registry 保持不变，并返回
`RegistrationError::DuplicateSelector`，其中包含现有 Provider 和新 Provider 的 ID。

## 三个库与 App 的完整模式

Qubit SPI 有意不定义统一的全局 Registry：每个服务族都有不同的 `ServiceSpec`。拥有
Service trait 的领域 crate 应暴露适合自己的单体。下面的完整示例把四项职责拆分到三个
独立发布的库和一个 App 中。

下面的 Cargo package 名使用连字符；Rust 在 `use` 路径中会把连字符转换成下划线。
为简洁起见，示例省略各个 `Cargo.toml` 文件。

### 1. `lib-greeter`：定义 Service 和全局 Registry

`lib-greeter` 持有 Service 契约，以及供消费者、Provider 和最终 App 共享的唯一
Registry 实例。

```rust
// lib-greeter/src/lib.rs
use std::sync::{Arc, LazyLock};

use qubit_spi::{ProviderRegistry, ServiceSpec, SyncServiceSpec};

/// 所有 Greeter Service 都要实现的业务接口。
pub trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

/// Provider 创建 Greeter 时接收的配置。
#[derive(Clone)]
pub struct GreeterConfig {
    /// 每条问候语中放在名字前面的文本。
    pub prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

/// 向 Qubit SPI 绑定 Greeter 的配置类型和输出类型。
pub struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Provider 创建 Greeter 时接收的输入类型。
    type Config = GreeterConfig;
}

impl SyncServiceSpec for GreeterSpec {
    // 创建成功后返回给消费者的 Service 类型。
    type Output = Arc<dyn Greeter>;
}

/// 供 App 和所有下游库共享的进程级 Greeter Provider Registry。
pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. `lib-foo`：使用默认 Service

`lib-foo` 依赖 `lib-greeter` 和 `qubit-spi`，但不依赖任何具体 Greeter 实现。

```rust
// lib-foo/src/lib.rs
use lib_greeter::GREETER_REGISTRY;

/// 创建 App 选定的默认 Greeter，并打印一条问候语。
pub fn foo() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GREETER_REGISTRY.resolve()?;
    let greeter = provider.create()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. `lib-friendly-greeter`：提供第三方 Provider

`lib-friendly-greeter` 依赖 `lib-greeter` 和 `qubit-spi`。它实现 Greeter 契约并发布一个
自描述 Provider，但不会通过自行注册来修改全局状态。

```rust
// lib-friendly-greeter/src/lib.rs
use std::sync::Arc;

use lib_greeter::{Greeter, GreeterConfig, GreeterSpec};
use qubit_spi::error::ProviderError;
use qubit_spi::{
    ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider,
};

/// friendly Provider 创建的具体 Greeter 实现。
struct FriendlyGreeter {
    /// 从创建配置复制得到的问候语前缀。
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

/// 导出给 App，由 App 显式注册的自描述 Provider。
pub struct FriendlyGreeterProvider;

impl ServiceProvider<GreeterSpec> for FriendlyGreeterProvider {
    fn create_configured(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderError> {
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderMetadata for FriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_priority(100)
    }
}
```

### 4. `app.rs`：注册 Provider 并运行 `lib-foo`

App 依赖这三个库并负责装配策略。它在任何下游代码请求 Greeter 之前注册第三方 Provider，
将其设为进程默认实现，然后调用 `foo()`。

```rust
// app.rs
use lib_foo::foo;
use lib_friendly_greeter::FriendlyGreeterProvider;
use lib_greeter::GREETER_REGISTRY;
use qubit_spi::ProviderSelection;

// 应用装配入口：先安装 Provider，再调用 lib-foo。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    GREETER_REGISTRY.register(FriendlyGreeterProvider)?;
    GREETER_REGISTRY
        .set_default_selection(ProviderSelection::named("friendly")?);
    foo()
}
```

程序会打印 `Hello, Rust!`。App 与 `lib-foo` 通过 `lib-greeter` 中同一个
`GREETER_REGISTRY` 协作；`lib-foo` 和 `lib-greeter` 都不依赖
`lib-friendly-greeter`。

启动顺序很重要：必须在下游代码首次请求 Service 前配置全局 Registry。消费者已经拿到
的 `ResolvingServiceProvider` 是时间点快照；后续注册只影响未来的解析，不会修改现有
快照。

Cargo 通常会统一兼容版本的 `lib-greeter`。如果同时链接不兼容版本，每个 crate 版本
会拥有独立的静态 Registry。App 和 `lib-foo` 必须使用同一个 `lib-greeter` 实例才能
共享单体。

## 选择 Provider

Selection 是一个值对象，可以来自配置文件、命令行、库内硬编码需求或 App 默认值。
它不要求存放在 Service config 类型中。

### Named selection

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = registry.resolve_selected(&selection)?;
```

named selection 只解析一个 canonical ID 或 alias。selector 不存在时返回
`ProviderResolutionError::UnknownProviders`。它只有一个候选，因此 fallback policy
不会让其他 Provider 运行。

### 有序 chain

```rust,ignore
let selection = ProviderSelection::chain([
    "remote-greeter",
    "friendly",
    "minimal",
])?;
let provider = registry.resolve_selected(&selection)?;
```

chain 按调用方顺序排列。`chain()` 是严格模式，只要存在未知 selector 就拒绝整个
selection；只有确实允许可选插件未安装时才使用 `chain_allowing_missing()`。如果多个
selector 通过 ID 和 alias 指向同一个 Provider，该 Provider 只在首次出现的位置保留
一次。宽松 chain 的所有项都不匹配时返回 `NoCandidates`。

### 自动选择

```rust,ignore
let provider = registry.resolve_selected(&ProviderSelection::auto())?;
```

自动选择按照确定顺序包含全部已注册 Provider：

1. priority 降序；
2. priority 相同时 canonical ID 升序。

Registry 为空时返回 `ProviderResolutionError::EmptyRegistry`。

### Registry 默认 selection

新 Registry 默认使用 `ProviderSelection::auto()` 和
`FallbackPolicy::OnAbsence`。App 可以在运行时替换：

```rust,ignore
let default = ProviderSelection::chain(["remote", "friendly"])?
    .with_fallback_policy(FallbackPolicy::OnAbsence);
registry.set_default_selection(default);

let snapshot = registry.default_selection();
let provider = registry.resolve()?;
```

`set_default_selection` 保存已经校验的 selection，但不要求对应 Provider 当时已经
存在，因此可以先设置策略、后注册实现。`resolve` 使用当前 selection 和当前
Registry 状态进行解析。

### Selection 与 config 相互独立

以下四种组合都合法：

```rust,ignore
// Registry 默认 selection + 默认 config。
let service = registry.resolve()?.create()?;

// 显式 selection + 默认 config。
let service = registry.resolve_selected(&selection)?.create()?;

// Registry 默认 selection + 显式 config。
let service = registry.resolve()?.create_configured(&config)?;

// 显式 selection + 显式 config。
let service = registry.resolve_selected(&selection)?.create_configured(&config)?;
```

不要强迫每种 Service config 都包含 Provider selection 字段。config 可以把 selection
作为一种便利来源，但没有 config 对象的调用方仍然必须能够使用 Registry 默认值。

## 创建 Service

`ProviderRegistry::resolve` 和 `ProviderRegistry::resolve_selected` 返回
`ResolvingServiceProvider<S>`。它是一个组合型 resolver：持有候选 Provider handle，
并在调用 `create` 时执行 selection 中的 fallback policy。它的固有创建方法返回聚合
`ProviderCreationError`，而不是叶 Provider 接口要求的 `ProviderError`。

对应的 `AsyncProviderRegistry` 方法返回 `AsyncResolvingServiceProvider<S>`。它的固有创建
方法是异步方法；await 后得到异步 `S::Output`。叶 `AsyncServiceProvider<S>` 接口才返回
boxed 的 `ProviderFuture` 类型。

```rust,ignore
let provider = registry.resolve_selected(&selection)?;
let service = provider.create_configured(&config)?;

let async_provider = async_registry.resolve_selected(&selection)?;
let async_service = async_provider.create_configured(&config).await?;
```

同步默认 `create()` 要求 `S::Config: Default`；异步默认 `create()` 要求
`S::Config: Default + Send`。只要满足服务规范本身的约束，两种模式都可以传入显式
config。

创建成功直接返回 `S::Output`。成功 fallback 的观测属于库内部职责，不属于公共
Service 值。当前实现不对外暴露成功 attempt 数据；内部收集能力将通过 IoC 注入的
collector 和 processor 另行实现。

Qubit SPI 每次调用 `create` 都会创建一个新输出。构造成本较高时，应由 App 或库缓存
返回值，或者 clone 返回的 handle。

## Fallback policy

Fallback 属于 `ProviderSelection`，因为它是调用方的请求策略，而不是 Registry 永久
状态，也不是 Service 配置。

| Policy | `Unsupported` 后继续 | `Unavailable` 后继续 | 非法配置或初始化失败后继续 |
| --- | --- | --- | --- |
| `Never` | 否 | 否 | 否 |
| `OnAbsence` | 是 | 是 | 否 |
| `OnAnyError` | 是 | 是 | 是 |

`OnAbsence` 是默认值，也是一般场景下最安全的策略。能力或环境缺失时可以尝试备选实现；
可能属于编程或部署错误的问题则立即停止。只有明确需要降级的 best-effort 行为时才使用
`OnAnyError`。

Provider 返回叶子 `ProviderError` 后才会判断 fallback。只有同步或异步 resolver
会把实际尝试聚合成 `ProviderCreationError`。

## 错误模型

错误按照三个生命周期阶段和输入校验边界组织。

### 定义与注册错误

- `ProviderIdError`：canonical ID 为空，或不符合小写 ASCII token 规则
  （首尾为字母数字；分隔符仅限 `-`、`_`、`.`、`+`）。
- `ProviderSelectorError`：规范化后的用户/配置输入为空或非法。
- `ProviderDescriptorError`：alias 非法、重复或与 ID 相同。
- `RegistrationError`：ID 或 alias 与 Registry 状态冲突。

### Selection 构造错误

`ProviderSelectionBuildError` 在构造已校验 selection 时返回：

- `InvalidSelector`：原始 selection 输入非法；
- `EmptyChain`：调用方没有提供 chain 项。

### Provider 解析错误

`ProviderResolutionError` 在调用任何 Provider 之前返回：

- `UnknownProviders`：named 或严格 chain 中存在未知项；
- `NoCandidates`：非空 chain 中没有任何项匹配；
- `EmptyRegistry`：自动选择时没有 Provider。

这些错误不包含 Provider 创建 attempt，因为 Provider 尚未被调用。

### 叶子 Provider 错误

具体 Provider 使用 `ProviderErrorKind` 对 `ProviderError` 分类：

- `Unsupported`：Provider 不支持本次请求；
- `Unavailable`：Provider 或依赖环境不存在；
- `InvalidConfiguration`：Provider 拒绝给定配置；
- `InitializationFailed`：Provider 构造过程中发生意外失败。

使用 `_with_source` 构造器保留底层错误。Registry 内部不会替外部消费者执行日志或观测
收集；操作失败时，消费者可以获得完整的错误链。

### 聚合创建错误

`ProviderCreationError` 始终是 resolver 产生的非空聚合错误。

每个 `ProviderAttemptFailure` 保存实际调用 Provider 的 canonical ID 和原始
`ProviderError`。chain 中不存在的 selector 不会伪造 attempt。

`ProviderCreationTermination` 说明遍历为何结束：

- `Exhausted`：selection 接纳的全部候选都已尝试；
- `StoppedByPolicy`：terminal failure 后 fallback policy 不允许继续。

常用查询如下：

```rust,ignore
if error.is_absence() {
    // 所有相关失败都是 Unsupported 或 Unavailable。
}

for attempt in error.attempts() {
    eprintln!("{}: {}", attempt.provider_id(), attempt.error());
}

match error.termination() {
    ProviderCreationTermination::Exhausted => { /* ... */ }
    ProviderCreationTermination::StoppedByPolicy => { /* ... */ }
    _ => { /* 未来新增的 non-exhaustive variant */ }
}
```

`decisive_attempt()` 始终返回最后一次实际尝试；它直接导致 policy stop 或候选耗尽。

## 并发与快照语义

Provider trait 要求存储的定义满足线程安全约束，因此 `ProviderRegistry<S>` 与
`AsyncProviderRegistry<S>` 都可以跨线程共享。每个 Registry 各自拥有共享的 `RwLock`
状态。

- 注册先调用 `descriptor()`，之后才获取写锁。
- 替换默认 selection 只短暂持有写锁。
- 解析在复制候选 handle 时持有读锁。
- 释放锁之后才执行同步 Provider 创建或 poll 异步 Future；pending future 不会阻塞注册
  或查询。
- `parking_lot::RwLock` 不会发生锁中毒；panic 后锁会正常释放。

同步或异步解析出的 Provider 都持有候选的 `Arc` handle，因此对应 Registry 被 clone、
修改或 drop 后仍可使用。两种快照都不会看到后续注册；需要新候选时重新解析。

## 推荐实践

1. 每个需要独立选择的服务族定义一个 `ServiceSpec`。
2. 由领域 crate 持有 Service trait 和可选全局 facade。
3. 每个可注册 Provider 直接实现 `ProviderMetadata`。
4. 根据创建模式选择 `ProviderRegistry` 或 `AsyncProviderRegistry`；同时支持两种模式的
   实现分别注册到两个 Registry。
5. 在下游首次使用 Service 前完成 App Provider 注册。
6. 把默认策略放在 Registry 中；只有调用方有真实要求时才传显式 selection。
7. 保持 selection 与 Service config 相互独立。
8. 默认使用 `OnAbsence`；在调用点说明为何需要 `OnAnyError`。
9. 返回分类清晰并保留 causal source 的 `ProviderError`。
10. 在 Qubit SPI 外缓存构造成本较高的 Service 输出。
11. 修改注册或默认值的测试使用隔离 Registry。

## 故障排查

### 已注册 Provider 无法找到

检查 `descriptor()` 返回的 canonical ID 和规范化 alias。使用
`registry.provider_ids()` 和 `registry.descriptors()` 查看快照。注意
`ProviderId` 不做规范化，且必须已满足 canonical token 规则；而
`ProviderSelector` 会 trim 并转小写。

### `resolve()` 选择了意外 Provider

检查 `registry.default_selection()`。新 Registry 默认自动选择，按 priority 降序和
canonical ID 升序排列。如果 App 启动时应固定一个 Provider，请显式调用
`set_default_selection`。

### Fallback 没有继续

检查 terminal attempt 的 `ProviderErrorKind` 和 selection policy。`OnAbsence` 有意在
`InvalidConfiguration` 和 `InitializationFailed` 后停止。named selection 也没有第二
个候选。

### 新注册 Provider 不可见

Registry clone 可以看到新注册，但已经解析的 `ResolvingServiceProvider` 是快照，需要
重新解析。对于全局 facade，还应确认 App 与库链接的是同一领域 crate 版本。

### 无法调用 `create()`

同步 `create()` 要求 `S::Config: Default`；异步 `create()` 要求
`S::Config: Default + Send`。否则构造 config 并调用 `create_configured(&config)`
（异步路径还需 `.await` future）。

### 重复执行测试时全局注册冲突

进程级 Registry 有意保留状态。优先为每个测试创建隔离的
`ProviderRegistry::default()`；或者在独立子进程中运行需要修改全局状态的场景。

## API 参考

| API | 用途 |
| --- | --- |
| `ServiceSpec` | 绑定 config 类型 |
| `SyncServiceSpec` / `AsyncServiceSpec` | 绑定同步与异步 output 类型 |
| `ServiceProvider::create_configured` | 使用显式 config 创建 |
| `ServiceProvider::create` | 使用 `Config::default()` 创建 |
| `AsyncServiceProvider::create_configured` | 使用显式 config 异步创建 |
| `AsyncServiceProvider::create` | 在 `Config: Default + Send` 时使用默认 config 异步创建 |
| `ProviderFuture` | `AsyncServiceProvider` 实现返回的、与运行时无关且可发送的 boxed future |
| `ProviderMetadata::descriptor` | 让可注册 Provider 自描述 |
| `ProviderDefinition` / `AsyncProviderDefinition` | 组合元数据与同步或异步创建能力的 marker trait |
| `ProviderRegistry::register` | 运行时注册 owned Provider |
| `ProviderRegistry::register_shared` | 注册已有 shared Provider |
| `ProviderRegistry::set_default_selection` | 替换进程或组件默认策略 |
| `ProviderRegistry::resolve_selected` | 解析显式 selection |
| `ProviderRegistry::resolve` | 解析 Registry 当前默认值 |
| `ProviderRegistry::descriptors` | 获取注册元数据快照 |
| `ProviderRegistry::provider_ids` | 获取 canonical ID 快照 |
| `AsyncProviderRegistry::register` / `register_shared` | 同步注册 owned 或 shared 异步 Provider |
| `AsyncProviderRegistry::set_default_selection` / `default_selection` | 替换或获取异步 Registry 默认策略快照 |
| `AsyncProviderRegistry::resolve_selected` / `resolve` | 同步解析显式 selection 或当前默认值 |
| `AsyncProviderRegistry::descriptors` / `provider_ids` | 获取异步注册元数据或 canonical ID 快照 |
| `AsyncProviderRegistry::len` / `is_empty` | 查询异步 Registry 大小或是否为空 |
| `ProviderSelection::named` | 选择一个 ID 或 alias |
| `ProviderSelection::chain` | 严格选择调用方排序的候选 |
| `ProviderSelection::chain_allowing_missing` | 显式忽略未注册的 chain 项 |
| `ProviderSelection::auto` | 按确定顺序选择全部 Provider |
| `ProviderSelection::with_fallback_policy` | 附加创建阶段 fallback policy |
| `ResolvingServiceProvider` | 通过解析后的候选快照创建 Service |
| `AsyncResolvingServiceProvider` | 返回 future，通过异步候选快照创建 Service |

准确签名和 non-exhaustive 错误 variant 请查阅[自动生成的 API 文档](https://docs.rs/qubit-spi)。
