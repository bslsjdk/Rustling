# Rustling P0：共享核心与 Android 桥接

状态：P0 主机验证通过，完整 APK 构建转 GitHub runner；P1 UI 尚未开始。范围由 [项目计划](rustling-plan.md) 确认。

## 边界与决策

P0 的交付不是新启动器，而是**独立的只读 Rust 核心、JNI/Kotlin 协议和迁移约束**。普通
`LauncherActivity` 仍然直接进入游戏；文件导入、重启恢复、崩溃恢复和 `NativeMods.java`
都没有被新桥接替换。Kotlin facade 只有显式调用时才 `System.loadLibrary("rustling_launcher")`。

新 crate 是 `crates/starling-launcher-core`，输出 `librustling_launcher.so`。它不链接
`horizon-client`、GPUI、wgpu 或游戏渲染器，避免启动器初始化游戏，也避免原生 Mod 替换游戏库
时同时替换启动器业务。Java/Kotlin 继续编译为 JVM 8 字节码，沿用 `javac + d8`，不引入 Gradle。

## 模块审计

| 模块 | 决定 | 原因和下一步 |
|---|---|---|
| `launcher/src/instances.rs` | **抽取，不整体链接** | 文件格式可复用，但依赖 `backend`、注册表整合包、进程 `Command` 和桌面 Trash。`Store::load` 将读取/解析失败静默当空表，`save` 直接覆盖，不能用于 Android 多写入者。 |
| `launcher/src/export.rs` | **抽取算法** | 固定允许列表与 `.mrpack` 写出有复用价值；依赖 browser、horizon-world NBT 和 flate2。Android 实例迁移与原生 Mod/世界策略须先定，不得将旧导出称为完整 Rustling 整合包。 |
| `launcher/src/import.rs` | **抽取安全路径与格式判断** | 路径约束可复用；调用 `instances` 和下载器，依赖桌面选择语义。SAF/content URI 先在 Android 适配层变为受控本地文件。 |
| `launcher/src/modpacks.rs` | **后续按命令拆分** | `horizon-port-pack` 可复用；同时依赖全局 `backend::game_home`、桌面 registry pack 和 `Command`，不能直接当 Android 核心。 |
| `launcher/src/backend.rs` | **保持桌面专用** | 5146 行，包含 GitHub、安装、桌面更新、进程和全局目录；不能通过隐藏 GPUI 变成手机核心。Android 已有 `starling-account`、资源准备和 `WorkService`。 |
| `starling-catalog` | **复用浏览器逻辑，改实例寻址** | 已能编译 Android 目录逻辑，但 `set_game_home` 是一次性 `OnceLock`。P1 前应让安装请求显式携带根目录，不能沿用全局单实例。 |
| `horizon-client-render::mod_list` | **仅下沉文件解析** | `ModSettings` 现在在 `horizon-client-core`，保持开关/按键序列化单一实现。游戏的 `Installed`、`NativeGame` 和编译按钮仍留渲染层。 |
| Android `LauncherActivity` | **扩展现有入口，不在 P0 改路由** | 已有独立进程、文件接收、重启和启动恢复；P1 才加入启动器路由和界面。 |
| `NativeMods.java` / `native_builds.rs` | **复用生产构建路径** | `files/native/current` 是 key，`files/native/<key>/build.json` 才含 commit/mod 列表。APK 时间戳、两次首帧前退出回退和动态库选择不可绕过。 |

## P0 协议

所有请求是 UTF-8 JSON，最大 64 KiB。Rust 的 `dispatch` 与 JNI 使用同一实现；JNI 捕获 panic，
返回协议错误而不是把 Rust panic 抛进 JVM。协议 `schema` 为 1。

```json
{"command":"ping"}
```

```json
{
  "command": "inspect_instance",
  "instanceId": "survival",
  "gameRevision": "<40 lowercase hex>",
  "nativeHooksApi": 46,
  "instanceDir": "/absolute/authorized/instance"
}
```

成功：

```json
{"schema":1,"ok":true,"result":{"kind":"pong","protocol":"rustling.launcher.ping.v1","readOnly":true}}
```

失败：

```json
{"schema":1,"ok":false,"error":{"code":"invalid_request","detail":"...","fields":[]}}
```

稳定错误码：`invalid_request`、`invalid_instance`、`instance_unreadable`、`settings_unreadable`、
`mod_invalid`、`mod_unreadable`、`namespace_conflict`、`revision_mismatch`、`hook_api_unsupported`、
`build_invalid`、`build_unreadable`、`internal_error`。`detail` 供日志，不让 UI 匹配英文句子；
Kotlin 根据 `code` 与 `fields` 选择中文消息。可选字段的缺失与 `null` 都须由消费者显式处理。

### 实例草案与扫描语义

草案包含稳定 `instanceId`、精确 `gameRevision`、可选 `nativeHooksApi`、所选原生构建和模组快照。
模组记录包含 id、显示名、版本、content/native、文件名、可选文件 SHA-256、大小、选中状态、side、
namespaces 与原生兼容信息。开发用模组目录的 `sha256` 是 `null`，不伪造目录哈希。

- 内容模组默认关闭，只有 `mod.<id>:true` 才选中。
- 原生模组默认选中，只有 `mod.<id>:false` 才排除。
- 不同 Horizon revision 且无稳定 Hook 契约：`revision_mismatch`。
- 有 Hook 契约但当前 API 太低：`hook_api_unsupported`；满足 API 时可进入现有构建流程，不能宣称已编译成功。
- 单个坏包标在自己的行上，不隐藏其他模组；根目录或开关文件不可读则整个请求失败。
- 扫描不创建实例、不写 `horizon-mods.txt`、不修改桌面 `instances.json`，也不生成迁移后的 manifest。
- `buildRequired` 只表示启用原生模组且当前没有同 revision 的所选构建；它**不是完整组合匹配证明**。
  P1 必须复用生产的模组集合/build key 逻辑，另列 APK stamp 和加载状态，不能只比较 commit。

## 写入与迁移规则

P0 没有第二写入者。P1 开启写入前必须满足：

1. 默认实例继续使用现有游戏目录，绝不移动世界；旧 `instances.json` 先解析并验证，再原子迁移。
2. 无法读取/解析的配置显式报错并保留，不能当空表覆盖。
3. 新 manifest 带 schema/revision；写入临时文件、flush 后原子 rename，崩溃后恢复上一份有效版本。
4. 桌面 launcher 与 Android 不共享可写实例目录。各自写入自己的元数据，交接使用显式导出。
5. 游戏仍在写 `horizon-mods.txt` 时，启动器采用同一文件锁/修订协议；未实现前只允许游戏写入。
6. 不修改 application ID、签名升级链、shared `Starling` 目录、账号数据或原生 Mod 选择规则。

回滚就是继续走旧 `LauncherActivity` 和旧游戏文件；P0 的库和 facade 没有接管启动，不需要数据回滚。

## 验证记录

| 项目 | 当前证据 |
|---|---|
| Rust 主机扫描与协议 | `cargo test -p starling-launcher-core`：8 个测试通过。覆盖 ping、错误/请求上限、两类模组、坏包、revision/Hook、命名空间冲突、目录与原生 build 指针。 |
| 游戏原开关行为 | `cargo test -p horizon-client-render mod_list::tests --lib`：3 个测试通过。 |
| Kotlin 编译 | Kotlin 1.9.20 实际编译 facade，`javap` 确认 class major 52（JVM 8）。 |
| Android target | `cargo check -p starling-launcher-core --target aarch64-linux-android` 通过。 |
| APK / CI | [Android run 44784413009](https://github.com/bslsjdk/Rustling/actions/runs/44784413009) 使用 x86_64 GitHub runner 生成 Preview，不发布 Release；[checks run 44784420714](https://github.com/bslsjdk/Rustling/actions/runs/44784420714)。 |
| 真机与旧路径 | 未跑真实游戏前，不宣称 UI、账号、存档、导入、重启和原生 Mod 已在设备回归。 |

复现环境：Rust stable 1.100.0；JDK 21；Kotlin 1.9.20；项目锁定 Android platform 36、
build-tools 35.0.0、NDK 28.2.13676358。Rust 测试只用临时目录，不读真实玩家目录。

本机是 ARM64 Ubuntu，官方 NDK/build-tools 只提供 x86_64 宿主编译器；QEMU 的显式调用虽能运行工具，
但完整构建成本不应由手机承担，已按项目所有者要求停止本地全量编译。Android CI 固定 `ubuntu-latest`
与 `--preview`，application ID 为 `io.github.juanluci.starling.preview`，可与正式版并存。

## 尚未完成与 P1 入口

- 实际 APK 构建、DEX 内容检查和至少一台真实 ARM64 设备的 `ping`/只读扫描结果必须记录后，P0 才能完全验收。
- 当前 Kotlin facade 返回原始 JSON；P1 引入有界 DTO 解析、稳定中文错误资源和后台任务，不在 UI 线程扫描/哈希。
- P1 首个纵向切片：启动器入口 → 当前默认实例 → 模组只读列表 → 启动原游戏；不改默认目录。
- 多实例写入、`starling-catalog` 根目录参数化、完整 build 组合指纹和原生构建状态是后续独立小 PR。
- 仓库 Issues 被禁用（创建返回 HTTP 410），无法按常规 issue `/claim`；已检查无重复开放 PR，后续 PR
  首行注明代理归属和 P0 范围。没有把禁用 Issues 当成跳过验证的理由。
