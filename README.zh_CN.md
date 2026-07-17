# Qubit SPI

[![Rust CI](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-spi/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-spi/coverage-badge.json)](https://qubit-ltd.github.io/rs-spi/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-spi.svg?color=blue)](https://crates.io/crates/qubit-spi)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

## 为什么需要这个库

应用真正依赖的通常是某种能力，而不是某一个具体实现。例如，一个 MIME 子系统可能在
模型已经安装时使用模型检测器，在系统命令可用时调用系统命令，否则回退到轻量检测器。

如果没有统一基础设施，每个服务族都要重复编写同一套启动逻辑：解析配置名称、寻找工厂、
排列候选实现、判断哪些错误允许回退、创建服务，并保留足够的上下文解释最终结果。这些
手写分支很容易出现不一致，也很难诊断。

Qubit SPI 用一套类型安全、显式装配的模型统一这个生命周期。它不依赖全局状态，也不会
从容器中按名称查找无类型对象。

## 它提供什么

- 在编译期约束同一服务族中的所有 Provider 使用相同构造配置并返回相同服务类型。
- 在应用启动阶段显式装配、随后不可变的 Registry。
- named、auto 和调用方指定顺序的三种 Provider 选择方式。
- 确定性的 priority 排序、分类后的创建错误、受控回退和结构化尝试诊断。
- 实际成功创建服务的 Provider canonical ID。

## 适用场景

当一种能力存在多个可互换实现，并且应用需要根据配置、运行环境或回退规则选择实现时，
适合使用 Qubit SPI。典型场景包括 MIME 检测器、文件系统、序列化器、模型后端和平台
适配器。

如果服务只有一个实现，通常不需要这个库。它也不是动态库加载器、依赖注入框架或服务
缓存。

## 核心模型

| 角色 | 职责 |
| --- | --- |
| Service | 业务代码可以反复调用的应用能力。 |
| `ServiceProvider` | 根据构造配置创建一种 Service 实现的工厂。 |
| `ServiceSpec` | 将所有 Provider 共用的 `Config` 与完整的 `Output` 服务句柄绑定起来。 |
| `ProviderDescriptor` | 将 canonical ID、alias、priority 与工厂代码分离。 |
| `ProviderRegistry` | 保存已经注册的 Provider 工厂的不可变目录。 |
| `ProviderResolver` | 选择候选 Provider、调用 `create` 并应用回退策略。 |
| `CreatedService` | 返回可用服务以及实际胜出的 Provider canonical ID。 |

最重要的边界是：Provider 负责**创建**服务；返回的 Service 才负责处理业务操作。

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
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ServiceProvider, ServiceSpec,
};

trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

struct MimeConfig {
    default_type: String,
}

struct MimeDetectorSpec;

impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}

struct ExtensionDetector {
    default_type: String,
}

impl MimeDetector for ExtensionDetector {
    fn detect(&self, file_name: &str, _content: &[u8]) -> &str {
        if file_name.ends_with(".png") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

struct ExtensionProvider;

impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(ExtensionDetector {
            default_type: config.default_type.clone(),
        }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();
    builder.register(
        ProviderDescriptor::new(ProviderId::new("extension")?),
        ExtensionProvider,
    )?;

    let resolver = ProviderResolver::new(builder.build(), FallbackPolicy::OnAbsence);
    let config = MimeConfig {
        default_type: "application/octet-stream".to_owned(),
    };
    let created = resolver.create_named("extension", &config)?;

    assert_eq!("extension", created.provider_id().as_str());
    assert_eq!(
        "image/png",
        created.service().detect("photo.png", b"PNG contents"),
    );
    Ok(())
}
```

## 示例如何工作

1. `MimeDetector` 是可复用的服务。文件名和内容字节属于它的 `detect` 业务操作。
2. `MimeConfig` 只保存构造期配置；`MimeDetectorSpec` 要求所有 Provider 都返回
   `Arc<dyn MimeDetector>`。
3. `ExtensionProvider::create` 校验构造配置并创建完整检测器，它本身不检测文件。
4. `ProviderDescriptor` 为工厂指定 canonical ID `extension`，Registry 将它保存到
   不可变目录中。
5. `create_named` 选择 Provider 并调用其工厂。返回的 `CreatedService` 同时提供胜出
   ID 和可用检测器。
6. 创建完成之后，应用才调用 `detect("photo.png", ...)`。

## 常用选择方式

| 需求 | 方法 | 行为 |
| --- | --- | --- |
| 使用配置指定的一个 Provider | `create_named` | 只尝试一个 canonical ID 或 alias，不会回退。 |
| 使用当前最合适的 Provider | `create_auto` | 先按 priority 降序，再按 canonical ID 升序尝试。 |
| 按偏好顺序尝试 | `create_chain` | 按调用方给出的 selector 顺序尝试，并避免通过 alias 重复调用同一个 Provider。 |

每个 Resolver 都有回退策略。`FallbackPolicy::OnAbsence` 是更安全的默认选择：它会在
Provider 不支持或暂时不可用时继续，但会在配置错误或初始化错误时停止。只有明确需要
尽力回退时才使用 `OnAnyError`。

Resolver 不会缓存创建结果。如果构造服务的成本较高，应在启动阶段创建一次，然后保存
或克隆返回的 `Arc`。

## 继续学习

- 阅读[用户手册](doc/user_guide.zh_CN.md)，了解完整心智模型，并通过带详细注释的示例
  理解 alias、priority、回退、诊断、生命周期、共享与性能细节。
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
