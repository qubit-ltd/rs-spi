# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

Qubit SPI 是 Rust 中构建 Service Provider Registry 的类型安全基础设施。
App 在启动时注册自描述 Provider；独立开发的下游库无需了解具体实现，只需按照显式
selection 或 App 设置的默认 selection 解析 Provider，再使用显式或默认配置创建服务。

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

- `ServiceSpec` 绑定同一服务族的配置类型和输出类型。
- `ServiceProvider` 创建 Service，并直接返回 Service 实体。
- `ProviderDefinition` 为 Provider 增加稳定 ID、alias 和 priority。
- `ProviderRegistry` 允许运行时修改、线程安全，clone 后共享同一状态。
- `ProviderSelection` 同时保存选择目标和创建阶段 fallback policy。
- `ResolvingServiceProvider` 是 Registry 解析后返回的 Provider，在创建服务时执行回退。
- 注册、选择、叶子 Provider 和聚合创建错误相互分离，并保留失败时真正需要的上下文。

Qubit SPI 不负责动态库加载、自动发现 crate、缓存已创建的 Service，也不强制提供统一的
进程全局单体。需要 App 与下游库共享时，应由领域 crate 暴露自己的全局 Registry facade。

## 核心生命周期

```text
App 启动
  注册 ProviderDefinition
  设置 Registry 默认 ProviderSelection
                         │
                         ▼
共享 ProviderRegistry<ServiceSpec>
                         │ resolve / resolve_default
                         ▼
ResolvingServiceProvider<ServiceSpec>
                         │ create(config) / create_default()
                         ▼
ServiceSpec::Output
```

| 阶段 | 主要 API | 成功结果 | 失败类型 |
| --- | --- | --- | --- |
| 注册 | `register(provider)` | Provider 对所有 Registry clone 可见 | `RegistrationError` |
| 选择 | `resolve(&selection)` 或 `resolve_default()` | `ResolvingServiceProvider` 中的候选快照 | `ProviderSelectionError` |
| 创建 | `create(&config)` 或 `create_default()` | 直接返回 `ServiceSpec::Output` | `ProviderCreationError` |

## 安装

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI 要求 Rust 1.94 或更高版本。

## 快速开始

下面的示例对应最常见的 App/库 X 场景：App 在启动时配置共享 Registry；库 X 只负责
解析并创建服务，不依赖具体 Provider 实现。

```rust
use std::sync::{Arc, LazyLock};

use qubit_spi::error::{ProviderCreationError, ProviderError};
use qubit_spi::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ProviderRegistry,
    ProviderSelection, ServiceProvider, ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

#[derive(Clone)]
struct GreeterConfig {
    prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}

struct FriendlyGreeter {
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

struct FriendlyProvider;

impl ServiceProvider<GreeterSpec> for FriendlyProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            )
            .into());
        }
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderDefinition<GreeterSpec> for FriendlyProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_aliases(["default-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}

// 领域 crate 通常持有这个 facade，并暴露类型化的 global() 方法。
// Qubit SPI 提供 Registry，而不是一个适用于所有服务族的全局实例。
static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);

fn greeter_registry() -> &'static ProviderRegistry<GreeterSpec> {
    &GREETER_REGISTRY
}

// App 启动代码负责安装 Provider 和设置进程默认 selection。
fn configure_app() -> Result<(), Box<dyn std::error::Error>> {
    let registry = greeter_registry();
    registry.register(FriendlyProvider)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    Ok(())
}

// 这个函数代表独立发布的库 X。
fn library_x_greeter() -> Result<Arc<dyn Greeter>, Box<dyn std::error::Error>> {
    let provider = greeter_registry().resolve_default()?;
    Ok(provider.create_default()?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    configure_app()?;

    let greeter = library_x_greeter()?;
    assert_eq!("Hello, Rust!", greeter.greet("Rust"));
    Ok(())
}
```

Registry 默认 selection 与 Service 配置相互独立。有明确需求的调用方可以只显式提供
其中一个，也可以同时提供：

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = greeter_registry().resolve(&selection)?;
let config = GreeterConfig {
    prefix: "Welcome".to_owned(),
};
let greeter = provider.create(&config)?;
```

## 选择与回退

| Selection | 候选顺序 | selector 不存在时 |
| --- | --- | --- |
| `ProviderSelection::named("id")` | 只包含一个 Provider | 解析阶段返回 `UnknownProvider` |
| `ProviderSelection::chain([..])` | 调用方顺序，并去除指向同一 Provider 的重复项 | 跳过不存在的项；全部不匹配时失败 |
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
| `ProviderIdError` | Provider 定义 | canonical ID 非法 |
| `ProviderSelectorError` | 输入解析 | selector 无法规范化或校验失败 |
| `ProviderDescriptorError` | Provider 定义 | alias 非法或 descriptor 内部重复 |
| `RegistrationError` | 注册 | ID 或 alias 已被占用 |
| `ProviderSelectionError` | 选择 | 无法解析出候选 Provider |
| `ProviderError` | 叶子创建 | 某个具体 Provider 返回分类后的失败 |
| `ProviderCreationError` | 创建 | 直接或聚合创建失败，并保留实际尝试记录 |

聚合创建错误只记录真正调用过的 Provider，并说明候选遍历是全部耗尽，还是因为 fallback
policy 不允许继续而停止。消费者通常直接向上传递错误；只有需要针对失败采取动作时，才
检查 attempt 列表。

## 运行时 Registry 与全局 facade

`ProviderRegistry` 内部使用同步共享状态。clone 成本低，通过任意 clone 完成的注册或默认
selection 修改，都对其他 clone 可见。descriptor 和候选查询返回自有快照，因此执行
Provider 代码时不会持有 Registry 锁。

可复用的领域 crate 可以用 `LazyLock` 持有一个 Registry，并暴露领域专用的 `global()`
方法。这样 App 启动时注册的 Provider，之后可以被独立发布的库通过 `resolve_default()`
获得。App 必须在下游代码首次需要服务之前完成配置。如果 Cargo 链接了同一领域 crate
的不兼容版本，每个被链接的 crate 版本会拥有各自的静态 Registry。

测试或局部组件需要隔离状态时，可以使用 `ProviderRegistry::default()` 或
`ProviderRegistry::builder()`。builder 构造完成后的 Registry 仍然允许运行时注册。

## 破坏性迁移

本版本有意移除以前的“不可变目录 + 独立 resolver”工作流，主要迁移关系如下：

| 旧工作流 | 当前工作流 |
| --- | --- |
| descriptor 与 provider 分开注册 | 实现 `ProviderDefinition::descriptor()`，调用 `register(provider)` |
| 单独创建 `ProviderResolver` | 调用 `ProviderRegistry::resolve()` 或 `resolve_default()` |
| fallback policy 保存在 resolver 中 | fallback policy 保存在 `ProviderSelection` 中 |
| 一次操作同时解析并创建 | 先解析 `ResolvingServiceProvider`，再调用 `create` |
| 返回 `CreatedService` 包装 | 直接返回 `ServiceSpec::Output` |
| 处理统一的 `ResolutionError` | 在正确阶段处理 `ProviderSelectionError` 或 `ProviderCreationError` |

本版本不提供兼容层。下游 crate 必须一起迁移 Provider 定义、注册、选择、创建和错误转换。

## 延伸阅读

- 阅读[用户手册](doc/user_guide.zh_CN.md)，了解完整生命周期、Provider 实现、运行时共享、
  selection 语义、fallback、诊断、全局 facade 模式和迁移方法。
- 浏览 [API 文档](https://docs.rs/qubit-spi)。
- Read the [English README](README.md).

## 测试

```bash
# 测试核心 API
cargo test --no-default-features

# 测试全部 feature 和文档示例
cargo test --all-features

# 运行完整项目 CI 检查
./ci-check.sh

# 生成覆盖率报告
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交 Pull Request 前
依次运行 `./align-ci.sh` 和 `./ci-check.sh`。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
