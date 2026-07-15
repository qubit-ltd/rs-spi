# Qubit SPI

面向 Rust 的类型安全、显式装配式服务提供者基础设施。

## 模型

应用在启动阶段使用 ProviderRegistryBuilder 注册 Provider，随后调用 build()
得到不可变且可低成本克隆的 ProviderRegistry。运行期通过 ProviderResolver、
ProviderSelection 和 FallbackPolicy 创建服务。

ServiceSpec 同时定义配置类型和完整输出句柄；核心不会在 Box、Arc、Rc 之间
转换。已知下游应在自己的 Spec 中统一选择所需句柄，例如 Arc trait object。

## 安装

~~~toml
[dependencies]
qubit-spi = "0.4"
~~~

## 核心 API

- ServiceSpec::Output：Provider 返回的完整服务句柄。
- ServiceProvider::create：唯一的 Provider 工厂方法。
- ProviderDescriptor：注册时提供 canonical ID、别名和自动选择优先级。
- ProviderRegistryBuilder：仅用于启动期装配。
- ProviderRegistry：不可变 Provider 目录，支持 ID/别名查询。
- ProviderResolver：按选择与回退策略完成创建。
- CreatedService：包含实际胜出的 Provider ID 和服务实例。

自动选择按 priority 降序、Provider ID 升序进行。默认
FallbackPolicy::OnAbsence 只在 Provider 不存在、不支持请求或不可用时继续回退；
初始化失败和配置错误会立即停止。若业务明确要求尽力而为，可选择
FallbackPolicy::OnAnyError。

## 0.4 破坏性迁移

| 0.3 API | 0.4 替代方案 |
| --- | --- |
| ServiceSpec::Service | ServiceSpec::Output |
| create_box / create_arc / create_rc | 单一 ServiceProvider::create |
| Provider 的 descriptor() | 向 Builder 注册时传入 ProviderDescriptor |
| availability() | create() 返回分类的 ProviderError |
| 可变 ProviderRegistry::register | Builder 注册后调用 build() |
| create_auto_* / create_selected_* | ProviderResolver::create |
| register_default | 应用启动期显式装配 |

本版本不提供兼容层。qubit-fs、qubit-mime 与 qubit-magika 的迁移将在各自工作区
独立进行；rs-llmsdk-core 保持 Provider-neutral，不依赖 qubit-spi。
