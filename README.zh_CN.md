# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

Qubit SPI 为 Rust 提供类型安全、允许运行时注册的 Service Provider Registry。App 在
启动时注册 Provider；下游库无需依赖具体实现，即可创建显式选择或 App 默认选择的 Service。

## 安装

```toml
[dependencies]
qubit-spi = "0.10"
```

Qubit SPI 要求 Rust 1.94 或更高版本。

## 快速开始

下面的示例由三个独立发布的库和一个 App 组成，分别承载 Service 契约、下游消费者、
第三方 Provider 和应用装配入口，明确展示每一部分在运行时的职责。

下面的 Cargo package 名使用连字符；Rust 在 `use` 路径中会把连字符转换成下划线。
为简洁起见，示例省略各个 `Cargo.toml` 文件。

### 1. `lib-greeter`：定义 Service 和全局 Registry

`lib-greeter` 持有 Service 契约。所有消费者和 Provider 都使用这个 crate 中同一个
`GreeterSpec` 和 `GREETER_REGISTRY` 单体。

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

`lib-foo` 只了解 Service 契约，不了解具体实现。`foo()` 从共享 Registry 中解析默认
Provider，用默认配置创建 Greeter，然后把结果打印到控制台。

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

`lib-friendly-greeter` 依赖 `lib-greeter` 中的契约，实现 Service，并导出一个自描述
Provider。它不会自行注册；最终 App 负责决定是否安装这个实现。

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

`ProviderId::new` 只接受已经是 canonical 的 token：非空小写 ASCII、首尾为字母或数字，
分隔符仅限 `-`、`_`、`.`、`+`。

### 4. `app.rs`：注册 Provider 并运行 `lib-foo`

App 是应用的装配入口。它在启动时把第三方 Provider 安装到 `lib-greeter` 持有的单体中，
将其设为默认 Provider，然后调用 `foo()`。

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

程序会打印 `Hello, Rust!`。虽然 `lib-foo` 与第三方 Provider 互不依赖，`lib-foo` 仍会
获得 App 选定的实现；它们的共享协调点是 `lib-greeter` 定义的单体。

Registry 默认 selection 与 Service 配置相互独立。有明确需求的调用方可以只显式提供
其中一个，也可以同时提供：

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = GREETER_REGISTRY.resolve_selected(&selection)?;
let config = GreeterConfig {
    prefix: "Welcome".to_owned(),
};
let greeter = provider.create_configured(&config)?;
```

### 5. 异步快速入门

异步 API 保持目录操作同步，仅让 Service 创建异步。因此 Registry 不依赖 executor：

```rust
use qubit_spi::error::ProviderError;
use qubit_spi::{
    AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec,
    ProviderDescriptor, ProviderFuture, ProviderId, ProviderMetadata,
    ProviderSelection, ServiceSpec,
};

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = str;
}

impl AsyncServiceSpec for GreetingSpec {
    type Output = String;
}

struct FriendlyProvider;

impl ProviderMetadata for FriendlyProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
    }
}

impl AsyncServiceProvider<GreetingSpec> for FriendlyProvider {
    fn create_configured<'a>(
        &'a self,
        name: &'a str,
    ) -> ProviderFuture<'a, Result<String, ProviderError>> {
        Box::pin(async move { Ok(format!("Hello, {name}!")) })
    }
}

async fn greet() -> Result<String, Box<dyn std::error::Error>> {
    let registry = AsyncProviderRegistry::<GreetingSpec>::default();
    registry.register(FriendlyProvider)?;
    let selection = ProviderSelection::named("friendly")?;
    let resolver = registry.resolve_selected(&selection)?;
    Ok(resolver.create_configured("Rust").await?)
}
```

在 Registry 工作流中，`register`、元数据查询、默认 selection 更新和解析均为同步操作。
解析得到的 `AsyncResolvingServiceProvider` 创建方法返回 `ProviderFuture`，必须
`.await` 才能获得 output。直接调用异步叶 Provider 的创建方法也会返回
`ProviderFuture`。`ProviderFuture` 是 `Send` 且与 runtime 无关。异步 spec 要求
`Config: Sync` 和 `Output: Send + 'static`；使用默认配置的 `create()` 还要求
`Config: Default + Send`。

## 为什么需要这个库

应用真正依赖的通常是一种能力，而不是某个固定实现。例如 MIME 检测器可以由模型、
系统命令或特征库实现；文件系统可以是本地、内存或远程实现。App 应该决定进程中安装
哪些实现以及默认使用哪个；下游库只应请求自己需要的能力。

完整生命周期包含三个不同问题：

1. **注册：** 当前进程中有哪些 Provider 实现？
2. **选择：** 本次请求应考虑哪个 Provider，或者哪组有序候选？
3. **创建：** 已选 Provider 能否使用给定配置创建 Service？

Qubit SPI 明确分离这三个阶段，并为每个失败边界提供不同错误类型。这样，Service 配置
不会意外变成查找 Provider 的前提，Provider 初始化失败也不会与选择失败混在一起。

## 它提供什么

- `ServiceSpec` 绑定同一服务族的配置类型。
- `SyncServiceSpec` 与 `AsyncServiceSpec` 分别绑定同步和异步输出类型。
- `ServiceProvider` 与 `AsyncServiceProvider` 是互不混合的创建契约。
- `ProviderMetadata` 为 Provider 增加稳定 ID、alias 和 priority。
- `ProviderId` 是严格的 canonical token：非空小写 ASCII、首尾为字母或数字，
  中间仅允许分隔符 `-`、`_`、`.`、`+`；构造时不会 trim 或转小写。
- `ProviderRegistry` 与 `AsyncProviderRegistry` 是相互独立的运行时目录；
  两者的注册、查询和 resolve 方法都保持同步。
- `ProviderSelection` 同时保存选择目标和创建阶段 fallback policy。
- `ResolvingServiceProvider` 与 `AsyncResolvingServiceProvider` 分别由对应 Registry
  解析后返回，并在创建 Service 时执行回退。
- `ProviderFuture` 是异步创建返回的、与 runtime 无关的 `Send` future。
- 注册、选择、叶子 Provider 和聚合创建错误相互分离，并保留失败时真正需要的上下文。

Qubit SPI 不负责动态库加载、自动发现 crate、缓存已创建的 Service，也不强制提供统一的
进程全局单体。需要 App 与下游库共享时，应由领域 crate 暴露自己的全局 Registry facade。

## 核心生命周期

```text
App 启动
  注册 ProviderMetadata + 创建能力
  设置 Registry 默认 ProviderSelection
                         │
                         ▼
共享 ProviderRegistry<SyncServiceSpec>
                         │ resolve_selected / resolve
                         ▼
ResolvingServiceProvider<SyncServiceSpec>
                         │ create_configured(config) / create()
                         ▼
SyncServiceSpec::Output
```

| 阶段 | 主要 API | 成功结果 | 失败类型 |
| --- | --- | --- | --- |
| 注册 | `register(provider)` | Provider 对所有 Registry clone 可见 | `RegistrationError` |
| 选择 | `resolve_selected(&selection)` 或 `resolve()` | `ResolvingServiceProvider` 中的候选快照 | `ProviderResolutionError` |
| 创建 | `create_configured(&config)` 或 `create()` | 直接返回 `SyncServiceSpec::Output` | `ProviderCreationError` |

异步路径遵循相同的三阶段生命周期。如[异步快速入门](#5-异步快速入门)所示，目录操作
保持同步，只有通过 `AsyncResolvingServiceProvider` 创建 Service 时需要 `.await`，
因此 SPI 不绑定任何 executor。

## 选择与回退

| Selection | 候选顺序 | selector 不存在时 |
| --- | --- | --- |
| `ProviderSelection::named("id")` | 只包含一个 Provider | 解析阶段返回 `UnknownProviders` |
| `ProviderSelection::chain([..])` | 调用方顺序，并去除指向同一 Provider 的重复项 | 严格拒绝任何不存在的项 |
| `ProviderSelection::chain_allowing_missing([..])` | 调用方顺序，并去重 | 跳过不存在的项；全部不匹配时失败 |
| `ProviderSelection::auto()` | priority 降序，再按 canonical ID 升序 | Registry 为空时失败 |

每个 selection 都携带一个创建阶段使用的 `FallbackPolicy`：

- `Never`：第一个 Provider 创建失败后立即停止。
- `OnAbsence`（默认）：仅在 `Unsupported` 或 `Unavailable` 后继续。
- `OnAnyError`：任意叶子 Provider 错误后都继续。

named selection 只有一个候选，因此不会回退。选择阶段不调用 Provider 代码。创建阶段使用
解析时得到的候选快照，并且调用 Provider 时不会持有 Registry 锁。

## 错误边界

| 错误 | 所属边界 | 含义 |
| --- | --- | --- |
| `ProviderIdError` | Provider 定义 | canonical ID 为空，或不符合小写 ASCII token 规则 |
| `ProviderSelectorError` | 输入解析 | selector 无法规范化或校验失败 |
| `ProviderSelectionBuildError` | selection 构造 | named 或 chain selection 输入非法 |
| `ProviderDescriptorError` | Provider 定义 | alias 非法或 descriptor 内部重复 |
| `RegistrationError` | 注册 | ID 或 alias 已被占用 |
| `ProviderResolutionError` | selection 解析 | 无法解析出候选 Provider |
| `ProviderError` | 叶子创建 | 某个具体 Provider 返回分类后的失败 |
| `ProviderCreationError` | resolver 创建 | 仅包含实际 Provider 尝试的非空聚合错误 |

聚合创建错误只记录真正调用过的 Provider，并说明候选遍历是全部耗尽，还是因为 fallback
policy 不允许继续而停止。消费者通常直接向上传递错误；只有需要针对失败采取动作时，才
检查 attempt 列表。

## 运行时 Registry 与全局 facade

`ProviderRegistry` 与 `AsyncProviderRegistry` 各自使用同步共享状态，并具有相同的低成本
clone 语义：通过一个 clone 完成的注册或默认 selection 修改，对同一 Registry 的其他
clone 可见。两者的 descriptor 和候选查询都返回自有快照，并在执行 Provider 代码或
轮询异步创建 future 前释放 Registry 锁。两者的注册状态相互独立；在一个 Registry 中
注册 Provider 不会将其注册到另一个 Registry。

可复用的领域 crate 可以用 `LazyLock` 持有一个 Registry，并暴露领域专用的 `global()`
方法。这样 App 启动时注册的 Provider，之后可以被独立发布的库通过 `resolve()`
获得。App 必须在下游代码首次需要服务之前完成配置。如果 Cargo 链接了同一领域 crate
的不兼容版本，每个被链接的 crate 版本会拥有各自的静态 Registry。

测试或局部组件需要隔离状态时，可以使用 `ProviderRegistry::default()`。构造后的
Registry 仍然允许运行时注册。

## 延伸阅读

- 阅读[用户手册](doc/user_guide.zh_CN.md)，了解完整生命周期、Provider 实现、运行时共享、
  selection 语义、fallback、诊断和全局 facade 模式。
- 浏览 [API 文档](https://docs.rs/qubit-spi)。
- Read the [English README](README.md).

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
