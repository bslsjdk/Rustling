# Create 生态移植深挖（Rustling-090 · 48 模组口径）

> **证据口径**：本轮只读本地文件，不联网。所有许可与分层结论来自对本地 jar 的 `zipfile` 普查：
> `/storage/emulated/0/1.21.1mc/versions/1.21.1梦/mods/`（66 个，含全部 Create 系），读取 `META-INF/neoforge.mods.toml` 的 `license` / `modId` / 依赖段、jar 内 LICENSE 原文，并统计 blockstates / recipe / models / textures / class / mixin 数量。
> **权威清单**：`docs/modpack-list.md:1-70`（48 个，用户已排除性能优化模组）。**总盘点**：`docs/modpack-triage.md:42-61, 118-130`。
> **Create-Horizon 案例**：仅引用 lead 调研结论（225/246 方块、代码 MIT、美术从本地 jar 读），未联网复核。

## 0. 与总盘点的对齐与两处实证修正

总盘点结论成立：Create 本体内核已内置（`none` 内核 / `deep` 适配），Create 附属多为 deep，`createwheelsuponchairs`（data/logic）、`create_power_loader`（logic）、`createtreadmill`（machines）三个最浅，作为第一批；重附属长周期。本轮 jar 实证补充两点：

| 项 | 总盘点 | jar 实证 | 处置 |
|---|---|---|---|
| **mechanicals** | deep（Create 附属机械，#37 L59） | `LGPL3`，**0 blockstates / 0 recipes / 4 models / 106 class / 0 mixin**，依赖 jei+kubejs → 纯库（与 `create_mechanical_spawner` README 所述 "Mechanicals Lib" 一致） | 改判 **none（库映射）**，不是移植项；它只是 `create_mechanical_spawner` 的前置 |
| **createbigcannons** | LGPL-3.0⚠（copyleft） | `MIT and CC BY-NC-SA (see github for details)` | **代码 MIT 可参考，美术 CC BY-NC-SA（非商业）受限**；比总盘点更明确：不是纯 copyleft 代码，但美术红线同样让它进不了第一批 |

其余 jar 实证与总盘点方向一致（copycats=ARR、create-new-age=BSD-3-Clause、enchantment-industry / dragonsplus / mechanical_spawner=LGPL 系、create 本体=MIT 代码+ARR 美术）。

## 1. 逐模组分层、许可与数据规模（jar 实证）

| # | 模组 | 分层 | jar `license` 原文 | 数据规模（bs/rec/mdl/tex/class/mixin） | 判据与处置 |
|---|---|---|---|---|---|
| 5 | create 6.0.10 | **none（核心）/ deep（适配）** | `Read attached LICENSE.md`，jar 内 `LICENSE.md_create-1.21.1` 原文：**Assets = All Rights Reserved（`./src/main/resources/assets/`）；其他全部 MIT** | 643 / 1884 / 2684 / 1295 / 2692 / 99 | 内核已内置；只做 1.21.1→26.3 数据升级 + API 适配（§2-3）。可改代码，**不可碰 `assets/`** |
| 4 | copycats | deep | `All Rights Reserved` | 46 / 68 / 93 / 12 / 393 / 80 | ARR：不得参考代码，**只搬事实性 JSON（配方/标签/掉落）；美术与 CT 逻辑放弃或 clean-room**（`docs/modpacks.md:100,652-672`） |
| 6 | create-enchantment-industry | deep | `LGPL-3.0-or-later` | 20 / 48 / 90 / 58 / 464 / 52 | copyleft → MIT 引擎不纳入其代码；clean-room 或仅数据；且依赖 #13 |
| 7 | create-new-age | deep | `BSD-3-Clause` | 36 / 88 / 132 / 88 / 178 / 7 | 许可宽松可读，但电力/反应堆/热系统是全新能力体系，重做成本高 |
| 8 | create_jetpack | deep | 自定义许可 URL（`LICENSE.md`） | 2 / 4 / 9 / 5 / 26 / **0 mixin** | 规模小但许可需人工确认；飞行/悬停要玩家运动 hook，deep |
| 9 | create_mechanical_spawner | deep | `LGPL3` | 34 / 66 / 68 / 101 / 52 / 31 | copyleft + 实体生成/AI；且依赖 mechanicals（已判 none） |
| 10 | create_power_loader | **logic（→machines 边界）** | `MIT` | **4 / 2 / 21 / 11 / 65 / 8** | **第一批**：许可干净、体量小，核心是区块加载能力（§4.2） |
| 11 | createbigcannons | deep | `MIT and CC BY-NC-SA` | 139 / 151 / 487 / 374 / **855** / 69 | 代码 MIT 可读，美术 NC 受限；炮弹实体+弹道+多方块，**先放弃**（§5） |
| 12 | createdieselgenerators | deep | `MIT` | 65 / 137 / 297 / 263 / 239 / 39 | 许可干净但流体+能量machines/deep，重附属（§5） |
| 13 | CreateDragonsPlus | **logic / machines** | `LGPL-3.0-or-later` | 56 / 119 / 118 / 17 / 370 / 168 | copyleft 只搬数据；本质是风扇处理配方扩展，内置 fan processing 可承接，**第二梯队首选**（§5 例外） |
| 14 | createoreexcavation | deep | `MIT` | 6 / 42 / 18 / 26 / 145 / 0 | 许可干净；但无限矿脉 worldgen + 抽取器，deep，先放弃 |
| 15 | createtreadmill | **machines** | `MIT` | **2 / 2 / 10 / 8 / 44 / 6** | **第一批**：许可干净、极小，踩踏→应力走内置 kinetic 网络（§4.3） |
| 16 | createwheelsuponchairs | **data/logic** | `MIT` | **1 / 0 / 6 / 1 / 74 / 78** | **第一批的数据部分**；但 78 Mixin 与 0 配方反差大，行为需 hook（§4.1） |
| 37 | mechanicals | **none（库）** | `LGPL3` | **0 / 0 / 4 / 16 / 106 / 0** | 纯库，映射到 API，不写 port（实证修正，见 §0） |
| 18 | flerovium（库/配置组） | none | `LGPL3.0` | **0 / 0 / 0 / 0 / 34 / 24** | 无内容 + Mixin 密集，优化/库性质；用户已排除性能模组，与总盘点一致 |
| 41 | ritchiesprojectilelib（库/配置组） | none | `MIT` | 0 / 0 / 0 / 1 / 34 / 4 | 纯库；是 CBC 与女仆系的前置，映射到 API |

> 许可红线按 `docs/modpacks.md:126-181` + `docs/modpack-triage.md:92-104`：MIT 可署名参考；copyleft 不进 MIT 引擎（clean-room）；ARR 只搬事实性数据；美术一律不搬。
> `create_jetpack` 的自定义许可、`createbigcannons` 的 NC 美术属于"未放行前不碰代码/贴图"。

## 2. 内置内核覆盖了什么（Create 本体适配的地基）

**源码证据**（均为本地）：

- `crates/horizon-client/src/game/starling/kinetic_machines.rs:1-8`：注释明确 "Ported from Create 6.0.10 (MIT; licenses/create-MIT.txt)"，实现 drill / crushing wheel（对应 `BlockBreakingKineticBlockEntity`、`CrushingWheelControllerBlockEntity`）。
- `kinetic_machines.rs:85-92`：`RECIPE_TYPES = crushing, milling, pressing, splashing, haunting, cutting`。
- `crates/horizon-client/src/game/starling/basins.rs:38`：`create:mixing`、`create:compacting`（以及原版 crafting）。
- 同目录子模块：`kinetics.rs`（网络/应力/RPM）、`belts.rs`、`funnels.rs`、`chutes.rs`、`basins.rs`、`fans.rs`、`processing.rs`、`saws.rs`、`bearings.rs`、`arms.rs`、`machine_entities.rs`、`falling_trees.rs`。
- 内容侧声明：`docs/modpacks.md:283-336`（kinetic parts 全表、machine 类型、stress/capacity、加工配方）。

**jar 内 `data/create/recipe/<type>/` 真实文件数 × 内置支持：**

| 配方类型 | jar 文件数 | 内置支持 | 依据 |
|---|---:|---|---|
| milling | 246 | ✅ | `kinetic_machines.rs:87` |
| crushing | 211 | ✅ | `kinetic_machines.rs:86` |
| splashing | 52 | ✅ | `kinetic_machines.rs:89` |
| pressing | 43 | ✅ | `kinetic_machines.rs:88` |
| cutting | 32 | ✅ | `kinetic_machines.rs:91` |
| haunting | 22 | ✅ | `kinetic_machines.rs:90` |
| mixing | 14 | ✅ | `basins.rs:38` |
| compacting | 7 | ✅ | `basins.rs:38` |
| deploying | 167 | ❌ | 内置无 Deployer |
| filling / emptying | 19 / 5 | ❌ | 内置无 Spout / Item Drain |
| item_application | 8 | ❌ | 随 Deployer |
| mechanical_crafting | 4 | ❌ | 内置无合成器 |
| sequenced_assembly | 3 | ❌ | 内置无顺序装配 |
| sandpaper_polishing | 1 | ❌ | 内置无砂纸 |
| crafting / smelting / blasting / smoking / campfire_cooking | 369 / 37 / 27 / 1 / 1 | ✅（原版类型，carry 原样搬运） | `docs/modpacks.md:656-661` |

> 结论：Create 本体是 **"内核 none + 适配 deep"**——8 类 Create 配方和整套动力/物流机制已在引擎里；缺口是 7 类机器配方（deploying 为主）与火车/车站/Ponder 等外围。**不要把内核当成完整 Create**（`docs/modpack-triage.md:61`）。

## 3. 重点一：Create 本体 1.21.1 → 26.3 怎么适配内置内核

### 3.1 许可边界（jar 原文，最高优先级）

jar 内 `LICENSE.md_create-1.21.1` 原文：
- `## Assets License (All Rights Reserved)` — "This license applies to ... `./src/main/resources/assets/`"
- `## Code License (MIT)` — "All other files are licensed under the MIT License"

处置：代码可读可移植并署名（`licenses/create-MIT.txt` 已在本仓）；**`assets/` 不可拷贝**，走"玩家设备上从自己的 jar 抓取"路径，与 `docs/modpacks.md:152-177` 的 `assets.fetch` 策略一致。

### 3.2 升级坑（按本地文档）

| 坑 | 处理 | 来源 |
|---|---|---|
| 数据格式升级 | item models → `items/`（1.21.4）；recipe ingredients/results 变形（1.21.2）；loot table `conditions`/`functions` → `modifier`、数字 provider 类型化；不会升级的文件列出而不猜 | `docs/modpacks.md:656-661` |
| ID/注册表改名 | 1.21.1 `chain` → 26.3 `iron chain`，carry 改写 recipe / loot / worldgen；同类改名逐项处理 | `docs/modpacks.md:842-846` |
| NeoForge API 改名 | `Capabilities.ItemHandler/FluidHandler/EnergyStorage` 等 26.3 改名；改名只是起点，载荷可能变 | `docs/modpacks.md:514-520` |
| 代码不可照搬引擎 | 附属按 1.21.1 的 Create API 编译，jar 无法在 26.3 加载；须按本仓原生 mod API 重写 | `docs/modpack-triage.md:61`；`docs/modpacks.md:657-661` |
| Mixin → hook | Create 的 Mixin 进不了 26.3；用本仓 hook/Parts。Create-Horizon 用 trait API 而非 source mixin，正合本仓"mod API 干净断裂"规则（`Rustling-090/AGENTS.md`） | lead 调研结论 |

### 3.3 落地步骤（可验证）

1. **carry**：`horizon-port-pack carry` 提取 Create 的 `data/` + `assets/`（美术进 fetch，不进仓），升级到 26.3 格式（`docs/modpacks.md:652-672`）。
2. **oracle + harness**：用 Java oracle 导出 643 个 blockstate、1884 配方的事实与最终配方表，逐项与 Horizon 对照（`docs/modpacks.md:969-1012`）。
3. **复用内置**：8 类配方 + 动力网络直接吃现成实现，不为它们另写 port；只核验 jar 配方能被 `Recipes::load` 读到（`kinetic_machines.rs:108-127`）。
4. **补缺口**：7 类缺失配方按优先级补，deploying（167）最大；参考 Create-Horizon 的 MIT 规则实现，不读其 jar 美术。
5. **验收**：以 per-mod harness 和实机画面为准，不以"方块数"当完成度（`docs/modpacks.md:976-987`）。

## 4. 重点二：三个浅层附属的"能玩第一版"

### 4.1 createwheelsuponchairs（data/logic，MIT）

jar 实证：`redstone_radar.json` 1 个 blockstate，6 个模型（`redstone_radar*` + `simple_packer`、`simple_schematic`），**0 配方**，74 class，**78 mixin**，依赖 create + cloth_config。

- **反差告警**：名字是"轮子上的椅子"，但内容里没有椅子/轮子方块；78 Mixin 说明行为主要靠注入 Create 机器实现，不是纯注册。**carry 只能拿到 1 个方块 + 模型，拿不到行为**。
- **第一版（能玩）**：先把 `redstone_radar` 按 data 层搬进来（方块 + 模型 + 战利品），能放置/能看即可；Mixin 行为列为后续 hook 项，不承诺。
- **验收**：方块出现在创造栏、模型正确、可放置破坏；不声称雷达功能已移植。

### 4.2 create_power_loader（logic，MIT）

jar 实证：`andesite_chunk_loader` / `brass_chunk_loader` / 两个 `empty_*` 共 4 个 blockstate，2 个配方（空装载机），21 模型，65 class，8 mixin，依赖 create。

- **机制**：空装载机（捕获恶魂）→ 装载机；黄铜版支持 1×1 / 3×3 / 5×5 范围；可附着车站/机器。
- **第一版（能玩）**：data 层搬 4 方块 + 2 配方 + 模型；logic 层实现"旋转达到阈值 → 保持区块加载"的最小闭环，范围先用黄铜版固定 1×1，后续再放配置。
- **所需 API**：region/区块常驻加载接口 + 方块 tick + 应力输入（内置 kinetic）；车站附着依赖 Create 火车/车站，**第一版明确不做**。
- **验收**：放置装载机 → 供能 → 离开后区块仍加载；关掉后正常卸载。

### 4.3 createtreadmill（machines，MIT）

jar 实证：`treadmill` + `maid_motor` 2 个 blockstate，2 个配方，10 模型，44 class，6 mixin，依赖 create（模型名含 maid，可能为可选女仆联动，但依赖段无 touhoulittlemaid）。

- **机制**：实体/玩家踩踏跑步机 → 产生旋转动力；`maid_motor` 是变体。
- **第一版（能玩）**：data 层搬 2 方块 + 2 配方 + 模型；machines 层做"实体站上去 → 按 tick 输出 RPM/应力到内置 kinetic 网络"的最小闭环，不做女仆联动。
- **所需 API**：方块实体 tick + 站立实体检测 + 内置 `kinetics.rs` 的动力源接口（`capacity`/`stress`，`docs/modpacks.md:298-305`）。
- **验收**：玩家站上跑步机 → 相邻轴转动 → 能驱动一个内置 millstone/crusher。

### 4.4 三者的共性路径

全部 MIT，许可最干净；都只有个位数方块状态与配方，适合做 **data carry → oracle harness → 少量 logic/machines hook** 的标准样板，用来打通附属移植管线，再复制到第二梯队。

## 5. 重点三：重附属先放弃（长周期）

| 模组 | 许可 | jar 规模 | 先放弃的理由 | 例外/后续 |
|---|---|---|---|---|
| copycats | **ARR** | 46 bs / 393 cls / 80 mixin | 代码不可参考；CT 连接纹理与功能型 copycat 需自定义渲染 | 仅事实性 JSON 数据搬运；或 clean-room 自研 |
| createbigcannons | MIT + **CC BY-NC-SA 美术** | 139 bs / 855 cls / 487 模型 | 炮弹实体+弹道+多方块，deep 顶配；美术非商业受限 | 代码 MIT 可留作远期参考，美术不可碰 |
| createdieselgenerators | MIT | 65 bs / 239 cls / 297 模型 | 流体 + 能量体系未与 new-age 统一前，重复造轮子 | 许可干净，排在能量抽象定型之后 |
| create_mechanical_spawner | LGPL3 | 34 bs / 52 cls / 31 mixin | copyleft + 实体生成/AI，deep | 依赖 mechanicals（库，none）已解决，但 AI 仍重 |
| createoreexcavation | MIT | 6 bs / 145 cls / 0 mixin | 无限矿脉 worldgen + 流体抽取器，deep | 许可干净，可待 worldgen carry 能力成熟后复活 |
| create-new-age | BSD-3-Clause | 36 bs / 178 cls | 电力/电线/核反应堆/热系统是全新能力层 | 许可宽松，但要先设计能量 API |
| create_jetpack | 自定义许可 | 2 bs / 26 cls / 0 mixin | 飞行/悬停要玩家运动 hook；许可未放行 | 体积极小，许可确认后可提前 |
| create-enchantment-industry | LGPL-3.0-or-later | 20 bs / 464 cls / 52 mixin | copyleft + 附魔新 hook + 依赖 #13 | 等 #13 与第二梯队 |
| mechanicals | LGPL3 | 0 bs / 0 rec | **本来就是库，不是移植项**（§0） | 不需要"放弃"，直接 none |

**第二梯队例外**：`CreateDragonsPlus`（LGPL，只搬数据）本质是给内置 fan processing 加配方（56 bs / 119 rec），工程浅、不碰其代码，可在三个浅层之后立刻做；它是 `create-enchantment-industry` 的前置，先做它能解锁后续。

## 6. Create-Horizon 案例能复用什么（仅引用 lead 调研结论）

- **结论**：从零 Rust 重写 Create，达到 **225/246 方块（91.5%）广度**，代码 **MIT**，美术从玩家本地 jar 读取。
- **可复用**：① 用它的"已做方块清单"当我们的缺口清单，尤其 §2 表中内置缺失的 7 类配方；② MIT 代码可读、可署名参考，避开其美术；③ 它用**具名 trait API 而非 source mixin**，与本仓 mod API 规则一致，移植思路可直接借鉴。
- **不可当真**：**广度 ≠ 深度**，其大量模块据 lead 调研仅类型检查/单测、未实机验证；不把它当成画质或完成度验收依据。

## 7. 依赖顺序（Create 组）

| 层 | 内容 |
|---|---|
| L0 | flerovium、ritchiesprojectilelib、mechanicals、kotlinforforge → 全部 none，映射 API（`docs/modpack-triage.md:110`） |
| L1 | **create 本体**：内核 none + 1.21.1→26.3 适配（§3） |
| L2 第一批 | **createwheelsuponchairs → create_power_loader → createtreadmill**（§4，全 MIT、浅层） |
| L2' 第二梯队 | CreateDragonsPlus（只搬数据）→ 可解锁 enchantment-industry |
| L3 长周期 | createdieselgenerators / create-new-age（先统一能量抽象）；createoreexcavation、mechanical_spawner（deep） |
| L4 放弃/远期 | copycats（ARR）、createbigcannons（NC 美术）、create_jetpack（许可+hook） |

## 8. 来源索引（全部本地）

- `docs/modpack-list.md:1-70` — 48 模组权威清单与分组
- `docs/modpack-triage.md:42-61, 92-104, 110, 118-130` — 总盘点的 Create 分层、许可红线、顺序与放弃结论
- `docs/modpacks.md:67-76, 126-181, 283-336, 514-520, 638-672, 842-846, 969-1012` — 分层/许可/内置机制/升级坑/harness
- `crates/horizon-client/src/game/starling/kinetic_machines.rs:1-8, 85-92, 108-127` — 内置 Create MIT 核心与配方类型
- `crates/horizon-client/src/game/starling/basins.rs:38` — `create:mixing` / `create:compacting`
- `crates/horizon-client/src/game/starling/` 其余子模块（kinetics/belts/funnels/chutes/fans/processing/saws/bearings/arms/machine_entities 等）
- `licenses/create-MIT.txt:1-5`、`LICENSE:1` — Create 许可边界与引擎 MIT
- **本地 jar 实证**：`/storage/emulated/0/1.21.1mc/versions/1.21.1梦/mods/` 下各 Create 系 jar 的 `META-INF/neoforge.mods.toml`（license/modId/依赖）与 `LICENSE.md_create-1.21.1` 原文，以及 blockstate/recipe/model/class/mixin 文件普查结果（见 §1-2）
- lead 调研结论 — Create-Horizon（225/246、MIT、本地 jar 美术、trait API），未联网复核
