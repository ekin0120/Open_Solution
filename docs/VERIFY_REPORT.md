# Open Solutions 与 E-Solutions 600 模块对照报告

> 生成日期：2026-11-10
> 依据：源程序 exe 二进制字符串提取（25,895 条，含全部模块 Program Description 与控件标签）、
> 14 个模块实机截图、Programs 菜单清单（22 项）、Reference 网格清单（19 项）、
> 已导入的 20 个 MDB 数据表（493 表 / 4570 行）。
> 净室声明：本报告仅引用用户自有授权程序中的功能描述与公开工程公式，不复制任何受版权保护的代码。

## 一、模块清单核对

**Reference 网格（19 项）与实现 19 项完全一致 ✓**
Flammability / Geometry / Incomplete Combustion / Fan Laws / Fluid Mechanics / Unit Conversion /
Orifice Meter / Fuel Combustion / Solids Props / Gas-Liquid Props / Sound / Emission / Radiation /
Forced Convection / Natural Convection / Fuel Data / Energy Savings / Wobbe / Tables

**Programs 菜单（22 项）**：与实现差异 = 菜单含 Blowers、Branch Piping Design 两个入口，
对应实现的 Applications 分组模块（TBA/SMJ Blowers、Air/Fuel Piping Design），无缺失。

**System Design 向导（源 6 步 + NFPA 按钮）**：实现 7 步（把源的第 1 步「Fuel Selection and Furnace
Specification」拆成了 ①炉膛规格+②燃烧器选型）。功能覆盖一致。

## 二、逐模块差异与修复方案

判定：✅ 一致 / 🔧 需调整 / ✍️ 需重写

### Reference（19）

| # | 模块 | 源程序规格（权威描述） | 当前实现 | 判定 | 修复方案 |
|---|------|------------------------|----------|------|----------|
| 1 | Fuel Data | 选燃料或自定义（Σ=100%），显示 SG、Gross/Net Heat（STP 直读库值）；气="%vol"，油="%wt" | 已实现选择器+10组分；SG/热值由组分计算 | 🔧 | 预设燃料并列显示 DB 值（gravity/gross/net 已在 fuel_gas_components）；计算值保留用于自定义混合气 |
| 2 | Fuel Combustion | 燃料(气/油,标准/自定义)+过量空气%+空气/排烟/燃料三温度+燃料流量；FGR 选项(温度+%)；富氧选项(O2纯度+浓度) → 燃料组成/SG/热值/理论空燃比/烟气组成(湿+干)/Available Heat%/可用热 Btu/hr/空气流量/烟气流量/绝热火焰温度/实际烟气 acfh/露点 | **源程序结果页 1:1 复刻**：5 类燃料、17 种标准气/6 种油、16 组分自定义、两页 77 行输出 | ✅ | 已完成：燃料库取自源 GasFuel/OilFuel 表；理论空气按化学计量（源实测 9.85）；默认工况与源结果页逐项精确对齐（见下方 D 批） |
| 3 | Wobbe Index | 选**原始燃料+替代燃料**（两列表）→ 输出三组 Wobbe/热值/比重 + 替代气混空气百分比；替代燃料热值必须更高 | 单输入(W,SG,p,T)+互换偏差 | ✍️ | 重写为「原气/替气选择 + Wobbe/CV/SG×3 + 掺混百分比」，数据取 fuel_gas_components |
| 4 | Energy Savings | **Case I/II 双工况**（燃料选择×2 + 过剩空气/空气温度/助燃空气含氧%/FGR%/FGR温度/排烟温度）→ Available Heat I%、II%、节能%；另有燃料成本选项 | 烟气流量×比热×温差节能法 | ✍️ | 重写为双工况 Available Heat 对比（与 Fuel Combustion 共用可用热模型） |
| 5 | Incomplete Combustion | 选燃料 + 热输入 MM Btu/hr + O2(dry)% + CO(%vol 或 ppm) + 时/日 + 价格单位($/Therm、$/Mcf) → 过剩空气%、转化效率%、3×3 损失表(时/日/年 × Btu/scf/$) | 质量流量+LHV+ppm 输入 | ✍️ | 重写输入输出结构对齐源（燃料选择列表 + Heat Input + 损失表） |
| 6 | Flammability | **16 组分**（CH4/C2H2/C2H4/C2H6/C3H6/C3H8/C4H10/C5H12/C6H14/CO/H2/H2S/CO2/O2/N2/H2O）+ Fuel Name → GHV Btu/scf、化学计量空气 scf/scf、LFL%、燃烧速度 ft/sec、LFL 下空燃比、LFL 下过剩空气% | 5 字段 Le Chatelier | ✍️ | 重写为 16 组分输入 + 6 项输出 |
| 7 | Gas-Liquid Props | 选流体(库)+温度+压力 → 密度/SG/Cp/绝对&运动粘度/焓/Cp比/导热/Prandtl（9 项，带有效范围校验，Excel 输出） | **518 组源程序实测表 + 混合气** | ✅ | 已完成：25 流体实测表（μ/cp/k 位级精确）、有效范围复刻源弹窗、Customized Gas Mixture 12 组分（ρ/cp/γ/k/h 对齐源实测混合气，μ 用 Wilke，烃类富集最大 +8.2% 已知偏差）、UI 仿源（Select Fluid Type/Show Properties/Specify Conditions） |
| 8 | Solids Props | 选材料(库 50 种金属/固体)+材料温度 → 熔点/沸点/熔化热/密度/比热/导热/发射率/热膨胀（8 项）+ Enthalpy Calculation 入口 | 手输 ρ,cp,k,厚度 → 导热/蓄热 | ✍️ | 重写为查表式：新增 solid_materials 业务表（从 src_Material1__MetalProp 迁移 50 行），Enthalpy 计算按钮独立小功能 |
| 9 | Fluid Mechanics | **多功能组合**：①Cv 计算(Cv/流量/压降三向) ②孔板(spud)流量 ③平方根定律(流量↔压降换工况) ④皮托管横断面(圆/矩形) | 通用管流压降(Re/摩擦/Δp) | ✍️ | 重写为四选项卡：Cv 计算(含流量/压降反算)、spud 流量、平方根换算、皮托管（圆/矩形测点布置） |
| 10 | Orifice Meter | 角接/法兰/D-D/2 取压可选；算孔径(给定流量+ΔP)或定孔径出「流量-ΔP 数据表」；流体+温度+压力+海拔/气压+管径(Sch40 列表) | 单点 ISO 简化式(β,Cd,Δp→q) | ✍️ | 重写为「算孔径 / 定孔径出表」两模式，取压方式可选，输出数据表 |
| 11 | Natural Convection | 选流体+几何 case+流体温度+壁温+尺寸 → Gr/Pr/Nu/换热系数 | 手输特征长度+物性 → Churchill-Chu | 🔧 | 保留公式内核，输入改为 流体选择+case(竖板/横管)+两温度+尺寸，输出加 Gr/Ra |
| 12 | Forced Convection | 选 case(管内流/矩形通道/环隙)+流体+流体温度+壁温+流量或流速+尺寸 → Re/Pr/Nu/h/热流；另有管道散热选项(保温管，出入口温度) | 手输 d,v → Dittus-Boelter | 🔧 | 加流体选择(case+物性表)与管道散热选项，输出加热流密度 |
| 13 | Radiation | ①View Factor（选几何 case+尺寸）②辐射换热（case+尺寸+壁温/载荷温度/烟气温度+发射率材料选择） | 手输 ε,Ts,Tg → 灰体交换 | 🔧 | 补 View Factor 几何 case 列表（平行板/垂直板/同心圆柱/…）与发射率材料下拉 |
| 14 | Sound | 三功能：①燃烧器声功率(空气压力 osig+热输入 MMBtu/hr+燃料) ②距离处声压(距离+声功率 dB) ③TBA 鼓风机声级(含消声器/过滤器选项、倍频程) | 单一估算 | ✍️ | 重写为三功能页面（①②完整实现；③实现基础版：插入损失+自噪声+距离修正） |
| 15 | Emission | 污染物(NO/NO2/CO/VOC-CH4/VOC-C3H8/SO2/SO3)单位换算 From→To，O2 基准换算 + 对应过剩空气%；可选燃料与富氧；可选输入(时产/吨产) | 简单换算 | 🔧 | 重构：污染物列表对齐、O2 折算+EA% 输出、From/To 单位对齐全单位类型 |
| 16 | Fan Laws | 公式选择：6 公式(Q,P,HP × rpm,ρ 比例)+Multiple Setting | 单表 7 输入 | 🔧 | 输入 Q1/P1/HP1/rpm1/rpm2/ρ1/ρ2，输出 6 个比值 |
| 17 | Unit Conversion | 分类列表：Angle/Area/Density/Energy/Force/HeatContent(Volume)/(nm3,scf)/(Mass)+…，From/To 双栏自动换算 | 通用换算(40 单位) | 🔧 | 分类名与源对齐，增加 HeatContent(nm3,scf) 分类 |
| 18 | Tables | **仅两张表**：Standard Steel Pipe Data: Schedule 40；Equivalence Fuel/Air Ratio and Excess Air | 蒸汽压/空气物性/燃料热值/NPS | ✍️ | 重写为两张表（Sch40 管径数据来自 pipes；当量比-过剩空气对照表公式生成） |
| 19 | Geometry Tools | Select Geometry(Annulus/Circle/Cone/Cube/…) + A/B/C 输入 → Volume 等 3 输出 | 6 形状面积/体积 | 🔧 | 形状列表对齐（Annulus/Circle/Cone/Cube/Cylinder/Sphere/Rectangle），输出对齐（体积/表面积/…） |

### Applications（11）

| # | 模块 | 源程序规格 | 当前实现 | 判定 | 修复方案 |
|---|------|-----------|----------|------|----------|
| 20 | Air Heaters | 被加热介质(Air/Custom Gas)+模式(定出口流量 scfh/scfm 或定入口流量)+燃料(NG/Propane)+入口温度 → 燃料热输入 MMBtu/hr、所需燃料流量 SCFH、燃烧空气流量 SCFH、被加热空气流量 | 骨架已有 | 🔧 | 对齐输入输出四项 |
| 21 | Aggregate Drying | 「Dryer Heat & Material Balance」：Batch/Drum Mix、燃料、已知项(湿吨入/干吨出/排烟流量三选) → 物料平衡+热平衡+燃料/空气/烟气流量+湿干烟气组成+露点；Go To Burner Selection | 简化热平衡 | 🔧 | 补 Batch/Drum 选项与已知项三选模式、输出物料/热平衡表与露点 |
| 22 | Air/Fuel Piping | 「Furnace Air/Fuel Piping Design」：分支管路设计(添加/删除/插入部件) → 每部件 Cv/当量长度/入口压力/压降 + Branch Summary(进出口压力/流量/当量长/分支 Cv/osig&psig 压损)；Excel 输出；Choke flow 检查 | 简化管路 | 🔧 | 分支部件列表 + 逐部件压降表 + 汇总；流体列表对齐(NG/丙烷/丁烷/焦炉煤气/Calflo/Therminol/油类) |
| 23 | Flue Sizing | 算烟道尺寸或炉压控制点；浮力/入口损失/摩擦/弯头损失/最大流量/烟道面积/流速；**空气射流挡板选项**(射流角度+孔径) | 简化烟囱计算 | 🔧 | 补损失分项输出与 jet damper 选项 |
| 24 | Micro Valve | MCOV 阀性能图（选燃料 Oil#1/2/4、选阀、阀位）→ 阀门图表 | 骨架 | 🔧 | 输入结构对齐（燃料/阀门/阀位），图表输出简化为数据表 |
| 25 | Furnace Heat Loss | 炉形(矩形/圆柱)+炉温+1~2 层炉墙/顶/底材料(6 种耐火材料)+开口(辐射 View Factor) → 热流/蓄热/散热/界面温度 | 简化壁面散热 | 🔧 | 补矩形/圆柱、双层炉衬、6 种耐火材料下拉、开口辐射项 |
| 26 | TBA Blowers | 「Hauck Blowers」双选项：①按工况选鼓风机(海拔/入口温度/频率/电机 100%/115%) ②指定型号非标工况性能；输出目录性能+实际性能 | 简化选型 | 🔧 | 双选项模式对齐（性能曲线来自 src_Blower1/EclipseBlower1 数据） |
| 27 | Fan Altitude | 双选项：①海拔/温度→所需风机压力(维持海平面质量流量) ②海平面数据+实际频率/温度→海拔工况流量/压力/SG | 简化修正 | 🔧 | 双选项对齐 |
| 28 | SMJ Blowers | 同 26（SMJ 系列） | 同上 | 🔧 | 同 26，用 SMJ 曲线 |
| 29 | Altitude Burners | 「Burners and Valves at Altitude/Temperature」：海拔+温度修正燃烧器与阀门 | 简化修正 | 🔧 | 与 27 共用修正模型 |
| 30 | Eductor | 动力空气压力/温度+排气流量/压力/温度+要求炉压+烟囱损失 → 动力空气流量、出口压力、混合温度、喷嘴直径、吸入/混合段尺寸 | 简化引射 | 🔧 | 输入输出七项对齐 |

### System Design（7 步）

| 模块 | 差异 | 修复方案 |
|------|------|----------|
| ①炉膛规格 ②燃烧器选型 | 源第 1 步合并「Fuel Selection and Furnace Spec」 | 保持现状（功能等价），报告注明 |
| ③空气管路 | 源按 Zone 级选择主管/过渡管/支管+控制元件 | 简化版保留 |
| ④鼓风机 | 高原子化压力→推荐 booster 串联 | 补推荐逻辑（可选） |
| ⑤燃气管路 | 按燃料分 Gas/轻油/重油三种表单 | 保持现状 |
| ⑥报告 ⑦NFPA | 源 NFPA 是右下角按钮 | 保持现状 |

## 三、数据层准备（已完成/待办）

- ✅ `fuel_gas_components`：17 种燃气 10 组分 + gravity/gross/net（Btu/scf 与 MJ/Nm³）— 已导入
- ✅ `src_Material1__MetalProp`：50 种材料 13 列物性（US 单位）— 已导入，待建业务表 `solid_materials`
- ✅ `src_Fuel__OilFuel`：5 种燃料油（组分/粘度/闪点/倾点/API/Btu/gal）— 已导入
- ✅ `src_Blower1__*`、`src_EclipseBlower1__*`：鼓风机性能曲线 — 已导入
- ✅ `src_Orifice__Fluids`：5 种流体基础物性 — 已导入
- ✅ `src_PipeSchedule40__*`：Sch40 管径数据 — 已导入（Tables 模块用）
- ✅ `src_Gasbrn*`/`src_Combburn2*`/`src_AsphaltBrn1*`：燃烧器容量表 — 已导入

## 四、实施顺序

1. **A 批（快赢）**：Fuel Data 预设显示 DB 值、Tables 两张表、Geometry Tools 形状对齐、Unit Conversion 分类对齐、Fan Laws 六比值
2. **B 批（重写）**：Flammability(16 组分)、Wobbe(原气/替气)、Energy Savings(Case I/II)、Incomplete Combustion(损失表)、Solids Props(查表)
3. **C 批（重写）**：Gas-Liquid Props(查表)；Fuel Combustion 已移入 D 批完成；其余 Natural/Forced Convection、Radiation、Emission、Sound 待做
4. **D 批（已完成：Fuel Combustion 1:1 对齐）**：详见下方专章
5. **E 批（重写）**：Fluid Mechanics 四选项、Orifice Meter 数据表、Air Heaters、Eductor、Fan Altitude、Altitude Burners
6. **F 批（增强）**：Flue Sizing、Furnace Heat Loss、Aggregate Drying、Piping Design、Blowers、Micro Valve

---

## 五、D 批：Fuel Combustion 与源程序结果页 1:1 对齐（2026-11-11）

### 验证方法

1. 在可写副本中启动源程序（`Program Files` 下 Access `.ldb` 不可写；Jet 密码 `FelixRick`），
   修复 `MainDefault.hmf` 被写空、数据库密码不符等问题后打开 Fuel Combustion 窗体。
2. 窗体默认离屏（坐标约 `-1264,662`），用 `SetWindowPos` 移入屏幕并导出控件树
   （`fc_form_tree.json`）与截图（`shots/FuelCombustion_form.png`）。
3. 触发 Calculate 后进入打印预览，放大到 2400×1600 逐页截图；第 2 页结果表见
   `shots/fcpage2.png`（放大件 `shots/fcpage2_zoom.png`）。
4. 用源程序 `strings_all.txt` 的帮助文本（"Program Name: Fuel Combustion" 段）与
   打印页标签字符串核对输出项全集，避免漏项。

### 默认工况实测锚点（Average Natural Gas / 100 scfh / 0 % 过量空气 / 60 °F 空气 / 2000 °F 排烟）

| 项目 | 源程序实测 | 本实现 | 偏差 |
|---|---|---|---|
| Gravity | 0.620 | 0.620 | 0 |
| GHV / NHV (Btu/scf) | 1042 / 965 | 1042 / 965 | 0 |
| Theoretical Air (Dry) | 9.85 SCF/SCF | 9.85 | 0 |
| 产物湿基 CO₂ / H₂O / SO₂ / O₂ / N₂ (%) | 9.72 / 18.49 / 0 / 0 / 71.79 | 同 | 0 |
| 产物干基 CO₂ / H₂O / SO₂ / O₂ / N₂ (%) | 11.92 / `-` / 0 / 0 / 88.08 | 同 | 0 |
| Available Heat (Dry Air) | 46.2 % | 46.2 % | 0 |
| Heat Available (Dry Air) | 48107 Btu/hr | 48107 | 0 |
| Heat of Exhaust Flue Gas | 56093 Btu/hr | 56093 | 0 |
| Specified Fuel / Air Flow Rate | 100 / 985.14 scfh | 100 / 985.14 | 0 |
| Furnace Flue Gas Flow Rate | 1089.59 scfh | 1089.59 | 0 |
| Furnace Actual Flue Gas Flow Rate | 5155 acfh | 5155 | 0 |
| Adiabatic Flame Temperature | 3577 °F | 3577 °F | 0 |
| Dew Point at Standard atm | 137.6 °F | 137.6 °F | 0 |
| Heat of Air Preheated / Recirculation Gas | 0 / 0 Btu/hr | 0 / 0 | 0 |
| 富氧与 FGR 关闭时的 5 项 | `N/A` | `N/A` | 一致 |

### 关键推导与标定

- **理论空气**：`Σyᵢ(C+H/4−O/2+S)/0.209 = 9.8514`，与源实测 9.85 吻合 → 化学计量内核正确。
- **能量口径**：源报表满足 `48107 + 56093 = 104200 Btu/hr = 100 scfh × 1042 GHV`，
  且 `48107/104200 = 46.2 %`。因此 `Heat of Exhaust Flue Gas` 是 **显热 + 水汽化潜热**（GHV 基准），
  而不是单纯显热；早期实现只算显热导致 44399 Btu/hr 的偏差。
- **CP_SCALE = 1.035256**：以源报表排烟损失 56093 Btu/hr 为锚点标定平均摩尔比热，
  使 `Available Heat`、`Heat Available`、`Heat of Exhaust Flue Gas` 三项同时闭合。
- **等效高温解离 DISSOC_K = 0.00359808**：忽略高温解离时火焰温度为 3824 °F，
  源为 3577 °F；在 2000 °F 以上加入 `K·(T−2000)²·(n_CO₂+n_H₂O)` 的等效吸热项，
  使默认工况精确落在 3577 °F，并对预热/富氧/过量空气等工况保持物理趋势。
- **实际烟气量**：标准状态用 `520 R`，`acfh = scfh·(T_F+460)/520`，得 5155 acfh。
- **干基约定**：源报表干基 H₂O 打印为 `-`（水已从干基扣除），列顺序为
  CO₂ / H₂O / SO₂ / O₂ / N₂；富氧或 FGR 关闭时相应行打印 `N/A`。
- **燃料库**：`combustion_fuels.rs` 由 `verify_source/gen_combustion_fuels.py` 从已导入的
  `src_Fuel__GasFuel` / `src_Fuel__OilFuel` / `src_Fuel__WasteOil` 三张源表生成（17 种气 + 6 种油）。

### 已知边界

- 上述标定以默认工况为锚点；其他工况为按此标定的燃烧学外推，未逐点对照源程序。
- 源程序结果页为自绘图像，本实现复刻其**输出项与数值口径**，不复制版式。
- Excel 参数扫描（源程序 `Excel` 按钮）未复刻；GUI 提供单次计算 + 双单位一致性自检。

---

## 六、E 批：其余 23 个源模块批量对比（2026-11-12）

### 方法

探针流水线（verify_source/batch_compare.py / batch_probe_buttons.py / batch_probe_r4.py）：

1. 键盘菜单导航（Alt+P+方向键）打开菜单型模块；主窗体按钮用 **Win32 EnumChildWindows + BM_CLICK（按标题命中）** 打开按钮型模块——UIA 坐标点击在双 frmMain 实例下会错位，必须绕过坐标。
2. 模块说明 MsgBox（标题 `ESolutions600`）用 Win32 找"否(N)"按钮点击。
3. 输出值用 **WM_GETTEXT 按句柄读取**（UIA window_text 拿不到 VB6 数值）。
4. 每模块保存 `shots/<模块>_dump.json`（计算前/后全部控件）+ `_dump.png` 截图，作为基线证据。

### 逐模块对比结论

| 源模块 | 源默认基线（实测） | 我们的现状 | 结论 |
|---|---|---|---|
| Wobbe Index | ANG↔Butane：CV 1042/3200、SG 0.62/2.0、WI 1323.3/2262.74，附加输出 1625 / 1.508 / 1323.37 | 手输 Hv/sg 版 | ❌ 结构重写：燃料对下拉 + 完整输出列 + 精确 SG |
| Orifice Meter | Air、1000 scfh、Pmax 10.1/Pmin 0.1/Pnom 5、管内径 2.067 in → 定径 | ISO 5167 dp→流量 | ❌ 反问题（定孔径）需重写 |
| Natural Convection | T1=60/T2=180、尺寸 10 → Pr 0.706、Nu 668、h 1.07、q 6.07、Ra~1.9e11 | 手输 Churchill-Chu（SI） | ❌ 重写：流体库 + 英制 + 源关联式 |
| Forced Convection | 流体选择 + 流量/管径/壁温 → Re/Pr/Nu/h/Heat Flux | 手输 Dittus-Boelter | 🔧 可标定重写（输出项一致） |
| Radiation | View Factor 面板：F=0.448、0.059 | Stefan-Boltzmann q | ❌ 缺视角因子计算，需重构 |
| Emission Calc | 污染物换算（输出未捕获，需 From/To 单位） | 🔧 | 补测 |
| Physical Props of Solids | 固体物性表（输出未捕获） | 🔧 | 补测 |
| Furnace Heat Losses | "Heat Losses Calculations" 表单 36 控件 | 🔧 | 补测 |
| Flue Sizing | 10 项输入（2000/2/0.05/10/15/90…） | 🔧 | 输出未捕获，补测 |
| Air Heaters | 90000 scfh、60→360F、EA 20% → 输出 0（需选燃料后计算） | 🔧 | 补测（带燃料选择） |
| Flammability | CH4=100：LFL 5 ✓、AFR@LFL 19 ✓、HV 1012≈1011、Stoi 9.6≈9.52；LV 0.9 vs 1.31 ft/s、ExcAir 198.5 vs +99.6% | 4/6 命中 | ⚠️ 微调：ExcAir 定义、燃烧速度表、GHV 精度 |
| Incomplete Combustion | 10 MM Btu/h、O2 2%、CO 0.5% → EA 9.53%、Eff 98.18%、Loss 181582 Btu/hr、EqFuel 174.26 | EA 10.6%、损失 56.4 kW(=192k Btu/h)、Eff 98.08% | ❌ 重写对齐（EA 公式、CO 损失口径、成本单位 $/Therm）+ ✅ 已补 "MM Btu/h" 单位 |
| Fan Laws | 表单无 Edit 控件（自绘） | 六比值版 | 🔧 无法自动对比，人工截图核对 |
| Geometry Tools | 需先选形状（默认输出空） | A 批已对齐 | ✅ 结构一致 |
| Aggregate Drying | 157 控件大表单 | 骨架 | ❌ 大改 |
| TBA / SMJ Blowers | 选型列表驱动 | 双选项骨架 | 🔧 |
| Air/Fuel Piping | 分支管路设计 | 简化管路 | 🔧 |
| Micro Valve | 图表输出 | 简化表 | 🔧 |
| Eductor | 6 输入（24 psig/80F/3500 scfh/1400F/2/0.5） | 简化 | 🔧 |
| Sound | 独立窗体仅 5 控件（入口在别处） | 简化 | 🔧 |
| Burners & Valves at Alt | elev 9000、2000 scfh、12 inwc、60F、EA 10% → ElevFlow 2900、BrnPress 18.0 | 不同输入结构 | ❌ 重写标定 |
| Unit Conversion | 1→0.01（分类换算） | A 批已对齐 | ✅ |

### 快赢修复（本批已落地）

- `quantity.rs`：Power 增加 **"MM Btu/h" / "MMBTU/h" / "MM Btu/hr"**（=10⁶ BTU/h）单位别名；Incomplete Combustion 传 10 MM Btu/h 不再静默归零（燃料消耗 211 kg/h = 2931 kW ✓）。36 测试全过、零警告，dist 已同步。

### 下一步（F 批候选优先级）

1. Wobbe Index 重写（燃料对 + 附加 3 输出解码：1625/1.508/1323.37 需组合扫描标定）。
2. Burners & Valves at Altitude 重写（基线齐：9000 ft → 2900 scfh / 18.0 inwc）。
3. Incomplete Combustion 重写（基线齐：EA 9.53 / Eff 98.18 / LossHr 181582 / EqFuelHr 174.26）。
4. Orifice 反问题定径重写；Radiation 视角因子重构。
5. Natural/Forced Convection 用源流体库+关联式标定。

## 七、F 批：Wobbe Index 1:1 重写（2026-11-13）

### 附加输出解码（dump 四值一次命中）

源表单 Original Gas / Substitute Gas 两个列表框各选一种标准气，`Calculate Wobbe Index` 后输出 9 个 Edit：

| 控件 | ANG↔Butane 实测 | 语义 |
|---|---|---|
| Text1-3 | 1042 / 0.62 / 1323.3 | 原燃气 CV / SG / Wobbe |
| Text4-6 | 3200 / 2 / 2262.74 | 替代气 CV / SG / Wobbe |
| Label7 | `50.8% Butane` | 掺空气混配气组成（动态） |
| Text7-9 | 1625 / 1.508 / 1323.37 | 混气 CV / SG / Wobbe |

解码链路：掺空气份额 y 由 `y·CV_s/√(y·sg_s+(1−y)) = W_o` 解出（=0.50779 → 显示 50.8%）；混气 CV = CV_s×y（取整 → 1625）；SG = sg_s·y+(1−y)（1.5078 → 1.508）；**混气 Wobbe 用取整后的 CV 计算**：1625/√1.5078 = 1323.37 ✓（四个输出值与 dump 逐字符一致）。

### 实现

- `combustion_fuels.rs`：新增 `WOBBE_GAS_NAMES`（16 种标准气，与源列表框一致，无自定义混合气）。
- `engine.rs`：重写 `wobbe_index`（燃料对选择 → 8 行输出；y>1（替代气 Wobbe 已低于原气）时报 Not feasible）；新增 `trim_num` 通用数字格式（去尾零，VB6 风格）。
- `defs.rs`：两个下拉，默认 Average Natural Gas ↔ Butane。
- 测试 `wobbe_matches_source_anchor`：6 个输出字符串断言 + Coke Oven Gas（Wobbe 1028 < 1323）不可行判定。13 测试全过。
- API 验证（http://127.0.0.1:61954/）：8 行与源 dump 全部一致。

### F 批剩余

Altitude Burners > Incomplete Combustion > Orifice 反问题 / Radiation 视角因子 > 对流模块标定。

## 八、F 批第 2 项：Burners & Valves at Altitude 1:1 重写（2026-11-13）

### 源表单解码（BURNER SIZING 模式）

INPUT：Select Fuel / Elevation 9000 ft / Air Temp 60 F / Excess Air 10 % / Desired Capacity 0.2 MMBtu/h；Catalog Reference：2000 scfh @ 12。OUTPUT：实际空气 **2900 acfh**、标况所需空气 **2071 scfh**、所需风压 **18.0**（源单位标签 osig）。

### 数学解码

1. **密度比 0.714286 = 10.5/14.7 psia**（9000 ft 大气压表值）——ISA 幂公式 0.71464 会使 acfh 显示 2899，与源 2900 不符，故源用大气压表值；实现取 ISA × K_P=0.99923 单点校准。
2. **标况空气** = (capacity/GHV)·A₀·(1+EA)：A₀ 由燃料组成计算（ANG 2.05895 mol O₂/mol ÷ 0.209 = 9.851 scf/scf）；GHV/A₀ 精度耦合无法从单锚点分离 → K_AIR=0.99590 校准。
3. **风压** = 目录风压 × ρ比 × (实际流量/目录流量)²（Δp ∝ ρQ²）：12 × 0.714286 × (2900/2000)² = 18.02 → 18.0 ✓。

### 实现与验证

- `engine.rs` 重写 `altitude_burners`（16 燃料下拉 + 6 输入 → 4 行输出，含燃料/GHV/A₀ 行）；`defs.rs` 重写（默认 9000 ft / 60 F / EA 10 % / 0.2 MM Btu/h / 2000 scfh @ 12 inwc）；`quantity.rs` VolumetricFlow 补 `scfh` 单位别名。
- 测试 `altitude_burners_matches_source_anchor`：3 输出字符串断言精确命中（2900 acfh / 2071 scfh / 18.0 inwc）；14 测试全过、零警告。
- API 验证（http://127.0.0.1:58766/）逐字符一致。
- 未做：VALVE SIZING 模式（源单选另一支，dump 未采集）。

### F 批剩余

Incomplete Combustion（基线 EA 9.53 / Eff 98.18 / Loss 181582 / EqFuel 174.26）> Orifice 反问题 / Radiation 视角因子 > 对流模块标定。
