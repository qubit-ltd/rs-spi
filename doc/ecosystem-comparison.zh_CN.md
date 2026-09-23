# rs-spi 整体设计与 Rust 生态方案对比

## 版本基线与结论

- **对象**：`qubit-spi` 0.13.0；本文对比其当前 API、实现边界与 Rust 生态中的相关方案。
- **文档范围**：本文用于说明项目定位、架构取舍和适用场景。具体 API 与运行示例以双语 README、用户指南和 Rust API 文档为准。

**结论先行**：rs-spi 不是“Rust 动态插件框架”的同类替代品，而是一个用于**静态链接应用内、运行时可配置的多后端服务选择器**。它最突出的价值，是将“候选发现（显式注册）—选择（名称/链/自动）—构造（带失败语义的回退）”分成三个阶段，并让同步与异步在选择和错误语义上保持一致。对于文件系统、对象存储、编码器、认证后端这类“应用决定装配、库只依赖抽象”的场景，它比手写 `match` 或仅用 `inventory` 更完整、可观测且并发边界更清楚。

代价也同样明确：默认 feature 集不做跨 crate 自动发现；可选的 `inventory` feature
只能发现最终二进制中已链接的静态 factory，仍没有运行时 `.so/.dll` 加载、依赖图装配，
也不处理已成功创建对象的生命周期、缓存、健康检查或后续操作失败。因此应把它定位为
**轻量、显式、类型化 SPI 基础设施**，不要作为通用 DI 容器或第三方二进制插件平台来
评估或扩张。

## 1. 项目定位与边界

Rust 没有标准库版的 Java `ServiceLoader`。rs-spi 以一个服务族的零大小标记类型 `S` 为中心：`ServiceSpec` 绑定通用配置类型和领域错误类型，`SyncServiceSpec` 或 `AsyncServiceSpec` 另行绑定输出类型。[`ServiceSpec`](../src/service/service_spec.rs#L32) 允许 `Config: ?Sized`，但错误必须是 `Error + Send + Sync + 'static`；这让 provider 的错误保留领域诊断，而不是被擦除为字符串。

应用注册具体 provider、设置默认选择；被复用的库解析并构造服务。README 也明确表示：单一且不需要选择的实现应直接用构造函数；URI、凭据、缓存与输出身份校验属于领域适配层，而非 SPI 核心。[设计文档](design.md#responsibilities)

不覆盖的能力：插件二进制发现/加载、卸载、依赖注入图、服务运行期故障切换、自动缓存、provider 健康探测与配置协议。这是刻意克制的边界，而不是现有实现缺口。

## 2. 核心解法与架构

### 2.1 三阶段模型

```mermaid
flowchart LR
  A[应用：注册 ProviderDefinition] --> B[ProviderCatalog\nArc<RwLock<RegistryInner>>]
  C[应用/配置：ProviderSelection] --> B
  B -->|单个 read snapshot| D[ResolvingServiceProvider\n候选快照 + fallback policy]
  D -->|create_configured(config)| E{逐个工厂调用}
  E -->|成功| F[typed Output]
  E -->|ProviderFailure| G[记录 provider ID + 领域错误]
  G -->|策略允许且仍有候选| E
  G -->|终止| H[ProviderCreationError]
```

这不是一个把 provider 当作最终服务的容器；解析结果是组合式 resolver。解析先固定候选身份、顺序和回退策略，创建时才传入配置并依次调用工厂。同步实现的循环可直接见 [`ResolvingServiceProvider::create_configured`](../src/registry/resolving_service_provider.rs#L122)。因此，同一 resolver 不会因后续注册或默认选择变更而改变候选集合，但 provider 内部状态仍可变——快照不是深拷贝。

### 2.2 组件职责

| 组件 | 职责 | 关键设计结果 |
| --- | --- | --- |
| `ServiceSpec` / 同异步 capability trait | 将配置、领域错误、输出按服务族静态绑定 | 无需 `Any`、字符串 service key 或全局弱类型容器 |
| `ProviderMetadata` + `ProviderDefinition` | 使 provider 自描述：规范 ID、别名、优先级、工厂 | 元数据和工厂分离，领域接口仍由业务 crate 定义 |
| `ProviderCatalog` | 注册、索引、默认选择、解析 | 一个 `RwLock` 保护不可分割的目录状态 |
| `ProviderSelection` | `named`、严格/宽松 `chain`、`auto` | 选择目标与 creation fallback policy 是两个正交维度 |
| `ResolvingServiceProvider` / async 版本 | 持有候选快照、按策略创建 | 不在锁内执行用户代码或 await |
| `ProviderFailure` / `ProviderCreationError` | 叶子失败分类和聚合诊断 | 保留实际尝试的 provider ID 及领域错误 |

目录内部维护三套互补视图：按 canonical ID 的条目及自动排序索引、selector 到 canonical ID 的映射、成功注册顺序；默认选择与 `sealed` 状态同样在锁内。[`RegistryInner`](../src/registry/internal/registry_inner.rs#L25) 因而它能同时提供按名称查找、别名查找、稳定的优先级自动选择和注册序枚举。

### 2.3 并发与一致性边界

注册先在锁外调用不受信任的 `descriptor()`，随后在一个写锁内检查所有 canonical ID/alias 冲突，并写入所有索引；冲突时本次 provider 不部分可见。[`ProviderCatalog::register_shared`](../src/registry/internal/provider_catalog.rs#L71) 这避免了元数据回调重入造成的死锁，也实现了本次注册的原子性；但回调本身已经完成的重入修改不会回滚——这是合理且已文档化的边界。

解析也在一个读快照内完成；`resolve_default_snapshot()` 同时捕获 default selection 和候选，避免“先读默认值、后解析”在并发修改下混配。[`ProviderCatalog::resolve_default_snapshot`](../src/registry/internal/provider_catalog.rs#L180) 更重要的是，factory 调用和异步 future 的轮询发生在锁外；异步 resolver 在每个候选完成后才决定是否尝试下一项。[`AsyncResolvingServiceProvider::create_configured`](../src/registry/async_resolving_service_provider.rs#L120)

## 3. 值得肯定的设计选择

1. **显式装配与可选发现并存。** 默认路径由应用显式注册；启用 `inventory` 时，应用仍
   需决定哪些 provider crate 被固定链接、何时构建、是否追加有状态 provider、如何选择
   与何时封存。仅在 `Cargo.toml` 中声明依赖不会使 provider 自动可见。这很适合安全边界
   明确、部署差异大的基础设施库，也使测试可在局部 registry 中完成。
2. **选择与构造分离。** `named`/`chain`/`auto` 解决“试谁、按何顺序”；`Never`/`OnAbsence`/`OnAnyError` 解决“失败后是否继续”。普通 registry 往往把这两层混成单个 `get()`，从而无法解释为什么没有尝试下一个 provider。
3. **失败分类不是布尔回退。** `Unsupported`、`Unavailable`、`InvalidConfiguration`、`InitializationFailed` 使默认 `OnAbsence` 不会把用户配置错误掩盖成“换个后端试试”。这把可靠性策略显式放进 API，但要求每个 provider 正确分类。
4. **可复现的选择和诊断。** `auto` 的顺序是 priority 降序、canonical ID 升序，不依赖哈希遍历；链式选择对重复 provider 去重并保留首次顺序。[`resolve_from_inner`](../src/registry/internal/provider_catalog.rs#L267) 这对配置审计、测试和故障复现很有价值。
5. **同步/异步语义同构且不绑 executor。** async API 只要求 `Send` 的 boxed future，输出为 `Send + 'static`，没有把 Tokio、async-std 等运行时漏进公共 ABI。[`ProviderFuture`](../src/service/provider_future.rs#L13) 这是一种库基础设施应有的克制。
6. **初始化完成后可封存。** 当前代码提供共享、幂等的 `seal()`，阻止后续注册及默认选择修改；这对启动后配置冻结很实用。版本与 API 约束应以 0.13 的 README、用户指南及 Rust API 文档为准。

## 4. 与主流 Rust 方案的比较

这里的“主流”按机制分层，而不是假设存在一个占统治地位的通用 SPI crate：Rust 生态通常选择直接 trait 注入、链接期聚合或 ABI 插件三条路线。

| 方案 | 发现与加载模型 | 与 rs-spi 相比的优势 | 相对 rs-spi 的短板 | 最适合的场景 |
| --- | --- | --- | --- | --- |
| **直接 trait + 构造函数/参数注入** | 无 registry；调用者显式传 `Arc<dyn Trait>`、泛型或具体值 | 最小 API/依赖/运行时成本；所有依赖在类型签名中可见；无全局可变目录 | 不提供多 provider 的名称、优先级、链式回退与聚合诊断；这些逻辑很容易散落在业务层 | 单实现或调用点本就掌握依赖的库；这是 rs-spi README 所建议的默认方案 |
| **rs-spi** | 默认显式 runtime 注册；可选 `inventory` feature 读取最终二进制中已链接的 provider factory | 类型化服务族；选择与构造分离；一致的 sync/async 回退、快照和错误链；可将链接期发现纳入同一原子构建路径 | 注册/锚定仍有样板；无动态加载/对象缓存；配置和错误必须在一个服务族内统一 | 多可选后端、应用决定部署策略、需明确 fallback 的库生态；可选用静态扩展 crate 贡献 factory |
| **`inventory`** | 链接期收集 `submit!` 的静态条目；最终二进制迭代 | 扩展 crate 不需要中央注册表；适合 handler、测试、schema 等“贡献声明” | 仅负责发现，不定义选择、优先级、并发目录、构造、异步或失败协议；Cargo 依赖不等于条目进入最终链接图 | 希望静态插件自行声明、条目可用 `'static` 表示的场景；也是 rs-spi `inventory` feature 实际采用的发现层 |
| **`linkme` distributed slice** | linker 将散布的 `static` 元素拼为只读连续切片 | 极低运行期开销，跨平台的分布式静态表；不需启动期可变 registry | 元素必须适于静态初始化；顺序不是 rs-spi 这种显式优先级/ID 协议；仍不解决构造、选择与错误语义 | 编译期命令表、codec 表、测试/基准登记；也可用来给 rs-spi 批量登记 factory |
| **`shaku` 等 DI 容器** | 宏声明 module、component 与依赖图，编译期检查一部分错误 | 把多服务依赖关系和构造图整体表达，可检出环；适合应用 composition root | 比 rs-spi 更侵入业务类型和构造方式；不天然等同于“同一服务多个后端 + 失败时按策略尝试” | 应用内有稳定、较大的对象图；不是跨 crate 后端选择的最小抽象 |
| **`abi_stable` / `dynamic-plugin`** | 从动态库加载二进制插件，经 FFI/stable-ABI 包装调用 | 可安装、更新或由第三方提供而不重编 host；解决 rs-spi 明确不处理的物理插件边界 | FFI-safe 类型、版本/布局、分发、加载错误和卸载语义显著复杂；不能把普通 Rust trait object 直接跨边界传递 | IDE、宿主程序、可下载扩展、独立发布插件；不应为同一 Cargo 二进制内的后端选择引入 |

外部事实依据：[`inventory` 的 `collect!`/链接期插件注册说明](https://snix.dev/rustdoc/inventory/index.html)、[`linkme` 的 distributed slice 定义及跨平台支持](https://docs.rs/linkme/latest/linkme/)、[`shaku` 的编译期 DI 定位](https://docs.rs/shaku/latest/shaku/)、[`abi_stable` 的加载期布局检查与 FFI-safe trait object](https://docs.rs/abi_stable/latest/abi_stable/)，以及 [`dynamic-plugin` 的 host/client 与 C-compatible 接口模型](https://docs.rs/dynamic-plugin/latest/dynamic_plugin/)。Rust Reference 也明确说明 Rust ABI **不提供稳定性保证**，这正是后二者比 rs-spi 更复杂的根本原因。[Rust Reference](https://doc.rust-lang.org/nightly/reference/items/external-blocks.html)

### 可组合，而非互斥

`inventory` 已是 rs-spi 的可选集成：服务契约 crate 声明与服务族绑定的 collection，
provider crate 提交 factory，应用用 `use provider_friendly as _;` 等锚点决定链接集合，
再调用 `build_registry()`。rs-spi 随后完成 ID 冲突校验、选择、快照和 fallback；注册
冲突时构建不会返回部分 registry。factory 和 `descriptor()` panic 原样传播。`linkme`
仍可作为上层来源，但需要适配为同样的显式构建步骤。无论来源为何，条目发现/链接顺序
都不是自动选择顺序，后者固定为优先级降序和规范 ID 升序。

动态插件场景也可采用两层结构：ABI 插件层只暴露稳定的 factory/descriptor 协议，宿主把成功加载的适配器注册进 rs-spi。但这是一项新产品能力，需要单独设计 ABI、版本协商、隔离、资源所有权和不可安全卸载等规则；绝不应把现在的 `dyn ProviderDefinition<S>` 当作跨动态库 ABI。

## 5. 关键流程与失败语义

### 流程 A：注册与封存

`provider.descriptor()`（锁外） → 获取写锁 → 检查 `sealed` → 校验 ID 与全部 alias 的所有权 → 同时写 selector 索引、自动排序索引和注册序 → 返回。任何 selector 冲突都不插入本 provider；`seal()` 后注册与默认选择变更均返回 `RegistryMutationError::Sealed`。代码在进入 `descriptor()` 前后都检查封存状态，因而并发 `seal` 不会留下绕过封存的插入窗口。[`ProviderCatalog`](../src/registry/internal/provider_catalog.rs#L71)

### 流程 B：选择快照

`named` 从 selector 索引得到一个条目；严格 chain 聚合所有未知 selector 并报错，宽松 chain 忽略未知但不允许最终为空；`auto` 遍历有序索引。[`resolve_from_inner`](../src/registry/internal/provider_catalog.rs#L267) 所得 resolver 拥有 `Box<[RegistryEntry]>`，所以目录变更仅影响未来解析，不会改变历史 resolver。

### 流程 C：构造与回退

工厂返回第一个成功 output 即结束。失败则记录 canonical ID 与 `ProviderFailure<E>`；若没有候选剩余，返回 exhausted 聚合错误；仍有候选但 policy 不许可时返回 stopped-by-policy；否则继续。provider panic 不会被转成可回退失败。异步版本一次只 await 一个 provider future；取消会 drop 当前 future，不执行下一次 fallback，也不承诺回滚外部副作用。[设计契约](design.md#selection-and-failure-state-machine)

## 6. 成本、风险和建议的收敛方向

### 目前的主要限制

1. **服务族配置模型偏统一。** 一个 `ServiceSpec` 只有一个 `Config` 和一个 `Error`。这对共同配置很干净，但 provider 特有配置、多个错误域或运行时 capability 协商只能由上层定义 enum/配置视图/适配器承担。不要急于在 SPI 中加入 `Any` 配置；那会破坏它最重要的静态契约。
2. **异步对象安全的成本。** `ProviderFuture` 是 `Pin<Box<dyn Future + Send>>`，每次异步构造通常有一次堆分配，且 `Config: Sync`、future/输出 `Send` 排除了 local executor 和 `!Send` 资源。[`AsyncServiceSpec`](../src/service/async_service_spec.rs#L25) 对跨线程基础设施这是可接受的默认；对极端热路径或单线程 UI/嵌入式系统则不一定合适。
3. **回退的正确性依赖 provider 作者。** 将“请求不支持/资源不可用”误标为 `InitializationFailed` 会过早终止；把错误误标为 absence 又可能掩盖真实配置问题。应在每个领域 SPI 包中给出错误分类规范和契约测试，而不是把类别继续泛化。
4. **目录是共享可变状态。** `Arc<RwLock<_>>` 和 snapshot 已处理内部一致性，但不替调用者决定 registry 的作用域。长期运行的应用应在 composition root 创建实例、完成注册后 `seal()`，并避免把可写 registry 当隐式全局单例。
5. **未观测运行表现。** 项目有 Criterion bench 和 fuzz target，但本评估没有执行，不能宣称锁竞争、解析或 boxed future 的实际性能。设计文档中的性能结论应视作项目作者给出的历史测量，需要在目标平台复测后才可用于容量决策。

### 文档版本一致性

本项目的 README、用户指南和设计说明以 0.13 API 为基线。注册表支持启动配置完成后的不可逆 `seal()`，没有 `unregister` 或 `unseal`；相关行为以 [`ProviderRegistry::seal`](../src/registry/provider_registry.rs#L174) 及其 Rustdoc 为准。

### 采用判断

- **应采用 rs-spi**：一个领域服务有 2+ 可选后端，部署/配置决定顺序，且“不可用可回退、配置错误应停下”是业务语义；尤其适用于 Qubit 内多个 domain crate 共享同一选择模型。
- **应只用 trait 注入**：只有一个实现，或调用点天然知道具体实现；引入 registry 只会制造间接层。
- **启用内置 `inventory`，或另行适配 `linkme`**：provider 来自多个静态 extension crate，中央应用不想维护长注册列表；仍须保留显式锚定、封存和冲突处理。
- **改用 ABI 插件框架**：用户需要把未在编译期链接的第三方二进制放入目录后加载。此时 rs-spi 可保留为 host 内的选择层，但不能独自完成目标。

## 7. 阅读路线与研究边界

建议后续阅读顺序：先读 [README](../README.zh_CN.md) 了解使用者模型，再读[设计说明](design.zh_CN.md) 的契约，然后沿 `ProviderCatalog → ResolvingServiceProvider → FallbackState` 追实现，最后看 `tests/registry` 与 `tests/selection` 验证边界。下游消费者可用于进一步验证领域适配层如何使用 SPI，同时保持业务专用职责位于适配层。
