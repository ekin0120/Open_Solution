# KARDOX Open Solutions

燃烧与传热工程计算平台 —— 用 Rust 从零实现的工业燃烧计算工具集，覆盖 **41 个工程计算模块**、双单位制（SI / US）、燃料物性数据库与可打印的工程报告输出。

> **版本**：v0.1.0（初始版本，2026-11-13） · **平台**：Windows 10+ x64 · **许可**：MIT（仅原创代码）
> **下载**：https://github.com/ekin0120/Open_Solution/releases

---

## 一、软件介绍

Open Solutions 是一台「装进单个 exe」的燃烧工程计算器：

- **单文件、免安装**：`open-solutions-gui.exe` 内嵌全部界面、图标、燃料数据与计算内核，双击即用。启动后在本地回环地址（127.0.0.1，仅本机可访问）起一个 HTTP 服务，并打开 Edge 应用模式窗口（无地址栏、无标签页，观感即桌面软件，**无 cmd 黑窗**）。
- **纯净计算内核**：全部工程公式为教科书级通用关系（公开的燃烧学 / 传热学 / 流体力学关系式），类型化的量纲系统保证公制与英制输入走**完全相同**的计算路径。
- **数据层**：SQLite（schema v2），内置 16 种标准气体 + 6 种油燃料的物性库，支持用户自录燃料与设备目录数据；每次计算自动写入历史记录。
- **双语界面**：全站中英双语（`EN · 中文`），可跟随切换。
- **品牌**：KARDOX（界面侧边栏竖版 logo、窗口图标、exe 程序图标、打印报告页眉）。

### 架构

```
Rust 后端 (os-gui.exe) ── 本地回环 HTTP（127.0.0.1:随机端口，不监听外网）
   ├─ /api/modules        模块注册表（41 项，数据驱动）
   ├─ /api/compute        os-modules 计算内核
   ├─ /api/convert-fields SI ↔ US 字段换算
   ├─ /api/audit          单位约定审计
   ├─ /api/db /api/fuel   SQLite 状态与数据录入
   └─ /                   内嵌单文件 UI（编译期 include_str!）
                              ↓
                    Edge --app 独立应用窗口
```

| crate | 职责 |
|---|---|
| `os-core` | 类型化单位系统（Length/Pressure/MassFlow/…，内部统一 SI）+ 存储层 |
| `os-modules` | 41 个工程模块的计算引擎 + Quantity 边界层 |
| `os-gui` | 本地 HTTP 服务 + 单文件 HTML/CSS/JS 界面 |
| `os-cli` | 命令行工具（单位自检、换算、燃料查询、快速计算） |

### 关键设计：类型化单位系统

界面右上角 `SI 公制 / US 英制` 一键切换，任何输入量的公制值与等价英制值经边界换算后走同一计算路径，**结果与显示单位制无关**。核心对策：

- 每个物理量独立类型（`Length` / `PressureAbs` / `VolumetricFlow`…），内部统一 SI，换算只在 I/O 边界做**一次**；
- 温度用 K，°F 为仿射变换；表压 / 绝压、标况 / 工况流量、LHV / HHV 均带标签，错用直接报错；
- `38/38` 单元测试通过（含 SI↔US 往返、双单位一致性锚点）。

---

## 二、使用方法

### 1. 安装与启动（推荐：Release 便携包）

1. 从 [Releases](https://github.com/ekin0120/Open_Solution/releases) 下载 `Open_Solution-v0.1.0-win64.zip`；
2. 解压到任意目录；
3. 双击 **`open-solutions-gui.exe`** → 自动打开应用窗口，无需安装、无需 Python。

> 需要系统自带 Microsoft Edge（缺失时回退默认浏览器）。关闭窗口或点「退出」即结束服务。

### 2. 界面操作

| 操作 | 说明 |
|---|---|
| **选模块** | 左侧导航四个分组（参考 / 应用 / 系统设计 / 工具），顶部支持中英文关键词搜索 |
| **输入** | 每个字段带显式单位，下拉字段直接选燃料 / 流体 / 形状 |
| **单位制** | 右上角 `SI 公制 / US 英制` 一键切换，输入与结果同步换算 |
| **计算** | 点「▶ 计算」出结果表：主列为当前单位制，对照列为另一单位制 |
| **自检** | 点「⇄ 双单位一致性自检」验证当前模块 SI↔US 往返一致性 |
| **历史** | 每次计算自动写入 SQLite `calc_runs`，可在「设备目录」页查看 |
| **打印 / PDF** | 右上角「打印 / PDF」→ 弹出打印对话框 → 选「另存为 PDF」，生成带 KARDOX 页眉的 A4 报告 |

### 3. 打印报告

全 41 模块通用。报告排版：KARDOX 横版 logo 页眉 + 蓝色分隔线、输入参数表、双单位对照结果表（长表自动分页且行不跨页断开）、页脚品牌落款。背景色强制打印，黑白打印机亦可读。

### 4. CLI（可选）

```sh
open-solutions verify-units            # 端到端单位一致性自证（不一致退出码 1）
open-solutions unit-check              # 17 条基准换算对照表
open-solutions modules                 # 模块清单
open-solutions list-fuels              # 燃料库查询
open-solutions convert --length "4 in" # 显式带单位换算，双单位输出
open-solutions calc-heat --mdot 1000 --lhv 50   # 放热功率
```

### 5. 从源码构建

```sh
cargo test --workspace          # 38 个测试
cargo build --release --workspace
# Windows 图标注入（GNU 工具链无 rc.exe，用 UpdateResource 后处理）
python tools/set_icon.py target/release/os-gui.exe
```

`rusqlite` 使用 `bundled` feature，需要 C 编译器（mingw-w64 GCC 即可，无需 MSVC）。

---

## 三、功能总览（41 个模块）

| 分组 | 模块 |
|---|---|
| **Reference 参考**（19） | 燃料数据、气体与液体物性、固体物性、燃料燃烧、沃泊指数、余热回收节能、流体力学、不完全燃烧损失、混合气可燃性、孔板流量计、自然对流、强制对流、辐射换热、燃烧噪声、排放换算、风机定律、单位换算、工程数据表、几何工具 |
| **Applications 应用**（11） | 低温骨料干燥、空气预热器、空气/燃气管路设计、烟道与烟囱尺寸、微调阀流量特性、炉体散热损失、TBA 鼓风机选型、风机海拔/温度修正、SMJ 鼓风机选型、燃烧器/阀门海拔修正、引射器 |
| **System Design 系统设计**（7） | ① 炉膛规格 ② 燃烧器选型 ③ 空气管路与部件 ④ 鼓风机选型 ⑤ 燃气管路与部件 ⑥ 设计报告 ⑦ 安全联锁检查 |
| **Tools 工具**（4） | 关键词检索、术语表、设备目录、单位设置 |

### 重点模块

- **燃料数据**：交互式燃料选择器——17 种预设气体燃料（组分锁定只读）+ 自定义混合气（10 组分体积分数，自动归一化），从第一性原理重算 12 项物性（分子量、比重、密度、质量/体积 LHV·HHV、理论需氧/空气、湿烟气、干基 CO₂）。
- **气体与液体物性**：25 种流体（18 气 + 7 液）× 19 温度点 × 9 项物性（ρ/SG/cp/μ/ν/h/γ/k/Pr），纯流体粘度/比热/导热与参考实测数据位级对齐（0.00%）；自定义混合气支持 12 组分（不足 100% 自动补 N₂），混合规则与参考数据 ≤0.4%。
- **燃料燃烧**：5 类燃料选择 + 16 组分自定义气 + 3 种燃料输入单位 + 环境/湿度/富氧/FGR；两页 77 行完整燃烧报表（组成、理论空气、湿/干烟气产物、可用热、绝热火焰温度、露点、流量、质量平衡）。
- **余热回收节能**：Case I/II 各自选燃料（25 项清单）+ 6 项燃烧条件，Shomate 真实气体焓内核（含空气预热、富氧化学计量、FGR 修正），输出可用热对比与节能率。
- **沃泊指数**：燃料对互换性判据 + 掺空气混配解（混配比例、混气热值/比重/Wobbe）。
- **燃烧器/阀门海拔修正**：16 燃料 → 海拔/温度/过剩空气 → 实际风量、标况风量、工况风压（Δp ∝ ρQ²）。

### 数据与录入

- 内置燃料库：16 种标准气 + 6 种油（`os-modules/combustion_fuels.rs` 编译期内嵌）；
- SQLite 业务表：`fuels` / `burners` / `blowers` / `valves` / `orifices` / `pipes` / `fittings` / `materials` / `regulators`，界面「设备目录」页可直接录入；
- `unit_conventions` 表把每列规范单位与陷阱写成数据，CLI `unit-rules` 与「单位设置」页可随时审计；
- 详细的 MDB 数据导入流程见下文「附：源数据导入」。

---

## 四、版本历史

### v0.1.0 — 初始版本（2026-11-13）

- **41 个工程计算模块**全部实装：类型化量纲边界、SI/US 双单位、一致性自检全模块 PASS；
- **单文件 GUI**：Edge 应用窗口、KARDOX 品牌化（exe 图标 / 侧边栏 logo / 打印页眉）、无 cmd 控制台窗口；
- **打印 / PDF 输出**：全模块通用的 A4 报告（logo 页眉、双单位对照表、自动分页）；
- **燃料数据层**：16 气 + 6 油内置库、自定义混合气、SQLite 历史/目录/单位审计；
- **源程序一致性验证**：对参考实现逐模块对照（详见 `docs/VERIFY_REPORT.md` 八个批次），Energy Savings / Wobbe Index / Fuel Combustion / Burners at Altitude / 气液物性等核心模块锚点**逐位吻合**；
- **CLI**：单位自检、换算、燃料查询、快速计算命令集；
- **测试**：`cargo test --workspace` 38/38 通过。

---

## 附：源数据导入（可选，内部使用）

源程序安装目录下的 20 个加密 Access `.mdb` 目录数据可导入 SQLite（原样层 496 张 `src_*` 表 + 映射层 10 张业务表）。这些是厂商专有工程数据，**仅供个人研究 / 内部自用，不要随本项目源码或成品对外分发**；`dist\data\` 中的库如需分享，请先清空 `src_*` 表与业务表中的导入行。

```bat
app\tools\import_source_data.bat    # 一键复现导入（幂等，不删用户自录数据）
```

单位换算假设、逐表对应关系与踩坑记录见 `tools/` 内脚本注释与 git 历史。

**净室（Clean room）边界**：本项目**不复制**任何 Honeywell / Hauck 的程序代码、图标、商标或界面文案；模块划分仅依据可见界面文字推导，所有公式均为公开通用工程关系。

---

## 目录结构

```
app/
├── Cargo.toml                 # workspace：os-core / os-modules / os-cli / os-gui
├── schema.sql                 # SQLite DDL v2
├── docs/VERIFY_REPORT.md      # 源程序一致性验证报告（A–F 批）
├── data/open_solutions.db     # 数据层
├── tools/
│   ├── set_icon.py            # Windows exe 图标注入（纯 ctypes，零依赖）
│   ├── import_source_data.bat # MDB 一键导入
│   └── import_mdb.py          # JSON → SQLite
├── crates/
│   ├── os-core/src/           # 单位系统 / 存储层（24 测试）
│   ├── os-modules/src/        # 41 模块引擎（14 测试）
│   ├── os-cli/src/main.rs     # CLI
│   └── os-gui/src/
│       ├── main.rs            # 本地 HTTP 服务 + 应用窗口启动器（windows_subsystem=windows）
│       ├── ui.html            # 单文件界面（含 KARDOX logo / 打印视图）
│       └── assets/            # icon.ico / favicon / logo 资源
└── dist/                      # 本机成品目录（不入库）
```

## 边界声明

`os-core::calc` 与 `os-modules::engine` 只实现教科书级通用关系，**不含任何厂商系数**；用于实际工程设计前请用自己的基准数据校验。导入的厂商目录数据不在 MIT 许可范围内。
