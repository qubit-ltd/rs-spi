# Qubit SPI 用户手册

本手册介绍 Qubit SPI 0.8 的运行时 Provider 模型，覆盖从 App 启动注册到下游使用
Service 的完整生命周期，包括 selection、配置、fallback、错误诊断、并发、全局 facade
以及从旧 resolver API 迁移的方法。

如果只需要快速了解，请先阅读[项目说明](../README.zh_CN.md)。

## Qubit SPI 解决什么问题

假设一个可复用的库 X 需要 MIME 检测器。库 X 不应该自己选择或构造具体实现，因为最终
App 可能需要模型检测器、系统命令检测器，或者部署环境提供的自定义 Provider。

预期的运行时关系是：

1. App 在启动时注册当前可用的 Provider。
2. App 可以设置进程级默认 Provider selection。
3. 库 X 随后解析自己的显式 selection，或者使用这个默认值。
4. 解析出的 Provider 使用显式或默认 config 创建 Service。
5. 库 X 使用返回的 Service，不需要了解其具体类型。

这是 Service Provider Registry，而不是通用依赖注入框架。它统一实现的注册、选择和
创建方式；Service 的业务接口仍然属于具体领域 crate。

## 第一性原理：三个独立阶段

最重要的设计规则是：注册、选择和创建回答的是三个不同问题，不能压缩成一次操作。

### 注册：当前有什么实现

注册把一个 `ProviderDefinition<S>` 安装到 `ProviderRegistry<S>`。Provider 同时提供
创建行为和自己的 descriptor。Registry 保存 Provider 身份和查找元数据，而不是已经
创建好的 Service。

canonical ID 或 alias 已被占用时，注册会失败。注册不会解析某次请求的 selection，
也不会创建 Service。

### 选择：本次允许尝试什么

`ProviderSelection` 描述 named Provider、调用方指定顺序的 chain，或者 Registry 自动
顺序。`ProviderRegistry::resolve` 把这个 selection 解析为一个时间点上的候选快照，
用 `ResolvingServiceProvider<S>` 表示。

选择阶段不需要 `S::Config`，也不会调用 Provider 代码。请求的 Provider 或候选集合
不存在时，选择失败。

### 创建：候选能否构造服务

`ResolvingServiceProvider<S>` 实现 `ServiceProvider<S>`。它的 `create` 使用
`S::Config` 调用候选 Provider，执行 selection 中保存的 fallback policy，并在成功时
直接返回 `S::Output`。

Provider 不支持请求、运行环境不可用、配置非法或初始化失败都会导致创建错误。聚合错误
只保留真正调用过的 Provider。

```text
ProviderDefinition --register--> ProviderRegistry
                                      │
ProviderSelection ---------------- resolve
                                      │
                                      ▼
                         ResolvingServiceProvider
                                      │
S::Config ------------------------- create
                                      │
                                      ▼
                                 S::Output
```

## 核心类型

| 类型 | 职责 |
| --- | --- |
| `ServiceSpec` | 绑定一个服务族的 `Config` 和 `Output` 类型 |
| `ServiceProvider<S>` | 根据 `S::Config` 创建 `S::Output` |
| `ProviderDefinition<S>` | 为 Service Provider 增加自有 descriptor |
| `ProviderDescriptor` | 保存 canonical ID、alias 和自动选择 priority |
| `ProviderRegistry<S>` | 保存共享的运行时注册状态和默认 selection |
| `ProviderSelection` | 描述候选目标和创建阶段 fallback policy |
| `ResolvingServiceProvider<S>` | 持有解析后的候选快照并创建 Service |

泛型参数 `S` 防止不同服务族的 Provider 被混用。MIME Provider 无法注册到文件系统
Registry，因为它们使用不同的 `ServiceSpec`。

## 定义服务族

首先定义业务能力，其中只放消费者在初始化完成后反复调用的操作。构造参数放在独立的
config 类型中。

```rust
use std::sync::Arc;

use qubit_spi::ServiceSpec;

trait Greeter: Send + Sync {
    fn greet(&self, name: &str) -> String;
}

#[derive(Clone)]
struct GreeterConfig {
    prefix: String,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            prefix: "Hello".to_owned(),
        }
    }
}

struct GreeterSpec;

impl ServiceSpec for GreeterSpec {
    type Config = GreeterConfig;
    type Output = Arc<dyn Greeter>;
}
```

`ServiceSpec::Output` 是消费者需要的完整实体，常见形式包括 `Arc<dyn Trait>`、具体
client 或轻量 handle。Qubit SPI 不会用 Provider 元数据包装成功结果，也不会缓存它。

`ServiceSpec::Config` 可以是 unsized 类型。只有 config 实现 `Default` 时才能调用
`create_default()`；`create(&config)` 始终可用。

## 实现自描述 Provider

可以注册的 Provider 实现两个契约：

1. `ServiceProvider<S>`：提供创建行为。
2. `ProviderDefinition<S>`：提供稳定的注册元数据。

```rust
use std::sync::Arc;

use qubit_spi::error::{ProviderCreationError, ProviderError};
use qubit_spi::{
    ProviderDefinition, ProviderDescriptor, ProviderId, ServiceProvider,
};

struct FriendlyGreeter {
    prefix: String,
}

impl Greeter for FriendlyGreeter {
    fn greet(&self, name: &str) -> String {
        format!("{}, {}!", self.prefix, name)
    }
}

struct FriendlyProvider;

impl ServiceProvider<GreeterSpec> for FriendlyProvider {
    fn create(
        &self,
        config: &GreeterConfig,
    ) -> Result<Arc<dyn Greeter>, ProviderCreationError> {
        if config.prefix.trim().is_empty() {
            return Err(ProviderError::invalid_configuration(
                "the greeting prefix must not be empty",
            )
            .into());
        }
        Ok(Arc::new(FriendlyGreeter {
            prefix: config.prefix.clone(),
        }))
    }
}

impl ProviderDefinition<GreeterSpec> for FriendlyProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_aliases(["default-greeter", "friendly-greeter"])
        .expect("static aliases are valid")
        .with_priority(100)
    }
}
```

### 为什么 descriptor 属于 Provider

Provider 身份和创建实现共同构成一个注册单元。要求调用方分别传递两者，既可能造成
元数据与实现不匹配，也会让第三方安装过程变得繁琐。自描述 Provider 让 App 只需：

```rust,ignore
registry.register(FriendlyProvider)?;
```

注册会先调用 `descriptor()`，随后才获取 Registry 写锁，并保存 descriptor 快照。
Provider 后续状态变化无法修改已经注册的 ID、alias 或 priority。

### Canonical ID、alias 和 priority

`ProviderId` 必须已经是 canonical 小写 ASCII，只允许字母数字和 `-`、`_`、`.`、`+`
分隔符，并且首尾必须是字母数字。

`ProviderSelector` 用于输入边界。解析时会去除首尾空白、把 ASCII 字母转成小写，再按照
同一 token 语法校验。因此配置值 `" Friendly-Greeter "` 会解析成规范化 alias
`friendly-greeter`。

alias 规范化后不能与 canonical ID 或另一个 alias 重复。priority 只影响自动选择：
值越大越靠前；priority 相同时，canonical ID 按升序排列。

## 创建并共享运行时 Registry

最简单的 Registry 初始为空，并允许运行时修改：

```rust,ignore
let registry = ProviderRegistry::<GreeterSpec>::default();
registry.register(FriendlyProvider)?;
```

`ProviderRegistry::builder()` 是可选的流式装配工具：

```rust,ignore
let mut builder = ProviderRegistry::<GreeterSpec>::builder();
builder.register(FriendlyProvider)?;
let registry = builder.build();

// builder 产出的 Registry 仍然允许运行时注册。
registry.register(AnotherProvider)?;
```

Provider 已经保存在 `Arc<dyn ProviderDefinition<S>>` 中时使用 `register_shared`；
其他情况优先使用 `register(provider)`。

### Clone 与同步语义

Registry clone 共享同一个 `Arc<RwLock<...>>` 状态：

```rust,ignore
let library_registry = registry.clone();
registry.register(FriendlyProvider)?;
assert_eq!(1, library_registry.len());
```

通过任何 clone 完成的注册和默认 selection 修改，都对其他 clone 可见。返回 descriptor、
ID、默认 selection 或解析结果的方法提供自有快照。执行第三方 Provider 代码时不会持有
Registry 锁。

对于 selector 冲突，注册是原子的。ID 或 alias 已被占用时，Registry 保持不变，并返回
`RegistrationError::DuplicateSelector`，其中包含现有 Provider 和新 Provider 的 ID。

## 完整的 App 与库 X 模式

Qubit SPI 有意不定义统一的全局 Registry：每个服务族都有不同的 `ServiceSpec`。拥有
Service trait 的领域 crate 应暴露适合自己的单体或 facade。

```rust
use std::sync::{Arc, LazyLock};

use qubit_spi::{ProviderRegistry, ProviderSelection, ServiceProvider};

static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);

fn greeter_registry() -> &'static ProviderRegistry<GreeterSpec> {
    &GREETER_REGISTRY
}

// App 启动：
fn configure_app() -> Result<(), Box<dyn std::error::Error>> {
    let registry = greeter_registry();
    registry.register(FriendlyProvider)?;
    registry.set_default_selection(ProviderSelection::named("friendly")?);
    Ok(())
}

// 独立发布的库 X：
fn library_x_greeter() -> Result<Arc<dyn Greeter>, Box<dyn std::error::Error>> {
    let provider = greeter_registry().resolve_default()?;
    Ok(provider.create_default()?)
}
```

App 控制安装哪些实现以及默认策略。库 X 只依赖服务族和 Registry facade。如果库 X 有
明确要求，也可以解析自己的显式 `ProviderSelection`。

启动顺序很重要：必须在下游代码首次请求 Service 前配置全局 Registry。消费者已经拿到
的 `ResolvingServiceProvider` 是时间点快照；后续注册只影响未来的解析，不会修改现有
快照。

Cargo 通常会统一兼容版本的领域 crate。如果同时链接不兼容版本，每个 crate 版本会拥有
独立的静态 Registry。App 和库必须使用同一个领域 crate 实例才能共享单体。

## 选择 Provider

Selection 是一个值对象，可以来自配置文件、命令行、库内硬编码需求或 App 默认值。
它不要求存放在 Service config 类型中。

### Named selection

```rust,ignore
let selection = ProviderSelection::named("friendly")?;
let provider = registry.resolve(&selection)?;
```

named selection 只解析一个 canonical ID 或 alias。selector 不存在时返回
`ProviderSelectionError::UnknownProvider`。它只有一个候选，因此 fallback policy
不会让其他 Provider 运行。

### 有序 chain

```rust,ignore
let selection = ProviderSelection::chain([
    "remote-greeter",
    "friendly",
    "minimal",
])?;
let provider = registry.resolve(&selection)?;
```

chain 按调用方顺序排列。不存在的 selector 会被跳过。如果多个 selector 通过 ID 和
alias 指向同一个 Provider，该 Provider 只在首次出现的位置保留一次。只有所有 chain
项都不匹配时，解析才返回 `NoCandidates`。

### 自动选择

```rust,ignore
let provider = registry.resolve(&ProviderSelection::auto())?;
```

自动选择按照确定顺序包含全部已注册 Provider：

1. priority 降序；
2. priority 相同时 canonical ID 升序。

Registry 为空时返回 `ProviderSelectionError::EmptyRegistry`。

### Registry 默认 selection

新 Registry 默认使用 `ProviderSelection::auto()` 和
`FallbackPolicy::OnAbsence`。App 可以在运行时替换：

```rust,ignore
let default = ProviderSelection::chain(["remote", "friendly"])?
    .with_fallback_policy(FallbackPolicy::OnAbsence);
registry.set_default_selection(default);

let snapshot = registry.default_selection();
let provider = registry.resolve_default()?;
```

`set_default_selection` 保存已经校验的 selection，但不要求对应 Provider 当时已经
存在，因此可以先设置策略、后注册实现。`resolve_default` 使用当前 selection 和当前
Registry 状态进行解析。

### Selection 与 config 相互独立

以下四种组合都合法：

```rust,ignore
// Registry 默认 selection + 默认 config。
let service = registry.resolve_default()?.create_default()?;

// 显式 selection + 默认 config。
let service = registry.resolve(&selection)?.create_default()?;

// Registry 默认 selection + 显式 config。
let service = registry.resolve_default()?.create(&config)?;

// 显式 selection + 显式 config。
let service = registry.resolve(&selection)?.create(&config)?;
```

不要强迫每种 Service config 都包含 Provider selection 字段。config 可以把 selection
作为一种便利来源，但没有 config 对象的调用方仍然必须能够使用 Registry 默认值。

## 创建 Service

`ProviderRegistry::resolve` 返回 `ResolvingServiceProvider<S>`。它是一个组合型
`ServiceProvider<S>`：持有候选 Provider handle，并在调用 `create` 时执行 selection
中的 fallback policy。

```rust,ignore
use qubit_spi::ServiceProvider;

let provider = registry.resolve(&selection)?;
let service = provider.create(&config)?;
```

必须导入 `ServiceProvider` trait，才能调用它的方法。

创建成功直接返回 `S::Output`。如果消费者希望在成功路径知道实际 Provider ID，那属于
领域层观测需求，不是通用 Service 值的职责。创建失败时的诊断已经保留错误处理需要的
实际 attempt。

Qubit SPI 每次调用 `create` 都会创建一个新输出。构造成本较高时，应由 App 或库缓存
返回值，或者 clone 返回的 handle。

## Fallback policy

Fallback 属于 `ProviderSelection`，因为它是调用方的请求策略，而不是 Registry 永久
状态，也不是 Service 配置。

| Policy | `Unsupported` 后继续 | `Unavailable` 后继续 | 非法配置或初始化失败后继续 |
| --- | --- | --- | --- |
| `Never` | 否 | 否 | 否 |
| `OnAbsence` | 是 | 是 | 否 |
| `OnAnyError` | 是 | 是 | 是 |

`OnAbsence` 是默认值，也是一般场景下最安全的策略。能力或环境缺失时可以尝试备选实现；
可能属于编程或部署错误的问题则立即停止。只有明确需要降级的 best-effort 行为时才使用
`OnAnyError`。

Provider 返回叶子 `ProviderError` 后才会判断 fallback。普通注册 Provider 应把
`ProviderError` 通过 `.into()` 转成 `ProviderCreationError::Provider` 返回。

## 错误模型

错误按照三个生命周期阶段和输入校验边界组织。

### 定义与注册错误

- `ProviderIdError`：canonical ID 为空或不规范。
- `ProviderSelectorError`：规范化后的用户/配置输入为空或非法。
- `ProviderDescriptorError`：alias 非法、重复或与 ID 相同。
- `RegistrationError`：ID 或 alias 与 Registry 状态冲突。

### 选择错误

`ProviderSelectionError` 在调用任何 Provider 之前返回：

- `InvalidSelector`：原始 selection 输入非法；
- `EmptyChain`：调用方没有提供 chain 项；
- `UnknownProvider`：named selection 没有匹配项；
- `NoCandidates`：非空 chain 中没有任何项匹配；
- `EmptyRegistry`：自动选择时没有 Provider。

这些错误不包含 Provider 创建 attempt，因为 Provider 尚未被调用。

### 叶子 Provider 错误

具体 Provider 使用 `ProviderErrorKind` 对 `ProviderError` 分类：

- `Unsupported`：Provider 不支持本次请求；
- `Unavailable`：Provider 或依赖环境不存在；
- `InvalidConfiguration`：Provider 拒绝给定配置；
- `InitializationFailed`：Provider 构造过程中发生意外失败。

使用 `_with_source` 构造器保留底层错误。Registry 内部不会替外部消费者执行日志或观测
收集；操作失败时，消费者可以获得完整的错误链。

### 聚合创建错误

`ProviderCreationError` 有两种形态：

- `Provider(error)`：直接调用一个 Provider 失败；
- `NoProviderSucceeded { attempts, termination }`：组合创建失败。

每个 `ProviderAttemptFailure` 保存实际调用 Provider 的 canonical ID 和原始
`ProviderError`。chain 中不存在的 selector 不会伪造 attempt。

`ProviderCreationTermination` 说明遍历为何结束：

- `Exhausted`：selection 接纳的全部候选都已尝试；
- `StoppedByPolicy`：terminal failure 后 fallback policy 不允许继续。

常用查询如下：

```rust,ignore
if error.is_absence() {
    // 所有相关失败都是 Unsupported 或 Unavailable。
}

for attempt in error.attempts() {
    eprintln!("{}: {}", attempt.provider_id(), attempt.error());
}

match error.termination() {
    Some(ProviderCreationTermination::Exhausted) => { /* ... */ }
    Some(ProviderCreationTermination::StoppedByPolicy) => { /* ... */ }
    None => { /* 直接 Provider 错误 */ }
    _ => { /* 未来新增的 non-exhaustive variant */ }
}
```

`decisive_attempt()` 返回可以单独解释 policy stop 或单候选耗尽的 attempt。多候选全部
耗尽时，有多个同等重要的失败，因此不会虚构唯一 decisive source。

## 并发与快照语义

Provider trait 要求存储的定义满足线程安全约束，因此 `ProviderRegistry<S>` 可以跨线程
共享。内部状态使用 `RwLock`。

- 注册先调用 `descriptor()`，之后才获取写锁。
- 替换默认 selection 只短暂持有写锁。
- 解析在复制候选 handle 时持有读锁。
- 释放锁之后才调用 Provider 创建 Service。
- 锁中毒时恢复其中保留的状态，而不是再次 panic。

解析出的 Provider 持有候选的 `Arc` handle，因此 Registry 被 clone、修改或 drop 后仍
可使用，但不会看到后续注册。需要新候选时重新解析。

## 从旧 API 迁移

0.8 是有意的破坏性版本，不提供兼容 facade。

### Provider 定义

以前调用方同时传递 `ProviderDescriptor` 和 Provider。现在 Provider 自己实现
`ProviderDefinition<S>`，注册只接收一个值：

```rust,ignore
// 当前 API
registry.register(FriendlyProvider)?;
```

这样可以防止元数据与实现错配，并把第三方安装简化为一次操作。

### Registry 与解析

以前的 `ProviderResolver` 已删除。`ProviderRegistry` 现在同时负责运行时修改、默认
selection 和解析：

```rust,ignore
let provider = registry.resolve(&selection)?;
let service = provider.create(&config)?;
```

`FallbackPolicy` 从 resolver 移入 `ProviderSelection`。

### 成功结果

以前的 `CreatedService<T>` 和 `ResolvedProvider` 包装已删除。创建直接返回
`S::Output`。解析对象现在是具体的 `ResolvingServiceProvider<S>`，它本身实现
`ServiceProvider<S>`。

### 失败结果

以前统一的 `ResolutionError`、`ResolutionTermination` 和 `AttemptFailure` 模型改为：

- 创建前使用 `ProviderSelectionError`；
- 创建阶段使用 `ProviderCreationError`；
- 聚合创建诊断内部使用 `ProviderCreationTermination` 和
  `ProviderAttemptFailure`。

迁移下游错误转换时，应根据失败阶段重新划分，而不是把一个旧类型机械改名成一个新类型。

## 推荐实践

1. 每个需要独立选择的服务族定义一个 `ServiceSpec`。
2. 由领域 crate 持有 Service trait 和可选全局 facade。
3. 每个可注册 Provider 直接实现 `ProviderDefinition`。
4. 在下游首次使用 Service 前完成 App Provider 注册。
5. 把默认策略放在 Registry 中；只有调用方有真实要求时才传显式 selection。
6. 保持 selection 与 Service config 相互独立。
7. 默认使用 `OnAbsence`；在调用点说明为何需要 `OnAnyError`。
8. 返回分类清晰并保留 causal source 的 `ProviderError`。
9. 在 Qubit SPI 外缓存构造成本较高的 Service 输出。
10. 修改注册或默认值的测试使用隔离 Registry。

## 故障排查

### 已注册 Provider 无法找到

检查 `descriptor()` 返回的 canonical ID 和规范化 alias。使用
`registry.provider_ids()` 和 `registry.descriptors()` 查看快照。注意
`ProviderId` 不执行规范化，而 `ProviderSelector` 会规范化。

### `resolve_default()` 选择了意外 Provider

检查 `registry.default_selection()`。新 Registry 默认自动选择，按 priority 降序和
canonical ID 升序排列。如果 App 启动时应固定一个 Provider，请显式调用
`set_default_selection`。

### Fallback 没有继续

检查 terminal attempt 的 `ProviderErrorKind` 和 selection policy。`OnAbsence` 有意在
`InvalidConfiguration` 和 `InitializationFailed` 后停止。named selection 也没有第二
个候选。

### 新注册 Provider 不可见

Registry clone 可以看到新注册，但已经解析的 `ResolvingServiceProvider` 是快照，需要
重新解析。对于全局 facade，还应确认 App 与库链接的是同一领域 crate 版本。

### 无法调用 `create_default()`

`S::Config` 必须实现 `Default`，并且必须导入 `ServiceProvider` trait。否则构造 config
并调用 `create(&config)`。

### 重复执行测试时全局注册冲突

进程级 Registry 有意保留状态。优先为每个测试创建隔离的
`ProviderRegistry::default()`；或者在独立子进程中运行需要修改全局状态的场景。

## API 参考

| API | 用途 |
| --- | --- |
| `ServiceSpec` | 绑定 config 与 output 类型 |
| `ServiceProvider::create` | 使用显式 config 创建 |
| `ServiceProvider::create_default` | 使用 `Config::default()` 创建 |
| `ProviderDefinition::descriptor` | 让可注册 Provider 自描述 |
| `ProviderRegistry::register` | 运行时注册 owned Provider |
| `ProviderRegistry::register_shared` | 注册已有 shared Provider |
| `ProviderRegistry::set_default_selection` | 替换进程或组件默认策略 |
| `ProviderRegistry::resolve` | 解析显式 selection |
| `ProviderRegistry::resolve_default` | 解析 Registry 当前默认值 |
| `ProviderRegistry::descriptors` | 获取注册元数据快照 |
| `ProviderRegistry::provider_ids` | 获取 canonical ID 快照 |
| `ProviderSelection::named` | 选择一个 ID 或 alias |
| `ProviderSelection::chain` | 选择调用方排序的候选 |
| `ProviderSelection::auto` | 按确定顺序选择全部 Provider |
| `ProviderSelection::with_fallback_policy` | 附加创建阶段 fallback policy |
| `ResolvingServiceProvider` | 通过解析后的候选快照创建 Service |

准确签名和 non-exhaustive 错误 variant 请查阅[自动生成的 API 文档](https://docs.rs/qubit-spi)。
