# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

## 这个库解决什么问题

应用可以为同一种服务注册多个实现，并在运行时选择合适的实现；整个过程不依赖全局
状态，也不需要通过无类型的名称查找对象。

例如，应用可以优先使用云端后端，在云端不可用时回退到本地后端，也可以根据配置指定
某一个后端。Rust 会检查所有 Provider 是否接收相同的配置类型并返回相同的输出类型。

## 安装

```toml
[dependencies]
qubit-spi = "0.8"
```

Qubit SPI 要求 Rust 1.94 或更高版本。

## 快速开始

```rust
use qubit_spi::error::ProviderError;
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = ();
    type Output = &'static str;
}

struct EnglishProvider;

impl ServiceProvider<GreetingSpec> for EnglishProvider {
    fn create(&self, _config: &()) -> Result<&'static str, ProviderError> {
        Ok("hello")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("english")?),
        EnglishProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let created = resolver.create_named("english", &())?;

    assert_eq!("english", created.provider_id().as_str());
    assert_eq!("hello", *created.service());
    Ok(())
}
```

## 示例如何工作

1. `GreetingSpec` 将 Provider 的输入固定为 `()`，将输出固定为
   `&'static str`。
2. `EnglishProvider` 实现工厂操作并返回问候语。
3. `ProviderDescriptor` 在注册时为 Provider 指定 canonical ID `english`。
4. `ProviderRegistry::builder()` 在启动阶段收集 Provider，`build()` 将目录冻结为
   运行时使用的不可变 Registry。
5. `ProviderResolver::create_named` 选择 `english`；返回的 `CreatedService` 同时包含
   输出和实际胜出的 canonical ID。

## 常用选择方式

| 需求 | 方法 | 行为 |
| --- | --- | --- |
| 使用配置指定的一个 Provider | `create_named` | 只尝试一个 canonical ID 或 alias，不会回退。 |
| 使用当前最合适的 Provider | `create_auto` | 先按 priority 降序，再按 canonical ID 升序尝试。 |
| 按偏好顺序尝试 | `create_chain` | 按调用方给出的 selector 顺序尝试，并避免通过 alias 重复调用同一个 Provider。 |

每个 Resolver 都有回退策略。`FallbackPolicy::OnAbsence` 是更安全的默认选择：它会在
Provider 不支持或暂时不可用时继续，但会在配置错误或初始化错误时停止。只有明确需要
尽力回退时才使用 `OnAnyError`。

## 继续学习

- 阅读[用户手册](doc/user_guide.zh_CN.md)，通过带详细注释的完整示例理解真实输出句柄、
  alias、priority、回退、诊断、共享与性能细节。
- 浏览 [API 参考](https://docs.rs/qubit-spi)。
- 阅读[英文 README](README.md)。

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
