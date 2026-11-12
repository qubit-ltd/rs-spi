# 下游兼容性验证

此命令使用选定的 `qubit-spi` 源码检查本地消费者仓库。它独立于
`rs-spi` 常规 CI 运行，因此普通检查不要求同级消费者仓库存在。

## 前置条件

- Python 3.11 或更高版本、`rustup`，以及已安装的 Rust/Cargo 1.94.0
  工具链。验证器不会安装或更新工具链。可先用
  `rustup which --toolchain 1.94.0 cargo` 检查原生 Cargo 路径。
- workspace 根目录下存在 `rs-fs-registry`、`rs-mime`、`rs-magika` 和
  `rs-event-bus` 仓库，以及它们引用的所有本地路径依赖。
- 当 Cargo 需要下载 crate 时可访问网络。`rs-magika` 默认启用捆绑版 ONNX
  Runtime；其构建可能会下载运行库二进制文件。
- 临时源码快照、Cargo 日志和构建产物所需的磁盘空间。构建产物共用一个临时
  target 目录。
- 验证工具要求 POSIX 进程组能力，并要求 `/proc` 或 `ps` 来确认命令组已停止。
  这是外部验证工具的限制，不改变 `qubit-spi` crate 支持的平台。

## 运行

在 `rs-spi` checkout 中运行；若仓库位于其他位置，请替换路径：

```bash
python3 scripts/check-downstream.py \
  --workspace-root /data/working/qubit/rust-common \
  --spi-root /data/working/qubit/rust-common/rs-spi \
  --report /tmp/rs-spi-downstream-report.json
```

消费者集合固定，没有部分成功模式。工具为每个消费者分别使用默认 feature
和全部 feature 执行依赖元数据解析及测试。它注入 Cargo 配置覆盖，使
`qubit-spi` 指向 `--spi-root`，再核对解析图中的包 manifest 路径和来源。测试以
快照中的 lockfile 和 `--locked` 运行。

验证器会将原生 Cargo 1.94.0 复制到自己的临时目录，命名为
`rs-spi-cargo-driver`；不会重命名或修改工具链中的 Cargo。wrapper 直接启动该副本。
这一机制依赖 Cargo 1.94.0 对可执行文件名称和 `CARGO` 的选择行为；升级工具链时，
必须重新核对实现，并通过启动探针和嵌套入口回归。生产探针会构建一个无 Cargo 环境
补丁、仅使用标准库的测试 crate，再分别通过 `PATH`、编译期 `CARGO` 路径和运行期
`CARGO` 执行 `cargo --version`。三个入口都必须被 wrapper 记录后才会启动消费者。
驱动准备或探针失败会终止本次运行，不会回退到其他 Cargo。

wrapper 会记录嵌套 fixture Cargo 命令并检查其解析到的 SPI 来源；它不会拦截使用其他
硬编码 Cargo 路径的程序。每条顶层 metadata 命令预算为 180 秒，启动探针预算为
180 秒，每条消费者 test 命令预算为 1,800 秒。命令结束、失败或超时后，工具会终止并
回收该命令进程组中的剩余进程；升级强制终止前最多再等待两秒。进程组清理失败会使
验证失败，并保留诊断报告和快照。

工具会将消费者、`rs-spi` 以及递归发现的本地路径依赖闭包复制到所有输入仓库
之外的临时目录。快照包含已有的已修改和未跟踪文件，并记录输入 revision、dirty
状态及内容摘要。工具不会编辑源仓库或其 lockfile。若任一输入有未提交修改，结果
代表报告中记录的快照验证，而不只是干净 revision 的验证。

## 查看报告

原生 Cargo 和副本均成功通过版本检查后，JSON 报告会记录原始 Cargo 的路径、版本和
SHA-256、临时驱动路径和 SHA-256、启动探针结果、输入快照、Cargo 覆盖项、每个消费者
和 feature 模式、解析得到的 `qubit-spi` manifest 和 source、命令参数及工作目录、退出
状态、耗时、超时状态和日志路径。如果驱动准备在完成这些检查前失败，请查看报告错误
中的失败路径和原因；只有两个二进制都可成功运行后才会写入摘要元数据。验证器只写自己
的临时副本。探针失败时，查看 `cargo_entry_probe.command.log_path` 和
`cargo_entry_probe.records`，确认哪个入口缺失或失败。命令失败时，报告及临时快照/日志
目录会保留供诊断；成功时，先写报告，再清理工具创建的临时目录。

命令失败时，请检查报告中的日志，并确认 `qubit-spi` manifest 位于 SPI 快照内且
`source` 为 `null`。工具不会改写源仓库或消费者 helper；探针和嵌套回归会观察它们原有
的 `PATH`、编译期 `CARGO` 和运行期 `CARGO` 启动方式。解决缺失的本地仓库、Cargo/网络
错误或原生运行库前置条件后，再判断消费者是否通过验证。此命令提供可复现的本地兼容性
检查；`rs-spi` 标准 CI 不会运行这些下游仓库。
