# Qubit SPI

[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Rust 的强类型服务提供者注册基础设施。

`qubit-spi` 提供一个小而明确的 SPI 层，用于“基础 crate 定义 trait，扩展
crate 提供可选实现”的场景。它面向静态链接的 Rust crate：应用程序决定链接
哪些扩展 crate，也决定在启动时注册哪些 provider。

## 功能特性

- 针对一个服务 trait、配置类型和 provider 错误类型的强类型 registry。
- 稳定 provider id 和大小写不敏感 alias。
- 可选后端的运行时可用性检查。
- 基于 priority 的自动 provider 选择。
- 显式 default 加 fallback chain 的选择机制。
- 通过 `Arc` 注册共享 provider 实例。
- 错误中保留 unknown、unavailable 和 creation failure 等候选状态。

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
    ProviderRegistry,
    ProviderRegistryError,
    ServiceProvider,
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
struct EnglishProvider;

impl ServiceProvider for EnglishProvider {
    type Config = ();
    type Service = dyn Greeter;

    fn id(&self) -> &'static str {
        "english"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["en"]
    }

    fn create(&self, _config: &Self::Config) -> Result<Box<Self::Service>, ProviderRegistryError> {
        Ok(Box::new(EnglishGreeter))
    }
}

let mut registry = ProviderRegistry::<dyn Greeter, ()>::new();
registry
    .register(EnglishProvider)
    .expect("provider names should be unique");

let greeter = registry
    .create("en", &())
    .expect("registered provider should create a greeter");
assert_eq!("hello", greeter.greet());
```

## 核心概念

### ServiceProvider

`ServiceProvider` 是每个后端实现的工厂协议。provider 提供：

| 方法 | 用途 |
| --- | --- |
| `id()` | 规范、稳定的 provider id |
| `aliases()` | registry 接受的其他名称 |
| `priority()` | 自动选择时优先级越高越优先 |
| `availability(config)` | 检查可选依赖在当前环境是否可用 |
| `create(config)` | 创建 boxed service 实现 |

关联类型 `Service` 可以是 `dyn Greeter` 这样的 trait object。

### ProviderRegistry

`ProviderRegistry<S, C>` 存储同一个服务类型 `S` 和配置类型 `C` 下的所有
provider。

provider id 和 alias 采用大小写不敏感匹配。注册时会拒绝重复名称，包括同一个
provider 自身 id 与 alias 之间的冲突。

### ProviderSelection

`ProviderSelection` 描述 `create_default()` 如何构造候选 provider：

- default 为空或为 `auto`：按 priority 降序、provider id 升序尝试所有已注册
  provider。
- default 是显式名称：先尝试 default，再按顺序尝试 fallbacks。

只要某个 provider 可用并成功创建 service，选择过程就会停止。

## Fallback 示例

```rust
use std::fmt::Debug;

use qubit_spi::{
    ProviderRegistry,
    ProviderRegistryError,
    ProviderSelection,
    ServiceProvider,
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
struct Provider(&'static str, i32);

impl ServiceProvider for Provider {
    type Config = ();
    type Service = dyn Greeter;

    fn id(&self) -> &'static str {
        self.0
    }

    fn priority(&self) -> i32 {
        self.1
    }

    fn create(&self, _config: &()) -> Result<Box<Self::Service>, ProviderRegistryError> {
        Ok(Box::new(GreeterImpl(self.0)))
    }
}

let mut registry = ProviderRegistry::<dyn Greeter, ()>::new();
registry
    .register(Provider("repository", 0))
    .expect("unique provider");
registry
    .register(Provider("native", 10))
    .expect("unique provider");

let selection = ProviderSelection::from_names("native", &["repository"]);
let greeter = registry
    .create_default(&selection, &())
    .expect("one provider should create a greeter");

assert_eq!("native", greeter.greet());
```

## 错误模型

`ProviderRegistryError` 区分注册、查找和选择失败：

| Variant | 含义 |
| --- | --- |
| `EmptyProviderName` | provider id、alias 或 selector 为空 |
| `DuplicateProviderName` | provider id 或 alias 与已有名称冲突 |
| `UnknownProvider` | 没有 provider 匹配请求的 selector |
| `ProviderUnavailable` | 选中的 provider 报告不可用 |
| `ProviderCreate` | 选中的 provider 创建服务失败 |
| `NoAvailableProvider` | fallback chain 中所有候选都失败 |
| `EmptyRegistry` | 对空 registry 请求自动或默认创建 |

`NoAvailableProvider` 会保留有序的 `ProviderFailure`，调用方可以解释完整 fallback
链路。

## 与 Java ServiceLoader 的关系

Rust 标准库没有 Java `ServiceLoader` 的等价机制。`qubit-spi` 刻意保持显式发现：
扩展 crate 暴露 provider 类型或注册函数，应用程序注册需要对外可见的 provider。
这种方式避免 linker 魔法，也让测试隔离更容易。

如果将来某个 crate 需要链接期自动发现，可以在 `ProviderRegistry` 之上叠加
`inventory` 或 `linkme` 这类 crate。

## API 概览

| API | 用途 |
| --- | --- |
| `ServiceProvider` | 每个后端实现的 provider trait |
| `ProviderRegistry::new()` | 创建空 registry |
| `ProviderRegistry::register(provider)` | 注册 owned provider |
| `ProviderRegistry::register_arc(provider)` | 注册共享 provider |
| `ProviderRegistry::find_provider(name)` | 按 id 或 alias 解析 provider |
| `ProviderRegistry::create(name, config)` | 按 provider 名称创建 service |
| `ProviderRegistry::create_auto(config)` | 按自动优先级创建 service |
| `ProviderRegistry::create_default(selection, config)` | 按 default 和 fallbacks 创建 |
| `ProviderSelection` | default 与 fallback 候选配置 |
| `ProviderAvailability` | provider 可用性状态 |
| `ProviderFailure` | fallback chain 中的单个失败候选 |
| `ProviderRegistryError` | registry 错误类型 |

## Rust 版本

本 crate 使用 Rust 2024 edition，要求 Rust 1.94 或更新版本。

## 许可证

本项目基于 Apache License, Version 2.0 授权。详见 [LICENSE](LICENSE)。
