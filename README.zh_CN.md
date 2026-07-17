# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

为 Rust 应用与库提供类型安全的服务 Provider 注册、选择和创建基础设施。

## 模型

`ServiceSpec` 定义一个服务族的配置类型与输出类型；`ServiceProvider` 负责创建该
输出；`ProviderDefinition` 则通过 `descriptor()` 为 Provider 增加稳定 ID、别名和
自动选择优先级。

`ProviderRegistry` 是可克隆的同步目录。应用既可以在启动阶段注册自描述 Provider，
也可以在运行期间继续注册。所有 clone 都能看到后续注册和默认 selection 的更新。

服务获取包含两个彼此独立的输入：

1. `ProviderSelection` 选择候选 Provider，并携带相应的 `FallbackPolicy`。
2. `S::Config` 配置由已选 Provider 创建的服务。

`resolve()` 或 `resolve_default()` 会把 Registry 当前状态转换为一个候选快照
`ResolvingServiceProvider`。随后调用 `create()` 或 `create_default()`，成功时直接
返回 `S::Output`。

## 安装

```toml
[dependencies]
qubit-spi = "0.8"
```

## 快速开始

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderCreationError;
use qubit_spi::{
    ProviderDefinition,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self) -> &'static str;
}

struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Output = Arc<dyn Greeter>;
}

struct EnglishProvider {
    descriptor: ProviderDescriptor,
}

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(
        &self,
        _config: &(),
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

impl ProviderDefinition<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(EnglishProvider {
    descriptor: ProviderDescriptor::new(ProviderId::new("english")?)
        .with_aliases(["en"])?
        .with_priority(100),
})?;

registry.set_default_selection(ProviderSelection::named("en")?);
let greeter = registry.resolve_default()?.create_default()?;

assert_eq!("hello", greeter.greet());
# Ok(())
# }
```

## 选择与回退

- `ProviderSelection::auto()` 按 priority 降序、canonical Provider ID 升序生成
  候选快照。
- `ProviderSelection::named(...)` 解析一个 canonical ID 或别名。
- `ProviderSelection::chain(...)` 保持配置顺序、跳过未知 selector，并根据实际
  Provider 去重其多个别名。
- `FallbackPolicy::Never` 在首次 Provider 创建失败后停止。
- `FallbackPolicy::OnAbsence` 仅在 leaf failure 为 `Unsupported` 或
  `Unavailable` 时继续，也是默认策略。
- `FallbackPolicy::OnAnyError` 在任意 leaf failure 后继续。

Selection 是不可变值，可以用 `with_fallback_policy()` 派生不同回退行为。解析完成的
Provider 持有当时的候选快照：此后新增的注册只影响未来的解析，不改变已有快照。

## 错误边界

选择失败与创建失败发生在不同生命周期：

- `ProviderSelectionError` 表示 selection 构造无效、named Provider 未知、chain
  没有匹配候选，或 auto 面对空 Registry。返回该错误时不会调用任何 Provider。
- `ProviderCreationError` 表示候选确定后的创建失败。leaf `ProviderError` 对单个
  Provider 失败分类；聚合错误保留按顺序排列的 `ProviderAttemptFailure`，并通过
  `Exhausted` 与 `StoppedByPolicy` 区分终止原因。

尝试诊断只包含真正调用过的 Provider，错误对象会保留完整 source 链。成功调用只返回
服务值；本 crate 不提供成功包装，也不提供面向消费者的观测 API。

## 注册与全局门面

注册只接收一个自描述 Provider：

```rust,ignore
registry.register(provider)?;
registry.register_shared(shared_provider)?;
```

Registry 会在获取写锁前调用并快照 `ProviderDefinition::descriptor()`。它在修改状态前
完整校验 canonical ID 与所有别名，因此冲突注册不会留下部分 selector。

这个通用 crate 不为任何具体服务族定义全局单例。领域 crate 可以在
`ProviderRegistry<MimeDetectorSpec>` 之上提供 MIME Detector Registry 之类的全局
门面。App 在启动时通过该门面注册自定义 Provider；下游库则从同一个共享 Registry
按显式或默认 selection 解析服务，无需依赖具体实现。

## 测试

```bash
# 使用默认的空 feature 集测试核心 API
cargo test --no-default-features

# 测试核心 API 和正则校验
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
