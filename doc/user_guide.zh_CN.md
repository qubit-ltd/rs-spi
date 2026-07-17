# Qubit SPI 用户手册

本手册先给出能直接运行的程序，再把它扩展为真实示例，最后详细解释公共 API 中的每个
使用决策。

## 从这里开始：五分钟上手

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

本手册适用于 `qubit-spi` 0.8，该版本要求 Rust 1.94 或更高版本。上面的示例构建了
一个只包含单个 Provider 的 Registry，选择 `english`，并同时得到服务值 `"hello"`
与实际胜出的 Provider ID。

运行示例前，在应用中添加依赖：

```toml
[dependencies]
qubit-spi = "0.8"
```

## 理解核心流程

### 1. 定义输入和输出

`impl ServiceSpec for GreetingSpec` 表示这个服务族中的所有 Provider 都接收 `&()`，
并且必须返回 `&'static str`。`GreetingSpec` 只是将这两个类型绑定在一起的标记类型。

### 2. 实现工厂

`EnglishProvider::create` 是工厂操作。它接收 `GreetingSpec` 指定的配置，创建完整的
输出；无法创建时则返回经过分类的 `ProviderError`。

### 3. 指定 Provider 身份

`ProviderDescriptor::new(ProviderId::new("english")?)` 为工厂指定稳定的 canonical
ID。身份属于注册信息，因此 Provider 类型本身不需要知道配置给它的名称、alias 或
priority。

### 4. 装配 Registry

`ProviderRegistry::builder()` 开始可变的启动装配阶段。每次 `register` 都会检查
身份冲突。`build()` 消费 `ProviderRegistryBuilder`，得到用于运行时查询和共享的
不可变 `ProviderRegistry`。

### 5. 解析并创建

`ProviderResolver::new` 将 Registry 与 `FallbackPolicy` 组合起来。
`create_named("english", &())` 会规范化 selector，找到唯一的 Provider，调用它的
工厂并返回结果。

### 6. 使用结果

返回的 `CreatedService` 通过 `service()` 提供输出，通过 `provider_id()` 提供实际
成功的 canonical ID。保留胜出者身份便于记录日志、指标和支持诊断。

完整流程如下：

```text
ServiceSpec -> ServiceProvider -> ProviderDescriptor -> Registry Builder
            -> immutable Registry -> Resolver -> CreatedService
```

## 带详细注释的完整示例

下面的程序加入真实的服务 trait、两个 Provider、alias、priority、三种选择方式、回退
和结构化诊断。请按顺序阅读代码注释；每条注释都说明对应设计存在的原因以及运行时
结果。

```rust
use std::sync::Arc;

use qubit_spi::error::{AttemptFailure, ProviderError, ResolutionError};
use qubit_spi::{
    FallbackPolicy, ProviderDescriptor, ProviderId, ProviderRegistry, ProviderResolver,
    ResolutionTermination, ServiceProvider, ServiceSpec,
};

/*
 * 面向应用的 trait 才是真正有用的服务。SPI 返回 Arc 后，调用方无需知道具体是哪个
 * Provider 创建了实现，就能以较低成本克隆并在线程间共享同一个句柄。
 */
trait Greeter: Send + Sync {
    fn greet(&self) -> String;
}

struct TextGreeter {
    message: String,
}

impl Greeter for TextGreeter {
    fn greet(&self) -> String {
        self.message.clone()
    }
}

/*
 * ServiceSpec 是所有 Provider 共同遵守的编译期契约：每个工厂接收相同配置，并返回
 * 相同且完整的、由调用方持有的服务句柄。
 */
struct GreetingConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = GreetingConfig;
    type Output = Arc<dyn Greeter>;
}

/*
 * Provider 只负责创建服务，不拥有注册身份。名称和排序信息独立于类型后，启动代码便可
 * 复用同一种工厂实现，并按部署环境分别设置元数据。
 */
struct CloudProvider;

impl ServiceProvider<GreetingSpec> for CloudProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        /*
         * Unavailable 表示该 Provider 能处理请求，但现在暂时无法提供服务。因此
         * OnAbsence 可以继续尝试其他 Provider。
         */
        if !config.cloud_available {
            return Err(ProviderError::unavailable(
                "the cloud greeting service is offline",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from cloud", config.prefix),
        }))
    }
}

struct LocalProvider;

impl ServiceProvider<GreetingSpec> for LocalProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        /*
         * InvalidConfiguration 表示调用方输入有误。OnAbsence 会在这里停止，避免继续
         * 尝试其他 Provider 而掩盖错误配置。
         */
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from local", config.prefix),
        }))
    }
}

fn build_resolver() -> Result<ProviderResolver<GreetingSpec>, Box<dyn std::error::Error>> {
    let mut builder = ProviderRegistry::<GreetingSpec>::builder();

    /*
     * canonical ID 是稳定身份，alias 是可接受的输入名称。priority 100 让 cloud 成为
     * 自动选择时的首选；具名选择和链式选择仍然遵循调用方明确给出的顺序。
     */
    builder.register(
        ProviderDescriptor::new(ProviderId::new("cloud")?)
            .with_aliases(["remote"])?
            .with_priority(100),
        CloudProvider,
    )?;
    builder.register(
        ProviderDescriptor::new(ProviderId::new("local")?)
            .with_aliases(["builtin"])?
            .with_priority(10),
        LocalProvider,
    )?;

    /*
     * build() 结束可变的启动装配阶段。Resolver 共享构建出的不可变 Registry，并在
     * 运行时应用一个明确的回退策略。OnAbsence 会保护调用方错误；OnAnyError 则会在
     * InvalidConfiguration 以及其他非缺失类错误后继续。
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
    let config = GreetingConfig {
        prefix: "hello".to_owned(),
        cloud_available: false,
    };

    /*
     * 自动选择按 priority 顺序尝试。cloud 排在前面，但它返回的 Unavailable 允许
     * OnAbsence 继续到 local。结果保留 local 的 canonical ID，因此日志不依赖 alias。
     */
    let automatic = resolver.create_auto(&config)?;
    assert_eq!("local", automatic.provider_id().as_str());
    assert_eq!("hello from local", automatic.service().greet());

    /*
     * 具名选择只解析一个 canonical ID 或 alias。"builtin" 映射到 local，而且具名
     * 选择不会回退到 cloud。
     */
    let named = resolver.create_named("builtin", &config)?;
    assert_eq!("local", named.provider_id().as_str());
    assert_eq!("hello from local", named.service().greet());

    /*
     * 链式选择保留调用方顺序。未知名称会进入诊断，remote 到达暂时不可用的 cloud，
     * 最后 builtin 通过 local 成功。
     */
    let chained = resolver.create_chain(["missing", "remote", "builtin"], &config)?;
    assert_eq!("local", chained.provider_id().as_str());
    assert_eq!("hello from local", chained.service().greet());

    /*
     * 第二个请求故意失败以展示诊断：cloud 不可用，local 随后拒绝空 prefix；因为无效
     * 配置不属于“缺失”，OnAbsence 会停止遍历。
     */
    let invalid_config = GreetingConfig {
        prefix: "  ".to_owned(),
        cloud_available: false,
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

当 `cloud_available: false` 时，自动选择先到达 `cloud`，在收到 `Unavailable` 后
继续，并返回 `local`。具名选择与链式选择也会返回 `local`。最后一个故意构造的无效
请求在 `LocalProvider` 处被策略终止，并运行诊断函数。

## 定义服务

当需要引入一组可独立配置的 Provider 实现时，定义一个服务族。

以下片段使用完整示例中已经定义的类型：

```rust,ignore
struct GreetingConfig {
    prefix: String,
    cloud_available: bool,
}

struct GreetingSpec;

impl ServiceSpec for GreetingSpec {
    type Config = GreetingConfig;
    type Output = Arc<dyn Greeter>;
}
```

可观察到的结果是一条编译期契约：每个 `ServiceProvider<GreetingSpec>` 都接收
`&GreetingConfig` 并返回 `Arc<dyn Greeter>`。

`Config` 可以是 unsized 类型，因此服务可以使用 `str` 或 trait object 等视图。
`Output` 是调用方最终持有的完整值；应根据应用的所有权和并发要求选择普通值、
`Box<dyn Trait>`、`Arc<dyn Trait>` 或其他句柄。SPI 不会自动添加或移除包装。

**常见误区：**为互不相关的服务定义一个过于宽泛的 specification。当配置、输出、
Provider 集合或选择策略需要独立演进时，应使用不同的标记类型。

## 实现 Provider

当需要加入一个能够创建 `ServiceSpec` 所指定输出的工厂时，实现 Provider。

以下片段来自完整示例：

```rust,ignore
impl ServiceProvider<GreetingSpec> for LocalProvider {
    fn create(&self, config: &GreetingConfig) -> Result<Arc<dyn Greeter>, ProviderError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "prefix must not be empty",
            ));
        }
        Ok(Arc::new(TextGreeter {
            message: format!("{} from local", config.prefix),
        }))
    }
}
```

结果是一个在 Resolver 到达该 Provider 时被调用的工厂。Provider 实现必须满足
`Send + Sync + 'static`，因为 Registry 会保留它，并且可能在线程间共享它。配置以
借用方式传入，每次调用成功时都会返回一个新的完整输出。

错误分类会直接控制回退，因此应按真实含义选择：

| `ProviderError` 构造器 | 含义 | `OnAbsence` |
| --- | --- | --- |
| `unsupported` | 该 Provider 无法处理此请求。 | 继续 |
| `unavailable` | 它能够处理，但现在暂时不可用。 | 继续 |
| `invalid_configuration` | 调用方提供了无效配置。 | 停止 |
| `initialization_failed` | 创建该实现时发生意外失败。 | 停止 |

每种分类还有对应的 `_with_source` 构造器，可保留底层的
`Error + Send + Sync + 'static`。

**常见误区：**把无效配置报告成 `Unavailable`。这会允许 `OnAbsence` 静默选择其他
Provider，从而掩盖调用方错误。

## 命名并排序 Provider

当需要为一次工厂注册指定稳定身份、可接受的配置名称以及自动选择顺序时，使用
descriptor。

以下片段来自完整示例：

```rust,ignore
let cloud = ProviderDescriptor::new(ProviderId::new("cloud")?)
    .with_aliases(["remote"])?
    .with_priority(100);
```

该 descriptor 将 `cloud` 设为 canonical ID，接受 `remote` 作为 alias，并为自动
选择设置 priority 100。

canonical `ProviderId` 是严格的小写 ASCII token：首尾必须是 ASCII 字母或数字，
中间还可以包含 `-`、`_`、`.` 和 `+`。`ProviderId::new` 不会修剪或规范化输入。
运行时 `ProviderSelector` 则不同：它会先修剪空白并把 ASCII 字母转为小写，再执行
校验，因此 `" REMOTE "` 可以解析 alias `remote`。

alias 与 canonical ID 共享同一个 selector 命名空间。descriptor 会拒绝无效 alias、
与自身 ID 相同的 alias 以及重复 alias；Builder 会拒绝已被其他注册占用的 selector。
priority 只影响 `create_auto`，具名选择和链式选择遵循调用方给出的 selector 或顺序。

无效 canonical ID 返回 `ProviderIdError`；无效或重复 alias 返回
`ProviderDescriptorError`。

**常见误区：**把 alias 当作 Provider 身份。即使请求使用 alias，结果和诊断仍然报告
canonical ID。

## 构建并检查 Registry

当需要在应用启动阶段装配所有可用工厂，或在之后检查不可变目录时，构建 Registry。

以下片段使用完整示例中的类型：

```rust,ignore
let shared_cloud: Arc<dyn ServiceProvider<GreetingSpec>> = Arc::new(CloudProvider);
let mut builder = ProviderRegistry::<GreetingSpec>::builder();
builder.register(
    ProviderDescriptor::new(ProviderId::new("local")?),
    LocalProvider,
)?;
builder.register_shared(
    ProviderDescriptor::new(ProviderId::new("cloud")?),
    shared_cloud,
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
let named = resolver.create_named("builtin", &config)?;
let chained = resolver.create_chain(["missing", "remote", "builtin"], &config)?;
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

## 选择回退策略

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

chain 中的未知 selector 会被记录后继续，因为此时没有调用任何 Provider。只有存在后续
候选项时回退策略才有实际作用；具名选择仍然只有一个候选项。策略提前停止会产生
`ResolutionTermination::StoppedByPolicy`；访问完所有允许的候选项会产生
`ResolutionTermination::Exhausted`。

**常见误区：**仅仅为了让请求成功而选择 `OnAnyError`。该策略可能掩盖配置和初始化
缺陷，只应在明确的尽力而为流程中使用。

## 复用已校验的选择

当同一个配置选择会用于多次创建调用时，预先构造 `ProviderSelection`。

以下片段使用完整示例中的类型：

```rust,ignore
use qubit_spi::ProviderSelection;

let selection = ProviderSelection::chain(["remote", "builtin"])?;

let first = resolver.create(&selection, &config)?;
let second = resolver.create(&selection, &config)?;
```

`ProviderSelection::auto()` 不会失败。`named(...)` 会规范化并校验一个 selector；
`chain(...)` 会校验所有 selector、保留顺序并拒绝空 chain。之后可通过
`ProviderResolver::create` 反复使用该已校验值。`Default` 是自动选择；`kind()` 返回
模式，`selector()` 在具名选择中借用 selector，`selectors()` 返回 chain，并在其他
模式下返回空 slice。

在运行时输入边界，更适合直接使用 `create_named` 和 `create_chain`：它们会把解析
失败转换为 `ResolutionError`，同时保留无效输入和 chain 索引。但这些方法每次调用
都会解析并分配拥有所有权的 selector 数据。复用 `ProviderSelection` 可以把这项工作
移到配置加载阶段，并以 `ProviderSelectionError` 报告校验失败。

**常见误区：**在热路径上反复解析固定配置字符串。应只校验一次并保留 selection。

## 检查成功结果

当需要消费输出或记录实际胜出的 Provider 时，检查 `CreatedService`。

以下片段来自完整示例：

```rust,ignore
let created = resolver.create_named("builtin", &config)?;
println!("winner: {}", created.provider_id());
created.service().greet();

let (provider_id, service) = created.into_parts();
```

`provider_id()` 和 `service()` 分别借用两个值。`into_service()` 消费包装并只返回输出；
`into_parts()` 消费包装并返回 `(ProviderId, Output)`。返回的 ID 始终是 canonical ID，
因此可观察性不会依赖调用方使用了哪个 alias。

**常见误区：**在记录日志或指标之前丢弃胜出者 ID。它是确认生产环境回退行为最直接
的信息。

## 诊断失败

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

## 共享 Registry 和 Resolver

`ProviderRegistry` 和 `ProviderResolver` 都可以低成本克隆。Registry 将不可变 entry
和索引存放在内部 `Arc` 后面，克隆 Registry 不会复制 Provider。克隆 Resolver 会共享
同一 Registry，并复制很小的回退策略值。`registry()` 和 `fallback_policy()` 提供对
Resolver 配置的只读访问。

Provider 满足 `Send + Sync + 'static`，因此目录可以共享。SPI 没有为
`ServiceSpec::Output` 添加 `Send` 或 `Sync` 约束；如果创建出的服务本身需要跨线程，
应选择 `Arc<dyn Trait + Send + Sync>` 等线程安全输出。

原始 selector 解析会规范化并持有 selector 文本，使错误和 selection 能安全地保留
它。该分配成本重要时应复用 `ProviderSelection`。Registry 查询和自动顺序使用
`build()` 时一次性准备的索引。

## 推荐实践

- 在启动阶段只装配一次 Provider，并让注册错误直接导致启动失败。
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
| 分类工厂失败并诊断解析 | [`ProviderError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/struct.ProviderError.html)、[`ResolutionError`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/enum.ResolutionError.html) |
| 处理校验、注册、Provider 和解析错误 | [`qubit_spi::error`](https://docs.rs/qubit-spi/0.8.0/qubit_spi/error/index.html) |
