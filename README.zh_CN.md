# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Documentation](https://docs.rs/qubit-spi/badge.svg)](https://docs.rs/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的类型安全、显式装配式服务提供者基础设施。

## 模型

应用在启动阶段通过 ProviderRegistryBuilder 注册 Provider。调用 build() 后得到
不可变且可低成本克隆的 ProviderRegistry。ProviderResolver 将该目录与
ProviderSelection、FallbackPolicy 组合起来创建服务。Resolver 持有 registry，
通过 `registry()` 提供只读访问，并通过 `fallback_policy()` 暴露当前回退策略。

ServiceSpec 同时定义配置类型和完整输出句柄；SPI 核心不会在 Box、Arc 和 Rc 之间
转换。

## 安装

~~~toml
[dependencies]
qubit-spi = "0.5"
~~~

## 快速开始

~~~rust
use std::sync::Arc;

use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderError,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
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

struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeter>, ProviderError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("english")?).with_aliases(["en"])?,
    EnglishProvider,
)?;
let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
let created = resolver.create_named("en", &())?;
assert_eq!("hello", created.service().greet());
# Ok(())
# }
~~~

## 选择与失败

- `ProviderSelection::auto()` 按 descriptor priority 降序、canonical Provider ID
  升序选择。
- `ProviderSelection::named(...)` 只选择一个 Provider。
- `ProviderSelection::chain(...)` 按配置顺序尝试候选项，并避免通过多个别名重复尝试同一
  Provider。
- `ProviderResolver::create_auto`、`create_named`、`create_chain` 可直接接收运行时
  原始输入，并把解析失败统一报告为 `ResolutionError`。
- FallbackPolicy::OnAbsence 会在未知、不支持或不可用时继续回退；遇到无效配置和
  初始化失败时停止。
- FallbackPolicy::OnAnyError 用于明确要求尽力而为的回退链。

`ProviderError` 对单次工厂失败分类。`ResolutionError` 会记录已尝试的候选项，保留
无效 selector 的原始输入及校验错误链，并明确区分空 registry 与空原始 chain；其
显示文本包含按顺序排列的尝试诊断。每个 `AttemptFailure` 会显式区分未知 selector
与 Provider 创建失败。

校验和装配错误按生命周期拆分为 `ProviderIdError`、`ProviderSelectorError`、
`ProviderDescriptorError`、`ProviderSelectionError` 与 `RegistrationError`；其中
registration error 只表示 registry 内部的 selector 冲突。

`CreatedService` 暴露实际胜出的 canonical Provider ID，可通过 `into_service()` 或
`into_parts()` 消费。`ProviderRegistry::len()` 与 `is_empty()` 可无分配查看目录大小。

## 注册

Provider 身份属于注册过程，而不是 Provider 工厂本身。持有具体 Provider 时使用
`register(descriptor, provider)`；已经持有 `Arc` 工厂时使用
`register_shared(descriptor, provider)`。Builder 会在修改内部状态前完整校验
canonical ID 和全部别名，因此失败的注册不会占用部分 selector。

核心不提供全局 registry。应用应在启动阶段显式装配所需 Provider，并共享构建出的
不可变 registry 或 resolver。
