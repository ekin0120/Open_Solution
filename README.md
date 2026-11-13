# Open Solutions

独立的燃烧设备选型工具，用 Rust 编写，数据层使用 **SQLite**，带完整中文桌面图形界面。

> **净室（Clean room）实现边界**：本项目**不复制**任何 Honeywell / Hauck 的程序代码、图标、商标或界面文案；模块划分与名称仅依据**可见界面按钮文字**与安装目录中**可见文件名**推导，所有公式均为公开通用工程关系。
>
> **2026-11-10 起的数据例外**：按使用者明确要求，已把**本机已安装程序** `DatabasesPwd\` 下 20 个 `.mdb` 的**目录数据**导入到本地 `open_solutions.db`。这些是厂商专有工程数据，仅供个人研究 / 内部自用，**不要随本项目源码或成品对外分发**；`dist\data\` 中的库如需分享，请先清空 `src_*` 表与 9 张业务表中的导入行（重跑 `tools\import_source_data.bat` 可随时重建）。项目源码树内**不内嵌**任何源数据。

## 交钥匙状态（2026-11-10）

**不需要安装任何东西**：把 `dist\` 整个目录拷走，双击 `启动 Open Solutions.bat` 即可。

| 项 | 状态 |
|---|---|
| 工具链 | Rust 1.99.1 `x86_64-pc-windows-gnu` + CLion 自带 mingw-w64 **GCC 15.2.0**（本机编译用，成品不需要） |
| 测试 | `cargo test --workspace` **35/35 通过**（os-core 24 + os-modules 11） |
| 模块 | **41 个**：Reference 19 + Applications 11 + System Design 7 + Tools 4；其中 **37 个计算模块全部实跑通过**，共 281 行结果 |
| 界面 | 41 个页面逐个打开，0 空白 / 0 JS 报错；Edge `--app` 独立窗口，无地址栏、无首次运行弹窗 |
| 单位自检 | 界面「双单位一致性自检」逐模块 **全项 PASS**（如燃料数据 12/12、燃料燃烧 8/8）；CLI `verify-units` **6 项 delta 全 0.000e0** |
| SQLite | `schema.sql` **v2**：14 张应用表 + 2 张导入清单表 + 496 张源表 = **512 张表**、24 条单位约定；导入 **20 个 .mdb / 496 表 / 4570 行**，`integrity_check = ok` |
| 成品依赖 | 两个 exe 均自包含；仅需系统自带的 Microsoft Edge（找不到时回退默认浏览器） |
| 目录数据 | 已导入 **`ESolutions600:*`** 来源 3024 行到 10 张业务表（`fuels` + `fuel_gas_components` 17 种气体燃料 + 其余 8 张）；其余 4570 行以 `src_*` 原样保留。用户自有数据可继续录入 |

## 图形界面

启动：`dist\启动 Open Solutions.bat`（或直接双击 `dist\open-solutions-gui.exe`）。

- **左侧导航**：四个可折叠分组 + 顶部中英文关键词搜索，共 41 项。
- **右上操作区**：`SI 公制 / US 英制` 一键切换、数据库状态徽章、退出按钮。
- **模块页**：方法说明 + 带显式单位的输入表单 + 「计算 / 恢复默认 / 双单位一致性自检」+ 结果表（**主列当前单位制、对照列另一单位制**）。每次计算自动写入 SQLite `calc_runs` 历史。
- **四个工具页**：关键词检索、术语表、设备目录（读 SQLite 业务表、导入统计与计算历史、可录入燃料）、单位设置（含单位约定审计表）。
- **右上徽章**：`SQLite v2 · 14 表 · 源数据 4570 行`，应用表数量保持 14，导入的 496 张源表不计入该列表以免刷屏。

### 模块清单（41）

| 分组 | 模块 |
|---|---|
| **Reference 参考**（19） | 燃料数据、气体与液体物性、固体物性、燃料燃烧、沃泊指数、余热回收节能、流体力学、不完全燃烧损失、混合气可燃性、孔板流量计、自然对流、强制对流、辐射换热、燃烧噪声、排放换算、风机定律、单位换算、工程数据表、几何工具 |
| **Applications 应用**（11） | 低温骨料干燥、空气预热器、空气/燃气管路设计、烟道与烟囱尺寸、微调阀流量特性、炉体散热损失、TBA 鼓风机选型、风机海拔/温度修正、SMJ 鼓风机选型、燃烧器/阀门海拔修正、引射器 |
| **System Design 系统设计**（7） | ① 炉膛规格 ② 燃烧器选型 ③ 空气管路与部件 ④ 鼓风机选型 ⑤ 燃气管路与部件 ⑥ 设计报告 ⑦ 安全联锁检查 |
| **Tools 工具**（4） | 关键词检索、术语表、设备目录、单位设置 |

### 燃料数据：交互式燃料选择器

「燃料数据」模块复刻了 E-Solutions 的 Fuel Data 窗口交互：

- **选择燃料**：顶部下拉框，首项为 `Customized Gas Mixture（自定义混合气）`，其后列出 SQLite `fuel_gas_components` 表中已导入的全部 17 种气体燃料（如 Average Natural Gas、Methane、Propane…），名称取自 `ESolutions600:Fuel.mdb/GasFuel`。
- **组分显示**：下方 10 个组分输入框（CH4 / C2H6 / C3H8 / C4H10 / CO / H2 / CO2 / O2 / N2 / H2O），单位为体积分数 %。
  - 选中**预设燃料**→ 自动填入该燃料的组分并**锁定为只读**（灰显 `.locked`）；
  - 选中 **Customized Gas Mixture** → 全部可编辑，总和不必等于 100（计算时自动归一化）。
- **计算**：点「▶ 计算」按组分从第一性原理重算 12 项结果（平均分子量 / 相对比重 / 标况密度 / 质量 LHV·HHV / 体积 LHV·HHV / 理论需氧 / 理论空气量 / 理论湿烟气量 / 干基最大 CO₂ / 归一化说明），并写入 `calc_runs` 历史。
- **双单位自检**：燃料组分为 %（SI/US 无换算差异），点「⇄ 双单位一致性自检」全项 PASS。

> 注：引擎从组分**重新计算**物性。源库 `GasFuel` 的 `GrossHeatSI/NetHeatSI` 实为**体积基 MJ/Nm³**（与界面显示的 Btu/scf 一致），与本引擎算出的体积 LHV 36.94 / HHV 40.89 MJ/Nm³ 量级吻合；质量基 LHV≈46.4 MJ/kg 由组分推导得出。

### 源程序一致性核对（A/B 批对齐）

对照 E-Solutions 600 实机截图（pywinauto 自动化捕获）与 exe 内嵌控件字符串，逐模块核对后完成两批对齐（详见 `docs/VERIFY_REPORT.md`）：

- **A 批（快赢）**：
  - `tables` → 改为源的「Standard Steel Pipe Data: Schedule 40」（NPT 1/8″–30″ 全 22 行）+「当量比 φ ↔ 过剩空气」两张表；
  - `geometry_tools` → 形状清单对齐源 `frmGeometry`（Annulus/Circle/Cone/Cube/Cylinder…）；
  - `fan_laws` → 输出 6 个比值（转速律 Q/p/HP + 等质量流量 Q/p/HP）+ 目标值；
  - Fuel Data 前端预设燃料时同时显示源库参考值（SG / GHV / NHV，Btu/scf）。
- **B 批（重写）**：
  - `flammability` → **16 组分**（CH4…H2S + CO2/O2/N2/H2O）+ 温度修正，输出 12 项：LFL/UFL(25°C)、Zabetakis 温度修正系数与修正值、GHV (Btu/scf)、化学计量空气 (scf/scf)、燃烧速度 (m/s·ft/s)、LFL 下空燃比与过剩空气；
  - `wobbe_index` → 原燃气/替代气对比（可从燃料库一键填入 Hv·sg）+ 掺空气互换配比（含不可行判定）；
  - `energy_savings` → **Case I/II 可用热（Available Heat）对比**：AH=(LHV+空气预热焓−烟气损失)/LHV，输出节能率；
  - `incomplete_combustion` → 热输入 + 干基 O₂/CO → 过剩空气、CO 化学损失、效率损失 + **时/日/年 3×3 成本矩阵**；
  - `solids_props` → **查表化**：内置 53 种金属/非金属物性（源自本地导入 `MetalProp/Solid` 库，US 单位已换算 SI），显示密度/比热/导热/辐射率/熔点/使用上限 + α、q″、蓄热评估。
- **C 批（气体与液体物性 1:1 重做）**：
  - `gas_liquid_props` → **对源程序 518 组实测数据逐点对齐**：25 种流体（18 气 + 7 液）× 19 温度点 × 9 项物性；纯流体 μ/cp/k **位级精确（0.00%）**，ρ/SG ≤0.56%（源显示位数舍入）；有效温度范围复刻源弹窗（CH₄ 0–930°F、乙烯 0–350°F、H₂ ≥1000°F、蒸汽 ≥600°F、液体 40–290°F…）；
  - **Customized Gas Mixture**：12 组分体积分数（CH₄/C₂H₆/C₂H₄/C₃H₆/C₃H₈/C₄H₁₀/H₂/CO/N₂/CO₂/O₂/H₂O），不足 100% 自动补 N₂（与源一致）；混合规则 ρ/SG/cp/γ/k/h 与源实测 6 组混合气全部对齐（≤0.4%，多数精确），μ 用 Wilke（Brokaw 形式，5/6 组 ≤4.5%，烃类富集混合最大 +8.2% 为已知偏差）；
  - **界面仿源**：Select Fluid Type + Show Properties + Specify Conditions + 9 项英文物性面板（Density/Specific Gravity/Specific Heat/Absolute & Kinematic Viscosity/Enthalpy/Ratio of Specific Heats/Thermal Conductivity/Prandtl Number）。
- **D 批（燃料燃烧 1:1 对齐源程序结果页）**：
  - `fuel_combustion` → 从源 `GasFuel`/`OilFuel`/`WasteOil` 表生成 17 种标准气 + 6 种油燃料库（`combustion_fuels.rs`）；5 类燃料选择、16 组分自定义气（不足 100% 自动补 N₂）、3 种燃料输入单位、环境温度/湿度、烟气再循环、富氧；
  - **两页 77 行输出**对齐源报表：燃料组成、SG/GHV/NHV/理论空气、燃烧条件、湿/干烟气产物、可用热、各类流量、绝热火焰温度、露点、预热/排烟热量、主要质量流量；富氧/FGR 关闭时源程序打印 `N/A` 的项也照此显示；
  - **默认工况逐项精确吻合源程序实测**（Average Natural Gas / 100 scfh / 0 % 过量空气 / 2000 °F 排烟）：可用热 46.2 %、可用热量 48107 Btu/hr、排烟热损失 56093 Btu/hr、绝热火焰温度 3577 °F、露点 137.6 °F、烟气量 1089.59 scfh / 5155 acfh、湿基 CO₂ 9.72 % / H₂O 18.49 % / N₂ 71.79 %、理论空气 9.85；
  - 关键是发现源报表能量口径 `48107 + 56093 = 104200`（= 100 scfh × 1042 GHV），据此把「排烟热损失」定为显热 + 水汽化潜热（GHV 基准），并标定平均比热与高温等效解离项；详见 `docs/VERIFY_REPORT.md` 第五章；
  - **界面仿源**：Fuel Selection 5 单选 + Select Gas/Oil 列表、Specify Combustion Conditions、三个选项区、分区式两页报表渲染。
- **D2 批（界面单位制联动与中英双语）**：
  - `fuel_combustion` 的 Fuel Input 三种单位随右上角 SI/US 切换：US `scfh / Mill.Btu/h / lb/hr` ↔ SI `Nm³/h / kW / kg/h`；显示值按物理当量换算（100 scfh = 2.679 Nm³/h），提交时换回源程序 US 基准，保证两套单位制下结果完全一致；
  - 结果表主列随单位制切换，另一单位制列为对照：`48107 BTU/hr ↔ 14.10 kW`、`1089.59 scfh ↔ 29.19 Nm³/h`、`3577 °F ↔ 1969.4 °C`；
  - 全站 265 个字段标签、燃料类型单选、三个选项复选框、16 组分、78 行结果标签/说明、`N/A`/`Off` 状态、表头与按钮统一为 `EN · 中文` 双语；
  - 双单位一致性自检 78/78 PASS（SI ↔ US 往返后输入物理量保持不变）。
- **D7 批（Energy Savings 1:1 重做）**：
  - `energy_savings` → 按源程序 Energy Saving 表单重做：**Case I / Case II 各自选择燃料（源 25 项清单：16 种标准气 + Customized Gas Mixture / Oil #1–#6 / Customized Oil Mixture / Custom Oil / Electricity，后 9 项按源实测行为回退 Average Natural Gas 数据）** + 各 6 项燃烧条件（过量空气、排烟温度、空气温度、助燃空气含氧、烟气再循环 % 与温度）+ 燃料预热温度（源程序实测该输入不参与计算，仅为表单对齐保留）；
  - **可用热内核按源程序 59 点扫描标定**（空气/排烟/过剩空气/富氧/FGR 扫描 + 16 种燃料）：Shomate 真实气体焓（CO₂/H₂O/N₂/O₂，60 °F 基准）×1.02788、H₂O 潜热 41.63 Btu/scf、富氧 >20.9 % 时氧化剂按化学计量 s/x（源程序行为）、FGR 按湿烟气份额 × 干基比焓差 ×1.05430；拟合 rms 0.39 点、最大残差 1.96（3000 °F 解离区边缘）；
  - **锚点验证**：默认工况 AH_I=AH_II=48.0 %、节能率 0；空气预热 800 °F → 62.5 %（源 62.4）；FGR 20 % @400 °F → 39.1 %（精确）；Methane → 47.9 %（源 47.8）；复合工况 → 68.6 %（源 68.7）；新增 `energy_savings_matches_source_sweep` 回归测试（36 测试全过）。
- **E 批（其余 23 个源模块批量对比，2026-11-12）**：
  - **探针方法**：菜单型模块用 Alt+P 键盘导航打开；按钮型模块用 Win32 `EnumChildWindows + BM_CLICK`（按按钮标题命中，绕过双 frmMain 实例的 UIA 坐标错位）；说明弹窗点掉"否(N)"；数值用 `WM_GETTEXT` 按句柄读取。23 份基线证据存档于 `verify_source/shots/`（`<模块>_dump.json` + 截图）；
  - **结论分布**：✅ 已对齐 3 个（Geometry Tools、Unit Conversion、Fan Laws 结构一致）、⚠️ 微调 1 个（Flammability 4/6 命中：过剩空气定义、燃烧速度表、GHV 精度）、❌ 需重写 7 个（Wobbe Index、Orifice Meter 反问题、Natural/Forced Convection、Radiation 视角因子、Incomplete Combustion、Burners & Valves at Altitude、Aggregate Drying）、🔧 补测/可标定 12 个（Emission Calc、Solids Props、Furnace Heat Losses、Flue Sizing、Air Heaters 等）；
  - **快赢修复**：`quantity.rs` Power 增加 `MM Btu/h` 单位别名，Incomplete Combustion 传 10 MM Btu/h 不再静默归零；
  - **F 批优先级**：Wobbe 重写 > Altitude Burners（基线 9000 ft → 2900 scfh / 18.0 inwc）> Incomplete Combustion（基线 EA 9.53 / Eff 98.18 / Loss 181582 Btu/hr）> Orifice 反问题、Radiation 视角因子 > 对流模块标定；逐模块结论表见 `docs/VERIFY_REPORT.md` 第六章。

### GUI 架构

```
Rust 后端 (os-gui.exe)  ── 本地回环 HTTP（127.0.0.1:随机空闲端口，仅监听本机）
   ├─ /api/modules       模块注册表（数据驱动，41 项）
   ├─ /api/compute       调用 os-modules 计算内核
   ├─ /api/convert-fields SI ↔ US 字段换算
   ├─ /api/audit         单位约定审计
   ├─ /api/db /api/fuel  SQLite 状态与录入
   └─ /                 内嵌 ui.html（编译期 include_str!，单文件 HTML/CSS/JS）
                              ↓
                    Edge --app 独立应用窗口
```

本地服务**不监听外网**，端口每次随机分配（避免多开冲突），关闭窗口或点「退出」即结束进程。浏览器配置档案写在程序目录 `edge-profile\`，不污染系统 Edge 配置。调试端口默认关闭，需要浏览器级验证时设环境变量 `OS_CDP_PORT=9222`。

## 单位转换 bug 的修复（核心）

原程序在公英制切换后输出不对。根因是**裸 `f64` 携带隐含单位、在流程中反复就地换算**。本实现改为：每个物理量一个独立类型、内部统一 SI、换算只发生在 I/O 边界且只做一次。

| 根因 | 典型错误表现 | 本实现的对策 |
|---|---|---|
| 裸数值隐含单位 | mm 被当 inch 读 | `Length`/`Pressure`/`MassFlow`… 独立新类型，类型不匹配直接编译失败 |
| 双重转换 | 输入转 SI、输出又转一次 | 单一规范表示；`no_double_conversion_roundtrip` 往返测试锁死 |
| 温度当比例量 | °C/°F 直接乘 9/5，或比值用 °C | 内部 K，°F 为仿射变换 `(F-32)*5/9+273.15`；比值必须用 K |
| 表压 / 绝压混用 | `0 psig` 当成 0 压力 | 内部 **Pa 绝压**；`from_psig(0) = 101325 Pa`，`from_psia(0) = 0 Pa` |
| 标况 / 工况流量混用 | Nm³/h 直接当 m³/h 算流速 | 流量带 `Standard`/`Actual` 标签；`rebase(T,P)` 显式状态修正，未 rebase 直接报错 |
| LHV / HHV 互换 | 热值取错，出力偏差约 10% | 热值带 basis 标签，`require_basis()` 拒绝错用 |
| 平方 / 立方按线性换算 | in² 只乘 25.4（应为 645.16） | `Length::squared()/cubed()` 与 `Area`/`Volume` 类型 |
| NPS 名义尺寸当实测外径 | NPS × 25.4 得出错误 OD | `unit_conventions` 表明确写「NPS is a nominal size, NOT a measured OD」 |
| Cv / Kv 混用 | 差 0.865 倍 | `cv_to_kv()` 单一换算点 |
| 输出无单位标签 | 数值错了也看不出来 | 界面与 CLI **同时输出 SI 与 Imperial**；界面还有一键一致性自检 |

`os-modules` 用另一套 `Quantity` 边界层把 **37 个计算模块**也接到同一套 SI 规范上：任何模块的公制输入与等价英制输入，经边界换算后走**完全相同**的计算路径，因此结果不随显示单位制变化。UI 里 SI → US → SI 往返也不会出现 `20.00000000000853` 这类浮点尾巴（显示层 `nice()` 处理，存储值不变）。

核对工具：

```sh
open-solutions verify-units            # 端到端一致性自证（不一致退出码 1）
open-solutions unit-check              # 17 条基准换算对照表
open-solutions unit-rules              # 数据库每一列的规范单位与警示
open-solutions convert --length "4 in"
open-solutions convert --temp "212 F"
```

## SQLite 数据层

- DDL 单一来源 `schema.sql` v2，编译期用 `include_str!` 嵌入，`Database::migrate()` 幂等。
- 默认库 **`data/open_solutions.db`（相对 exe）**；`--db <path>` 或环境变量 `OPEN_SOLUTIONS_DB` 可覆盖 CLI。
- 连接强制 `PRAGMA foreign_keys = ON`；`projects` / `calc_runs` 记录工程与计算历史。
- **导入清单表** `source_files` / `source_tables`：记录源 `.mdb` 文件名、字节数、SHA256、Jet 版本、原表名与 SQLite 表名的对应关系。
- **`unit_conventions` 表**把每个数值列的规范单位与陷阱写成数据，`unit-rules` 命令与界面「单位设置」页可随时审计。

业务表 ↔ 源文件对应关系：

| 表名 | 可见文件名线索 |
|---|---|
| `fuels` | `Fuel.mdb` |
| `burners` | `Gasbrn.mdb`, `GasbrnConvert.mdb`, `Combburn2.mdb`, `AsphaltBrn1.mdb` |
| `blowers` | `Blower1.mdb`, `EclipseBlower1.mdb` |
| `valves` | `Valve1.mdb`, `Valve1Convert.mdb` |
| `orifices` | `Orifice.mdb`, `Orifice1.mdb` |
| `pipes` | `Pipe1.mdb`, `PipeSchedule40.mdb` |
| `fittings` | `Fitting1.mdb` |
| `materials` | `Material.mdb`, `Material1.mdb` |
| `regulators` | `GasPressRegulator.mdb` |
| `catalog_refs` | 使用者自有厂商资料索引（不存厂商数据） |

录入自己的数据（界面「设备目录」页可直接新增燃料）：

```sql
INSERT INTO fuels (code, name, lhv_mj_kg, hhv_mj_kg, density_kg_nm3, source)
VALUES ('MY-NG', 'My natural gas', 48.0, 53.1, 0.78, 'own lab measurement');

INSERT INTO pipes (code, schedule, nps_inch, od_mm, wall_mm, source)
VALUES ('P-2-SCH40', 'SCH40', 2.0, 60.3, 3.91, 'own catalog');
```

界面「设备目录」页可直接新增燃料，并会显示 20 个源文件 / 496 张源表 / 4570 行的导入统计。

## 源程序 MDB 数据导入（2026-11-10）

源程序安装目录 `C:\Program Files (x86)\Honeywell International\ESolutions600\DatabasesPwd\` 下有 **20 个加密 Access `.mdb`**（Jet3 / Jet4 混合，共约 11 MB）。主程序内明文存有连接串口令（`;pwd=FelixRick`、`;pwd=Leonardo`）；读取走 **32 位 PowerShell + `Microsoft.Jet.OLEDB.4.0`**（64 位 ACE 拒绝 Jet3 且 `Program Files` 只读无法建 `.ldb` 锁文件，所以先复制到可写工作区）。

> **踩坑**：Jet 4.0 在一个进程里第一次密码校验失败后会污染整个进程的密码缓存，同一进程内重试必然继续失败；而且同一批库在同一进程里连开时，个别库会随机报 `Not a valid password`。最终方案是**每个 .mdb 用独立的 32 位进程导出**，失败就换进程重试（最多 15 次），20/20 全部成功。

导入分两层，全部写进同一个 `open_solutions.db`：

| 层 | 内容 |
|---|---|
| **原样层** | 496 张 `src_<库名>__<原表名>` 表，**4570 行**逐字段原值保存（列名、类型、空值都不改）。逐表比对 0 处行数不符 |
| **映射层** | 10 张业务表共 **3024 行**：`fuels` 23、`fuel_gas_components` 17、`burners` 1131、`blowers` 479、`valves` 1001、`orifices` 115、`pipes` 55、`fittings` 132、`materials` 50、`regulators` 21 |

每一行业务数据都带 `source = 'ESolutions600:<库>.mdb/<原表>'`，并在 `note` 里保存**完整原始行 JSON + 单位假设**，派生值不会被误认为实测值。

### 单位换算假设（重要）

| 表 | 假设 |
|---|---|
| `fuels` | 气体 `GrossHeatSI/NetHeatSI` 为 **MJ/Nm³ @0 °C**，`Gravity` 为相对空气比重；密度 `= Gravity × 1.2929 kg/Nm³`，`MJ/kg = MJ/Nm³ ÷ 密度`。燃油的 SI 列已是 MJ/kg，液体密度**不写入** `density_kg_nm3` |
| `burners` | 仅 `AsphaltBrn1.HeatCapMax` 与 `dbTriOx1.NominalBurnerHeat` 按 **MMBtu/h × 293.07107 → kW** 换算；其余容量/压力字段单位无据可查，**保持 NULL**，原值留在 `note`（1131 行里 138 行有容量） |
| `blowers` | `SCFH/FlowRate` 按标准 ft³/h × 0.028316846592 → Nm³/h，`BHP × 0.745699872 → kW`；`Pressure` 单位未明（psig / osig / in.w.c. 皆可能），**不猜**，留 NULL |
| `valves` / `fittings` | `CV` 即美制 gpm @1 psi（与 `unit_conventions` 一致）；`ID` 是**实测内径 inch** × 25.4 → mm，**不是公称 DN**；`RESISTK` 为无量纲 K 因子 |
| `orifices` / `pipes` | `Bore` inch × 25.4 → mm；Sch40 的 SI 列已是 mm，`WallThicknessUS` inch × 25.4；`nps_inch` 由分数标签解析，保持**名义尺寸**语义 |
| `materials` | `Density` lb/ft³ × 16.018463 → kg/m³；`Tmax` °F → °C |
| `regulators` | `Pmax/Pout1/Pout2` **假定 psig** × 6.894757 → kPa（源库未声明，`Poffset` 又是 in.w.c.）——**设计使用前必须核对样本** |

### 复现导入

```bat
app\tools\import_source_data.bat
```

依次完成：复制源 `.mdb` → `tools\export_all.ps1`（独立 32 位进程逐库导出 JSON 到 `import_work\dump2\`）→ `tools\import_mdb.py`（写 SQLite）。可用环境变量 `ESOLUTIONS_MDB_DIR` 覆盖源目录。导入器**幂等**：重跑只替换 `src_*` 表与 `source` 以 `ESolutions600:` 开头的行，**用户自行录入的数据不会被删除**。导入前自动备份见 `dist\data\open_solutions.db.bak_before_import`。

## CLI

| 命令 | 作用 |
|---|---|
| `init-db` / `tables` / `modules` | 建库迁移、行数、模块清单 |
| `verify-units` | 端到端自证：同一物理量分别用 SI 与英制输入，结果必须一致（不一致退出码 1） |
| `unit-check` / `unit-rules` | 基准换算表、各列规范单位 |
| `convert --length "4 in"` / `--temp "212 F"` | 显式带单位输入，SI + 英制双输出 |
| `list-fuels` / `add-fuel` | 燃料查询与录入 |
| `calc-gas --m 28.97 --tc 0 --p 101.325` | 理想气体密度，也可用 `--tf` / `--psia` / `--psig` |
| `calc-pipe --d "4 in" --cfm 100` | 管内流速，也可用 `--d "100 mm"` / `--flow <m3/h>`；`--rho --mu` 算 Re |
| `calc-heat --mdot 1000 --lhv 50` | 放热功率；也可用 `--lbh` / `--btulb`，`--hhv` 切换基准 |

## 本机重新编译

成品无需编译。若要改代码，在本机已配好的环境执行：

```sh
export PATH="/c/Users/ekinz/.cargo/bin:/c/Program Files/JetBrains/CLion 2026.2.4/bin/mingw/bin:$PATH"
export CC=gcc
cd "Open Solutions/app"
cargo test --workspace
cargo build --release --workspace
```

`rusqlite` 使用 `bundled` feature 从源码编译 SQLite，因此需要 C 编译器；这里用 CLion 自带的 mingw-w64，而不是 MSVC Build Tools。

## 目录结构

```
app/
├── Cargo.toml                 # workspace：os-core / os-modules / os-cli / os-gui
├── schema.sql                 # SQLite DDL v2（仅结构，不含源数据）
├── README.md
├── data/open_solutions.db
├── tools/
│   ├── import_source_data.bat # ★ 一键复现导入
│   ├── export_all.ps1         # 逐库独立 32 位进程导出驱动
│   ├── export_one.ps1         # 单库导出（Jet OLEDB 4.0）
│   └── import_mdb.py          # JSON → SQLite（原样层 + 映射层）
├── crates/
│   ├── os-core/src/           # units / model / calc / storage + 24 个测试
│   ├── os-modules/src/        # quantity / defs(41 模块) / engine + 11 个测试
│   ├── os-cli/src/main.rs     # CLI
│   └── os-gui/src/
│       ├── main.rs            # 本地 HTTP 服务 + Edge 应用窗口启动器
│       └── ui.html            # 单文件界面（编译期内嵌）
└── dist/                      # ★ 交钥匙成品
    ├── 启动 Open Solutions.bat
    ├── open-solutions-gui.exe # 图形界面
    ├── open-solutions.exe     # CLI
    ├── verify.bat             # 一键自检
    ├── schema.sql / README.md
    └── data/open_solutions.db
```

## 边界与后续

`os-core::calc` 与 `os-modules::engine` 只实现教科书级通用关系，**不含任何厂商系数**；用于实际设计前请用自己的基准数据校验。

**当前状态**：导入的目录数据已在 SQLite 中可查询、可在「设备目录」页浏览，但**计算内核尚未按目录查表选型**（37 个计算模块仍用输入参数计算）。把 `src_*` 表接到选型逻辑、按样本校准上面的单位假设，是下一步工作。

其余可迭代：把 VB6 基准值做成自动对比测试、打印设计报告。

Craft Anything 的自动 rebuild 在本机仍缺 coding agent（Copilot CLI 未安装），Study 阶段 Clean-room 已通过但 Build 失败；本项目代码由 WorkBuddy 直接实现并编译验证，**不依赖那条链路**。项目骨架保留在上一级 `Open Solutions/`（`craft.json`、`capture/`、`investigation.cany/`）。

许可：MIT，**仅覆盖本项目原创代码**；导入的厂商目录数据不在许可范围内，亦不包含在源码树中。
