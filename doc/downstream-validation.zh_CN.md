# 下游兼容性验证

此命令使用选定的 `qubit-spi` 源码检查本地消费者仓库。它独立于
`rs-spi` 常规 CI 运行，因此普通检查不要求同级消费者仓库存在。

## 前置条件

- Python 3.11 或更高版本，以及本 checkout 要求的 Rust/Cargo 工具链
  （当前为 Rust 1.94.0）。
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

通过 `PATH`、编译期 `CARGO` 可执行文件路径或运行期 `CARGO` 环境变量启动的
Cargo 都会经过验证 wrapper。wrapper 会记录嵌套 fixture Cargo 命令并检查其解析到的
SPI 来源；它不会拦截使用其他硬编码 Cargo 路径的程序。每条顶层 metadata 命令的
预算为 180 秒，每条 test 命令为 1,800 秒。命令结束、失败或超时后，工具会终止并
回收该命令进程组中的剩余进程；升级强制终止前最多再等待两秒。进程组清理失败会使
验证失败，并保留诊断报告和快照。

工具会将消费者、`rs-spi` 以及递归发现的本地路径依赖闭包复制到所有输入仓库
之外的临时目录。快照包含已有的已修改和未跟踪文件，并记录输入 revision、dirty
状态及内容摘要。工具不会编辑源仓库或其 lockfile。若任一输入有未提交修改，结果
代表报告中记录的快照验证，而不只是干净 revision 的验证。

## 查看报告

JSON 报告记录工具链版本、输入快照、Cargo 覆盖项、每个消费者和 feature 模式、
解析得到的 `qubit-spi` manifest 和 source、命令参数及工作目录、退出状态、耗时、
超时状态和日志路径。命令失败时，报告及临时快照/日志目录会保留供诊断；成功时，
先写报告，再清理工具创建的临时目录。

命令失败时，请检查报告中的日志，并确认 `qubit-spi` manifest 位于 SPI 快照内且
`source` 为 `null`。解决缺失的本地仓库、Cargo/网络错误或原生运行库前置条件后，
再判断消费者是否通过验证。此命令提供可复现的本地兼容性检查；`rs-spi` 标准 CI
不会运行这些下游仓库。
