# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的类型安全、显式装配式服务提供者基础设施。

## 概述

Qubit SPI 支持应用定义服务族、在启动阶段注册 Provider 工厂，并通过自动、具名或
有序选择解析 Provider。构建后的 `ProviderRegistry` 不可变且可低成本克隆；
`ProviderResolver` 则在 Provider 无法创建所需服务时应用配置的
`FallbackPolicy`。

本 crate 管理 Provider 身份和选择元数据，但不限定服务句柄类型，也不会在 `Box`、
`Arc` 和 `Rc` 之间转换。

## 文档

- [用户手册](doc/user_guide.zh_CN.md)
- [API 参考](https://docs.rs/qubit-spi)
- [英文 README](README.md)

## 安装

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI 要求 Rust 1.94 或更高版本。

## 快速开始

```rust
use std::sync::Arc;

use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy,
    ProviderDescriptor,
    ProviderId,
    ProviderRegistry,
    ProviderResolver,
    ServiceProvider,
    ServiceSpec,
};

trait Greeter: Send + Sync {
    fn greet(&self) -> &'static str;
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = ();
    type Output = Arc<dyn Greeter>;
}

struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> &'static str {
        "hello"
    }
}

struct EnglishProvider;

impl ServiceProvider<GreeterSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<Arc<dyn Greeter>, ProviderError> {
        Ok(Arc::new(EnglishGreeter))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreeterSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("english")?)
            .with_aliases(["en"])?
            .with_priority(100),
        EnglishProvider,
    )?;

    let resolver = ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    );
    let created = resolver.create_named("en", &())?;

    assert_eq!("english", created.provider_id().as_str());
    assert_eq!("hello", created.service().greet());
    Ok(())
}
```

## 常用选择方式

- `create_auto` 按 priority 降序、canonical Provider ID 升序尝试 Provider。
- `create_named` 只解析一个 canonical ID 或 alias，并且从不回退。
- `create_chain` 按调用方给出的 selector 顺序尝试，记录未知 selector，并避免通过
  多个 alias 重复调用同一 Provider。
- `FallbackPolicy::OnAbsence` 会在 Provider 不支持或不可用时继续；
  `FallbackPolicy::OnAnyError` 会在任意 Provider 创建错误后继续。

有关可复用的已校验选择、Registry 查询、完整回退语义、错误诊断、并发和推荐实践，
请参阅[用户手册](doc/user_guide.zh_CN.md)。

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
