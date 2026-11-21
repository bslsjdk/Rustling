# 车万女仆（Touhou Little Maid）生态移植可行性深挖

> 范围：`docs/modpack-list.md` 48 模组清单中「车万女仆生态」10 个（本体 1 + 附属 9）。
> 方法：**全程只读本地文件**，未联网。`jar` 证据由 Python `zipfile` 直接读取 `META-INF/neoforge.mods.toml`、类常量池与 `assets/`/`data/` 结构得到。
> 结论先行：**本体原样移植不可行；9 个附属无一能脱离本体独立移植；许可（CC BY-NC-SA 非商业）是独立于技术之外的第二道红线。**

## 0. 口径与来源

| 项 | 口径 | 来源 |
|---|---|---|
| 权威清单 | 48 个模组，1.21.1 NeoForge；车万女仆组 10 个 | `docs/modpack-list.md:1-5,63-65`（对应清单 #29-36、#44-45） |
| 分层定义 | data / logic / machines / deep / client / none；**deep = 实体与 AI、自定义渲染与动画模型、新 hook** | `docs/modpacks.md:67-76`（deep 行见 `:74`）；分类规则 `docs/modpacks.md:638-643` |
| 能力缺口表 | 「实体：类型、属性、AI goals 与 brain、生成」与「自定义渲染与动画模型（GeckoLib 格式）」均列为 deep | `docs/modpacks.md:206-207`；GeckoLib→动画模型格式 `docs/modpacks.md:94` |
| 许可判据 | 含 `-nc`/`noncommercial` 归为 **restricted**；引擎本身 MIT | `crates/horizon-port-pack/src/report.rs:111-114`（restricted 分支）；`LICENSE:1` |
| 总盘点结论 | 10 个全 deep、全不移植，9 附属依赖本体 | `docs/modpack-triage.md:63-78,92-104,126-127` |
| 本地 jar | 1.21.1 全套 10 个 + 26.3 本体 2.0.0 | `/storage/emulated/0/1.21.1mc/versions/1.21.1梦/mods/`、`/storage/emulated/0/26.3/versions/26.3 NeoForge 测试整合包/mods/` |

> 校正一处任务表述：本仓库**没有**名为 `java-to-horizon.json` 的字面文件（全库文件名与全文检索 0 命中）。实体硬边界的真实出处是 `docs/modding/src/how-to/content-mods.md:552-606` 与 `docs/modpacks.md:74,206-207`，下文按真实出处引用。

## 1. 本体为什么不可行

### 1.1 撞到的 API 硬边界（逐条，带出处）

| # | 本体实际用到的能力（本地 jar 实证） | 引擎现状（文件:行号） | 判定 |
|---|---|---|---|
| 1 | 自定义实体类型 `EntityMaid`、扫帚 `EntityBroom` 等；本体 `entity/passive` 类 17（1.5.3）/49（26.3） | 内容实体只能**复制模板**：`minecraft:snowball` 投掷物，或 zombie / villager / wandering_trader / wolf / cat / parrot 六种活体；活体类型「`attributes` 只能覆盖模板已有属性」，非模板物种无法声明。`docs/modding/src/how-to/content-mods.md:552-553,599-606` | **硬撞**：没有「注册全新物种」入口 |
| 2 | AI 体系：AI/brain/task 类 **537**（1.5.3）/533（26.3），含 `MaidPathFindingBFS` 自定义寻路（pathfinding 类 4） | `EntityPart::register_goals` / `register_behaviors` 只能给**已存在的**模板物种挂 goal/behavior，且写入限于 `move_to`/`stop_navigation`/`look_at`/`jump`/`set_target`，brain-backed 物种还会拒绝目标写入。`docs/modding/src/concepts/parts/entity-parts-and-mob-goals.md:5-8,22-27,29-40,47-49` | **硬撞**：能「加行为」≠能「定义物种的 brain/sensor/navigation」 |
| 3 | 动画模型：实体模型 JSON **292**、动画 JSON **63**（两版一致）；geckolib 相关条目 395（1.5.3）/453（26.3）；`simplebedrockmodel`/bedrock 263/192 | 模型侧只支持 OptiFine/EMF `.jem` 自定义实体模型 + 表达式动画（`crates/horizon-client/src/mobs/cem_anim.rs:1-3`；`docs/resource-packs.md:45`，且标注 *partly*）；全库检索**无 Bedrock `geo.json` 解析器、无 GeckoLib 骨骼动画运行时** | **硬撞**：格式与运行时都不存在 |
| 4 | 渲染：client render/model/animation 类 178（1.5.3）/129（26.3），GUI screen 143/133 | 有 `EntityRenderer`/`AddLayers` 注册 API（`docs/modding/src/how-to/client/entity-renderers.md:3-5,87`；`docs/modding/src/how-to/client/entity-layers.md:1-7`），但提交的是 quad/cube/model_part，**不是骨骼蒙皮/关键帧动画**；且只能画已注册的类型 | **半撞**：渲染入口有，驱动动画的模型和姿势系统没有 |
| 5 | Mixin 注入：mixin 类/相关条目 37（1.5.3）/43（26.3），涉及导航、玩家、方块烧炼缓存等 | deep 层「新 vanilla hook（Mixin 做的事）」整类列为缺口 `docs/modpacks.md:208-210`；且本项目要求每个 hook 一一对应 vanilla 调用点、带 `Host`、侧别、目录条目与测试（`AGENTS.md`「Mod API on this fork」段） | **硬撞**：需逐条设计 hook，不能批量搬 |
| 6 | 规模与自带生态：本体 2349 类（1.5.3）/2289 类（26.3），data JSON 231，assets JSON 731，另有 **1321 个 `tlm_custom_pack` 运行时皮肤包资产** | 皮肤包热加载、LLM/聊天 AI、备份管理（`MaidBackupsManager`）等玩法无对应 API | **硬撞 + 工程量爆炸** |

**小结**：本体不是「缺一两个 API」，而是同时撞上**物种注册、AI/寻路、模型动画格式、hook 体系、皮肤包分发**五个 deep 级缺口。任何一条单独补成都远大于一次常规移植。

### 1.2 许可：能不能移植、能不能分发

| 维度 | 事实 | 后果 |
|---|---|---|
| 许可证原文 | 1.5.3 与 26.3 本体 `META-INF/neoforge.mods.toml` 均写 `license = "MIT / CC BY-NC-SA 4.0"`；jar 内 `licenses/` 还含 GPLv2、Vorbis、chess 等第三方许可 | 双许可：**代码 MIT，美术（模型/贴图/皮肤）CC BY-NC-SA 4.0** |
| 引擎判定 | `report.rs:113-114` 把含 `-nc` 的串判为 `restricted`；总盘点已按此归类 `docs/modpack-triage.md:98` | 与 Jade 同属 restricted 红线 |
| **能不能移植（技术上）** | MIT 代码部分可**参考/clean-room 重写**；但重写出来的实体、AI、动画系统仍需 §1.1 的引擎能力，许可放行≠技术可行 | 两条门槛必须**同时**过 |
| **能不能分发** | CC BY-NC-SA 4.0 = 非商业 + 相同方式共享：非商业前提下可分发衍生，但须同协议并署名；**商业分发不可**；`assets/` 的模型/贴图/1321 个皮肤包资产不能原样拷进 MIT 引擎仓库 | 即使引擎补齐能力，美术仍需原作者授权或**全部原创重画** |
| 引擎侧兼容性 | 引擎 MIT（`LICENSE:1`）；copyleft/NC 代码不能直接纳入（`docs/modpack-triage.md:96-101`） | 不能把 TLM 源码或其衍生直接并入引擎主线 |

## 2. 附属模组：逐个判定分层与「能否独立」

**关键实证：9 个附属的 `neoforge.mods.toml` 全部声明 `mandatory=true` 的 `touhou_little_maid` 依赖**（如 `maidtask`：`dependencies.maidtask / modId="touhou_little_maid" / mandatory=true / versionRange="[1.3.7,)" / ordering="AFTER"`）。类常量池进一步证实它们直接调用 `com.github.tartaricacid.touhoulittlemaid.api.task.IMaidTask`、`entity.passive.EntityMaid`、`MaidPathFindingBFS` 等本体类。**没有任何一个是纯数据包（全部含 19–557 个 class）。**

| 模组（清单 #） | 本地 jar 实证（类数 / mixin / 依赖 / 许可） | 分层 | 能否脱离本体独立移植 | 理由 |
|---|---|---|---|---|
| maidtask（#34） | 34 类、0 mixin 类；依赖 TLM；**MIT**；26 类引用 TLM，含 `api/task/IMaidTask`、`EntityMaid`、`MaidPathFindingBFS` | deep | **否** | 看似「纯逻辑任务系统」，但 `BuildHouseBehavior`/`CollectFluidBehavior`/`DestoryBlockBehavior` 全是挂在 `EntityMaid` 上的 AI 行为；没有本体就没有挂载对象。`docs/modpack-triage.md:73` 的 deep 判定成立 |
| maidbeacon（#33） | 60 类、mixin 配置 1；依赖 TLM；`license="${mod_license}"`（**未解析，需核实**）；13 类引用 `api/ILittleMaid`、`LittleMaidExtension`、`api/bauble/IMaidBauble`、`EntityMaid`、`BeaconRenderer` | machines→deep | **否** | 主体是「信标方块 + 客户端 UI + 女仆饰品槽增益」。信标方块本身是通用 machines 概念，但**重写一个信标不叫移植 maidbeacon**，且无女仆时无玩法价值 |
| maid_restaurant_storage（#31） | 25 类、1 mixin；依赖 `maid_restaurant` + refinedstorage/ironchest/farmersdelight；**MIT**；2 类引用 TLM | machines→deep | **否** | 容器/流体能力扩展，经 `MaidGetFromStorageTaskMixin` 接入女仆任务；二级依赖链 `→maid_restaurant→TLM` |
| MaidUseHandCrank（#35） | 25 类、4 mixin；依赖 TLM + create + create_connected + sable；**MIT / CC BY-NC-SA 4.0**；`HandCrankTask implements IMaidTask` | logic（集成胶水） | **否** | 唯一接近 logic 层的附属，但功能就是「让女仆转 Create 曲柄」；两端（TLM 任务 API + Create 曲柄方块实体）缺一不可，且带 NC 许可 |
| maid_useful_task（#36） | 87 类、5 mixin；依赖 TLM + playerrevive；**MIT** | deep | **否** | 任务行为扩展，同类 `DestoryBlockBehavior`/`BlockUpPlaceBehavior`，强耦合本体 API |
| littlemaidmoreaction（#29） | 557 类（附属中最大）；依赖 TLM + cloth_config；**All rights reserved** | deep | **否** | 行为 + 菜单 + 网络包 + 客户端入口，ARR 不可参考代码 |
| maidaddition（#32） | 62 类；依赖 TLM + create + ponder + flywheel；**All Rights Reserved** | client→deep | **否** | 动作队列（考古/破坏方块等）+ Create 展示集成，依赖本体动作系统 |
| maid_restaurant（#30） | 147 类、15 data JSON；依赖 TLM + kaleidoscope_cookery；**BSD 3-Clause** | machines→deep | **否** | 餐厅方块 + `ICookTask`/`IMaidStorage` 任务接口，许可虽宽松但依赖本体与烹饪模组 |
| vanilla_companion（#44） | 19 类、15 个 mixin 类；依赖 TLM；**All Rights Reserved** | deep | **否** | `MixinBowUse`/`MixinCrossbowUse`/`MixinEntityBroom` 等改的是本体扫帚和女仆行为，纯本体集成 |

**直接回答 lead 的问题：有「纯数据/纯逻辑」能单独移植的吗？——没有。**

- 数据层面：9 个附属合计 data JSON 极少（0–15 个），且都是自身方块/物品的配方标签，离开本体物品无意义。
- 逻辑层面：最接近的 `maidtask`（MIT、0 mixin、34 类）也不是独立系统，它实现的是本体的 `IMaidTask` 接口。
- 唯一可「借用概念」的是信标增益方块和容器能力（`maidbeacon`/`maid_restaurant_storage`），但这属于**自研通用方块**，不构成对女仆模组的移植，也没有女仆玩法价值。

## 3. 若要让女仆本体变得可能：引擎要补什么、多大工程

| 需要的能力 | 引擎现状 | 要补的东西 | 工程量 |
|---|---|---|---|
| 可注册生物类型 | 只能复制 6 种 vanilla 模板 + snowball（`content-mods.md:552-553,599-606`） | 自定义 species：独立类/NBT/网络追踪/属性/尺寸/生成规则；从「复制模板」升级为「注册物种」 | **大**（实体系统核心改造） |
| goals / brain / pathfinding | `EntityPart` 可给既有模板挂 goal/behavior（`entity-parts-and-mob-goals.md:5-49`），这是唯一的「苗头」 | 自定义 sensor、memory、activity、导航算法（本体用 `MaidPathFindingBFS`）；现有 API 明确拒绝部分写入且状态不保存 | **中大** |
| 动画模型格式 | 仅 OptiFine/EMF `.jem` + 表达式动画（`cem_anim.rs:1-3`、`docs/resource-packs.md:45`）；无 Bedrock `geo.json` 解析、无 GeckoLib | Bedrock 几何 JSON 解析器 + 骨骼层级/关键帧/插值运行时 + 皮肤包热加载 UI（本体 292 模型 + 63 动画 + 1321 皮肤资产） | **大**（全新子系统） |
| 渲染 | `EntityRenderer`/`AddLayers` 已存在（`entity-renderers.md:3-5,87`、`entity-layers.md:1-7`） | 扩展 draw context 以提交骨骼姿势；与模型格式、动画运行时联动 | **中大** |
| 新 vanilla hook（Mixin 替代） | 整类列为 deep 缺口（`docs/modpacks.md:210`）；本项目要求一 hook 一 Host 一目录条目 | 本体 37–43 个 mixin 逐条设计、实现、测试 | **大**（逐条长尾） |
| 配套 | — | 女仆数据持久化、网络同步、GUI/配置、LLM/聊天（本体含 `ai/manager` 大量类） | **大** |

**有没有可参考的相近实现？**

- **Rust/Horizon 女仆移植：无。** 采用 lead 的调研结论；本地全库检索亦无命中。
- **引擎内真正相近的两处苗头**：① CEM/EMF 的 `.jem` 模型 + 表达式动画（`mobs/cem.rs`、`mobs/cem_anim.rs`）——证明「资源包驱动实体模型 + 逐帧表达式」这条路走得通，但它是 OptiFine 模型，**不等同 GeckoLib 的骨骼动画**；② `EntityPart` 的 goals/behaviors——证明可给模板实体挂 AI，但不能定义新物种。
- **本地非 Rust 参考数据（仅作格式参考，不是可移植实现）**：26.3 Java 版本体 `touhoulittlemaid-2.0.0-neoforge+mc26.3-all.jar` 存在（2289 类）——目标版本虽对齐，但它仍是 NeoForge Java 字节码，Horizon 无法加载，需要全量重写；手机上的 `TouhouLittleMaidBP.mcpack.zip` / `TouhouLittleMaidRP.zip` 含 271 个 `models/entity/*.geo.json` + 24 个动画 JSON，可作 Bedrock/Gecko 格式的**结构参考**，但引擎无对应解析器，且**美术资产受 NC 许可限制，不可直接取用**。

**工程量定性**：这不是「移植一个 mod」，而是「在引擎里重建一套 TLM 级实体 + 动画框架，再重写 2000+ 类玩法」。按本仓库的 clean-break、一 hook 一 Host、一行为一测试规范，属于**专项团队长周期工程**，不是某个 PR 或一次版本升级能顺带完成的。

## 4. 明确结论

| 档位 | 对象 | 结论 |
|---|---|---|
| **做不了（原样移植）** | 本体 `touhoulittlemaid` 1.5.3；9 个附属全部 | 技术：五个 deep 级硬缺口叠加；许可：美术 CC BY-NC-SA 非商业。两条均不放行 → **不列入移植计划**（与 `docs/modpack-triage.md:127` 一致） |
| **勉强能做（但要换性质）** | 「原创女仆风格 companion」 | 用**现有**能力起步：wolf/cat 模板实体 + `EntityPart` goals + `.jem` 原创模型 + attachments 存女仆状态 + machines 方块做信标/容器。可行但：①不是 TLM 移植；②玩法大幅缩水（无 GeckoLib 皮肤系统、无原版任务链）；③**模型/贴图必须原创**，不能碰 TLM 资产。属自研 spike，不占 48 模组移植名额 |
| **最小可玩版要不要等引擎升级** | 分两档 | **要像 TLM**（多皮肤、骨骼动画、完整任务/餐厅/信标）→ **必须等**引擎补齐 §3 的物种注册 + 动画格式 + hook，且先拿到许可授权；**只要一个会跟随/能干活的原创女仆**→ **不用等**，现有 EntityPart + CEM + 模板实体即可做原型，但这是自研而非移植 |

**给 lead 的执行建议**：① 维持「不移植」总判定，不投入 10 个 mod 的移植工；② 许可未授权前**零代码**，尤其不得把 `assets/`、`tlm_custom_pack/` 拷入仓库；③ 若想保留选项，只做上表「原创 companion」的可行性 spike（1–2 周，验证 wolf 模板 + EntityPart + CEM 能否拼出可玩循环），与 TLM 资产完全隔离。

## 5. 证据索引

- 边界与分层：`docs/modpacks.md:67-76,94,206-210,638-643`；`docs/modding/src/how-to/content-mods.md:552-606`
- AI/渲染 API：`docs/modding/src/concepts/parts/entity-parts-and-mob-goals.md:5-49`；`docs/modding/src/how-to/client/entity-renderers.md:3-5,87`；`docs/modding/src/how-to/client/entity-layers.md:1-7`
- 模型/动画：`crates/horizon-client/src/mobs/cem_anim.rs:1-3`；`crates/horizon-client/src/mobs/cem.rs:1-10`；`docs/resource-packs.md:45`
- 许可：`crates/horizon-port-pack/src/report.rs:111-114`；`LICENSE:1`；`docs/modpack-triage.md:92-104`
- 清单与总判定：`docs/modpack-list.md:63-65`；`docs/modpack-triage.md:63-78,126-127`
- 本地 jar：1.21.1 目录 10 个 jar 的 `META-INF/neoforge.mods.toml` 与类常量池；26.3 目录 `touhoulittlemaid-2.0.0-neoforge+mc26.3-all.jar`；`/sdcard/Download/TouhouLittleMaid{BP.mcpack,RP}.zip`
