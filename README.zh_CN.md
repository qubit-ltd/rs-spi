# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-spi/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-spi?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

Rust 的强类型服务提供者注册基础设施。

## 概述

`qubit-spi` 提供一个小而明确的 SPI 层，用于“基础 crate 定义 trait，扩展
crate 提供可选实现”的场景。它面向静态链接的 Rust crate：应用程序决定链接
哪些扩展 crate，也决定在启动时注册哪些 provider。

公共 API 围绕三个核心类型组织：

- `ServiceSpec`：绑定配置类型和 service contract。
- `ServiceProvider`：为一个 service specification 创建某个具体实现。
- `ProviderRegistry`：存储 provider，并按名称、fallback chain 或基于优先级的
  自动选择解析 provider。

## 设计目标

- **显式发现**：由应用程序决定链接和注册哪些 provider。
- **类型可读性**：registry 只使用一个 `ServiceSpec` 泛型参数，而不是拆成 service、
  config 和 error 三个参数。
- **类型安全**：service contract 和配置类型由 spec 在编译期固定。
- **确定性选择**：自动选择使用稳定的 priority 和名称排序。
- **Fallback 透明**：失败候选会被保留下来，便于诊断。
- **小运行时表面积**：crate 只依赖 `log` 和 `thiserror`。

## 功能特性

- 基于 `ServiceSpec` 的单泛型 registry。
- 稳定的 `ProviderName` 校验和规范化 provider descriptor。
- 可选后端的运行时可用性检查。
- 基于 priority 的自动 provider 选择。
- 显式 named provider 加 fallback chain 的选择机制。
- 通过 `Arc` 注册共享 provider 实例。
- provider 创建错误与 registry 错误分层。
- provider 创建错误可以保留下层 source error。
- 通过 `log` 门面提供低噪声诊断日志。

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
qubit-spi = "0.2"
```

## 快速开始

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Debug + Send + Sync {
    fn greet(&self) -> &'static str;
}

#[derive(Debug)]
struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

#[derive(Debug)]
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Service = dyn Greeter;
}

#[derive(Debug)]
struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("english")?.with_aliases(&["en"])
    }

    fn create_box(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create_box("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## 核心概念

### ServiceSpec

`ServiceSpec` 绑定一个服务族的配置类型和 service contract。这个 contract 可以是
`dyn MyService` 这样的 trait object；调用方再决定 registry 返回
`Box<dyn MyService>`、`Arc<dyn MyService>` 还是 `Rc<dyn MyService>`。

### ServiceProvider

`ServiceProvider<Spec>` 是每个后端实现的工厂协议。provider 提供：

| 方法 | 用途 |
| --- | --- |
| `descriptor()` | 稳定 provider id、alias 和 priority |
| `availability(config)` | 检查可选依赖在当前环境是否可用 |
| `create_box(config)` | 创建 boxed service 值 |
| `create_arc(config)` | 创建原子引用计数 service 值 |
| `create_rc(config)` | 创建单线程引用计数 service 值 |

### ProviderRegistry

`ProviderRegistry<Spec>` 存储同一个 service specification 下的所有 provider。

registry 在注册时捕获 provider descriptor。provider id 和 alias 会规范化为
`ProviderName` 并建立索引，所以 provider 实例内部状态变化不会影响 registry 的
名称解析不变量。

### ProviderSelection

`ProviderSelection` 是一个枚举：

- `Auto`：按 priority 降序、provider id 升序尝试所有已注册 provider。
- `Named`：先尝试 primary provider，再按顺序尝试 fallbacks。

只要某个 provider 可用并成功创建 service，选择过程就会停止。

## Fallback 示例

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderCreateError,
    ProviderDescriptor,
    ProviderRegistry,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Debug + Send + Sync {
    fn greet(&self) -> &'static str;
}

#[derive(Debug)]
struct GreeterImpl(&'static str);

impl Greeter for GreeterImpl {
    fn greet(&self) -> &'static str {
        self.0
    }
}

#[derive(Debug)]
struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Service = dyn Greeter;
}

#[derive(Debug)]
struct Provider(&'static str, i32);

impl ServiceProvider<GreeterSpec> for Provider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        Ok(ProviderDescriptor::new(self.0)?.with_priority(self.1))
    }

    fn create_box(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(GreeterImpl(self.0)))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(Provider("repository", 0))
    .expect("unique provider");
registry
    .register(Provider("native", 10))
    .expect("unique provider");

let selection = ProviderSelection::from_names("native", &["repository"])
    .expect("selection names should be valid");
let greeter = registry
    .create_selected_box(&selection, &())
    .expect("one provider should create a greeter");

assert_eq!("native", greeter.greet());
```

## 错误模型

provider 错误和 registry 错误分层：

- `ProviderCreateError` 由 provider factory 返回。
- `ProviderRegistryError` 由注册、查找和选择过程返回。
- `ProviderFailure` 记录 fallback chain 中每个失败候选。

`ProviderRegistryError` 的主要 variant：

| Variant | 含义 |
| --- | --- |
| `EmptyProviderName` | provider id、alias 或 selector 为空 |
| `InvalidProviderName` | provider id、alias 或 selector 包含非法字符 |
| `DuplicateProviderName` | provider id 或 alias 与已有名称冲突 |
| `UnknownProvider` | 没有 provider 匹配请求的 selector |
| `ProviderUnavailable` | 选中的 provider 报告不可用 |
| `ProviderCreate` | 选中的 provider 创建服务失败 |
| `NoAvailableProvider` | fallback chain 中所有候选都失败 |
| `EmptyRegistry` | 对空 registry 请求自动或 selected 创建 |

`NoAvailableProvider` 会保留有序的 `ProviderFailure`，调用方可以解释完整 fallback
链路。

`ProviderCreateError::failed_with_source()` 和
`ProviderCreateError::unavailable_with_source()` 会保留下层错误原因，并通过直接
registry 创建和 fallback failure 报告继续向外暴露。

## 诊断日志

`qubit-spi` 通过 `log` 门面输出低噪声诊断日志。应用程序仍然负责安装自己选择的
logger 实现。本 crate 使用 `debug` 记录成功注册和选择结果，使用 `trace` 记录名称
解析、候选顺序和 fallback 失败。日志不会记录 service 配置值或 service 实例。

## 生命周期模型

`ProviderRegistry` 会把 provider 存储为共享 trait object。因此，已注册的
provider 和 service specification 都要求是 `'static`。这是有意设计：本 crate
面向由应用 crate 与扩展 crate 组装出来的长期存活 provider registry，而不是借用
栈上临时 provider 状态的短生命周期 registry。

## 与 Java ServiceLoader 的关系

Rust 标准库没有 Java `ServiceLoader` 的等价机制。`qubit-spi` 刻意保持显式发现：
扩展 crate 暴露 provider 类型或注册函数，应用程序注册需要对外可见的 provider。
这种方式避免 linker 魔法，也让测试隔离更容易。

如果将来某个 crate 需要链接期自动发现，可以在 `ProviderRegistry` 之上叠加
`inventory` 或 `linkme` 这类 crate。

## API 概览

| API | 用途 |
| --- | --- |
| `ServiceSpec` | 绑定 provider 配置类型和 service contract |
| `ServiceProvider` | 每个后端实现的 provider trait |
| `ProviderDescriptor` | 捕获 provider id、alias 和 priority |
| `ProviderName` | 已校验并规范化的 provider 名称 |
| `ProviderRegistry::new()` | 创建空 registry |
| `ProviderRegistry::register(provider)` | 注册 owned provider |
| `ProviderRegistry::register_shared(provider)` | 注册共享 provider |
| `ProviderRegistry::resolve_provider(name)` | 解析 provider，失败时返回精确错误 |
| `ProviderRegistry::find_provider(name)` | 返回 `Option` 的 provider 查询便利方法 |
| `ProviderRegistry::iter_provider_names()` | 无分配遍历 provider id |
| `ProviderRegistry::iter_provider_descriptors()` | 无分配遍历 descriptor |
| `ProviderRegistry::create_box(name, config)` | 按 provider 名称创建 boxed service |
| `ProviderRegistry::create_arc(name, config)` | 按 provider 名称创建原子引用计数 service |
| `ProviderRegistry::create_rc(name, config)` | 按 provider 名称创建单线程引用计数 service |
| `ProviderRegistry::create_auto_box(config)` | 按自动优先级创建 boxed service |
| `ProviderRegistry::create_auto_arc(config)` | 按自动优先级创建原子引用计数 service |
| `ProviderRegistry::create_auto_rc(config)` | 按自动优先级创建单线程引用计数 service |
| `ProviderRegistry::create_selected_box(selection, config)` | 按 selection 创建 boxed service |
| `ProviderRegistry::create_selected_arc(selection, config)` | 按 selection 创建原子引用计数 service |
| `ProviderRegistry::create_selected_rc(selection, config)` | 按 selection 创建单线程引用计数 service |
| `ProviderSelection` | 自动或 named fallback 候选选择 |
| `ProviderAvailability` | provider 可用性状态 |
| `ProviderCreateError` | provider 层创建错误 |
| `ProviderFailure` | fallback chain 中的单个失败候选 |
| `ProviderRegistryError` | registry 层错误类型 |

## Rust 版本

本 crate 使用 Rust 2024 edition，要求 Rust 1.94 或更新版本。

## 测试与代码覆盖率

本项目测试统一放在 `tests/` 目录下，覆盖 provider 名称处理、descriptor 规范化、
注册、查找、provider 选择、fallback 失败报告和错误格式化。

### 运行测试

```bash
# 运行所有测试
cargo test

# 生成覆盖率报告
./coverage.sh

# 生成文本格式覆盖率报告
./coverage.sh text

# 对齐 CI 格式化要求
./align-ci.sh

# 运行 CI 检查（格式化、clippy、测试、文档、覆盖率、audit）
./ci-check.sh
```

## 依赖项

运行时依赖保持很少：

- `log` 通过标准日志门面提供低噪声诊断日志。
- `thiserror` 用于实现具体错误类型。

## 许可证

Copyright (c) 2026. Haixing Hu.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

<http://www.apache.org/licenses/LICENSE-2.0>

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献。请保持改动与现有 Rust 项目结构一致，并在提交 Pull Request 前运行
`./ci-check.sh`。

## 作者

**Haixing Hu**

## 相关项目

Qubit 旗下的更多 Rust 库发布在 GitHub 组织
[qubit-ltd](https://github.com/qubit-ltd)。

---

仓库地址：[https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
