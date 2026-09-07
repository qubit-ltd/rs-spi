# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

Qubit SPI 为 Rust 库提供可由应用选择的服务实现。库只依赖服务接口，应用负责注册
可用的服务提供者、设置默认选择，并在创建服务时传入配置。例如，业务库需要问候
服务时，可以直接取得应用选定的实现，无需自行决定使用哪个后端。

## 安装

```toml
[dependencies]
qubit-spi = "0.11"
```

需要 Rust 1.94 或更高版本。本 crate 没有可选的运行时 feature。

## 快速开始

新建二进制 crate，添加上述依赖，将以下代码写入 `src/main.rs`。应用先注册服务
提供者并设置默认选择，再解析和创建服务。运行 `cargo run`，输出为 `Hello, Rust!`。

<!-- spi-example: quick-start; file: app/src/main.rs -->
```rust
use qubit_spi::{ProviderDescriptor, ProviderMetadata, ProviderRegistry, ProviderSelection};
use qubit_spi::{ServiceProvider, ServiceSpec, SyncServiceSpec, provider_descriptor};
use qubit_spi::error::ProviderFailure;

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = std::io::Error; }
impl SyncServiceSpec for Greeting { type Output = String; }
struct Friendly;
impl ProviderMetadata for Friendly {
    fn descriptor(&self) -> ProviderDescriptor { provider_descriptor!("friendly") }
}
impl ServiceProvider<Greeting> for Friendly {
    fn create_configured(&self, name: &String) -> Result<String, ProviderFailure<std::io::Error>> {
        Ok(format!("Hello, {name}!"))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ProviderRegistry::<Greeting>::default();
    registry.register(Friendly)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    let greeter = registry.resolve()?;
    println!("{}", greeter.create_configured(&"Rust".to_owned())?);
    Ok(())
}
```

## 何时使用

当多个库需要共享应用选定的实现，或部署环境需要在多个后端之间选择时，可以使用
注册表。如果只有一个固定实现且无需选择，直接调用构造函数即可。SPI 负责注册、
选择和创建；业务接口、资源缓存及服务提供者的专用配置校验仍由领域 crate 负责。

## 核心能力

- 用类型区分服务族，避免注册不相关的服务提供者。
- 通过规范 ID 和归一化别名进行具名选择、有序链选择或自动选择。
- 用明确的回退策略区分后端缺席、配置错误与初始化错误。
- 同步与异步注册表提供相同的目录操作；异步创建返回可跨线程传递的 future，不绑定执行器。
- 注册表的克隆共享运行时变更；解析后的候选快照保留当时的身份、顺序和策略。
- 类型化错误保留实际调用记录和领域诊断。每次创建都会调用工厂，但工厂可以复用已有资源。

## 延伸阅读

- [中文用户指南](doc/user_guide.zh_CN.md)：三个库与应用的完整集成、配置、回退和排障。
- [English User Guide](doc/user_guide.md)。
- [中文设计说明](doc/design.zh_CN.md)与 [English Design](doc/design.md)：行为契约与实现决策。
- [API 文档](https://docs.rs/qubit-spi)。
- [English README](README.md)。

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
