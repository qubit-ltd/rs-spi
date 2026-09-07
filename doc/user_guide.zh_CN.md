# Qubit SPI 用户指南

[English](user_guide.md) · [README](../README.zh_CN.md) · [设计说明](design.zh_CN.md)

本指南适用于 `qubit-spi` 0.11，需要 Rust 1.94 或更高版本。领域库作者、服务提供者
实现者和应用开发者可以沿同一个示例完成集成：`lib-foo` 使用应用选定的问候服务，
但不依赖具体的服务提供者 crate。

## 三个独立阶段

| 阶段 | 输入和结果 | 错误边界 |
| --- | --- | --- |
| 注册 | 将元数据和同步或异步工厂加入注册表 | 规范 ID 或别名冲突时整体失败，不留下部分注册 |
| 选择 | 将经过校验的选择配置解析成候选快照 | 必选标识不存在、自动选择时注册表为空，或没有候选 |
| 创建 | 向候选工厂传入显式配置或默认配置 | 按实际调用的服务提供者聚合类型化错误，并记录终止原因 |

`ServiceSpec` 绑定配置 `Config` 和领域错误 `Error`；`SyncServiceSpec` 与
`AsyncServiceSpec` 分别绑定输出类型。解析器通过自身的创建方法返回
`ProviderCreationError<E>`，并不实现返回 `ProviderFailure<E>` 的叶级
`ServiceProvider<S>` 接口。

## 安装与组装示例

新建 Cargo workspace，包含 `lib-greeter`、`lib-foo`、`lib-friendly-greeter` 和
`app` 四个成员。每个包使用版本 `0.1.0`、edition `2024`，并在依赖中加入
`qubit-spi = "0.11"`。workspace 根目录的 `Cargo.toml` 为：

```toml
[workspace]
members = ["lib-greeter", "lib-foo", "lib-friendly-greeter", "app"]
resolver = "3"
```

各成员的 `[dependencies]` 还需要以下本地依赖：

| 包 | 本地依赖 |
| --- | --- |
| lib-greeter | 无 |
| lib-foo | `lib-greeter = { path = "../lib-greeter" }` |
| lib-friendly-greeter | `lib-greeter = { path = "../lib-greeter" }` |
| app | `lib-greeter`、`lib-foo`、`lib-friendly-greeter`，分别设置 `path = "../<包名>"` |

将下面四段完整代码放入对应文件，在 workspace 根目录运行 `cargo run -p app`，
成功时输出 `Hello, Rust!`。[示例验证清单](../tests/fixtures/documentation_examples/scenarios.json)
保存了检查器使用的完整成员清单；它用路径依赖验证当前源码，自建项目时应改用发布版本。

### 1. 服务接口与共享注册表

`lib-greeter` 定义业务接口，并持有一个 `LazyLock` 注册表。参与集成的各方必须使用同一个服务族类型与注册表实例。

<!-- spi-example: three-crates; file: lib-greeter/src/lib.rs -->
```rust
// lib-greeter/src/lib.rs
use std::{
    error::Error,
    fmt,
    sync::{Arc, LazyLock},
};

use qubit_spi::{ProviderRegistry, ServiceSpec, SyncServiceSpec};

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

/// 供 App 和所有下游库共享的进程级 Greeter Provider Registry。
pub static GREETER_REGISTRY: LazyLock<ProviderRegistry<GreeterSpec>> =
    LazyLock::new(ProviderRegistry::default);
```

### 2. 独立的消费方

`lib-foo` 解析应用设置的默认选择，再创建服务，不自行决定具体后端。

<!-- spi-example: three-crates; file: lib-foo/src/lib.rs -->
```rust
// lib-foo/src/lib.rs
use lib_greeter::GREETER_REGISTRY;

/// 创建 App 选定的默认 Greeter，并打印一条问候语。
pub fn foo() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GREETER_REGISTRY.resolve()?;
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
use qubit_spi::{
    ProviderDescriptor, ProviderId, ProviderMetadata, ServiceProvider,
};

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
        ProviderDescriptor::new(
            ProviderId::new("friendly").expect("static provider ID is valid"),
        )
        .with_priority(100)
    }
}
```

### 4. 应用负责组装

应用先完成注册和默认选择，再调用业务库。全局单例属于领域 crate，SPI 自身不持有这样的全局注册表。

<!-- spi-example: three-crates; file: app/src/main.rs -->
```rust
// app.rs
use lib_foo::foo;
use lib_friendly_greeter::FriendlyGreeterProvider;
use lib_greeter::GREETER_REGISTRY;
use qubit_spi::ProviderSelection;

// 应用装配入口：先安装 Provider，再调用 lib-foo。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    GREETER_REGISTRY.register(FriendlyGreeterProvider)?;
    GREETER_REGISTRY
        .set_default_selection(ProviderSelection::named("friendly")?);
    foo()
}
```

## 选择、配置与诊断

注册表的默认选择与服务配置相互独立。新注册表默认采用自动选择和 `OnAbsence`
策略；修改默认值只影响后续解析，不改变已有候选快照。若配置校验需要知道本次
解析究竟使用了哪个默认值，应调用 `resolve_default_snapshot()`：返回的选择配置
与解析结果来自同一次读快照。`resolve()` 仅返回该操作的解析结果。

| 选择方式 | 含义 |
| --- | --- |
| `named` | 选择一个规范 ID 或别名，没有第二个候选 |
| `chain` | 按调用方顺序选择；任何未知标识都会使整个选择失败 |
| `chain_allowing_missing` | 忽略未知标识；没有剩余候选时仍然失败 |
| `auto` | 按优先级降序、规范 ID 升序排列所有服务提供者 |

链中指向同一服务提供者的标识会去重，包括不同别名；未知标识则按原顺序保留在
错误诊断中，也保留重复项。选择配置经过构造校验，内部表示不公开；可通过
`target()` 和 `ProviderSelectionTargetRef` 查看，并为非穷尽枚举保留通配分支。

规范 ID 必须是非空的小写 ASCII，首尾为字母或数字，中间只允许字母、数字及
`-`、`_`、`.`、`+`。ID 不做裁剪或大小写转换。选择标识和描述符别名会先去掉首尾
空白、将 ASCII 转为小写，再按相同语法校验。别名不能与规范 ID 或其他归一化别名
重复。`with_aliases` 遇到第一个错误后停止消费迭代器。静态宏
`provider_descriptor!` 在编译期校验规范字面量。

| 回退策略 | 哪些错误允许尝试下一候选 |
| --- | --- |
| `Never` | 都不允许 |
| `OnAbsence` | `Unsupported`、`Unavailable` |
| `OnAnyError` | 全部四种叶级错误 |

`Unsupported` 表示无法服务当前请求；`Unavailable` 表示依赖或环境缺席。
`InvalidConfiguration` 表示请求被接受后配置不合法；`InitializationFailed`
表示接受请求后的初始化失败。错误分类需要由服务提供者认真判断：如果把所有错误
都当成缺席，回退成功可能掩盖真正的配置问题。

下面是独立的二进制示例，覆盖别名、回退、默认快照和结构化诊断。使用快速开始
中的依赖，将代码放入 `src/main.rs`，运行 `cargo run`；全部断言通过且没有标准
输出即表示成功。

<!-- spi-example: guide-sync; file: app/src/main.rs -->
```rust
use std::error::Error;
use std::io;
use qubit_spi::{FallbackPolicy, ProviderCreationTermination, ProviderDescriptor, ProviderId};
use qubit_spi::{ProviderMetadata, ProviderRegistry, ProviderSelection, ServiceProvider};
use qubit_spi::{ServiceSpec, SyncServiceSpec};
use qubit_spi::error::{ProviderFailure, ProviderFailureKind};

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = io::Error; }
impl SyncServiceSpec for Greeting { type Output = String; }
struct Backend { remote: bool }
impl ProviderMetadata for Backend {
    fn descriptor(&self) -> ProviderDescriptor {
        let id = if self.remote { "remote" } else { "local" };
        ProviderDescriptor::new(ProviderId::new(id).expect("valid static ID"))
            .with_aliases([if self.remote { "network" } else { "disk" }])
            .expect("valid static alias")
            .with_priority(if self.remote { 100 } else { 0 })
    }
}
impl ServiceProvider<Greeting> for Backend {
    fn create_configured(&self, name: &String) -> Result<String, ProviderFailure<io::Error>> {
        if self.remote {
            return Err(ProviderFailure::unavailable(io::Error::new(
                io::ErrorKind::NotConnected, "remote greeting service is offline",
            )));
        }
        if name.is_empty() {
            return Err(ProviderFailure::invalid_configuration(io::Error::new(
                io::ErrorKind::InvalidInput, "a name is required",
            )));
        }
        Ok(format!("Hello, {name}!"))
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let registry = ProviderRegistry::<Greeting>::default();
    registry.register(Backend { remote: true })?;
    registry.register(Backend { remote: false })?;
    let config = String::from("Rust");
    let chain = ProviderSelection::chain([" NETWORK ", "disk", "local"])?
        .with_fallback_policy(FallbackPolicy::OnAbsence);
    registry.set_default_selection(chain.clone());
    let (captured, result) = registry.resolve_default_snapshot();
    assert_eq!(chain, captured);
    let snapshot = result?;
    registry.set_default_selection(ProviderSelection::named("remote")?);
    assert_eq!("Hello, Rust!", snapshot.create_configured(&config)?);

    let error = registry.resolve()?.create_configured(&config).expect_err("remote is absent");
    assert_eq!(ProviderCreationTermination::Exhausted, error.termination());
    assert!(error.is_absence());
    for attempt in error.attempts() {
        assert_eq!("remote", attempt.provider_id().as_str());
        assert_eq!(ProviderFailureKind::Unavailable, attempt.failure().kind());
        assert_eq!(io::ErrorKind::NotConnected, attempt.failure().error().kind());
        assert!(attempt.source().is_some());
        assert!(!attempt.failure().error().to_string().is_empty());
    }
    let stopped = registry.resolve_selected(
        &ProviderSelection::auto().with_fallback_policy(FallbackPolicy::Never),
    )?.create_configured(&config).expect_err("Never stops before local");
    assert_eq!(ProviderCreationTermination::StoppedByPolicy, stopped.termination());
    let failed = snapshot.create_configured(&String::new()).expect_err("local rejects empty name");
    assert_eq!(2, failed.attempts().len());
    assert!(!failed.is_absence());
    assert_eq!("local", failed.decisive_attempt().provider_id().as_str());
    assert!(registry.resolve_selected(&ProviderSelection::chain(["missing", "local"])?).is_err());
    assert_eq!("Hello, Rust!", registry.resolve_selected(
        &ProviderSelection::chain_allowing_missing(["missing", "disk"])?,
    )?.create_configured(&config)?);
    Ok(())
}
```

聚合错误只包含实际调用过的服务提供者。`attempt.failure()` 取得分类后的错误，
`attempt.failure().error()` 取得领域错误。将领域错误直接传入 `ProviderFailure`
构造函数即可；若领域错误还有底层原因，应由该类型实现 `Error::source`，SPI 会
保留这条因果链。API 没有专门的 `_with_source` 构造函数。

`Exhausted` 表示已尝试全部候选；`StoppedByPolicy` 表示仍有候选，但策略禁止继续。
最后一个候选失败时，即使使用 `Never`，结果也是 `Exhausted`。
`decisive_attempt()` 返回最后一次实际调用；`is_absence()` 表示所有实际失败均属于
缺席类别。`into_parts()` 可将诊断数据的所有权交给调用方。创建成功时，不返回成功前
各次失败的记录。SPI 不自行记录日志或
输出遥测数据。

## 异步创建

为下面的独立二进制示例添加 `futures = "0.3"` 依赖。SPI 本身不依赖该执行器。
注册和解析仍同步执行，只有创建需要等待，并且一次只尝试一个候选。

<!-- spi-example: guide-async; file: app/src/main.rs -->
```rust
use qubit_spi::{AsyncProviderRegistry, AsyncServiceProvider, AsyncServiceSpec, ProviderFuture};
use qubit_spi::{ProviderDescriptor, ProviderMetadata, ServiceSpec, provider_descriptor};
use qubit_spi::error::ProviderFailure;

struct Greeting;
impl ServiceSpec for Greeting { type Config = String; type Error = std::io::Error; }
impl AsyncServiceSpec for Greeting { type Output = String; }
struct Friendly;
impl ProviderMetadata for Friendly {
    fn descriptor(&self) -> ProviderDescriptor { provider_descriptor!("friendly") }
}
impl AsyncServiceProvider<Greeting> for Friendly {
    fn create_configured<'a>(&'a self, name: &'a String)
        -> ProviderFuture<'a, Result<String, ProviderFailure<std::io::Error>>> {
        Box::pin(async move { Ok(format!("Hello, {name}!")) })
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = AsyncProviderRegistry::<Greeting>::default();
    registry.register(Friendly)?;
    let resolver = registry.resolve()?;
    let config = String::from("Rust");
    let output = futures::executor::block_on(resolver.create_configured(&config))?;
    assert_eq!("Hello, Rust!", output);
    Ok(())
}
```

`ProviderFuture<'a, T>` 是可发送的装箱 future，在 `'a` 期间借用服务提供者和配置。
异步配置必须满足 `Sync`，异步输出必须满足 `Send + 'static`；同步输出没有对应的
Send 限制。`Config` 可以是非定长类型。同步 `create()` 要求 `Config: Default`，
异步 `create()` 还要求 `Send`；不满足时使用 `create_configured(&config)`。

未经 poll 就丢弃解析器 future，不会调用服务提供者。如果在某个候选 Pending 时
取消，会丢弃当前候选的 future，不会启动下一候选。取消不转换为 `ProviderFailure`，
也不触发回退。这不意味着撤销外部副作用或停止服务提供者独立启动的后台任务。
叶级方法返回 future 之前的 panic，以及 poll 时的 panic，都会直接传播；
`OnAnyError` 不会捕获 panic。

## 运行时所有权与边界

注册表的克隆共享注册内容与默认选择，同步和异步注册表彼此独立。每次注册尝试只
采样一次元数据，且在取得写锁之前执行；回调返回后才检查冲突。失败的注册尝试不会插入
自身的任何选择标识。元数据回调重入后产生的变更会保留，即使回调随后发生 panic
也不会回滚。工厂、格式化回调和被拒绝服务提供者的析构都在注册表锁外执行。

ID 和描述符快照按成功注册的顺序返回，自动候选则按独立的优先级顺序排列。
解析器固定候选身份、顺序与策略，但共享服务提供者对象，不会深拷贝可变后端状态。
每次创建都会调用工厂；工厂可以克隆已有 `Arc`，也可以返回缓存客户端的句柄。
SPI 不保证每次输出都对应新分配的底层资源。服务创建成功后的业务操作错误不会
再次触发回退。

测试和局部组件宜使用独立注册表。领域层可以用 `LazyLock` 包装 `global()` 门面，
并在消费方第一次解析前完成应用配置。全局注册表会在整个进程生命周期内保留已注册
的服务提供者及其持有的资源。同一领域 crate 的不同链接版本各有自己的
静态注册表。注册表允许运行时继续注册，但不提供注销、冻结、服务缓存、动态库
加载或自动依赖注入。URI 语义、凭证、输出身份校验及生命周期策略由领域 crate
或服务提供者负责。成功输出直接返回，不附加元数据包装。

## 排障

| 现象 | 检查与处理 |
| --- | --- |
| 找不到服务提供者 | 查看 `provider_ids()`/`descriptors()`，区分规范 ID 与归一化选择标识 |
| 默认选择不符合预期 | 查看 `default_selection()`、优先级和规范 ID 排序；必要时显式设置具名选择 |
| 看不到新注册项 | 重新解析，旧解析器保留旧快照；确认各方使用同一领域 crate 和注册表 |
| 回退没有继续 | 检查失败分类与策略；具名选择没有后续候选 |
| 无法使用 `create()` | 改用显式配置，并检查 Default/Send 约束 |
| 测试重复注册失败 | 使用每个测试独有的注册表，或用子进程隔离全局状态变更 |
| 意外共享底层资源 | 检查工厂缓存；SPI 会调用工厂，但不强制创建独立资源 |

## 后续集成与验证

已有 `Arc` 服务提供者可以使用 `register_shared`，也可以参照三个库的示例封装
领域门面。[API 文档](https://docs.rs/qubit-spi) 提供精确签名和错误变体；
[设计说明](design.zh_CN.md) 解释锁、快照和性能决策。

在本仓库运行 `python3 scripts/check-documentation.py`，会按语言分别编译和运行
全部带标记的 Rust 代码块。随后执行 `./align-ci.sh`；验证本地未提交变更时运行
`SPI_PACKAGE_ALLOW_DIRTY=1 ./ci-check.sh`，干净的 CI checkout 直接运行
`./ci-check.sh`，不设置该变量。
[README](../README.zh_CN.md) · [English Guide](user_guide.md)
