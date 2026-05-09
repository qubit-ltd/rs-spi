# Qubit SPI

[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Rust 的强类型服务提供者注册基础设施。

`qubit-spi` 提供一个小而明确的 SPI 层，用于“基础 crate 定义 trait，扩展
crate 提供可选实现”的场景。它面向静态链接的 Rust crate：应用程序决定链接
哪些扩展 crate，也决定在启动时注册哪些 provider。

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
qubit-spi = "0.1"
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
    type Service = Box<dyn Greeter>;
}

#[derive(Debug)]
struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        ProviderDescriptor::new("english")?.with_aliases(&["en"])
    }

    fn create(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<GreeterSpec>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## 核心概念

### ServiceSpec

`ServiceSpec` 绑定一个服务族的配置类型和 provider 输出类型。这样
`ProviderRegistry<Spec>` 只有一个泛型参数，同时每个 crate 仍然可以决定 provider
返回 `Box<dyn Trait>`、`Arc<dyn Trait>`、具体类型或其他 service handle。

### ServiceProvider

`ServiceProvider<Spec>` 是每个后端实现的工厂协议。provider 提供：

| 方法 | 用途 |
| --- | --- |
| `descriptor()` | 稳定 provider id、alias 和 priority |
| `availability(config)` | 检查可选依赖在当前环境是否可用 |
| `create(config)` | 创建 `Spec::Service` service 值 |

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
    type Service = Box<dyn Greeter>;
}

#[derive(Debug)]
struct Provider(&'static str, i32);

impl ServiceProvider<GreeterSpec> for Provider {
    fn descriptor(&self) -> Result<ProviderDescriptor, ProviderRegistryError> {
        Ok(ProviderDescriptor::new(self.0)?.with_priority(self.1))
    }

    fn create(&self, _config: &()) -> Result<Box<dyn Greeter>, ProviderCreateError> {
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
    .create_selected(&selection, &())
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
| `ServiceSpec` | 绑定 provider 配置类型和 service 类型 |
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
| `ProviderRegistry::create(name, config)` | 按 provider 名称创建 service |
| `ProviderRegistry::create_auto(config)` | 按自动优先级创建 service |
| `ProviderRegistry::create_selected(selection, config)` | 按 selection 创建 |
| `ProviderSelection` | 自动或 named fallback 候选选择 |
| `ProviderAvailability` | provider 可用性状态 |
| `ProviderCreateError` | provider 层创建错误 |
| `ProviderFailure` | fallback chain 中的单个失败候选 |
| `ProviderRegistryError` | registry 层错误类型 |

## Rust 版本

本 crate 使用 Rust 2024 edition，要求 Rust 1.94 或更新版本。

## 许可证

本项目基于 Apache License, Version 2.0 授权。详见 [LICENSE](LICENSE)。
