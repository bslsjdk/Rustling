# Rustling-090 整合包移植分层盘点（48 个模组 · 性能优化模组已按用户要求排除）

## 0. 口径与判据

- **权威清单**：`docs/modpack-list.md:1-70`，48 个 1.21.1 NeoForge Create 科技包模组，标题与说明明确「只移植这 48 个」（`docs/modpack-list.md:1-6`）。真实包目录里另有 18 个性能/辅助模组（BadOptimizations、ImmediatelyFast、c2me、modernfix、noisium、createbetterfps、iris、sodium、lithium、ferritecore、entityculling、spark、memguard、Clumps、CreateLazyTick、carryon 等），**按用户要求全部排除，不进本报告**。
- **取证方式**：48 个 jar 均可在 `/storage/emulated/0/1.21.1mc/versions/1.21.1梦/mods/` 找到；本报告逐 jar 读取 `META-INF/neoforge.mods.toml` 的 `license` / `modId` / `[[dependencies.*]]`，下表标 `[jar]` 即该元数据。
- **分层定义**：`docs/modpacks.md:67-76`（data=仅注册表+搬运数据；logic=+行为回调/服务端事件/配置；machines=+方块实体/菜单/能力/联机；deep=+实体 AI/自定义渲染/新 hook；client=客户端接口；none=不用移植）。
- **分类规则**：`docs/modpacks.md:638-643`；**carry-over 边界**：`docs/modpacks.md:652-672`（事实性 JSON 可搬，ARR/NC 美术不可搬）。
- **许可判定**：`crates/horizon-port-pack/src/report.rs:98-131`（permissive / copyleft / restricted(NC-ND) / ARR / custom）；`docs/modpacks.md:112-120`。引擎为 **MIT**（`LICENSE:1`），copyleft 代码不能直接并入。
- **来源标记**：`清单#N(Lx)`=`docs/modpack-list.md` 第 N 项（第 x 行，N+8）；`[jar]`=该 jar 的 `neoforge.mods.toml`；`[CSV]`=`docs/modpacks/atm10-mods.csv`；`[内置]`=引擎源码；`[规则]`=`docs/modpacks.md:638-643`。

## 1. 逐模组分层总表（48）

### 1.1 库 / 配置（11）→ 全部 none

| # | 模组 | 分层 | 判据与来源 | 许可 `[jar]` |
|---|---|---|---|---|
| 1 | architectury | none | 库，CSV `library,none`；清单#1(L9) | GNU LGPLv3 / LGPL-3（copyleft，不用代码） |
| 2 | balm | none | 库，CSV `library,none`；清单#2(L10) | **All Rights Reserved** |
| 3 | cloth-config | none | 配置库→config schema，`docs/modpacks.md:94-95`；清单#3(L11) | GNU LGPLv3 |
| 18 | flerovium | none | 库（Create/sodium 系前置）；清单#18(L26) | LGPL3.0 |
| 19 | ftb-library | none | 库（FTB 前置，依赖 architectury）；清单#19(L27) | **ARR** |
| 22 | fzzy_config | none | 配置库，依赖 kotlinforforge；清单#22(L30) | TDL-M（custom） |
| 28 | kotlinforforge | none | 库，CSV `library,none`；清单#28(L36) | 未声明（需读内嵌 LICENSE） |
| 38 | mezz_config | none | 库，CSV `library,none`；清单#38(L46) | MIT |
| 40 | particle_core | none | 粒子工具库，依赖 kotlinforforge/fzzy_config；清单#40(L48) | MIT |
| 41 | ritchiesprojectilelib | none | 弹射物工具库；清单#41(L49) | MIT |
| 43 | sophisticatedcore | none | 库，CSV `library,none`；依赖 jei；清单#43(L51) | **ARR** |

> 库/配置全部由引擎 mod API 与 config schema 替代，**无需移植也不引用其代码**（`docs/modpacks.md:92-95`、`report.rs:154`）。

### 1.2 客户端 / UI（6）

| # | 模组 | 分层 | 判据与来源 | 许可 `[jar]` |
|---|---|---|---|---|
| 24 | Jade | client | client-ui→client，`docs/modpacks.md:61,638`；清单#24(L32) | **CC-BY-NC-SA-4.0 → restricted** |
| 25 | JadeAddons | client（依赖 Jade） | client-ui 集成；清单#25(L33) | **ARR** |
| 26 | jei | **none**（内置配方书替代） | `docs/modpacks.md:635`「jei: recipe viewer built in」；`crates/horizon-client/src/recipe_book.rs:1-5`；清单#26(L34) | MIT |
| 39 | nowheel | client（输入微调） | client 接口 `docs/modpacks.md:75`；依赖 create/entityculling/cloth_config；清单#39(L47) | MIT |
| 47 | xaerominimap | **none**（内置小地图替代） | `crates/horizon-client-render/src/minimap.rs`；清单#47(L55) | **ARR** |
| 48 | xaeroworldmap | **none**（内置 BlueMap 远景替代） | `docs/graphics.md:111`、`crates/horizon-client-render/src/bluemap.rs`；清单#48(L56) | **ARR** |

### 1.3 Create 核心 + 附属（14）

| # | 模组 | 分层 | 判据与来源 | 许可 `[jar]` |
|---|---|---|---|---|
| 5 | create 6.0.10 | **none（核心已内置）/ deep（版本 API 适配）** | `crates/horizon-client/src/game/starling/kinetic_machines.rs:1-8`（6.0.10 MIT 核心，26.3 机制）；同类 belts/basins/fans/chutes/funnels/arms/bearings/saws；`docs/modpacks.md:101` 将 Create 列 deep；1.21.1→26.3 差异 `docs/modpacks.md:657-661`；清单#5(L13) | `Read attached LICENSE.md` → 按 `licenses/create-MIT.txt:1-5`：**代码 MIT、美术 ARR** |
| 4 | copycats | deep | CSV `content,deep`；依赖 create；清单#4(L12) | **All Rights Reserved** |
| 6 | create-enchantment-industry | deep | CSV `content,deep`；依赖 create/create_dragons_plus/touhou_little_maid（跨生态）；清单#6(L14) | LGPL-3.0-or-later（copyleft） |
| 7 | create-new-age | deep | 机器+电力，`docs/modpacks.md:73,643`；依赖 esl/create/computercraft；清单#7(L15) | BSD-3-Clause |
| 8 | create_jetpack | deep | 飞行装备=实体/渲染/输入；依赖 kotlinforforge/create/flightlib；清单#8(L16) | 外链 GitHub LICENSE.md（需核实） |
| 9 | create_mechanical_spawner | deep | 自定义生成/实体；依赖 create/mechanicals；清单#9(L17) | LGPL3（copyleft） |
| 10 | create_power_loader | logic | 充能能力，`docs/modpacks.md:73`；依赖 create；清单#10(L18) | MIT |
| 11 | createbigcannons | **deep（做不了，见 §4）** | 炮弹/渲染+多前置；依赖 create/ritchiesprojectilelib/copycats 等；清单#11(L19) | **MIT and CC BY-NC-SA → 美术 restricted** |
| 12 | createdieselgenerators | deep | 发电机=方块实体+流体；依赖 create/struts；清单#12(L20) | MIT |
| 13 | CreateDragonsPlus | deep | CSV `content,deep`；依赖 create；清单#13(L21) | LGPL-3.0-or-later |
| 14 | createoreexcavation | deep | 挖掘机构；依赖 create/kubejs；清单#14(L22) | MIT |
| 15 | createtreadmill | machines | 动力方块实体，`docs/modpacks.md:73`；依赖 create；清单#15(L23) | MIT |
| 16 | createwheelsuponchairs | data/logic | 载具变体，注册+行为 `[规则]`；依赖 create/cloth_config；清单#16(L24) | MIT |
| 37 | mechanicals | **none（纯库，实证修正）** | jar 内 0 blockstate/0 recipe/106 class，无方块无配方，是 spawner 前置库；依赖 jei/kubejs（库性质）；清单#37(L44) | LGPL3 |

> **版本红线**：内置实现移植的是 Create **6.0.10 的 MIT 源码**但按 26.3 机制运行（`kinetic_machines.rs:4-8`）；48 清单里的附属按 1.21.1 Create API 编译，不能直接加载，须按原生 mod API 重写（`docs/modpacks.md:657-661`）。`flywheel`（渲染后端）/ `ponder`（教程场景）由引擎 gfx / UI 层覆盖，不单列移植项。

### 1.4 车万女仆生态（10：本体 + 9 附属）

| # | 模组 | 分层 | 判据与来源 | 许可 `[jar]` |
|---|---|---|---|---|
| 45 | touhoulittlemaid 1.5.3 | **deep（做不了，见 §4）** | 自定义生物+AI+GeckoLib 动画模型，`docs/modpacks.md:74,206-207`；清单#45(L52) | **MIT / CC BY-NC-SA 4.0 → 非商业+相同共享** |
| 29 | littlemaidmoreaction | deep | 行为扩展；依赖 touhou_little_maid/cloth_config；清单#29(L36) | **All rights reserved** |
| 30 | maid_restaurant | deep | 餐厅方块+任务；依赖 touhou_little_maid/kaleidoscope_cookery；清单#30(L37) | BSD 3-Clause |
| 31 | maid_restaurant_storage | deep | 容器+任务；依赖 maid_restaurant/refinedstorage 等；清单#31(L38) | MIT |
| 32 | maidaddition | deep | 依赖 touhou_little_maid/create/ponder/flywheel；清单#32(L39) | **ARR** |
| 33 | maidbeacon | deep | 信标方块+效果；依赖 touhou_little_maid；清单#33(L40) | `${mod_license}`（未展开，需核实） |
| 34 | maidtask | deep | 任务/AI；依赖 touhou_little_maid；清单#34(L41) | MIT |
| 35 | MaidUseHandCrank | logic | 女仆操作 Create 曲柄；依赖 touhou_little_maid/create/create_connected/sable；清单#35(L42) | **MIT / CC BY-NC-SA 4.0** |
| 36 | maid_useful_task | deep | 任务扩展；依赖 touhou_little_maid/playerrevive；清单#36(L43) | MIT |
| 44 | touhou_little_maid_vanilla_companion | deep | 集成附属；依赖 touhou_little_maid；清单#44(L51) | **ARR** |

> 整组依赖本体；本体不落地则 9 个附属全不做。本体还与 `create-enchantment-industry`、`MaidUseHandCrank`、`maidaddition` 形成**跨生态依赖**，顺序见 §3。

### 1.5 内容 / 玩法 / 服务端（7）

| # | 模组 | 分层 | 判据与来源 | 许可 `[jar]` |
|---|---|---|---|---|
| 17 | FarmersDelight | machines | CSV `content,machines`；依赖 crafttweaker；清单#17(L25) | MIT |
| 20 | ftb-quests | logic | 任务数据+服务端事件；依赖 architectury/ftblibrary/ftbteams；清单#20(L28) | **ARR** |
| 21 | ftb-teams | logic | 队伍/服务端 API；依赖 architectury/ftblibrary；清单#21(L29) | **ARR** |
| 23 | incontrol | logic | CSV `server-only,logic`；依赖 lostcities/kubejs；清单#23(L31) | MIT |
| 27 | kaleidoscopecookery | machines | 烹饪机器；依赖 kubejs；清单#27(L35) | BSD-3-Clause **+ CC BY-NC-SA 4.0（美术 restricted）** |
| 42 | sophisticatedbackpacks | deep | CSV `content,deep`；依赖 sophisticatedcore；清单#42(L50) | **ARR** |
| 46 | waystones | machines | CSV `content,machines`；依赖 balm；清单#46(L53) | **ARR** |

## 2. 许可红线

| 类别 | 模组（jar 许可原文） | 处置 |
|---|---|---|
| **MIT / BSD 可合法参考代码** | create（仅代码）、farmersdelight、incontrol、jei、mezz_config、nowheel、particle_core、ritchiesprojectilelib、create_power_loader、createtreadmill、createwheelsuponchairs、createoreexcavation、createdieselgenerators、maidtask、maid_useful_task、maid_restaurant_storage、create-new-age(BSD)、maid_restaurant(BSD) | 署名后可参考实现；引擎 MIT（`LICENSE:1`）兼容。 |
| **MIT 代码 + ARR 美术** | **create**（`Read attached LICENSE.md` → `licenses/create-MIT.txt:1-5`） | 代码可移植；`assets/` 不可拷贝。内置 Create 已按此处理，附属只搬逻辑不搬贴图。 |
| **Restricted（NC / NC-SA）** | **touhoulittlemaid**（MIT / CC BY-NC-SA 4.0）、**createbigcannons**（MIT and CC BY-NC-SA）、**Jade**（CC-BY-NC-SA-4.0）、**MaidUseHandCrank**（MIT / CC BY-NC-SA）、**kaleidoscopecookery**（BSD + CC BY-NC-SA 4.0） | 非商业+相同方式共享：商业分发不可；美术/衍生须同协议。`report.rs:113-114` 判为 restricted。 |
| **ARR / All Rights Reserved** | balm、copycats、ftb-library/quests/teams、JadeAddons、littlemaidmoreaction、maidaddition、touhou_little_maid_vanilla_companion、sophisticatedbackpacks/core、waystones、xaerominimap/worldmap | **不得参考代码**；事实性 JSON（配方/标签/掉落）可机械搬运，**美术/模型不可搬运**（`docs/modpacks.md:652-672`）。 |
| **Copyleft（LGPL/GPL）** | architectury、cloth-config、flerovium、create-enchantment-industry、CreateDragonsPlus、create_mechanical_spawner、mechanicals | MIT 引擎不能直接纳入 copyleft 代码；**clean-room 重写或仅数据搬运**，否则衍生须同协议。 |
| **Custom / 未声明** | fzzy_config(TDL-M)、create_jetpack(外链 LICENSE)、maidbeacon(`${mod_license}`)、kotlinforforge(未声明) | 导入时读内嵌 LICENSE，未核实前不碰代码。 |

## 3. 依赖关系与移植顺序（依赖先行）

依赖取自各 jar 的 `[[dependencies.*]]`（含可选前置）。**层内并行、层间串行**（`docs/modpacks.md:647-650`）。

| 层 | 内容 | 说明 |
|---|---|---|
| **0 库/配置** | 11 个库（§1.1） | 全部 none，映射到引擎 API，不占移植工。 |
| **1 核心** | create 6.0.10 | 26.3 机制已内置；仅做 1.21.1→26.3 API/数据适配（`docs/modpacks.md:657-661`）。flywheel/ponder 由引擎覆盖。 |
| **2 Create 浅层** | createwheelsuponchairs(data/logic) → create_power_loader(logic) → createtreadmill(machines) | 均依赖层 1，先验证适配路径。 |
| **2' 女仆链** | touhoulittlemaid → {maidtask, maid_useful_task, littlemaidmoreaction, maid_restaurant → maid_restaurant_storage, maidbeacon, vanilla_companion, MaidUseHandCrank} | 本体先行；`MaidUseHandCrank`/`maidaddition` 还依赖 create/ponder/flywheel，**与 Create 链交叉**。 |
| **2'' 内容工具链** | ftb-library → ftb-teams → ftb-quests；sophisticatedcore → sophisticatedbackpacks；cloth-config → createwheelsuponchairs；ritchiesprojectilelib → createbigcannons | 库先行。 |
| **3 跨生态汇合** | create-enchantment-industry（依赖 create + create_dragons_plus + **touhou_little_maid**） | 必须等 Create 链与女仆链都落地。 |
| **4 深层 / 长周期** | copycats、createbigcannons、createdieselgenerators、createoreexcavation、create-new-age、create_jetpack、CreateDragonsPlus、kaleidoscopecookery、waystones、incontrol、Jade → JadeAddons | 需实体/渲染/新 hook；许可多数受限。（mechanicals 已实证修正为 none 纯库，移出此档） |

**包内缺失前置（jar 元数据声明但 48 清单内没有）**：Create 的 `flywheel/ponder/journeymap/radium/palladium`（其中 sodium/lithium 已被用户排除且由引擎替代）；`kubejs`/`kubejs_create`；`struts`；`esl`/`computercraft`；`flightlib`；大炮的 `framedblocks/curios/sable/simulated/aeronautics/offroad`；`refinedstorage/refinedcooking/ironchest`；`lostcities`；`playerrevive`；`create_connected`；`apotheosis/apothic_*`；xaero 的 `xaerolib/openpartiesandclaims/xaerobetterpvp`。多为可选依赖，功能完整性需先在 Java 侧确认，不凭空补造。

## 4. 推荐第一批 vs 明确做不了

**第一批（快速出成果，许可干净/判据明确）**
1. 11 个库/配置 + jei/xaerominimap/xaeroworldmap → 全部 none，0 移植工。
2. `createwheelsuponchairs`(data/logic, MIT)、`create_power_loader`(logic, MIT) → 验证 Create 附属 API 适配路径。
3. `createtreadmill`(machines, MIT)、`FarmersDelight`(machines, MIT) → machines 标杆 harness。
4. `incontrol`(MIT, logic) → 服务端逻辑标杆；`maidtask`/`maid_useful_task` 虽 MIT 但依赖受限本体，**不进第一批**。

**明确做不了 / 长周期**
- **车万女仆整组（10 个）**：本体 deep（自定义生物+AI+GeckoLib 模型，`docs/modpacks.md:74,206-207`）且美术 CC BY-NC-SA 非商业；9 附属全依赖本体 → **不移植**（除非授权+专项团队）。
- **createbigcannons**：MIT 代码但美术 CC BY-NC-SA，deep + 8 个缺失前置 → 第一批不做。
- **Create 深层附属**：copycats(ARR)、createdieselgenerators、createoreexcavation、create-new-age、create_jetpack 等，deep + copyleft/ARR，长周期。（mechanicals 经 Create 组实证为纯库 none，不在此列）
- **ARR 大系统**：sophisticatedbackpacks、waystones、ftb-quests/teams、Jade/JadeAddons、xaero 地图 → 不得参考代码；xaero 由内置替代，其余最多数据搬运或 clean-room 自研。

## 5. 证据索引

- `docs/modpack-list.md:1-70` — 权威 48 模组清单（性能类已按用户要求排除）
- `/storage/emulated/0/1.21.1mc/versions/1.21.1梦/mods/` — 48 个 jar 本体；本报告 `license`/`modId`/`dependencies` 均取自其 `META-INF/neoforge.mods.toml`
- `docs/modpacks.md:56-76,94-95,101,112-120,638-643,652-672` — 分类/分层/许可/规则/carry-over
- `docs/modpacks/atm10-mods.csv` — CSV 标注行（create/copycats/farmersdelight/jade/jei/waystones 等）
- `crates/horizon-port-pack/src/report.rs:98-131,152-159` — 许可类别与 none 处置
- `crates/horizon-client/src/game/starling/kinetic_machines.rs:1-8` — 内置 Create 6.0.10 MIT 核心
- `licenses/create-MIT.txt:1-5`、`LICENSE:1` — Create 代码/美术边界、引擎 MIT
- `crates/horizon-client/src/recipe_book.rs:1-5`、`crates/horizon-client-render/src/minimap.rs`、`docs/graphics.md:111` — JEI/小地图/世界地图内置替代
