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
qubit-spi = "0.13"
```

需要 Rust 1.94 或更高版本。默认 feature 集为空。

## 示例：应用选择 Greeter 实现

`lib-foo` 需要 Greeter，但由应用决定使用哪个实现。先看服务接口、消费方、
提供者和应用这四个文件，再看如何运行。

### 1. 服务接口

`lib-greeter` 定义业务接口；应用创建注册表并将其传给消费方，因此启动错误可以正常返回。

<!-- spi-example: three-crates; file: lib-greeter/src/lib.rs -->
```rust
// lib-greeter/src/lib.rs
use std::{
    error::Error,
    fmt,
    sync::Arc,
};

use qubit_spi::{ServiceSpec, SyncServiceSpec};

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

/// Greeter Provider 无法创建 Service 时返回的领域错误。
#[derive(Debug)]
pub struct GreeterError;

impl fmt::Display for GreeterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("greeter provider failed")
    }
}

impl Error for GreeterError {}

/// 向 Qubit SPI 绑定 Greeter 的配置类型和输出类型。
pub struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    // Provider 创建 Greeter 时接收的输入类型。
    type Config = GreeterConfig;
    // 分类后的 Provider failure 所保留的领域错误。
    type Error = GreeterError;
}

impl SyncServiceSpec for GreeterSpec {
    // 创建成功后返回给消费者的 Service 类型。
    type Output = Arc<dyn Greeter>;
}
```

### 2. 独立的消费方

`lib-foo` 解析应用设置的默认选择，再创建服务，不自行决定具体后端。

<!-- spi-example: three-crates; file: lib-foo/src/lib.rs -->
```rust
// lib-foo/src/lib.rs
use lib_greeter::GreeterSpec;
use qubit_spi::{ProviderRegistry, ServiceProvider};

/// 创建 App 选定的默认 Greeter，并打印一条问候语。
pub fn foo(registry: &ProviderRegistry<GreeterSpec>) -> Result<(), Box<dyn std::error::Error>> {
    let provider = registry.resolve()?;
    let greeter = provider.create()?;
    println!("{}", greeter.greet("Rust"));
    Ok(())
}
```

### 3. 第三方服务提供者

`lib-friendly-greeter` 提供元数据和创建逻辑。仅链接这个 crate 不会自动注册服务提供者。

<!-- spi-example: three-crates; file: lib-friendly-greeter/src/lib.rs -->
```rust
// lib-friendly-greeter/src/lib.rs
use std::sync::Arc;

use lib_greeter::{Greeter, GreeterConfig, GreeterError, GreeterSpec};
use qubit_spi::error::ProviderFailure;
use qubit_spi::{provider_descriptor, ProviderDescriptor, ProviderMetadata, ServiceProvider};

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
    ) -> Result<Arc<dyn Greeter>, ProviderFailure<GreeterError>> {
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderMetadata for FriendlyGreeterProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        provider_descriptor!("friendly", priority: 100)
    }
}
```

### 4. 应用负责组装

应用创建注册表、注册提供者并设置默认选择，然后将注册表传给业务库。

<!-- spi-example: three-crates; file: app/src/main.rs -->
```rust
// app/src/main.rs
use lib_foo::foo;
use lib_friendly_greeter::FriendlyGreeterProvider;
use lib_greeter::GreeterSpec;
use qubit_spi::{ProviderRegistry, ProviderSelection};

// 应用创建注册表并传给业务库中的消费方。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ProviderRegistry::<GreeterSpec>::default();
    registry.register(FriendlyGreeterProvider)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    foo(&registry)
}
```

### 运行示例

将四个文件放入 Cargo workspace，运行 `cargo run -p app`，输出为
`Hello, Rust!`。workspace 清单和本地依赖见[用户指南](doc/user_guide.zh_CN.md#运行示例)。

## 使用 `inventory` 简化注册

同一个 Greeter 示例可以从已链接的 crate 发现提供者，`lib-foo` 的代码保持不变。
提供者 crate 提交工厂，应用根据要链接的提供者构建一个注册表。

在 `lib-greeter/src/lib.rs` 中声明收集点：

```
qubit_spi::declare_sync_provider_inventory! {
    pub mod providers {
        spec = crate::GreeterSpec;
    }
}
```

在 `lib-friendly-greeter/src/lib.rs` 中提交工厂：

```
qubit_spi::submit_sync_provider! {
    inventory_entry = lib_greeter::providers::Entry;
    spec = lib_greeter::GreeterSpec;
    provider = FriendlyGreeterProvider;
}
```

在 `app/src/greeter_providers.rs` 中列出要链接的提供者 crate：

```
use lib_friendly_greeter as _;
```

每增加一个 Greeter 实现，就在这个文件里增加一行导入。它类似一份装配清单，
有点像 Spring 的 XML 配置：这里只决定哪些 crate 进入程序；工厂和元数据仍由
提供者实现，默认选择仍由应用设置。仅在 `Cargo.toml` 中声明依赖，不能保证
未被其他代码引用的提供者进入最终程序。

在 `app/src/main.rs` 中引入该文件并构建注册表，替代显式注册示例中的 `register` 调用：

```
// app/src/main.rs
mod greeter_providers;

use lib_foo::foo;
use lib_greeter::providers;
use qubit_spi::ProviderSelection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = providers::build_registry()?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    foo(&registry)
}
```

参与的 crate 启用 `qubit-spi = { version = "0.13", features = ["inventory"] }`。
[用户指南中的完整可运行版本](doc/user_guide.zh_CN.md#链接期发现简化同一个-greeter-示例)
列出了全部文件，并说明构建冲突的处理方式。

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

- [中文用户指南](doc/user_guide.zh_CN.md)：完整的显式注册和 `inventory` Greeter 示例，以及配置、回退与排障。
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
Pull Request 前运行 `./align-ci.sh` 格式化代码，运行 `./ci-check.sh` 对齐 CI 要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-spi](https://github.com/qubit-ltd/rs-spi)
