# Qubit SPI 用户手册

本手册先解释 Qubit SPI 解决的问题以及模型边界，再通过一个小示例创建服务，将它扩展
为真实的多 Provider 场景，最后详细解释公共 API。

## 为什么需要 Qubit SPI

应用真正依赖的通常是某种能力，而不是某一个具体实现。MIME 子系统需要检测媒体类型，
但最合适的实现会随部署环境变化：某个环境安装了训练模型，另一个环境提供系统命令，
受限环境则可能只能使用内置回退实现。

应用仍然只需要一个稳定的 `MimeDetector` 接口。发生变化的是如何构造这个接口，以及
当前环境能够使用哪个实现。Qubit SPI 正是为了解决这一区别而存在。

如果没有统一模型，每个服务族通常都会逐渐形成自己的配置解析器、工厂映射、priority
规则、回退循环和错误格式。一开始可能只是一个 `match`，但很快就难以回答生产环境中的
基本问题：

- 请求的是哪个实现，最后真正胜出的是哪个实现？
- 某个实现被跳过是预期行为，还是系统缺陷？
- 哪些失败允许回退，哪些失败必须终止解析？
- 不同服务族的 alias、priority 和尝试顺序是否遵循同一套规则？

Qubit SPI 将这些决策放进一套类型安全且显式的生命周期中。

## 它解决的问题

这个 crate 将手写选择逻辑中经常混在一起的职责分离开：

| 职责 | Qubit SPI 中的承担者 |
| --- | --- |
| 业务操作 | 应用定义的 Service trait，例如 `MimeDetector` |
| 构造输入与输出类型 | `ServiceSpec` |
| 构造一种实现 | `ServiceProvider::create` |
| canonical 名称、alias 和 priority | `ProviderDescriptor` |
| 启动装配与冲突检查 | `ProviderRegistryBuilder` |
| 不可变查询目录 | `ProviderRegistry` |
| 候选顺序、回退与诊断 | `ProviderResolver` |

它不会带来自动依赖注入。应用仍然需要决定注册哪些 Provider、显式构建 Registry、提供
构造配置，并决定何时创建服务。Rust 会检查同一服务族中的所有 Provider 是否接收相同
配置类型并返回相同的完整服务类型。

## 适用与不适用场景

同时满足以下条件时，适合使用 Qubit SPI：

- 一种应用能力存在两个或更多可互换实现；
- 需要根据配置、环境、偏好顺序或可用性选择实现；
- 构造过程可能以不同方式失败，而且这些失败需要不同回退行为；
- 调用方需要确定性的选择结果和结构化诊断。

典型服务族包括 MIME 检测器、文件系统、序列化器、密码学引擎、模型后端和平台适配器。

不要仅仅为了包装一个实现而引入这个 crate。它也不会加载动态库、从文件系统发现代码、
管理任意对象图或缓存已经创建的服务。Provider 的发现与注册仍然是应用的显式职责。

## 先建立心智模型

从业务能力出发，逐层向外理解：

| 角色 | 第一性原理含义 |
| --- | --- |
| Service | 可复用的能力，其方法负责处理业务请求。 |
| Provider | 知道如何构造一种 Service 实现的工厂。 |
| Config | 路径、endpoint、凭据、默认值等构造期输入。 |
| Output | 工厂返回的完整 Service 值或句柄。 |
| Descriptor | 用来标识工厂并决定排序的注册元数据。 |
| Registry | 在启动阶段装配完成的不可变工厂目录。 |
| Resolver | 选择工厂并调用 `create` 的策略执行者。 |
| CreatedService | 可用 Service 以及创建它的工厂 canonical ID。 |

完整生命周期如下：

```text
定义 Service 能力
  -> 用 ServiceSpec 绑定 Config 和 Output
  -> 为每个后端实现一个 ServiceProvider 工厂
  -> 在启动阶段注册工厂及其元数据
  -> 使用 named / auto / chain 选择候选者
  -> 调用 Provider::create(config)
  -> 保存返回的 Service，并调用它的业务方法
```

最重要的边界是：`create` 负责构造 Service，而不是执行一次业务操作。在 MIME 示例中，
数据库路径和默认类型属于构造配置；文件名及其字节属于之后的 `detect` 调用。

## 五分钟上手示例

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

本手册适用于 `qubit-spi` 0.8，该版本要求 Rust 1.94 或更高版本。上面的示例注册了
一个 Provider 工厂，创建基于扩展名的检测器，然后用这个检测器识别 PNG 文件。返回
结果同时包含服务句柄和实际胜出的 Provider ID。

运行示例前，在应用中添加依赖：

```toml
[dependencies]
qubit-spi = "0.8"
```

## 理解第一个示例

### 1. 定义 Service 能力

`MimeDetector` 是业务代码真正需要的接口。它的 `detect` 方法每次处理一个文件请求。
Resolver 和 Provider 都不会代替应用执行这个业务操作。

### 2. 分离构造配置

`MimeConfig` 保存构造检测器时需要的默认媒体类型。它不包含 `file_name` 或 `content`，
因为这些值会在检测器创建完成后随每次业务调用变化。

### 3. 绑定 Provider 契约

`MimeDetectorSpec` 将 `MimeConfig` 与 `Arc<dyn MimeDetector>` 绑定起来。因此 Rust
要求所有 `ServiceProvider<MimeDetectorSpec>` 都接收 `&MimeConfig`，并返回相同的、
完整且可共享的检测器句柄。

### 4. 实现工厂

`ExtensionProvider::create` 校验构造配置并创建 `ExtensionDetector`。无法构造时，
`ProviderError` 说明原因；构造成功时返回的是 Service 本身，而不是某次检测结果。

### 5. 注册并选择工厂

`ProviderDescriptor` 为工厂指定 canonical ID `extension`。
`ProviderRegistry::builder()` 在启动阶段收集工厂，`build()` 冻结目录。
`create_named("extension", &config)` 只选择这个工厂并调用 `create`。

### 6. 使用已经创建的 Service

返回的 `CreatedService` 通过 `service()` 提供检测器，通过 `provider_id()` 提供胜出的
canonical ID。此时业务代码才调用 `detect("photo.png", ...)`。同一个检测器还可以
处理之后的其他文件，不需要再次解析 Provider。

## 带详细注释的完整示例

下面的程序建立一个真实的 MIME 检测服务族，包含两个 Provider 工厂、alias、priority、
三种选择方式、回退和结构化诊断。为了保持示例自包含，生产环境中的 Magic Provider
会加载真实数据库，而这里仅保留足以说明构造边界的行为。请按顺序阅读注释；每条注释
都会说明对应部分存在的原因及其运行时行为。

```rust
use std::{
    path::PathBuf,
    sync::Arc,
};

use qubit_spi::error::{AttemptFailure, ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ResolutionTermination, ServiceProvider, ServiceSpec,
};

/*
 * 这个 trait 是面向应用的 Service。detect() 在构造完成后处理不断变化的业务输入。
 * SPI 返回 Arc 后，应用无需知道具体实现，就能保存并共享已经选出的检测器。
 */
trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

/*
 * Config 只包含构造检测器需要的值。某个文件的名称和字节不属于这里；它们会在之后
 * 调用 detect() 时传入。
 */
struct MimeConfig {
    default_type: String,
    magic_database: Option<PathBuf>,
}

struct MimeDetectorSpec;

/*
 * ServiceSpec 是所有 Provider 工厂共同遵守的编译期契约。两个工厂都必须接收
 * MimeConfig 并返回相同的完整 Service 句柄，因此切换实现不会改变业务代码看到的类型。
 */
impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}

struct MagicDatabaseDetector {
    _database: PathBuf,
    default_type: String,
}

/*
 * 创建完成的 Service 可以反复执行 MIME 检测。为了不依赖外部数据库，这个精简实现
 * 只识别一种签名。真实实现会保留并查询 create() 阶段加载的数据库。
 */
impl MimeDetector for MagicDatabaseDetector {
    fn detect(&self, _file_name: &str, content: &[u8]) -> &str {
        if content.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

/*
 * 这个回退 Service 使用文件名而不是内容数据库。它仍然实现同一个 MimeDetector 契约，
 * 因此业务代码不需要知道检测器由哪个 Provider 创建。
 */
struct ExtensionDetector {
    default_type: String,
}

impl MimeDetector for ExtensionDetector {
    fn detect(&self, file_name: &str, _content: &[u8]) -> &str {
        if file_name.to_ascii_lowercase().ends_with(".png") {
            "image/png"
        } else {
            &self.default_type
        }
    }
}

/*
 * Provider 类型是工厂，不是 Service，也不是注册身份。MagicDatabaseProvider 只负责
 * 根据共享的初始化配置构造一个可用的 MagicDatabaseDetector。
 */
struct MagicDatabaseProvider;

impl ServiceProvider<MimeDetectorSpec> for MagicDatabaseProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        /*
         * 没有数据库意味着该后端无法在当前部署环境运行。Unavailable 告诉
         * OnAbsence 可以继续尝试其他 Provider。
         */
        let Some(database) = &config.magic_database else {
            return Err(ProviderError::unavailable(
                "no magic database is configured",
            ));
        };

        /*
         * 已配置但格式错误的路径属于调用方配置错误，不是环境缺失。OnAbsence 必须停止，
         * 不能用另一个后端掩盖这个错误。
         */
        if database.extension().and_then(|value| value.to_str()) != Some("mgc") {
            return Err(ProviderError::invalid_configuration(
                "magic_database must point to an .mgc file",
            ));
        }
        if config.default_type.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "default_type must not be empty",
            ));
        }
        Ok(Arc::new(MagicDatabaseDetector {
            _database: database.clone(),
            default_type: config.default_type.clone(),
        }))
    }
}

struct ExtensionProvider;

impl ServiceProvider<MimeDetectorSpec> for ExtensionProvider {
    fn create(
        &self,
        config: &MimeConfig,
    ) -> Result<Arc<dyn MimeDetector>, ProviderError> {
        /*
         * 这个工厂创建完整的回退 Service。它只校验构造配置；具体文件的处理仍由
         * detect() 完成。
         */
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

fn build_resolver() -> Result<ProviderResolver<MimeDetectorSpec>, Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();

    /*
     * canonical ID 是稳定的可观察性身份，alias 是可接受的配置名称。priority 100
     * 让 magic 成为自动选择的第一候选项，但不会改变调用方控制的具名或链式顺序。
     */
    builder.register(
        ProviderDescriptor::new(ProviderId::new("magic")?)
            .with_aliases(["content", "libmagic"])?
            .with_priority(100),
        MagicDatabaseProvider,
    )?;
    builder.register(
        ProviderDescriptor::new(ProviderId::new("extension")?)
            .with_aliases(["filename", "suffix"])?
            .with_priority(10),
        ExtensionProvider,
    )?;

    /*
     * build() 结束可变的启动装配阶段。Resolver 共享得到的不可变 Registry，并在运行时
     * 应用一条明确的回退策略。OnAbsence 允许不可用后端回退，同时保护无效配置和意外
     * 初始化失败。
     */
    Ok(ProviderResolver::new(
        builder.build(),
        FallbackPolicy::OnAbsence,
    ))
}

/*
 * ResolutionError 提供结构化诊断。依据这些值分支既稳定又可测试；解析 Display 文本
 * 会让程序依赖原本只面向读者的措辞。
 */
fn report_resolution_error(error: &ResolutionError) {
    match error.termination() {
        Some(ResolutionTermination::Exhausted) => {
            eprintln!("all admitted candidates were exhausted");
        }
        Some(ResolutionTermination::StoppedByPolicy) => {
            eprintln!("fallback policy stopped candidate traversal");
        }
        Some(_) => eprintln!("resolution ended for a newer reason"),
        None => eprintln!("resolution failed before candidate traversal"),
    }

    for (index, attempt) in error.attempts().iter().enumerate() {
        match attempt {
            AttemptFailure::UnknownProvider {
                requested_selector, ..
            } => eprintln!("attempt {index}: unknown selector {requested_selector}"),
            AttemptFailure::ProviderError {
                requested_selector,
                provider_id,
                error,
                ..
            } => eprintln!(
                "attempt {index}: selector {requested_selector:?} reached {provider_id}, \
                 which failed with {:?}: {}",
                error.kind(),
                error.reason(),
            ),
            _ => eprintln!("attempt {index}: newer failure kind"),
        }
    }

    match error.decisive_attempt() {
        Some(attempt) => eprintln!("decisive attempt: {attempt}"),
        None => eprintln!("no single attempt explains the whole outcome"),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = build_resolver()?;
    let config = MimeConfig {
        default_type: "application/octet-stream".to_owned(),
        magic_database: None,
    };
    let png_header = b"\x89PNG\r\n\x1a\n";

    /*
     * 自动选择遵循 priority 顺序。magic 首先被尝试，但缺少数据库会产生 Unavailable，
     * 因此 OnAbsence 会到达 extension。返回值包含可复用检测器和胜出的 canonical ID。
     */
    let automatic = resolver.create_auto(&config)?;
    assert_eq!("extension", automatic.provider_id().as_str());
    assert_eq!(
        "image/png",
        automatic.service().detect("photo.png", png_header),
    );

    /*
     * 具名选择只解析一个 canonical ID 或 alias。filename 映射到 extension。
     * create_named 只构造这个 Service，不会回退到 magic；detect() 是另一个业务操作。
     */
    let named = resolver.create_named("filename", &config)?;
    assert_eq!("extension", named.provider_id().as_str());
    assert_eq!(
        "application/octet-stream",
        named.service().detect("README", b"plain text"),
    );

    /*
     * 链式选择保留调用方顺序。missing 被记录为未知 selector，content 到达不可用的
     * magic，最后 suffix 创建 extension Service。同一 Provider 的 alias 会去重。
     */
    let chained = resolver.create_chain(["missing", "content", "suffix"], &config)?;
    assert_eq!("extension", chained.provider_id().as_str());

    /*
     * 第二次构造请求故意失败。magic 仍然不可用，随后 extension 拒绝空的构造默认值。
     * 因为无效配置不属于缺失，OnAbsence 会停止遍历。
     */
    let invalid_config = MimeConfig {
        default_type: "  ".to_owned(),
        magic_database: None,
    };
    let failure = resolver
        .create_auto(&invalid_config)
        .err()
        .expect("the invalid configuration must fail");
    assert_eq!(
        Some(ResolutionTermination::StoppedByPolicy),
        failure.termination(),
    );
    report_resolution_error(&failure);

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("fatal error: {error}");
        std::process::exit(1);
    }
}
```

没有配置 magic 数据库时，自动选择先到达 `magic`，在收到 `Unavailable` 后继续，
并返回 `extension`。具名选择与链式选择也会创建 extension 检测器。最后一次故意无效
的构造在 `ExtensionProvider` 处终止，并运行结构化诊断函数。每次成功的 Resolver
调用都会创建一个新检测器；随后示例才在返回的 Service 上调用业务方法。

## 服务契约

当需要引入一组可独立配置的 Provider 实现时，定义一个服务族。

以下片段使用完整示例中已经定义的类型：

```rust,ignore
trait MimeDetector: Send + Sync {
    fn detect(&self, file_name: &str, content: &[u8]) -> &str;
}

struct MimeConfig {
    default_type: String,
    magic_database: Option<PathBuf>,
}

struct MimeDetectorSpec;

impl ServiceSpec for MimeDetectorSpec {
    type Config = MimeConfig;
    type Output = Arc<dyn MimeDetector>;
}
```

可观察到的结果是一条编译期契约：每个 `ServiceProvider<MimeDetectorSpec>` 都接收
`&MimeConfig` 并返回 `Arc<dyn MimeDetector>`。

`Config` 可以是 unsized 类型，因此服务可以使用 `str` 或 trait object 等视图。
`Output` 是调用方最终持有的完整值；应根据应用的所有权和并发要求选择普通值、
`Box<dyn Trait>`、`Arc<dyn Trait>` 或其他句柄。对于 Service Provider 服务族，
`Output` 通常应该是完整的可复用 Service 或其句柄，而不是某个业务方法的一次执行
结果。SPI 不会自动添加或移除包装。

**常见误区：**为互不相关的服务定义一个过于宽泛的 specification。当配置、输出、
Provider 集合或选择策略需要独立演进时，应使用不同的标记类型。

## create 到底做什么

`ServiceProvider::create` 是“选择工厂”和“使用工厂构造出的 Service”之间的边界。
它接收借用的构造配置，并且必须返回一个已经可以处理业务调用的完整 `S::Output`。

以下片段来自完整示例：

```rust,ignore
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
```

工厂可以校验 Provider 专用配置、检查所需命令或模型是否可用、初始化客户端或引擎，
并把具体实现包装成输出句柄。它不应代替 `MimeDetector::detect` 处理某个文件；单个文件
的值不是构造配置。

只要遍历到该 Provider，Resolver 就会调用这个方法。具名调用最多调用一个工厂；自动
选择和链式选择可能依次调用多个工厂，直到一个成功或策略停止。再次调用
`create_auto`、`create_named`、`create_chain` 或 `create` 会重新解析，并且可能创建
另一个 Service；Qubit SPI 不会缓存输出。

`create` 是同步方法。需要异步网络初始化的 Provider 通常应创建支持异步操作的惰性
客户端，有意识地完成不可避免的同步初始化，或者把异步初始化放到这个接口之外。在
`create` 中隐藏长时间 I/O 会让解析过程发生意外阻塞。

Provider 实现必须满足 `Send + Sync + 'static`，因为 Registry 会保留并共享工厂本身。
配置以借用方式传入，每次工厂调用成功时都会返回一个新的完整输出。

错误分类会直接控制回退，因此应按真实含义选择：

| `ProviderError` 构造器 | 含义 | `OnAbsence` |
| --- | --- | --- |
| `unsupported` | 该 Provider 无法构造请求的能力或配置。 | 继续 |
| `unavailable` | 该 Provider 无法在当前环境运行。 | 继续 |
| `invalid_configuration` | 调用方提供了无效构造配置。 | 停止 |
| `initialization_failed` | 构造该实现时发生意外失败。 | 停止 |

每种分类还有对应的 `_with_source` 构造器，可保留底层的
`Error + Send + Sync + 'static`。

**常见误区：**把无效配置报告成 `Unavailable`。这会允许 `OnAbsence` 静默选择其他
Provider，从而掩盖调用方错误。

## Provider 身份与排序

当需要为一次工厂注册指定稳定身份、可接受的配置名称以及自动选择顺序时，使用
descriptor。

以下片段来自完整示例：

```rust,ignore
let magic = ProviderDescriptor::new(ProviderId::new("magic")?)
    .with_aliases(["content", "libmagic"])?
    .with_priority(100);
```

该 descriptor 将 `magic` 设为 canonical ID，接受 `content` 和 `libmagic` 作为
alias，并为自动选择设置 priority 100。

canonical `ProviderId` 是严格的小写 ASCII token：首尾必须是 ASCII 字母或数字，
中间还可以包含 `-`、`_`、`.` 和 `+`。`ProviderId::new` 不会修剪或规范化输入。
运行时 `ProviderSelector` 则不同：它会先修剪空白并把 ASCII 字母转为小写，再执行
校验，因此 `" LIBMAGIC "` 可以解析 alias `libmagic`。

alias 与 canonical ID 共享同一个 selector 命名空间。descriptor 会拒绝无效 alias、
与自身 ID 相同的 alias 以及重复 alias；Builder 会拒绝已被其他注册占用的 selector。
priority 只影响 `create_auto`，具名选择和链式选择遵循调用方给出的 selector 或顺序。

无效 canonical ID 返回 `ProviderIdError`；无效或重复 alias 返回
`ProviderDescriptorError`。

**常见误区：**把 alias 当作 Provider 身份。即使请求使用 alias，结果和诊断仍然报告
canonical ID。

## 构建 Registry

当需要在应用启动阶段装配所有可用工厂，或在之后检查不可变目录时，构建 Registry。

以下片段使用完整示例中的类型：

```rust,ignore
let shared_magic: Arc<dyn ServiceProvider<MimeDetectorSpec>> =
    Arc::new(MagicDatabaseProvider);
let mut builder = ProviderRegistry::<MimeDetectorSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("extension")?),
    ExtensionProvider,
)?;
builder.register_shared(
    ProviderDescriptor::new(ProviderId::new("magic")?),
    shared_magic,
)?;

let registry = builder.build();
assert_eq!(2, registry.len());
assert!(!registry.is_empty());
for descriptor in registry.descriptors() {
    println!("{}", descriptor.id());
}
```

`register` 将拥有所有权的具体 Provider 移入 Registry 存储；`register_shared` 接收
已有的 `Arc<dyn ServiceProvider<S>>`。注册具有事务性：只有在所有 canonical ID 与
alias 冲突检查通过后才会修改 Builder，因此被拒绝的注册不会占用部分 selector。
冲突会报告为 `RegistrationError`。

`build()` 消费可变 Builder，并准备不可变的查询索引与自动排序索引。运行时检查均为
只读操作：

- `len()` 和 `is_empty()` 返回目录大小；
- `descriptors()` 和 `provider_ids()` 按注册顺序迭代；
- `find(raw)` 对无效输入和未知输入都返回 `None`；
- `resolve(raw)` 返回 `ResolvedProvider`，或者通过 `ResolutionError` 区分无效输入与
  未知输入。

`ResolvedProvider` 借用 Registry 中的一条 entry。`descriptor()` 暴露注册元数据，
`create(config)` 直接调用该工厂。直接创建返回 `ProviderError`，并且有意绕过 Resolver
回退和 `CreatedService` 的胜出者记录。crate 不提供全局 Registry 或隐式发现。

**常见误区：**把 Builder 保留为可变的运行时状态。应在启动阶段完成注册，只构建
一次，然后共享 Registry 或 Resolver。

## 选择 Provider

当需要决定由配置指定一个 Provider、选取当前最佳 Provider，或按偏好列表尝试时，
选择对应模式。

以下片段来自完整示例：

```rust,ignore
let automatic = resolver.create_auto(&config)?;
let named = resolver.create_named("filename", &config)?;
let chained = resolver.create_chain(["missing", "content", "suffix"], &config)?;
```

| 需求 | Resolver 原始输入方法 | 候选顺序 |
| --- | --- | --- |
| 当前最佳实现 | `create_auto` | priority 降序，然后 canonical ID 升序 |
| 配置指定的唯一实现 | `create_named` | 该 canonical ID 或 alias 对应的一个 Provider |
| 有序偏好列表 | `create_chain` | 调用方给出的 selector 顺序 |

具名选择不会尝试第二个 Provider。对空 Registry 自动选择会返回
`ResolutionError::EmptyRegistry`。chain 不能为空，而且所有 selector 都会在调用
任何 Provider 之前完成校验。未知但有效的 selector 会按顺序记录为 attempt。如果
chain 中两个 selector 是同一 Provider 的 alias，该 Provider 只会被调用一次。

实际胜出者始终可以通过 `CreatedService::provider_id()` 获取。

**常见误区：**期望 priority 对 chain 重新排序。priority 只用于自动选择；chain
始终保留调用方顺序。

### 复用已校验的选择

当同一个配置选择会用于多次创建调用时，使用 `ProviderSelection`。以下聚焦片段省略了
完整示例中已经构建的 Resolver 和 MIME 配置：

```rust,ignore
use qubit_spi::ProviderSelection;

let selection = ProviderSelection::chain(["content", "suffix"])?;

let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

`ProviderSelection::auto()` 不会失败。`named(...)` 会规范化并校验一个 selector；
`chain(...)` 会校验所有 selector、保留顺序并拒绝空 chain。之后可以通过
`ProviderResolver::create` 复用这个已校验值。`Default` 是自动选择；`kind()` 返回
模式，`selector()` 在具名选择中借用 selector，`selectors()` 返回 chain，并在其他
模式下返回空 slice。

在运行时输入边界，更适合直接使用 `create_named` 和 `create_chain`：它们会把解析失败
转换为 `ResolutionError`，同时保留无效输入和 chain 索引。但这些方法每次都会解析并
分配拥有所有权的 selector 数据。复用 `ProviderSelection` 可以把这项工作移到配置
加载阶段，并以 `ProviderSelectionError` 报告校验失败。

复用 selection 只会避免重复解析名称，不会缓存已经创建的 Service。`first` 和
`second` 都会重新解析，并且可能调用 Provider 工厂。

## 回退与错误分类

当需要决定哪些 Provider 失败可以通过尝试后续候选项来隐藏时，选择回退策略。

以下片段使用完整示例中的类型：

```rust,ignore
let safe_resolver = ProviderResolver::new(registry.clone(), FallbackPolicy::OnAbsence);
let best_effort_resolver = ProviderResolver::new(registry, FallbackPolicy::OnAnyError);
```

| 策略 | 在哪些错误后继续 | 在哪些错误后停止 |
| --- | --- | --- |
| `OnAbsence` | `Unsupported`、`Unavailable` | `InvalidConfiguration`、`InitializationFailed` |
| `OnAnyError` | 所有 `ProviderError` | 不会因 Provider 错误分类而停止 |

`OnAbsence` 同时也是 `FallbackPolicy::default()` 的值。

回退只覆盖构造 Service 时返回的失败。一旦 Provider 已经返回
`Arc<dyn MimeDetector>`，之后 `detect` 调用产生的错误或结果就属于该 Service 的 API；
Resolver 不会因为业务操作失败而重新遍历 Provider chain。

chain 中的未知 selector 会被记录后继续，因为此时没有调用任何 Provider。只有存在后续
候选项时回退策略才有实际作用；具名选择仍然只有一个候选项。策略提前停止会产生
`ResolutionTermination::StoppedByPolicy`；访问完所有允许的候选项会产生
`ResolutionTermination::Exhausted`。

**常见误区：**仅仅为了让请求成功而选择 `OnAnyError`。该策略可能掩盖配置和初始化
缺陷，只应在明确的尽力而为流程中使用。

## 成功结果与失败诊断

### 检查成功结果

当需要消费输出或记录实际胜出的 Provider 时，检查 `CreatedService`。

以下片段来自完整示例：

```rust,ignore
let created = resolver.create_named("filename", &config)?;
println!("winner: {}", created.provider_id());
let media_type = created.service().detect("photo.png", png_header);

let (provider_id, service) = created.into_parts();
```

`provider_id()` 和 `service()` 分别借用两个值。`into_service()` 消费包装并只返回输出；
`into_parts()` 消费包装并返回 `(ProviderId, Output)`。返回的 ID 始终是 canonical ID，
因此可观察性不会依赖调用方使用了哪个 alias。

**常见误区：**在记录日志或指标之前丢弃胜出者 ID。它是确认生产环境回退行为最直接
的信息。

### 诊断失败

当解析无法创建服务，并且调用方需要稳定、结构化的解释时，检查
`ResolutionError`。

以下片段来自完整示例：

```rust,ignore
match error.termination() {
    Some(ResolutionTermination::Exhausted) => println!("all candidates failed"),
    Some(ResolutionTermination::StoppedByPolicy) => println!("policy stopped"),
    Some(_) => println!("newer termination reason"),
    None => println!("failure occurred before traversal"),
}

for attempt in error.attempts() {
    println!("{attempt}");
}
```

直接返回的 `ResolutionError` variant 区分以下边界：

| Variant | 含义 |
| --- | --- |
| `InvalidSelector` | 原始输入未通过规范化或语法校验；适用时包含 chain 索引。 |
| `EmptySelection` | 原始或已校验的 chain 不包含 selector。 |
| `UnknownProvider` | 有效的直接具名 selector 没有匹配任何注册。 |
| `EmptyRegistry` | 对空 Registry 发起了自动选择。 |
| `NoProviderSucceeded` | 候选尝试失败，或策略终止了遍历。 |

对于聚合失败，`attempts()` 按顺序返回 `AttemptFailure`。每条 attempt 要么是未知
selector，要么是 Provider 错误；后者保留请求 selector、canonical Provider ID、
错误分类、原因和可选 source。`termination()` 区分候选耗尽与策略停止，
`terminal_attempt()` 返回最后一条记录。

`decisive_attempt()` 在策略停止后返回最后一条 attempt，在只有一次尝试的耗尽结果中
返回唯一 attempt。对于多次尝试共同导致的耗尽，它返回 `None`，因为不存在能够单独
解释整体结果的失败。只有未知、不支持和不可用结果才会让 `is_absence()` 返回 true。

标准 `Error::source()` 链会暴露 selector 解析错误或无歧义的 decisive attempt；
Provider attempt 继续暴露其 `ProviderError`，而 `_with_source` 构造器会保留更底层
原因。所有公开错误枚举都是 non-exhaustive，因此下游 match 必须保留通配分支。控制
流程应使用字段和访问器；`Display` 文本只面向读者，不应被解析。

**常见误区：**把最后一条 attempt 当作所有耗尽 chain 的根因。多次尝试耗尽时，应
检查完整的有序 attempt 列表。

## 生命周期、共享与性能

`ProviderRegistry` 和 `ProviderResolver` 都可以低成本克隆。Registry 将不可变 entry
和索引存放在内部 `Arc` 后面，克隆 Registry 不会复制 Provider。克隆 Resolver 会共享
同一 Registry，并复制很小的回退策略值。`registry()` 和 `fallback_policy()` 提供对
Resolver 配置的只读访问。

这种共享只覆盖 Provider 工厂和注册元数据，不覆盖这些工厂创建出的 Service。每次
Resolver 创建调用都会开始新的遍历，并且可能再次调用 `ServiceProvider::create`。这个
crate 不提供 singleton scope、memoization 或输出缓存。

对于构造成本较高的检测器、客户端、引擎或连接池，应在启动阶段解析一次，保存返回的
`Arc<dyn MimeDetector>`，并为使用方克隆这个输出句柄。只有确实需要新的 Service
实例，或者构造配置发生变化时，才应该反复调用 Resolver。

Provider 满足 `Send + Sync + 'static`，因此目录可以共享。SPI 没有为
`ServiceSpec::Output` 添加 `Send` 或 `Sync` 约束；如果创建出的服务本身需要跨线程，
应选择 `Arc<dyn Trait + Send + Sync>` 等线程安全输出。本手册中的
`MimeDetector: Send + Sync` 使 `Arc<dyn MimeDetector>` 适合这种用途。

原始 selector 解析会规范化并持有 selector 文本，使错误和 selection 能安全地保留
它。该分配成本重要时应复用 `ProviderSelection`。Registry 查询和自动顺序使用
`build()` 时一次性准备的索引。

## 推荐实践

- 在启动阶段只装配一次 Provider，并让注册错误直接导致启动失败。
- `Config` 只保存构造输入；每次操作的请求应传给 Service 方法。
- `create` 返回完整 Service 句柄；如果构造成本较高，应保存并共享这个句柄。
- 保持 canonical ID 稳定；使用 alias 接受旧名称或更友好的名称。
- 只有自动选择偏好具有实际产品含义时才设置明确 priority。
- 优先使用 `OnAbsence`；仅为已有文档说明的尽力而为行为采用 `OnAnyError`。
- 在热路径之前把可复用配置校验为 `ProviderSelection`。
- 成功时记录实际胜出的 canonical Provider ID。
- 匹配结构化错误并保留 source chain，不要解析错误消息。
- 使用小型、目的明确的 Registry 测试具名、自动、链式、策略停止和候选耗尽行为。

## 故障排查

| 现象 | 原因 | 处理方式 |
| --- | --- | --- |
| `ProviderId::new` 拒绝名称 | canonical ID 不会被修剪或转为小写，并且必须符合小写 ASCII token 语法。 | 在决定稳定 canonical ID 前规范化配置，再传入有效 token。 |
| 注册报告 selector 冲突 | canonical ID 或 alias 已由其他 Provider 占用。 | 检查两个 descriptor，重命名或移除重叠 selector；失败注册没有改变 Builder。 |
| `create_auto` 返回 `EmptyRegistry` | `build()` 前没有注册任何 Provider。 | 至少注册一个 Provider，或在解析前把该服务族作为可选项处理。 |
| chain 为空或包含无效项 | 空 chain 会被拒绝，且遍历前会校验所有 selector。 | 在启动阶段校验配置，并使用错误中的 selector 索引和原始输入定位问题。 |
| 遍历比预期更早停止 | `OnAbsence` 遇到了 `InvalidConfiguration` 或 `InitializationFailed`。 | 修复输入或初始化问题；只有明确需要掩盖它时才使用 `OnAnyError`。 |
| 高成本初始化反复执行 | 每次 Resolver 创建调用都会再次调用 Provider 工厂；输出不会被缓存。 | 在启动阶段解析一次，并保存或克隆返回的 Service 句柄。 |
| Service 方法失败，但没有尝试另一个 Provider | Provider 成功创建 Service 后，回退过程已经结束。 | 在 Service API 中处理业务操作错误；只有应用确实需要新 Service 时才再次显式解析。 |
| `decisive_attempt()` 返回 `None` | 错误不是聚合错误，或多条已耗尽 attempt 共同解释结果。 | 检查 `attempts()`、`termination()` 和 `terminal_attempt()`，不要假定只有一个原因。 |
| `find` 返回 `None`，但原因不明确 | `find` 有意合并无效输入和未知输入。 | 需要结构化区分时使用 `resolve`。 |
| 请求使用 alias，但结果报告了另一个 ID | 成功结果和 Provider 失败都报告 canonical 身份。 | 日志使用 canonical ID，把 alias 只当作可接受输入。 |

## API 参考

| 职责 | API |
| --- | --- |
| 绑定配置和输出类型 | [`ServiceSpec`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/trait.ServiceSpec.html) |
| 实现工厂 | [`ServiceProvider`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/trait.ServiceProvider.html) |
| 表示 canonical 名称和运行时查询名称 | [`ProviderId`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderId.html)、[`ProviderSelector`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderSelector.html) |
| 定义身份、alias 和 priority | [`ProviderDescriptor`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderDescriptor.html) |
| 装配注册信息 | [`ProviderRegistryBuilder`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderRegistryBuilder.html) |
| 检查并解析不可变目录 | [`ProviderRegistry`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderRegistry.html) |
| 直接使用一个已解析工厂 | [`ResolvedProvider`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ResolvedProvider.html) |
| 保存可复用的已校验选择 | [`ProviderSelection`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderSelection.html) |
| 应用选择与回退 | [`ProviderResolver`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.ProviderResolver.html) |
| 消费输出和胜出者 ID | [`CreatedService`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/struct.CreatedService.html) |
| 选择回退行为 | [`FallbackPolicy`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/enum.FallbackPolicy.html) |
| 解释聚合终止原因 | [`ResolutionTermination`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/enum.ResolutionTermination.html) |
| 检查 attempt 与工厂错误分类 | [`AttemptFailure`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.AttemptFailure.html)、[`ProviderErrorKind`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ProviderErrorKind.html) |
| 分类工厂失败并诊断解析 | [`ProviderError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/struct.ProviderError.html)、[`ResolutionError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ResolutionError.html) |
| 处理校验、注册、Provider 和解析错误 | [`qubit_spi::error`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/index.html) |
