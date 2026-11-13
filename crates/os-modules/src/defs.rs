//! Data-driven definition of every module visible in the reference program.
//!
//! Groups are derived from the visible button panels: "Reference" (19),
//! "Applications" (11), and the "System Design" wizard (7), plus the four
//! main-window utility buttons. No vendor data, text or assets are copied:
//! names are used only as functional labels, all numeric defaults are public
//! engineering starting values.

use crate::quantity::Quantity as Q;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Group {
    Reference,
    Applications,
    SystemDesign,
    Tools,
}

impl Group {
    pub fn label(self) -> &'static str {
        match self {
            Group::Reference => "Reference 参考",
            Group::Applications => "Applications 应用",
            Group::SystemDesign => "System Design 系统设计",
            Group::Tools => "Tools 工具",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub qty: Q,
    /// Canonical SI value. Display values are derived with `qty.from_si`.
    pub default_si: f64,
    /// If non-empty the field is a select; `default_si` is the selected index.
    pub options: &'static [&'static str],
    pub hint: &'static str,
}

impl Field {
    pub fn json(&self) -> FieldJson {
        FieldJson {
            key: self.key,
            label: self.label,
            qty: format!("{:?}", self.qty),
            unit_si: self.qty.display_unit(os_core::units::UnitSystem::Si),
            unit_us: self.qty.display_unit(os_core::units::UnitSystem::Imperial),
            units: self.qty.available_units(),
            default_si: self.default_si,
            default_display: if self.options.is_empty() {
                self.qty
                    .from_si(self.default_si, self.qty.display_unit(os_core::units::UnitSystem::Si))
                    .unwrap_or(self.default_si)
            } else {
                self.default_si
            },
            options: self.options,
            hint: self.hint,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FieldJson {
    pub key: &'static str,
    pub label: &'static str,
    pub qty: String,
    pub unit_si: &'static str,
    pub unit_us: &'static str,
    pub units: &'static [&'static str],
    pub default_si: f64,
    pub default_display: f64,
    pub options: &'static [&'static str],
    pub hint: &'static str,
}

#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub id: &'static str,
    pub group: Group,
    pub name: &'static str,
    pub en: &'static str,
    pub desc: &'static str,
    pub fields: Vec<Field>,
    /// Public formula / method note shown in the UI.
    pub method: &'static str,
}

impl ModuleDef {
    pub fn json(&self) -> ModuleJson {
        ModuleJson {
            id: self.id,
            group: self.group.label(),
            group_key: format!("{:?}", self.group),
            name: self.name,
            en: self.en,
            desc: self.desc,
            fields: self.fields.iter().map(Field::json).collect(),
            method: self.method,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ModuleJson {
    pub id: &'static str,
    pub group: &'static str,
    pub group_key: String,
    pub name: &'static str,
    pub en: &'static str,
    pub desc: &'static str,
    pub fields: Vec<FieldJson>,
    pub method: &'static str,
}

/// Numeric input. `val` is expressed in `unit` at definition time and stored
/// canonically, so defaults are immune to the current display system.
fn f(key: &'static str, label: &'static str, qty: Q, val: f64, unit: &'static str, hint: &'static str) -> Field {
    Field {
        key,
        label,
        qty,
        default_si: qty.to_si(val, unit).expect("valid default unit"),
        options: &[],
        hint,
    }
}

fn sel(key: &'static str, label: &'static str, options: &'static [&'static str], index: f64, hint: &'static str) -> Field {
    Field {
        key,
        label,
        qty: Q::Dimensionless,
        default_si: index,
        options,
        hint,
    }
}

fn m(
    id: &'static str,
    group: Group,
    name: &'static str,
    en: &'static str,
    desc: &'static str,
    fields: Vec<Field>,
    method: &'static str,
) -> ModuleDef {
    ModuleDef { id, group, name, en, desc, fields, method }
}

pub const UNITS_FOR_CONVERTER: &[&str] = &[
    "m", "mm", "in", "ft", "m2", "in2", "ft2", "m3", "ft3", "L", "gal",
    "C", "F", "K", "kPa", "bar", "psia", "psig", "inWC", "kg/h", "lb/h",
    "kg/s", "m3/h", "cfm", "Nm3/h", "SCFM", "kg/m3", "lb/ft3", "m/s",
    "ft/s", "kW", "BTU/h", "hp", "MJ", "BTU", "MJ/kg", "BTU/lb", "Kv", "Cv",
];

const SHAPES: &[&str] = &["环 Annulus", "圆 Circle", "圆锥 Cone", "立方体 Cube", "圆柱 Cylinder", "球 Sphere", "矩形 Rectangle"];
const TABLES: &[&str] = &["Standard Steel Pipe Data: Schedule 40", "Equivalence Fuel/Air Ratio and Excess Air"];

pub fn all_modules() -> Vec<ModuleDef> {
    use Group::*;
    vec![
        // ---------------------------------------------------------------- Reference
        m("fuel_data", Reference, "燃料数据", "Fuel Data",
          "选择燃料或自定义混合气，按体积组成计算平均分子量、相对比重、标况密度、低位/高位热值（质量与体积）、理论空气量与理论烟气量。",
          vec![
              f("x_ch4", "甲烷 CH4", Q::Percent, 89.0, "%", "体积分数 %"),
              f("x_c2h6", "乙烷 C2H6", Q::Percent, 5.0, "%", "体积分数 %"),
              f("x_c3h8", "丙烷 C3H8", Q::Percent, 1.4, "%", "体积分数 %"),
              f("x_c4h10", "丁烷 C4H10", Q::Percent, 0.3, "%", "体积分数 %"),
              f("x_co", "一氧化碳 CO", Q::Percent, 0.0, "%", "体积分数 %"),
              f("x_h2", "氢气 H2", Q::Percent, 0.0, "%", "体积分数 %"),
              f("x_co2", "二氧化碳 CO2", Q::Percent, 0.7, "%", "体积分数 %"),
              f("x_o2", "氧气 O2", Q::Percent, 0.0, "%", "体积分数 %"),
              f("x_n2", "氮气 N2", Q::Percent, 3.0, "%", "体积分数 %"),
              f("x_h2o", "水蒸气 H2O", Q::Percent, 0.0, "%", "体积分数 %"),
          ],
          "组分体积加权；理论氧气按化学计量，空气按 O2=21% / N2=79%；体积热值换算 1 MJ/Nm³≈26.84 Btu/scf。"),

        m("gas_liquid_props", Reference, "气体与液体物性", "Physical Properties of Gases & Fluids",
          "源程序 1:1：25 种流体（18 气体 + 7 液体）按温度/压力查表输出 9 项物性；选 Customized Gas Mixture 可按体积分数自定义 12 组分混合气（不足 100% 自动补 N2）。",
          vec![
              sel("fluid", "流体", crate::fluids_data::FLUID_NAMES_PLUS_MIX, 0.0, "最后一项为 Customized Gas Mixture（自定义混合气）"),
              f("t", "温度 Temperature", Q::Temperature, 15.5556, "C", "各流体有效范围与源程序一致，超范围拒绝计算"),
              f("p", "绝对压力 Pressure", Q::PressureAbs, 101.325, "kPa", "气体密度按 P/14.7 线性换算；Aver. Flue Gas 仅 14.7 psia"),
              f("y_ch4", "CH4 体积%", Q::Percent, 0.0, "%", "混合气组分（体积分数）"),
              f("y_c2h6", "C2H6 体积%", Q::Percent, 0.0, "%", ""),
              f("y_c2h4", "C2H4 体积%", Q::Percent, 0.0, "%", ""),
              f("y_c3h6", "C3H6 体积%", Q::Percent, 0.0, "%", ""),
              f("y_c3h8", "C3H8 体积%", Q::Percent, 0.0, "%", ""),
              f("y_c4h10", "C4H10 体积%", Q::Percent, 0.0, "%", ""),
              f("y_h2", "H2 体积%", Q::Percent, 0.0, "%", ""),
              f("y_co", "CO 体积%", Q::Percent, 0.0, "%", ""),
              f("y_n2", "N2 体积%", Q::Percent, 0.0, "%", ""),
              f("y_co2", "CO2 体积%", Q::Percent, 0.0, "%", ""),
              f("y_o2", "O2 体积%", Q::Percent, 0.0, "%", ""),
              f("y_h2o", "H2O 体积%", Q::Percent, 0.0, "%", "水蒸气组分"),
          ],
          "物性表来自对源程序 518 组实测数据；密度∝P、SG=ρ/0.0765、ν=μ·6.72e-4/ρ、Pr=cp·μ·2.4192/k 均已逐点验证。混合气：ρ 按理想气体 MW 法（已验证）、cp 按质量加权、γ/k/h 按体积加权（均已验证）、μ 按 Wilke（逐项分母形式）。H2/H2O 超出源有效范围的组分值按幂律外推补齐（混合气模式专用）。"),

        m("solids_props", Reference, "固体物性", "Physical Properties of Solids",
          "查表：53 种金属/非金属材料的密度、比热、导热与辐射率。",
          vec![
              sel("mat", "材料", crate::materials::MAT_NAMES, 0.0, "来源：本地导入 MetalProp/Solid 库"),
              f("l", "壁厚(评估)", Q::Length, 0.2, "m", ""),
              f("dt", "温差(评估)", Q::TemperatureDiff, 100.0, "K", ""),
          ],
          "物性为源库 US 单位换算 SI 后的表值；α=k/(ρ·cp)；q''=k·ΔT/L。"),

        m("fuel_combustion", Reference, "燃料燃烧", "Fuel Combustion",
          "按源程序 Fuel Combustion：选择燃料（标准气/液/自定义 16 组分）→ 显示燃料数据与组成 → 指定燃烧条件 → 输出两页燃烧分析（产物湿/干成分、可用热、绝热火焰温度、露点、流量与质量流量）。",
          vec![
              sel("fuel_type", "Fuel Selection 燃料类型",
                  &["Standard Gas", "Liquid Fuel", "Customized Gas Fuel", "Customized Oil Mixture", "Custom Oil"],
                  0.0, "与源程序 5 个单选一致"),
              sel("fuel", "Select Gas 选择燃气", crate::combustion_fuels::GAS_FUEL_NAMES,
                  0.0, "源 GasFuel 表 17 种标准气 + Customized Gas Fuel"),
              sel("oil", "Select Oil 选择燃油", crate::combustion_fuels::OIL_FUEL_NAMES,
                  0.0, "源 OilFuel 表 5 种油 + Average Waste Oil"),
              // Customized Gas Fuel: the 16 constituents printed on page 1
              f("y_ch4",   "CH4 (Methane) · 甲烷",          Q::Percent, 89.45, "%", "体积分数 %"),
              f("y_c2h2",  "C2H2 (Acetylene) · 乙炔",       Q::Percent, 0.0, "%", ""),
              f("y_c2h4",  "C2H4 (Ethylene) · 乙烯",        Q::Percent, 0.0, "%", ""),
              f("y_c2h6",  "C2H6 (Ethane) · 乙烷",          Q::Percent, 5.08, "%", ""),
              f("y_c3h6",  "C3H6 (Propylene) · 丙烯",       Q::Percent, 0.0, "%", ""),
              f("y_c3h8",  "C3H8 (Propane) · 丙烷",         Q::Percent, 1.44, "%", ""),
              f("y_c4h10", "C4H10 (Butane) · 丁烷",         Q::Percent, 0.31, "%", ""),
              f("y_c5h12", "C5H12 (N-Pentane) · 正戊烷",      Q::Percent, 0.0, "%", ""),
              f("y_c6h14", "C6H14 (N-Hexane) · 正己烷",       Q::Percent, 0.0, "%", ""),
              f("y_co",    "CO (Carbon Monoxide) · 一氧化碳",   Q::Percent, 0.0, "%", ""),
              f("y_h2",    "H2 (Hydrogen) · 氢气",          Q::Percent, 0.0, "%", ""),
              f("y_h2s",   "H2S (Hydrogen Sulfide) · 硫化氢", Q::Percent, 0.0, "%", ""),
              f("y_co2",   "CO2 (Carbon Dioxide) · 二氧化碳",   Q::Percent, 0.73, "%", ""),
              f("y_o2",    "O2 (Oxygen) · 氧气",            Q::Percent, 0.0, "%", ""),
              f("y_h2o",   "H2O (Water Vapor) · 水蒸气",      Q::Percent, 0.0, "%", ""),
              f("y_n2",    "N2 (Nitrogen) · 氮气",          Q::Percent, 2.99, "%", "不足 100% 自动补 N₂"),
              // Specify Combustion Conditions
              f("excess",     "% Excess Air · 过量空气百分数",                 Q::Percent,     0.0, "%", "过量空气百分数"),
              f("t_air",      "Combustion Air Temp. · 助燃空气温度",         Q::Temperature, 60.0, "F", "60–2200 °F  /  15.6–1204 °C"),
              f("t_exh",      "Exhaust Temperature · 排烟温度",          Q::Temperature, 2000.0, "F", "60–3600 °F  /  15.6–1982 °C"),
              f("t_fuel",     "Fuel Temperature · 燃料温度",             Q::Temperature, 60.0, "F", "气 60–700 °F / 油 60–260 °F（15.6–371 / 15.6–127 °C）"),
              f("fuel_input", "Fuel Input · 燃料输入",                   Q::Dimensionless, 100.0, "-", "数值，单位由 Fuel Input Unit 指定"),
              sel("input_unit", "Fuel Input Unit · 燃料输入单位", &["scfh", "Mill.Btu/h", "lb/hr"], 0.0,
                  "与源程序 3 个复选框一致"),
              // Options
              sel("ambient", "Set Ambient Conditions · 设置环境条件", &["Off", "On"], 0.0, "温度与相对湿度"),
              f("t_amb", "Ambient Air Temperature · 环境空气温度", Q::Temperature, 60.0, "F", "默认 60 °F"),
              f("rh",    "Relative Humidity · 相对湿度",       Q::Percent, 0.0, "%", ""),
              sel("fgr", "Flue Gas Recirculation Option · 烟气再循环选项", &["Off", "On"], 0.0, "烟气再循环"),
              f("t_fgr",   "Recirculation Flue Gas Temperature · 再循环烟气温度", Q::Temperature, 60.0, "F", "60–2200 °F  /  15.6–1204 °C"),
              f("fgr_pct", "Percent of Recirculation Gases · 再循环烟气百分比",     Q::Percent,     0.0, "%", "最大 72%"),
              sel("o2e", "Oxygen Enrichment Option · 富氧燃烧选项", &["Off", "On"], 0.0, "富氧燃烧"),
              f("o2_purity", "Purity of Oxygen · 氧气纯度",                        Q::Percent, 99.0, "%", "最大 99%"),
              f("o2_mix",    "% O2 in Combustion Air/Oxygen Mixture · 助燃空气/氧气混合物含氧量",  Q::Percent, 20.9, "%", "20.9–99%"),
          ],
          "燃料数据取自源 GasFuel/OilFuel 表；理论空气按组分化学计量 Σyᵢ(C+H/4−O/2+S)/0.209（源实测 Average Natural Gas 9.85 SCF/SCF）；两页输出项按源程序结果页逐项复刻。默认工况已与源程序实测对齐：湿/干产物 9.72/11.92 %CO₂、18.49 %H₂O、71.79/88.08 %N₂，可用热 46.2 %、可用热量 48107 BTU/hr、排烟热损失 56093 BTU/hr、绝热火焰温度 3577 °F、露点 137.6 °F、烟气量 1089.59 scfh / 5155 acfh。烟气焓与高温等效解离按该实测工况标定。"),

        m("wobbe_index", Reference, "沃泊指数", "Wobbe Index",
          "源程序 1:1：原燃气/替代气各从 16 种标准气清单选择，输出两者 Wobbe 及掺空气混配气（份额、热值、相对密度、Wobbe）。",
          vec![
              sel("fuel_o", "Original Gas · 原燃气选择", crate::combustion_fuels::WOBBE_GAS_NAMES, 0.0,
                  "源程序 Original Gas 列表 · 与 GasFuel 库一致"),
              sel("fuel_s", "Substitute Gas · 替代气选择", crate::combustion_fuels::WOBBE_GAS_NAMES, 2.0,
                  "源程序 Substitute Gas 列表"),
          ],
          "W = CV/√sg（CV Btu/scf）；掺空气份额 y 由 y·CV_s/√(y·sg_s+(1−y)) = W_o 解出，混气热值/相对密度按体积线性混合。默认工况（Average Natural Gas ↔ Butane）与源程序实测对齐：原气 1323.3、替代气 2262.74、混配 50.8% Butane、混气 1625 / 1.508 / 1323.37。"),

        m("energy_savings", Reference, "余热回收节能", "Energy Savings",
          "源程序 1:1：Case I / Case II 各自选择燃料并指定燃烧条件（过量空气、排烟温度、空气温度、助燃空气含氧、烟气再循环、再循环温度、燃料预热温度），分别计算可用热并给出 Case II 相对 Case I 的节能率。",
          vec![
              sel("fuel1", "Select Fuel · Case I 燃料选择", crate::combustion_fuels::ES_FUEL_NAMES, 0.0,
                  "与源程序 25 项清单一致；油/电/自定义项按源程序实测行为回退为 Average Natural Gas 数据"),
              f("ea1",   "Excess Air · Case I 过量空气",          Q::Percent,     10.0, "%", ""),
              f("exh1",  "Exhaust Temperature · Case I 排烟温度",   Q::Temperature, 1800.0, "F", "60–3600 °F  /  15.6–1982 °C"),
              f("air1",  "Combustion Air Temperature · Case I 空气温度", Q::Temperature, 60.0, "F", "60–2200 °F  /  15.6–1204 °C"),
              f("o21",   "Oxygen in Combustion Air · Case I 助燃空气含氧", Q::Percent, 20.9, "%", "20.9% 以上启用富氧"),
              f("fgr1",  "Flue Gas Recirculation · Case I 烟气再循环", Q::Percent, 0.0, "%", ""),
              f("tfgr1", "Recirculation Gas Temperature · Case I 再循环烟气温度", Q::Temperature, 60.0, "F", ""),
              f("fp1",   "Fuel Preheat Temperature · Case I 燃料预热温度", Q::Temperature, 60.0, "F", "源程序实测：该输入不参与计算"),
              sel("fuel2", "Select Fuel · Case II 燃料选择", crate::combustion_fuels::ES_FUEL_NAMES, 0.0,
                  "与源程序 25 项清单一致；油/电/自定义项按源程序实测行为回退为 Average Natural Gas 数据"),
              f("ea2",   "Excess Air · Case II 过量空气",          Q::Percent,     10.0, "%", ""),
              f("exh2",  "Exhaust Temperature · Case II 排烟温度",   Q::Temperature, 1800.0, "F", "60–3600 °F  /  15.6–1982 °C"),
              f("air2",  "Combustion Air Temperature · Case II 空气温度", Q::Temperature, 60.0, "F", "60–2200 °F  /  15.6–1204 °C"),
              f("o22",   "Oxygen in Combustion Air · Case II 助燃空气含氧", Q::Percent, 20.9, "%", "20.9% 以上启用富氧"),
              f("fgr2",  "Flue Gas Recirculation · Case II 烟气再循环", Q::Percent, 0.0, "%", ""),
              f("tfgr2", "Recirculation Gas Temperature · Case II 再循环烟气温度", Q::Temperature, 60.0, "F", ""),
              f("fp2",   "Fuel Preheat Temperature · Case II 燃料预热温度", Q::Temperature, 60.0, "F", "源程序实测：该输入不参与计算"),
          ],
          "可用热内核按源程序 Energy Saving 表单 59 点扫描标定：组分化学计量需氧 + 空气 20.9/79.1；烟气焓用 CO2/H2O/N2/O2 的 Shomate 真实气体焓（60 °F 基准）×1.0279；H2O 潜热 41.63 Btu/scf；含氧 >20.9% 时氧化剂按化学计量 s/x（源程序行为）；烟气再循环按湿烟气份额 × 干基比焓差 × 1.0543；空气预热按氧化剂焓计入。节能率 = 1 − AH_I/AH_II。默认工况与源程序实测对齐：AH_I=AH_II=48.0%、节能率 0；空气预热 800 °F → AH_II=62.4%、节能率 23.1%。"),

        m("fluid_mechanics", Reference, "流体力学", "Fluid Mechanics",
          "管内流速、雷诺数、Darcy 摩擦系数与沿程/局部压降。",
          vec![
              f("d", "管内径", Q::Length, 100.0, "mm", "NPS 是名义口径，不是实测内径"),
              f("q", "体积流量", Q::VolumetricFlow, 100.0, "m3/h", ""),
              f("rho", "密度", Q::Density, 1.2, "kg/m3", ""),
              f("mu", "动力粘度", Q::DynamicViscosity, 0.018, "cP", ""),
              f("l", "管长", Q::Length, 20.0, "m", ""),
              f("eps", "绝对粗糙度", Q::Length, 0.045, "mm", ""),
              f("k_loss", "局部阻力系数 ΣK", Q::Dimensionless, 3.0, "-", ""),
          ],
          "Re=ρvD/μ；Swamee-Jain 显式摩擦系数；Δp=(f·L/D+ΣK)·ρv²/2。"),

        m("incomplete_combustion", Reference, "不完全燃烧损失", "Energy Cost From Incomplete Combustion",
          "CO 未燃尽的化学热损失与逐时/逐日/年度成本。",
          vec![
              f("q_in", "热输入", Q::Power, 5000.0, "kW", "燃料输入功率"),
              f("lhv", "燃料 LHV", Q::HeatingValue, 50.0, "MJ/kg", ""),
              f("o2", "烟气 O2 干基", Q::Percent, 3.0, "%", ""),
              f("co_ppm", "烟气 CO", Q::Dimensionless, 200.0, "-", "ppm (dry)"),
              f("hours", "运行时间", Q::Time, 16.0, "h", "每天运行小时数"),
              f("price", "燃料单价", Q::Dimensionless, 3.5, "-", "元/kg"),
          ],
          "过剩空气 EA=O₂/(20.9−O₂)；CO 损失 283 kJ/mol；成本按 每时/每日/每年 输出。"),

        m("flammability", Reference, "混合气可燃性", "Flammability of Gas Mixture",
          "Le Chatelier 混合规则：LFL/UFL、温度修正、GHV、化学计量空气与燃烧速度。",
          vec![
              f("x_ch4", "CH4 (Methane) · 甲烷", Q::Dimensionless, 80.0, "-", "mol %"),
              f("x_c2h6", "C2H6 (Ethane) · 乙烷", Q::Dimensionless, 5.0, "-", "mol %"),
              f("x_c2h2", "C2H2 (Acetylene) · 乙炔", Q::Dimensionless, 0.0, "-", "mol %"),
              f("x_c2h4", "C2H4 (Ethylene) · 乙烯", Q::Dimensionless, 2.0, "-", "mol %"),
              f("x_c3h8", "C3H8 (Propane) · 丙烷", Q::Dimensionless, 1.5, "-", "mol %"),
              f("x_c3h6", "C3H6 (Propylene) · 丙烯", Q::Dimensionless, 0.0, "-", "mol %"),
              f("x_c4h10", "C4H10 (N-Butane) · 正丁烷", Q::Dimensionless, 0.3, "-", "mol %"),
              f("x_c5h12", "C5H12 (N-Pentane) · 正戊烷", Q::Dimensionless, 0.2, "-", "mol %"),
              f("x_c6h14", "C6H14 (N-Hexane) · 正己烷", Q::Dimensionless, 0.0, "-", "mol %"),
              f("x_h2", "H2 (Hydrogen) · 氢气", Q::Dimensionless, 5.0, "-", "mol %"),
              f("x_co", "CO (Carbon Monoxide) · 一氧化碳", Q::Dimensionless, 3.0, "-", "mol %"),
              f("x_h2s", "H2S (Hydrogen Sulfide) · 硫化氢", Q::Dimensionless, 0.0, "-", "mol %"),
              f("x_co2", "CO2 (Carbon Dioxide) · 二氧化碳", Q::Dimensionless, 0.5, "-", "mol %"),
              f("x_o2", "O2 (Oxygen) · 氧气", Q::Dimensionless, 0.0, "-", "mol %"),
              f("x_n2", "N2 (Nitrogen) · 氮气", Q::Dimensionless, 2.5, "-", "mol %"),
              f("x_h2o", "H2O (Water Vapor) · 水蒸气", Q::Dimensionless, 0.0, "-", "mol %"),
              f("t_mix", "混合气温度", Q::Temperature, 25.0, "C", "LFL/UFL 温度修正基准 25°C"),
          ],
          "LFL_mix=100/Σ(ci/LFLi)；温度修正按 Zabetakis 线性式；GHV/化学计量空气按体积混合。"),

        m("orifice_meter", Reference, "孔板流量计", "Orifice Meter Calculations",
          "ISO 5167 简化式：β 比、流出系数、差压流量与永久压损。",
          vec![
              f("d_pipe", "管道内径", Q::Length, 100.0, "mm", ""),
              f("d_orifice", "孔板开孔", Q::Length, 50.0, "mm", ""),
              f("dp", "差压", Q::PressureDrop, 5.0, "kPa", ""),
              f("rho", "流体密度", Q::Density, 1.2, "kg/m3", ""),
              f("mu", "动力粘度", Q::DynamicViscosity, 0.018, "cP", ""),
              f("cd", "流出系数 C", Q::Ratio, 0.61, "-", ""),
          ],
          "q=C·A2/√(1-β⁴)·√(2Δp/ρ)；永久压损≈(1-β^1.9)·Δp。"),

        m("natural_convection", Reference, "自然对流", "Natural Convection",
          "竖直平板/水平管的自然对流努塞尔数与换热量。",
          vec![
              f("l", "特征长度", Q::Length, 1.0, "m", ""),
              f("t_s", "壁面温度", Q::Temperature, 120.0, "C", ""),
              f("t_inf", "环境温度", Q::Temperature, 20.0, "C", ""),
              f("k", "流体导热系数", Q::ThermalConductivity, 0.03, "W/(m.K)", ""),
              f("nu", "运动粘度", Q::Dimensionless, 1.8e-5, "-", "m2/s"),
              f("pr", "普朗特数 Pr", Q::Ratio, 0.7, "-", ""),
              f("area", "换热面积", Q::Area, 2.0, "m2", ""),
          ],
          "β=1/T_film(K)；Ra=gβΔT·L³/(ν·α)；Churchill-Chu 竖板关联式。"),

        m("forced_convection", Reference, "强制对流", "Forced Convection",
          "管内强制对流 Re/Pr/Nu 与换热系数。",
          vec![
              f("d", "管内径", Q::Length, 50.0, "mm", ""),
              f("v", "流速", Q::Velocity, 10.0, "m/s", ""),
              f("rho", "密度", Q::Density, 1.2, "kg/m3", ""),
              f("mu", "动力粘度", Q::DynamicViscosity, 0.018, "cP", ""),
              f("k", "导热系数", Q::ThermalConductivity, 0.026, "W/(m.K)", ""),
              f("cp", "比热", Q::SpecificHeat, 1.005, "kJ/(kg.K)", ""),
              f("dt", "壁-流体温差", Q::TemperatureDiff, 80.0, "K", ""),
              f("l", "管长", Q::Length, 2.0, "m", ""),
          ],
          "Dittus-Boelter Nu=0.023Re^0.8Pr^0.4（加热）；Re<2300 时提示层流。"),

        m("radiation", Reference, "辐射换热", "Radiation",
          "Stefan-Boltzmann 辐射热流与等效辐射换热系数。",
          vec![
              f("t_h", "热源温度", Q::Temperature, 1000.0, "C", ""),
              f("t_c", "受热面温度", Q::Temperature, 200.0, "C", ""),
              f("eps", "系统发射率", Q::Ratio, 0.8, "-", ""),
              f("area", "辐射面积", Q::Area, 2.0, "m2", ""),
              f("f_view", "角系数", Q::Ratio, 0.9, "-", ""),
              f("h_conv", "对流换热系数", Q::HeatTransferCoeff, 15.0, "W/(m2.K)", ""),
          ],
          "q=ε·σ·F·A·(Th⁴−Tc⁴)；等效 h_rad=εσ(Th²+Tc²)(Th+Tc)。"),

        m("sound", Reference, "燃烧噪声", "Sound",
          "燃烧器声功率级与自由场声压级估算。",
          vec![
              f("power", "燃烧热功率", Q::Power, 1000.0, "kW", ""),
              f("n_burner", "燃烧器数量", Q::Dimensionless, 2.0, "-", ""),
              f("dist", "测点距离", Q::Length, 3.0, "m", ""),
              f("directivity", "指向性因子 Q", Q::Ratio, 2.0, "-", ""),
              f("attenuation", "附加衰减", Q::SoundLevel, 0.0, "dB", ""),
          ],
          "Lw≈97+10log10(P[MW])+10log10(n)；Lp=Lw+10log10(Q/(4πr²))−衰减。"),

        m("emission", Reference, "排放换算", "Emission Calculations",
          "实测 ppm 折算到基准含氧量、mg/Nm3 与质量排放率。",
          vec![
              f("ppm", "实测浓度", Q::Dimensionless, 100.0, "-", "ppm (dry)"),
              f("o2_meas", "实测 O2", Q::Percent, 3.0, "%", "dry"),
              f("o2_ref", "基准 O2", Q::Percent, 3.0, "%", "dry"),
              f("mw", "污染物分子量", Q::MolecularWeight, 46.0, "kg/kmol", "NO2=46, CO=28, SO2=64"),
              f("q_flue", "标态烟气量", Q::VolumetricFlow, 5000.0, "Nm3/h", ""),
              f("hours", "年运行小时", Q::Time, 6000.0, "h", ""),
          ],
          "C_ref=C_meas·(20.9−O2_ref)/(20.9−O2_meas)；mg/Nm3=ppm·M/22.414。"),

        m("fan_laws", Reference, "风机定律", "Fan Laws",
          "与源程序一致的六条相似律：Q/P/HP 随转速比与密度比换算。",
          vec![
              f("q1", "原始风量 Q₁", Q::VolumetricFlow, 10000.0, "m3/h", ""),
              f("n1", "原始转速 rpm₁", Q::Frequency, 1450.0, "rpm", ""),
              f("dp1", "原始静压 p₁", Q::PressureDrop, 2.0, "kPa", ""),
              f("p1", "原始轴功率 HP₁", Q::Power, 15.0, "kW", ""),
              f("n2", "目标转速 rpm₂", Q::Frequency, 1750.0, "rpm", ""),
              f("rho1", "原始密度 ρ₁", Q::Density, 1.2, "kg/m3", ""),
              f("rho2", "目标密度 ρ₂", Q::Density, 1.0, "kg/m3", ""),
          ],
          "Q₂/Q₁=rpm₂/rpm₁；p₂/p₁=(rpm₂/rpm₁)²(ρ₂/ρ₁)；HP₂/HP₁=(rpm₂/rpm₁)³(ρ₂/ρ₁)；等质量流量时 Q/p/HP ∝ ρ₂/ρ₁。"),

        m("unit_conversion", Reference, "单位换算", "Unit Conversion",
          "任意支持单位之间的显式换算，含 17 条基准对照表。",
          vec![
              f("value", "数值", Q::Dimensionless, 1.0, "-", ""),
              sel("from", "源单位", UNITS_FOR_CONVERTER, 2.0, "in"),
              sel("to", "目标单位", UNITS_FOR_CONVERTER, 0.0, "m"),
          ],
          "所有换算经由同一 SI 规范表示；温度用仿射变换，面积/体积用平方/立方。"),

        m("tables", Reference, "工程数据表", "Tables",
          "与源程序一致的两张参考表：Sch40 标准钢管数据、当量比-过剩空气对照。",
          vec![
              sel("table_sel", "Select Tables · 选择数据表", TABLES, 0.0, ""),
          ],
          "Sch40 尺寸为 ASME B36.10 公开标准；当量比 φ=(F/A)/(F/A)ₛₜ，EA%=100×(1/φ−1)。"),

        m("geometry_tools", Reference, "几何工具", "Geometry Tools",
          "炉膛/管道/贮罐常用几何的面积、体积与表面积（形状清单与源程序一致）。",
          vec![
              sel("shape", "Select Geometry · 选择几何形状", SHAPES, 4.0, "圆柱 Cylinder"),
              f("a", "尺寸 A", Q::Length, 1.0, "m", "直径/边长"),
              f("b", "尺寸 B", Q::Length, 0.5, "m", "内径/宽"),
              f("c", "尺寸 C", Q::Length, 2.0, "m", "高/长"),
          ],
          "与源程序相同的形状列表：Annulus/Circle/Cone/Cube/Cylinder/Sphere/Rectangle；输出体积、表面积与截面积。"),

        // -------------------------------------------------------------- Applications
        m("aggregate_drying", Applications, "低温骨料干燥", "Low Temperature Aggregate Drying",
          "沥青拌合骨料干燥筒的热平衡与燃料消耗。",
          vec![
              f("m_agg", "骨料产量", Q::MassFlow, 100.0, "t/h", ""),
              f("w_in", "初始含水率", Q::Percent, 5.0, "%", "湿基"),
              f("w_out", "出料含水率", Q::Percent, 0.5, "%", "湿基"),
              f("t_in", "骨料入口温度", Q::Temperature, 20.0, "C", ""),
              f("t_out", "骨料出口温度", Q::Temperature, 120.0, "C", ""),
              f("cp_agg", "骨料比热", Q::SpecificHeat, 0.9, "kJ/(kg.K)", ""),
              f("lhv", "燃料 LHV", Q::HeatingValue, 42.0, "MJ/kg", ""),
              f("eff", "系统热效率", Q::Percent, 80.0, "%", ""),
          ],
          "Q=骨料显热+水升温/汽化潜热(≈2260 kJ/kg)+过热；燃料=Q/(LHV·η)。"),

        m("air_heaters", Applications, "空气预热器", "Air Heaters",
          "烟气-空气换热器的热量、出口烟温、LMTD 与面积。",
          vec![
              f("q_air", "空气流量(标态)", Q::VolumetricFlow, 5000.0, "Nm3/h", ""),
              f("t_air_in", "空气入口", Q::Temperature, 20.0, "C", ""),
              f("t_air_out", "空气出口", Q::Temperature, 300.0, "C", ""),
              f("m_flue", "烟气流量", Q::MassFlow, 6000.0, "kg/h", ""),
              f("t_flue_in", "烟气入口", Q::Temperature, 450.0, "C", ""),
              f("cp_flue", "烟气比热", Q::SpecificHeat, 1.1, "kJ/(kg.K)", ""),
              f("u", "总传热系数", Q::HeatTransferCoeff, 30.0, "W/(m2.K)", ""),
          ],
          "Q=m_air·cp·ΔT；逆流 LMTD；A=Q/(U·LMTD)；烟气温降=Q/(m_flue·cp)。"),

        m("air_fuel_piping", Applications, "空气/燃气管路设计", "Air/Fuel Piping Design",
          "按热负荷确定燃气/空气量并按允许流速选管径、校核压降。",
          vec![
              f("power", "燃烧热功率", Q::Power, 1000.0, "kW", ""),
              f("lhv", "燃料 LHV", Q::HeatingValue, 50.0, "MJ/kg", ""),
              f("excess", "过量空气", Q::Percent, 10.0, "%", ""),
              f("v_gas", "燃气允许流速", Q::Velocity, 15.0, "m/s", ""),
              f("v_air", "空气允许流速", Q::Velocity, 15.0, "m/s", ""),
              f("l_pipe", "管长", Q::Length, 20.0, "m", ""),
              f("p_gas", "供气压力(表压)", Q::PressureGauge, 5.0, "kPa", ""),
          ],
          "m_fuel=P/LHV；V=ṁ/ρ；D=√(4Q/πv)；Δp 按 Darcy-Weisbach，NPS 取整后复核流速。"),

        m("flue_sizing", Applications, "烟道与烟囱尺寸", "Flue Sizing",
          "烟囱自然抽力、摩擦阻力与所需直径。",
          vec![
              f("q_flue", "烟气流量(工况)", Q::VolumetricFlow, 8000.0, "m3/h", ""),
              f("t_flue", "烟气平均温度", Q::Temperature, 250.0, "C", ""),
              f("t_amb", "环境温度", Q::Temperature, 20.0, "C", ""),
              f("h_stack", "烟囱高度", Q::Length, 15.0, "m", ""),
              f("d_stack", "烟囱内径", Q::Length, 0.8, "m", ""),
              f("roughness", "壁面粗糙度", Q::Length, 0.5, "mm", ""),
          ],
          "抽力 Δp=(ρ_air−ρ_flue)·g·H；ρ=pM/RT；摩擦按 f·H/D·ρv²/2。"),

        m("micro_valve", Applications, "微调阀流量特性", "Micro Valve Performance",
          "阀门 Cv/Kv 换算、液体/气体流量与临界流判定。",
          vec![
              f("kv", "阀门流量系数", Q::FlowCoefficient, 10.0, "Kv", "Kv=m3/h@1bar；Cv=US gpm@1psi"),
              f("dp", "阀前后压差", Q::PressureDrop, 10.0, "kPa", ""),
              f("p_up", "阀前绝压", Q::PressureAbs, 150.0, "kPa", "绝压"),
              f("t_gas", "气体温度", Q::Temperature, 15.0, "C", ""),
              f("sg", "气体相对密度", Q::Ratio, 0.6, "-", ""),
              f("opening", "阀门开度", Q::Percent, 100.0, "%", ""),
          ],
          "液：Q=Kv·√(Δp/bar)；气：简化可压缩式；p2/p1<0.53 判定临界流。"),

        m("furnace_heat_loss", Applications, "炉体散热损失", "Furnace Heat Losses",
          "炉墙多层导热+外表面对流散热与年度能耗。",
          vec![
              f("area", "炉体外表面", Q::Area, 100.0, "m2", ""),
              f("t_in", "炉内温度", Q::Temperature, 1200.0, "C", ""),
              f("t_amb", "环境温度", Q::Temperature, 20.0, "C", ""),
              f("k_ins", "保温层导热系数", Q::ThermalConductivity, 0.12, "W/(m.K)", ""),
              f("l_ins", "保温层厚度", Q::Length, 0.2, "m", ""),
              f("h_out", "外表面换热系数", Q::HeatTransferCoeff, 12.0, "W/(m2.K)", ""),
              f("hours", "年运行小时", Q::Time, 6000.0, "h", ""),
          ],
          "U=1/(1/h_in+L/k+1/h_out)；迭代求外壁温度使导热=表面散热。"),

        m("tba_blowers", Applications, "TBA 鼓风机选型", "TBA Blowers",
          "按风量/静压计算轴功率与电机裕量。",
          vec![
              f("q", "所需风量", Q::VolumetricFlow, 8000.0, "m3/h", ""),
              f("dp", "所需静压", Q::PressureDrop, 5.0, "kPa", ""),
              f("rho", "空气密度", Q::Density, 1.2, "kg/m3", ""),
              f("eff", "风机效率", Q::Percent, 65.0, "%", ""),
              f("margin", "电机裕量", Q::Percent, 15.0, "%", ""),
              f("altitude", "海拔", Q::Length, 0.0, "m", ""),
          ],
          "P_shaft=Q·Δp/η；海拔按气压公式修正密度；电机功率=P_shaft·(1+裕量)。"),

        m("fan_altitude", Applications, "风机海拔/温度修正", "Fan at Altitude/Temperature",
          "高海拔/高温下空气密度变化对风机性能的影响。",
          vec![
              f("q0", "海平面风量", Q::VolumetricFlow, 10000.0, "m3/h", ""),
              f("dp0", "海平面静压", Q::PressureDrop, 5.0, "kPa", ""),
              f("p0", "海平面功率", Q::Power, 20.0, "kW", ""),
              f("altitude", "海拔", Q::Length, 1500.0, "m", ""),
              f("t", "进气温度", Q::Temperature, 40.0, "C", ""),
              f("n", "转速", Q::Frequency, 1450.0, "rpm", ""),
          ],
          "p=p0·(1−2.25577e-5·H)^5.2559；ρ=p/(R·T)；体积风量不变，质量流量/静压/功率按密度比缩放。"),

        m("smj_blowers", Applications, "SMJ 鼓风机选型", "SMJ Blowers",
          "高压比鼓风机的功率、温升与噪声估算。",
          vec![
              f("q", "所需风量", Q::VolumetricFlow, 12000.0, "m3/h", ""),
              f("dp", "所需静压", Q::PressureDrop, 8.0, "kPa", ""),
              f("rho", "空气密度", Q::Density, 1.2, "kg/m3", ""),
              f("eff", "风机效率", Q::Percent, 70.0, "%", ""),
              f("margin", "电机裕量", Q::Percent, 15.0, "%", ""),
              f("t_in", "进气温度", Q::Temperature, 40.0, "C", ""),
          ],
          "P=Q·Δp/η；温升 ΔT≈Δp/(ρ·cp·η)；噪声 Lw≈87+10log10(P_kW)+20log10(Δp/Δp_ref)。"),

        m("altitude_burners", Applications, "燃烧器/阀门海拔修正", "Burners and Valves at Altitude/Temperature",
          "源程序 1:1（BURNER SIZING 模式）：由目录额定风量/风压与热负荷，计算高海拔/温度工况下的实际空气流量、标准状态所需空气量与所需风压。",
          vec![
              sel("fuel", "Select Fuel · 燃料选择", crate::combustion_fuels::WOBBE_GAS_NAMES, 0.0,
                  "源 Select Fuel 列表 · 化学计量空气由燃料组成计算"),
              f("elev", "Elevation above Sea Level · 海拔", Q::Length, 9000.0, "ft", ""),
              f("t_air", "Air Temperature · 空气温度", Q::Temperature, 60.0, "F", ""),
              f("exc_air", "Burner Excess Air · 燃烧器过剩空气", Q::Percent, 10.0, "%", ""),
              f("cap", "Desired Burner Capacity · 目标热负荷", Q::Power, 0.2, "MM Btu/h", ""),
              f("cat_flow", "Catalog Rated Air Flow · 目录额定空气流量", Q::VolumetricFlow, 2000.0, "scfh", "对应目录额定风压下的标况流量"),
              f("cat_press", "Catalog Rated Air Pressure · 目录额定空气压力", Q::PressureGauge, 12.0, "inwc", ""),
          ],
          "标况空气需求 = (热负荷/GHV)·化学计量空气·(1+过剩空气)；实际体积流量 = 标况需求 / 密度比（密度比 = 大气压比 × 60 °F 基准温度修正）；风压 Δp ∝ ρQ²，即目录风压 × 密度比 × (实际流量/目录流量)²。默认工况（9000 ft / 60 °F / EA 10 % / 0.2 MM Btu/h / 2000 scfh @ 12 inwc）与源程序实测对齐：2900 acfh · 2071 scfh · 18.0。大气压按 ISA 公式并以源 9000 ft 表值 10.5/14.7 psia 单点校准。"),

        m("eductor", Applications, "引射器", "Eductor",
          "引射比、混合流量、喷嘴速度与可达到的压升。",
          vec![
              f("p_motive", "工作流体压力(表压)", Q::PressureGauge, 100.0, "kPa", ""),
              f("q_motive", "工作流体流量", Q::VolumetricFlow, 100.0, "m3/h", ""),
              f("q_suction", "引射流量", Q::VolumetricFlow, 200.0, "m3/h", ""),
              f("eff", "引射效率", Q::Percent, 30.0, "%", ""),
              f("rho", "流体密度", Q::Density, 1.2, "kg/m3", ""),
              f("d_nozzle", "喷嘴直径", Q::Length, 20.0, "mm", ""),
          ],
          "喷嘴速度 v=√(2Δp/ρ)；动量守恒+效率估算混合室压升；引射比 u=Qs/Qm。"),

        // ------------------------------------------------------------- System Design
        m("sd_furnace_spec", SystemDesign, "① 炉膛规格", "Furnace Specification",
          "系统设计向导第 1 步：炉膛尺寸、产量与热负荷估算。",
          vec![
              f("len", "炉膛长", Q::Length, 6.0, "m", ""),
              f("width", "炉膛宽", Q::Length, 3.0, "m", ""),
              f("height", "炉膛高", Q::Length, 2.5, "m", ""),
              f("t_furnace", "炉温", Q::Temperature, 1200.0, "C", ""),
              f("production", "产量", Q::MassFlow, 10.0, "t/h", ""),
              f("cp", "工件比热", Q::SpecificHeat, 0.6, "kJ/(kg.K)", ""),
          ],
          "V=L·W·H；Q_work=ṁ·cp·ΔT；按 25% 附加散热估算装机热负荷。"),

        m("sd_burner_selection", SystemDesign, "② 燃烧器选型", "Burner Selection",
          "系统设计向导第 2 步：台数、燃气量与助燃空气量。",
          vec![
              f("q_total", "总热负荷", Q::Power, 2000.0, "kW", ""),
              f("q_unit", "单台燃烧器功率", Q::Power, 500.0, "kW", ""),
              f("spare", "备用裕量", Q::Percent, 20.0, "%", ""),
              f("excess", "过量空气", Q::Percent, 10.0, "%", ""),
              f("lhv", "燃料 LHV", Q::HeatingValue, 50.0, "MJ/kg", ""),
          ],
          "N=ceil(Q_total(1+裕量)/Q_unit)；燃气=Q/LHV；空气=燃气·0.34·LHV·(1+λ)。"),

        m("sd_air_line", SystemDesign, "③ 空气管路与部件", "Air Line Specification and Component Selection",
          "系统设计向导第 3 步：主管管径、压降与阀门/过滤器阻力。",
          vec![
              f("q_air", "总空气量", Q::VolumetricFlow, 20000.0, "m3/h", ""),
              f("l", "管长", Q::Length, 30.0, "m", ""),
              f("v", "设计流速", Q::Velocity, 15.0, "m/s", ""),
              f("k_loss", "部件 ΣK", Q::Dimensionless, 4.0, "-", "过滤器+阀+弯头"),
              f("dp_allow", "允许压降", Q::PressureDrop, 2.0, "kPa", ""),
          ],
          "D=√(4Q/πv)；Δp 含沿程与局部；与允许压降比较给出结论。"),

        m("sd_blower_selection", SystemDesign, "④ 鼓风机选型", "Blower Selection",
          "系统设计向导第 4 步：风量、静压与电机功率。",
          vec![
              f("q", "风量", Q::VolumetricFlow, 20000.0, "m3/h", ""),
              f("dp", "系统总压降", Q::PressureDrop, 3.0, "kPa", ""),
              f("eff", "风机效率", Q::Percent, 68.0, "%", ""),
              f("margin", "电机裕量", Q::Percent, 15.0, "%", ""),
              f("rho", "空气密度", Q::Density, 1.2, "kg/m3", ""),
          ],
          "P=Q·Δp/η；电机功率按裕量向上取标准档。"),

        m("sd_fuel_line", SystemDesign, "⑤ 燃气管路与部件", "Fuel Line Component Selection",
          "系统设计向导第 5 步：燃气管径、压降与调压器/切断阀 Kv。",
          vec![
              f("q_gas", "燃气量(标态)", Q::VolumetricFlow, 400.0, "Nm3/h", ""),
              f("p_supply", "供气压力(表压)", Q::PressureGauge, 10.0, "kPa", ""),
              f("l", "管长", Q::Length, 20.0, "m", ""),
              f("v", "设计流速", Q::Velocity, 15.0, "m/s", ""),
              f("dp_allow", "允许压降", Q::PressureDrop, 1.0, "kPa", ""),
          ],
          "按标态→工况状态修正体积流量；D 由允许流速定；Kv=Q/√Δp（低压气体简化）。"),

        m("sd_print_preview", SystemDesign, "⑥ 设计报告", "Print Preview",
          "系统设计向导第 6 步：汇总关键设计参数并生成可复制报告。",
          vec![
              f("q_total", "总热负荷", Q::Power, 2000.0, "kW", ""),
              f("lhv", "燃料 LHV", Q::HeatingValue, 50.0, "MJ/kg", ""),
              f("excess", "过量空气", Q::Percent, 10.0, "%", ""),
              f("q_air", "助燃空气量", Q::VolumetricFlow, 20000.0, "Nm3/h", ""),
              f("dp", "系统压降", Q::PressureDrop, 3.0, "kPa", ""),
          ],
          "汇总燃气/空气/烟气/电机功率，输出可复制的纯文本设计摘要。"),

        m("sd_nfpa", SystemDesign, "⑦ 安全联锁检查", "NFPA System Interpretation",
          "系统设计向导第 7 步：吹扫体积/时间与联锁项核对。",
          vec![
              f("volume", "炉膛+烟道容积", Q::Volume, 45.0, "m3", ""),
              f("q_purge", "吹扫风量", Q::VolumetricFlow, 20000.0, "m3/h", ""),
              f("changes", "要求换气次数", Q::Dimensionless, 4.0, "-", "NFPA 常用 ≥4 倍容积"),
              f("p_leak", "检漏试验压力(表压)", Q::PressureGauge, 5.0, "kPa", ""),
              f("trials", "点火尝试次数上限", Q::Dimensionless, 3.0, "-", ""),
          ],
          "V_purge=N·V；t=V_purge/Q；输出吹扫、检漏、火焰检测、双切断阀等联锁核对清单。"),
    ]
}

/// Utility pages that are not numerical modules (implemented in the GUI).
pub fn tool_pages() -> Vec<ModuleJson> {
    vec![
        ModuleJson {
            id: "keyword_search", group: "Tools 工具", group_key: "Tools".into(),
            name: "关键词检索", en: "Key Word Search", desc: "按中英文名称、说明与公式搜索全部模块。",
            fields: vec![], method: "前端实时过滤模块注册表。",
        },
        ModuleJson {
            id: "glossary", group: "Tools 工具", group_key: "Tools".into(),
            name: "术语表", en: "Glossary", desc: "燃烧与传热工程常用术语及单位约定。",
            fields: vec![], method: "内置公开术语定义，不含原程序文案。",
        },
        ModuleJson {
            id: "catalog", group: "Tools 工具", group_key: "Tools".into(),
            name: "设备目录", en: "Online Catalog", desc: "浏览与录入 SQLite 设备/燃料/材料目录及计算历史。",
            fields: vec![], method: "读写 open_solutions.db；已导入本机 E-Solutions MDB 目录，也可继续录入自有数据。",
        },
        ModuleJson {
            id: "unit_settings", group: "Tools 工具", group_key: "Tools".into(),
            name: "单位设置", en: "Set Default Units", desc: "切换 SI / US 单位制并查看单位约定审计表。",
            fields: vec![], method: "所有字段在 I/O 边界显式标注单位，内部统一 SI。",
        },
    ]
}
