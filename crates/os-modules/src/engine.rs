//! Engineering kernels for every module. All inputs arrive in canonical SI
//! (converted once at the boundary) and all results are emitted in canonical
//! SI, then formatted into both unit systems for display.
//!
//! All correlations are standard, publicly documented engineering relations.
//! Defaults are illustrative, not vendor data.

use crate::defs::{all_modules, ModuleDef, UNITS_FOR_CONVERTER};
use crate::quantity::{smart_fmt, Quantity as Q};
use os_core::units::{self as u, UnitSystem};
use serde::Serialize;
use serde_json::{Map, Value};
use std::f64::consts::PI;

const R: f64 = 8.314_462_618;
const P_STD: f64 = u::P_STD_PA;
const T_STD: f64 = 273.15;
const G: f64 = 9.80665;
const SIGMA: f64 = 5.670_374_419e-8;
const R_AIR: f64 = 287.05;

#[derive(Debug, Serialize)]
pub struct RowJson {
    pub label: &'static str,
    /// Value formatted in the SI display unit.
    pub si: String,
    /// Same physical value formatted in the US display unit.
    pub us: String,
    pub note: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ComputeResult {
    pub rows: Vec<RowJson>,
    pub method: String,
}

pub struct Row {
    label: &'static str,
    value: f64,
    qty: Q,
    note: &'static str,
    text: Option<String>,
}

fn rq(label: &'static str, value: f64, qty: Q) -> Row {
    Row { label, value, qty, note: "", text: None }
}
fn rqn(label: &'static str, value: f64, qty: Q, note: &'static str) -> Row {
    Row { label, value, qty, note, text: None }
}
fn tx(label: &'static str, text: impl Into<String>) -> Row {
    Row { label, value: f64::NAN, qty: Q::Dimensionless, note: "", text: Some(text.into()) }
}
fn txn(label: &'static str, text: impl Into<String>, note: &'static str) -> Row {
    Row { label, value: f64::NAN, qty: Q::Dimensionless, note, text: Some(text.into()) }
}

fn finish(rows: Vec<Row>) -> Vec<RowJson> {
    rows.into_iter()
        .map(|r| {
            if let Some(t) = r.text {
                RowJson { label: r.label, si: t, us: String::new(), note: r.note }
            } else {
                RowJson {
                    label: r.label,
                    si: r.qty.format(r.value, r.qty.display_unit(UnitSystem::Si)),
                    us: r.qty.format(r.value, r.qty.display_unit(UnitSystem::Imperial)),
                    note: r.note,
                }
            }
        })
        .collect()
}

struct Ctx<'a> {
    m: &'a Map<String, Value>,
    d: &'a ModuleDef,
}

impl Ctx<'_> {
    /// Read a field. Missing fields use their canonical SI default; supplied
    /// fields carry an explicit unit and are converted exactly once.
    fn v(&self, key: &str) -> f64 {
        let field = match self.d.fields.iter().find(|f| f.key == key) {
            Some(f) => f,
            None => return 0.0,
        };
        match self.m.get(key) {
            None => field.default_si,
            Some(Value::Object(o)) => {
                let val = o.get("value").and_then(Value::as_f64).unwrap_or(field.default_si);
                if !field.options.is_empty() {
                    return val; // select index, unitless
                }
                let unit = o.get("unit").and_then(Value::as_str).unwrap_or(field.qty.si_unit());
                field.qty.to_si(val, unit).unwrap_or(val)
            }
            Some(Value::Number(n)) => n.as_f64().unwrap_or(field.default_si),
            _ => field.default_si,
        }
    }
    fn s(&self, key: &str) -> usize {
        self.v(key).round().max(0.0) as usize
    }
}

/// Inclusive helper: clamp to avoid log/0 in engineering correlations.
fn safe(x: f64, fallback: f64) -> f64 {
    if x.is_finite() && x != 0.0 { x } else { fallback }
}

fn atmosphere(h_m: f64) -> f64 {
    P_STD * (1.0 - 2.255_77e-5 * h_m.max(0.0)).powf(5.2559)
}

fn swamee_jain(re: f64, eps_d: f64) -> f64 {
    if re < 2300.0 {
        64.0 / re.max(1.0)
    } else {
        0.25 / (f64::log10(eps_d / 3.7 + 5.74 / re.powf(0.9))).powi(2)
    }
}

fn circle_area(d: f64) -> f64 { PI * d * d / 4.0 }

pub fn compute(id: &str, inputs: &Map<String, Value>) -> Result<ComputeResult, String> {
    let d = all_modules()
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| format!("未知模块 / unknown module: {id}"))?;
    let c = Ctx { m: inputs, d: &d };
    let rows = match id {
        "fuel_data" => fuel_data(&c),
        "gas_liquid_props" => gas_liquid_props(&c),
        "solids_props" => solids_props(&c),
        "fuel_combustion" => fuel_combustion(&c),
        "wobbe_index" => wobbe_index(&c),
        "energy_savings" => energy_savings(&c),
        "fluid_mechanics" => fluid_mechanics(&c),
        "incomplete_combustion" => incomplete_combustion(&c),
        "flammability" => flammability(&c),
        "orifice_meter" => orifice_meter(&c),
        "natural_convection" => natural_convection(&c),
        "forced_convection" => forced_convection(&c),
        "radiation" => radiation(&c),
        "sound" => sound(&c),
        "emission" => emission(&c),
        "fan_laws" => fan_laws(&c),
        "unit_conversion" => unit_conversion(&c),
        "tables" => tables(&c),
        "geometry_tools" => geometry_tools(&c),
        "aggregate_drying" => aggregate_drying(&c),
        "air_heaters" => air_heaters(&c),
        "air_fuel_piping" => air_fuel_piping(&c),
        "flue_sizing" => flue_sizing(&c),
        "micro_valve" => micro_valve(&c),
        "furnace_heat_loss" => furnace_heat_loss(&c),
        "tba_blowers" => tba_blowers(&c),
        "fan_altitude" => fan_altitude(&c),
        "smj_blowers" => smj_blowers(&c),
        "altitude_burners" => altitude_burners(&c),
        "eductor" => eductor(&c),
        "sd_furnace_spec" => sd_furnace_spec(&c),
        "sd_burner_selection" => sd_burner_selection(&c),
        "sd_air_line" => sd_air_line(&c),
        "sd_blower_selection" => sd_blower_selection(&c),
        "sd_fuel_line" => sd_fuel_line(&c),
        "sd_print_preview" => sd_print_preview(&c),
        "sd_nfpa" => sd_nfpa(&c),
        _ => return Err(format!("模块 {id} 尚未实现")),
    };
    Ok(ComputeResult { rows: finish(rows), method: d.method.to_string() })
}

// ============================================================== Reference ====

fn fuel_data(c: &Ctx) -> Vec<Row> {
    // Volume-% composition of the ten tracked components. The UI prefills
    // these from the imported `fuel_gas_components` table and lets the user
    // edit them freely for a "customized gas mixture"; the engine only sees
    // the ten numbers and treats them as a volume-% analysis.
    let raw = [
        c.v("x_ch4"), c.v("x_c2h6"), c.v("x_c3h8"), c.v("x_c4h10"),
        c.v("x_co"), c.v("x_h2"), c.v("x_co2"), c.v("x_o2"), c.v("x_n2"), c.v("x_h2o"),
    ];
    let sum: f64 = raw.iter().sum();
    let n = if sum > 0.0 { 1.0 / sum } else { 0.0 };
    let x: Vec<f64> = raw.iter().map(|v| v * n).collect(); // mol fractions, Σ=1
    let mw   = [16.043, 30.070, 44.097, 58.124, 28.010, 2.016, 44.010, 31.999, 28.014, 18.015];
    let lhv  = [802.3, 1428.0, 2043.0, 2658.0, 283.0, 241.8, 0.0, 0.0, 0.0, 0.0];   // kJ/mol, LHV
    let hhv  = [890.3, 1559.9, 2220.0, 2878.0, 283.0, 285.8, 0.0, 0.0, 0.0, 0.0];   // kJ/mol, HHV
    let m_avg = x.iter().zip(mw.iter()).map(|(a, b)| a * b).sum::<f64>();             // kg/kmol
    let lhv_mol = x.iter().zip(lhv.iter()).map(|(a, b)| a * b).sum::<f64>();         // kJ/mol
    let hhv_mol = x.iter().zip(hhv.iter()).map(|(a, b)| a * b).sum::<f64>();         // kJ/mol
    let lhv_kg = if m_avg > 0.0 { lhv_mol / m_avg } else { 0.0 };                    // MJ/kg
    let hhv_kg = if m_avg > 0.0 { hhv_mol / m_avg } else { 0.0 };                    // MJ/kg
    let rho = if m_avg > 0.0 { m_avg / 22.414 } else { 0.0 };                        // kg/Nm3
    let sg = if m_avg > 0.0 { m_avg / 28.97 } else { 0.0 };
    let vol_lhv = lhv_mol / 22.414;                                                  // MJ/Nm3
    let vol_hhv = hhv_mol / 22.414;
    let vol_lhv_btu = vol_lhv / 0.0372587;                                           // Btu/scf
    let vol_hhv_btu = vol_hhv / 0.0372587;
    let o2_coef  = [2.0, 3.5, 5.0, 6.5, 0.5, 0.5, 0.0, -0.5, 0.0, 0.0];
    let co2_coef = [1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let h2o_coef = [2.0, 3.0, 4.0, 5.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let o2_req = x.iter().zip(o2_coef.iter()).map(|(a, b)| a * b).sum::<f64>();
    let air_mol = o2_req / 0.21;
    let a0_kg = if m_avg > 0.0 { air_mol * 28.97 / m_avg } else { 0.0 };
    let a0_nm3 = if m_avg > 0.0 { air_mol * 22.414 / m_avg } else { 0.0 };
    let co2_gen = x.iter().zip(co2_coef.iter()).map(|(a, b)| a * b).sum::<f64>();
    let h2o_gen = x.iter().zip(h2o_coef.iter()).map(|(a, b)| a * b).sum::<f64>();
    let n2_air = air_mol * 0.79;
    let v0_wet = (co2_gen + h2o_gen + n2_air) * 22.414 / m_avg.max(1e-9);
    let co2_dry = if (co2_gen + n2_air) > 0.0 { co2_gen / (co2_gen + n2_air) } else { 0.0 };

    vec![
        rqn("平均分子量 M", m_avg, Q::MolecularWeight, "Σ xi·Mi"),
        rqn("相对比重 SG", sg, Q::Ratio, "M / 28.97（空气）"),
        rq("标况密度 ρ₀", rho, Q::Density),
        rqn("质量低位热值 LHV", lhv_kg * 1e6, Q::HeatingValue, "摩尔热值 / M；质量基；MJ/kg→J/kg"),
        rqn("质量高位热值 HHV", hhv_kg * 1e6, Q::HeatingValue, "摩尔热值 / M；含 H2O 潜热；MJ/kg→J/kg"),
        txn("体积低位热值 LHV",
            format!("{:.3} MJ/Nm³ = {:.1} Btu/scf", vol_lhv, vol_lhv_btu),
            "体积基；与源库 NetHeatSI 量级一致"),
        txn("体积高位热值 HHV",
            format!("{:.3} MJ/Nm³ = {:.1} Btu/scf", vol_hhv, vol_hhv_btu),
            "含水蒸气潜热"),
        txn("理论需氧量 O₂", format!("{:.4} mol/mol 燃料", o2_req),
            "2·CH4+3.5·C2H6+5·C3H8+6.5·C4H10+0.5·CO+0.5·H2−0.5·O2"),
        txn("理论空气量 A₀", format!("{:.3} kg/kg = {:.3} Nm³/kg", a0_kg, a0_nm3),
            "空气 O₂=21% / M=28.97"),
        txn("理论湿烟气量 V₀", format!("{:.3} Nm³/kg", v0_wet),
            "无过量空气、完全燃烧"),
        rqn("理论最大 CO₂（干基）", co2_dry, Q::Percent, "CO₂/(CO₂+N₂,air)"),
        txn("组成归一化", format!("Σ = {:.2} % → 100 %", sum), "输入按 100% 归一"),
    ]
}

fn gas_liquid_props(c: &Ctx) -> Vec<Row> {
    use crate::fluids_data::{FluidKind, FLUID_DEFS, FLUID_NAMES_PLUS_MIX, RHO_AIR_STD};
    let i = c.s("fluid");
    if i >= FLUID_NAMES_PLUS_MIX.len() - 1 {
        return gas_mixture_props(c);
    }
    let f = FLUID_DEFS[i];
    let t_f = u::Temperature::from_k(c.v("t")).as_f();
    let p_psia = c.v("p") / u::PSI_IN_PA;

    // --- range validation, mirroring the source popups ---
    if t_f < f.min_t - 1e-9 || t_f > f.max_t + 1e-9 {
        return vec![txn(
            "温度超出范围",
            format!("{} 有效温度 {}–{} °F（当前 {:.0} °F）", f.name, f.min_t, f.max_t, t_f),
            "与源程序范围弹窗一致",
        )];
    }
    if p_psia < 14.7 - 1e-9 {
        return vec![txn(
            "压力低于下限",
            format!("压力范围 14.7–500 psia（当前 {:.1} psia）", p_psia),
            "源程序最低 14.7 psia（1 atm）",
        )];
    }
    if p_psia > 500.0 + 1e-9 {
        return vec![txn(
            "压力超出范围",
            format!("压力范围 14.7–500 psia（当前 {:.1} psia）", p_psia),
            "源程序最大 500 psia",
        )];
    }

    /// Linear interpolation over an ascending (T, ...) table, clamped at both
    /// ends. Returns the fully interpolated 7-tuple row.
    fn row_at(tab: &[(f64, f64, f64, f64, f64, f64, f64)], t: f64) -> (f64, f64, f64, f64, f64, f64, f64) {
        let last = tab.len() - 1;
        if t <= tab[0].0 {
            return tab[0];
        }
        if t >= tab[last].0 {
            return tab[last];
        }
        let mut lo = 0usize;
        while lo + 1 < tab.len() && tab[lo + 1].0 < t {
            lo += 1;
        }
        let (t0, a) = (tab[lo].0, tab[lo]);
        let (t1, b) = (tab[lo + 1].0, tab[lo + 1]);
        if (t1 - t0).abs() < 1e-12 {
            return a;
        }
        let w = (t - t0) / (t1 - t0);
        let lerp = |x: f64, y: f64| x + (y - x) * w;
        (t, lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3), lerp(a.4, b.4), lerp(a.5, b.5), lerp(a.6, b.6))
    }

    /// Linear interpolation over a liquid (T, rho, sg, cp, mu, k, h, nu, pr) table.
    fn lrow_at(tab: &[(f64, f64, f64, f64, f64, f64, f64, f64, f64)], t: f64)
        -> (f64, f64, f64, f64, f64, f64, f64, f64, f64)
    {
        let last = tab.len() - 1;
        if t <= tab[0].0 {
            return tab[0];
        }
        if t >= tab[last].0 {
            return tab[last];
        }
        let mut lo = 0usize;
        while lo + 1 < tab.len() && tab[lo + 1].0 < t {
            lo += 1;
        }
        let (t0, a) = (tab[lo].0, tab[lo]);
        let (t1, b) = (tab[lo + 1].0, tab[lo + 1]);
        if (t1 - t0).abs() < 1e-12 {
            return a;
        }
        let w = (t - t0) / (t1 - t0);
        let lerp = |x: f64, y: f64| x + (y - x) * w;
        (t, lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3), lerp(a.4, b.4),
            lerp(a.5, b.5), lerp(a.6, b.6), lerp(a.7, b.7), lerp(a.8, b.8))
    }

    const BTU_LBF_PER_KG_K: f64 = 4186.8;      // Btu/(lb·°F) -> J/(kg·K)
    const BTU_FT_PER_W_MK: f64 = 1.730_735;    // Btu/(ft·h·°F) -> W/(m·K)
    const LB_FT3_PER_KG_M3: f64 = 16.018_463;  // lb/ft³ -> kg/m³
    const FT2_PER_M2: f64 = 10.763_910_4;      // m²/s -> ft²/s

    match f.kind {
        FluidKind::Gas => {
            let r = row_at(f.tab, t_f);
            // density: ideal-gas law (verified against source for all gases
            // except water vapor, which uses table interpolation)
            let rho_us = if f.name == "Vapor H2O" {
                r.1 * p_psia / 14.7
            } else {
                f.rho0 * 460.0 / (460.0 + t_f) * p_psia / 14.7
            };
            let cp = r.2;                     // Btu/(lb·°F)
            let mu_cp = r.3;                  // cP
            let k_us = r.4;                   // Btu/(ft·h·°F)
            let h_us = r.5;                   // BTU/scf std, ref 60°F
            let gamma = r.6;
            let sg = rho_us / RHO_AIR_STD;
            let nu_ft2 = mu_cp * 6.72e-4 / rho_us.max(1e-9); // ft²/s
            let pr = cp * (mu_cp * 6.72e-4) * 3600.0 / k_us.max(1e-9);
            vec![
                rqn("密度 Density, lb/ft³", rho_us * LB_FT3_PER_KG_M3, Q::Density, "表值 × P/14.7psia（与源程序逐点一致）"),
                rqn("比重 Specific Gravity (Air=1.0)", sg, Q::Ratio, "ρ/0.076335 lb/ft³（标准空气）"),
                rqn("比热 Specific Heat, Btu/(lb·°F)", cp * BTU_LBF_PER_KG_K, Q::SpecificHeat, "Btu/(lb·°F) 表值"),
                rqn("动力粘度 Absolute Viscosity, cP", mu_cp * 1e-3, Q::DynamicViscosity, "cP 表值"),
                txn("运动粘度 Kinematic Viscosity, ft²/sec", format!("{:.4e} m²/s（{:.3e} ft²/s）", nu_ft2 / FT2_PER_M2, nu_ft2), "ν = μ[cP]×6.72e-4/ρ[lb/ft³]"),
                txn("焓 Enthalpy, BTU/SCF", format!("{:.2} BTU/scf（标况，60°F 基准）｜ {:.0} kJ/m³", h_us, h_us * 37_258.9 / 1000.0), "表值（14.7 psia）"),
                rqn("比热比 Ratio of Specific Heats", gamma, Q::Ratio, "表值"),
                rqn("导热系数 Thermal Conductivity, BTU/(h·ft·°F)", k_us * BTU_FT_PER_W_MK, Q::ThermalConductivity, "Btu/(ft·h·°F) 表值"),
                rqn("普朗特数 Prandtl Number", pr, Q::Ratio, "Pr = cp·μ·3600/k（已验证与源一致）"),
                txn("数据说明", "μ/k/γ/cp/h 表值基于 14.7 psia；密度按 P/14.7 精确换算。Aver. Flue Gas 仅在 14.7 psia 有效。", "源程序：Data At Pressure of 14.7 psia"),
            ]
        }
        FluidKind::Liquid => {
            let r = lrow_at(f.ltab, t_f);
            let rho_us = r.1; // lb/ft³
            let sg = r.2;
            let cp = r.3;     // Btu/(lb·°F)
            let mu_cp = r.4;
            let k_us = r.5;
            let h_us = r.6;   // BTU/lb
            let nu_ft2 = r.7; // ft²/s (表值)
            let pr = r.8;
            vec![
                rqn("密度 Density, lb/ft³", rho_us * LB_FT3_PER_KG_M3, Q::Density, "lb/ft³ 表值（14.7 psia 液体近似不可压）"),
                rqn("比重 Specific Gravity (Water=1.0)", sg, Q::Ratio, "ρ/62.3 lb/ft³（60°F 水）"),
                rqn("比热 Specific Heat, Btu/(lb·°F)", cp * BTU_LBF_PER_KG_K, Q::SpecificHeat, "Btu/(lb·°F) 表值"),
                rqn("动力粘度 Absolute Viscosity, cP", mu_cp * 1e-3, Q::DynamicViscosity, "cP 表值"),
                txn("运动粘度 Kinematic Viscosity, ft²/sec", format!("{:.4e} m²/s（{:.3e} ft²/s）", nu_ft2 / FT2_PER_M2, nu_ft2), "表值；ν=μ·6.72e-4/ρ"),
                rqn("焓 Enthalpy, BTU/LB", h_us * 2326.0, Q::HeatingValue, "BTU/lb 表值（基准随流体而异，1 BTU/lb=2.326 kJ/kg）"),
                txn("比热比 Ratio of Specific Heats", "1（液体近似不可压）", "Cp≈Cv"),
                rqn("导热系数 Thermal Conductivity, BTU/(h·ft·°F)", k_us * BTU_FT_PER_W_MK, Q::ThermalConductivity, "Btu/(ft·h·°F) 表值"),
                rqn("普朗特数 Prandtl Number", pr, Q::Ratio, "表值"),
            ]
        }
    }
}

/// Customized Gas Mixture — 12 components by volume %, mirroring the source:
/// sum < 100% auto-fills N2; component mu via Wilke; rho via ideal-gas MW;
/// cp mass-weighted; gamma/k/h volume-weighted (all rules verified against
/// 6 measured source mixtures).
fn gas_mixture_props(c: &Ctx) -> Vec<Row> {
    use crate::fluids_data::FLUID_DEFS;
    const KEYS: [&str; 12] = ["y_ch4","y_c2h6","y_c2h4","y_c3h6","y_c3h8","y_c4h10",
                              "y_h2","y_co","y_n2","y_co2","y_o2","y_h2o"];
    // component -> index into FLUID_DEFS (gas table)
    const DEF_IDX: [usize; 12] = [8, 10, 9, 11, 12, 13, 7, 2, 1, 3, 6, 4];
    const MWS: [f64; 12] = [16.043, 30.070, 28.052, 42.081, 44.097, 58.123,
                            2.016, 28.010, 28.013, 44.010, 31.999, 18.015];
    const SYMS: [&str; 12] = ["CH4","C2H6","C2H4","C3H6","C3H8","C4H10",
                              "H2","CO","N2","CO2","O2","H2O"];

    let t_f = u::Temperature::from_k(c.v("t")).as_f();
    let p_psia = c.v("p") / u::PSI_IN_PA;

    let mut y = [0.0f64; 12];
    let mut sum = 0.0;
    for k in 0..12 {
        y[k] = c.v(KEYS[k]) * 100.0; // fraction -> vol %
        if y[k] < 0.0 { y[k] = 0.0; }
        sum += y[k];
    }
    if sum > 100.0 + 1e-6 {
        return vec![txn("组分总和超限",
            format!("组分体积分数之和为 {:.2}%，超过 100%", sum),
            "总和不得超过 100%（源程序：不足部分自动补 N2）")];
    }
    let y_n2_fill = 100.0 - sum;
    y[8] += y_n2_fill; // auto N2

    fn row_at(tab: &[(f64, f64, f64, f64, f64, f64, f64)], t: f64) -> (f64, f64, f64, f64, f64, f64, f64) {
        let last = tab.len() - 1;
        if t <= tab[0].0 { return tab[0]; }
        if t >= tab[last].0 { return tab[last]; }
        let mut lo = 0usize;
        while lo + 1 < tab.len() && tab[lo + 1].0 < t { lo += 1; }
        let (t0, a) = (tab[lo].0, tab[lo]);
        let (t1, b) = (tab[lo + 1].0, tab[lo + 1]);
        if (t1 - t0).abs() < 1e-12 { return a; }
        let w = (t - t0) / (t1 - t0);
        let lerp = |x: f64, yy: f64| x + (yy - x) * w;
        (t, lerp(a.1, b.1), lerp(a.2, b.2), lerp(a.3, b.3), lerp(a.4, b.4), lerp(a.5, b.5), lerp(a.6, b.6))
    }

    // upper-bound check across components (source: "Max. Temperature for CH4 is 930 F")
    let mut tmax_name = "";
    let mut tmax = f64::INFINITY;
    for k in 0..12 {
        if y[k] <= 0.0 { continue; }
        let f = FLUID_DEFS[DEF_IDX[k]];
        if f.max_t < tmax { tmax = f.max_t; tmax_name = f.name; }
    }
    if t_f > tmax + 1e-9 {
        return vec![txn("温度超出组分范围",
            format!("Max. Temperature for {} is {:.0} F（当前 {:.0} °F）", tmax_name, tmax, t_f),
            "与源程序混合气温度校验一致")];
    }
    if p_psia < 14.7 - 1e-9 || p_psia > 500.0 + 1e-9 {
        return vec![txn("压力超出范围",
            format!("压力范围 14.7–500 psia（当前 {:.1} psia）", p_psia),
            "源程序压力范围 14.7–500 psia")];
    }

    // component properties at T (tables incl. synthetic low-T extension for H2/H2O)
    let mut cp_mw = 0.0;   // Σ y·MW·cp
    let mut gamma_v = 0.0; // Σ y·gamma
    let mut k_v = 0.0;     // Σ y·k
    let mut h_v = 0.0;     // Σ y·h
    let mut mw_mix = 0.0;
    // Wilke accumulator: (y, mu[cP], MW, symbol) per present component
    let mut wilke: Vec<(f64, f64, f64, &'static str)> = Vec::new();
    for k in 0..12 {
        if y[k] <= 0.0 { continue; }
        let f = FLUID_DEFS[DEF_IDX[k]];
        let r = row_at(f.tab, t_f);
        let yi = y[k] / 100.0;
        cp_mw += yi * MWS[k] * r.2;
        gamma_v += yi * r.6;
        k_v += yi * r.4;
        h_v += yi * r.5;
        mw_mix += yi * MWS[k];
        wilke.push((yi, r.3, MWS[k], SYMS[k]));
    }
    // Wilke (form with per-species denominator, Brokaw): mu = Σ x_i μ_i / D_i
    let mu_mix = {
        let mut acc = 0.0;
        for &(xi, mui, mi, _) in &wilke {
            let mut d = 0.0;
            for &(xj, muj, mj, _) in &wilke {
                let phi = if (mui - muj).abs() < 1e-15 && (mi - mj).abs() < 1e-15 {
                    1.0
                } else {
                    let num = (1.0 + (mui / muj).sqrt() * (mj / mi).powf(0.25)).powi(2);
                    let den = 8.0f64.sqrt() * (1.0 + mi / mj).sqrt();
                    num / den
                };
                d += xj * phi;
            }
            acc += xi * mui / d;
        }
        acc
    };

    // ideal-gas density (verified: SG = MW/28.882 × 520/(460+T), rho = SG × 0.0765)
    let sg = mw_mix / 28.882 * 520.0 / (460.0 + t_f);
    let rho_us = 0.0765 * sg * p_psia / 14.7;
    let cp_mix = cp_mw / mw_mix;
    let nu_ft2 = mu_mix * 6.72e-4 / rho_us.max(1e-9);
    let pr = cp_mix * (mu_mix * 6.72e-4) * 3600.0 / k_v.max(1e-9);

    let comp_txt: String = wilke.iter().map(|&(yi, _, _, sym)| {
        format!("{} {:.1}%", sym, yi * 100.0)
    }).collect::<Vec<_>>().join(" + ");

    const BTU_LBF_PER_KG_K: f64 = 4186.8;
    const BTU_FT_PER_W_MK: f64 = 1.730_735;
    const LB_FT3_PER_KG_M3: f64 = 16.018_463;
    const FT2_PER_M2: f64 = 10.763_910_4;

    vec![
        txn("混合气组成", comp_txt, "按体积分数；不足 100% 自动补 N2（与源程序一致）"),
        rqn("密度 Density, lb/ft³", rho_us * LB_FT3_PER_KG_M3, Q::Density, "理想气体：ρ=0.0765×(MW/28.882)×520/(460+T)×P/14.7（源实测验证）"),
        rqn("比重 Specific Gravity (Air=1.0)", rho_us / 0.0765, Q::Ratio, "SG = ρ/0.0765 lb/ft³（14.7 psia、60°F 空气基准；ρ=SG×0.0765）"),
        rqn("比热 Specific Heat, Btu/(lb·°F)", cp_mix * BTU_LBF_PER_KG_K, Q::SpecificHeat, "质量加权 Σy·MW·cp/Σy·MW（源实测验证）"),
        rqn("动力粘度 Absolute Viscosity, cP", mu_mix * 1e-3, Q::DynamicViscosity, "Wilke（Brokaw 形式）"),
        txn("运动粘度 Kinematic Viscosity, ft²/sec", format!("{:.4e} m²/s（{:.3e} ft²/s）", nu_ft2 / FT2_PER_M2, nu_ft2), "ν = μ[cP]×6.72e-4/ρ[lb/ft³]"),
        txn("焓 Enthalpy, BTU/SCF", format!("{:.2} BTU/scf（标况，60°F 基准）｜ {:.0} kJ/m³", h_v, h_v * 37_258.9 / 1000.0), "体积加权 Σy·h"),
        rqn("比热比 Ratio of Specific Heats", gamma_v, Q::Ratio, "体积加权 Σy·γ（源实测验证）"),
        rqn("导热系数 Thermal Conductivity, BTU/(h·ft·°F)", k_v * BTU_FT_PER_W_MK, Q::ThermalConductivity, "体积加权 Σy·k（源实测验证）"),
        rqn("普朗特数 Prandtl Number", pr, Q::Ratio, "Pr = cp·μ·2.4192/k"),
    ]
}

fn solids_props(c: &Ctx) -> Vec<Row> {
    use crate::materials::MATERIALS;
    let i = c.s("mat").min(MATERIALS.len() - 1);
    let m = &MATERIALS[i];
    let l = c.v("l").max(1e-6);
    let dt = c.v("dt").max(0.0);
    let alpha = m.k / (m.rho * m.cp).max(1e-9);
    let r_th = l / m.k.max(1e-9);
    let q_flux = m.k * dt / l;
    let storage = m.rho * m.cp * l * dt / 1000.0;
    vec![
        txn("材料", format!("{} ({})", m.name, m.formula), "选定材料"),
        txn("密度 ρ", format!("{:.0} kg/m³", m.rho), "源库 lb/ft³ 已换算 SI"),
        txn("比热 cp", format!("{:.0} J/(kg·K)", m.cp), "源库 Btu/(lb·°F) 已换算 SI"),
        txn("导热系数 k", format!("{:.1} W/(m·K)", m.k), "源库 Btu/(h·ft·°F)，参考温度表值"),
        txn("辐射率 ε", format!("{:.3}", m.emis), ""),
        txn("熔点 / 使用上限", format!("{:.0} °C / {:.0} °C", m.tmelt_c, m.tmax_c), ""),
        txn("热扩散率 α", format!("{:.3e} m²/s", alpha), "α=k/(ρ·cp)"),
        txn("单位面积热阻", format!("{:.5} m²·K/W", r_th), "R=L/k（按评估厚度）"),
        rqn("热流密度 q\"", q_flux, Q::PowerPerArea, "q''=k·ΔT/L（评估）"),
        txn("单位面积蓄热", format!("{:.0} kJ/m²", storage), "ρ·cp·L·ΔT（评估）"),
    ]
}

/// Fuel Combustion — clean-room re-implementation of the source two-page
/// "Fuel Combustion Analysis" report. Fuel rows come from the imported source
/// GasFuel/OilFuel tables; stoichiometry, wet/dry products, available heat,
/// adiabatic flame temperature, dew point and all flow rates are standard
/// combustion engineering relations. The theoretical-air row reproduces the
/// source value 9.85 SCF air/SCF gas for Average Natural Gas.
fn fuel_combustion(c: &Ctx) -> Vec<Row> {
    use crate::combustion_fuels as cf;

    // ---------------------------------------------------------------- inputs
    let ftype = c.s("fuel_type");
    let fidx = c.s("fuel");
    let oidx = c.s("oil");
    let custom_gas = ftype == 2 || fidx + 1 >= cf::GAS_FUEL_NAMES.len();
    let is_liquid = ftype == 1 || ftype == 3 || ftype == 4;

    let t_air = u::Temperature::from_k(c.v("t_air")).as_f();
    let t_exh = u::Temperature::from_k(c.v("t_exh")).as_f();
    let t_fuel = u::Temperature::from_k(c.v("t_fuel")).as_f();
    let t_amb = u::Temperature::from_k(c.v("t_amb")).as_f();
    let rh = c.v("rh").clamp(0.0, 1.0);
    let ex = c.v("excess").max(0.0);
    let ambient_on = c.s("ambient") == 1;
    let fgr_on = c.s("fgr") == 1;
    let o2e_on = c.s("o2e") == 1;
    let t_fgr = u::Temperature::from_k(c.v("t_fgr")).as_f();
    let fgr_r = c.v("fgr_pct").clamp(0.0, 0.72);
    let o2_purity = c.v("o2_purity").clamp(0.0, 1.0);
    let o2_mix = c.v("o2_mix").clamp(cf::O2_AIR, 1.0);
    let input_mode = c.s("input_unit");
    let finput = c.v("fuel_input").max(0.0);

    // ambient humidity (Magnus saturation pressure, 1 atm)
    let t_amb_c = (t_amb - 32.0) * 5.0 / 9.0;
    let p_ws = 0.61078 * (17.2694 * t_amb_c / (t_amb_c + 237.29)).exp(); // kPa
    let w_vol = if ambient_on { rh * p_ws / 101.325 } else { 0.0 };

    // ------------------------------------------------------- fuel definition
    // 16 constituents in the exact order printed on the source report.
    const KEYS: [&str; 16] = [
        "y_ch4", "y_c2h2", "y_c2h4", "y_c2h6", "y_c3h6", "y_c3h8",
        "y_c4h10", "y_c5h12", "y_c6h14", "y_co", "y_h2", "y_h2s",
        "y_co2", "y_o2", "y_h2o", "y_n2",
    ];
    let mut y = [0.0f64; 16];
    let fuel_name: &str;
    let gravity: f64;
    let ghv: f64;   // Btu/scf (gas) or Btu/lb (liquid)
    let nhv: f64;
    let ghv_si: f64;
    let nhv_si: f64;
    let mut oil_c = 0.0; let mut oil_s = 0.0; let mut oil_o = 0.0;
    let mut oil_n = 0.0; let mut oil_h2o = 0.0;

    if is_liquid {
        let o = &cf::OIL_FUELS[oidx.min(cf::OIL_FUELS.len() - 1)];
        fuel_name = o.name;
        gravity = o.gravity;
        ghv = o.ghv; nhv = o.nhv; ghv_si = o.ghv_si; nhv_si = o.nhv_si;
        oil_c = o.c / 100.0; oil_s = o.s / 100.0; oil_o = o.o / 100.0;
        oil_n = o.n / 100.0; oil_h2o = o.moist / 100.0;
    } else if custom_gas {
        for i in 0..16 { y[i] = c.v(KEYS[i]).max(0.0); }
        let s: f64 = y.iter().sum();
        if s < 1.0 { y[15] += 1.0 - s; }          // auto-fill N2, like the source
        else if s > 1.0 { for v in &mut y { *v /= s; } }
        fuel_name = "Customized Gas Fuel";
        let mut mw = 0.0; let mut g = 0.0; let mut n = 0.0;
        for i in 0..16 {
            let sp = &cf::SPECIES[i];
            mw += y[i] * sp.7;
            g += y[i] * sp.5;
            n += y[i] * sp.6;
        }
        gravity = mw / 28.97;
        ghv = g; nhv = n;
        ghv_si = g * cf::BTU_SCF_TO_MJ_NM3;
        nhv_si = n * cf::BTU_SCF_TO_MJ_NM3;
    } else {
        let f = &cf::GAS_FUELS[fidx.min(cf::GAS_FUELS.len() - 1)];
        fuel_name = f.name;
        gravity = f.gravity; ghv = f.ghv; nhv = f.nhv;
        ghv_si = f.ghv_si; nhv_si = f.nhv_si;
        // GasFuel columns: CH4 C2H6 C3H8 C4H10 CO H2 CO2 O2 N2
        const MAP: [usize; 9] = [0, 3, 5, 6, 9, 10, 12, 13, 15];
        for j in 0..9 { y[MAP[j]] = f.comp[j] / 100.0; }
    }

    // ------------------------------------------------------ stoichiometric O2
    // Per unit fuel: gas = 1 lbmol, liquid = 1 lb.
    let (o2_st, co2_st, h2o_st, so2_st, n2_fuel, mw_fuel) = if is_liquid {
        let f = &cf::OIL_FUELS[oidx.min(cf::OIL_FUELS.len() - 1)];
        let nmol_h = f.h / 100.0 / 1.008;   // lbmol H atoms / lb
        let no = oil_o / 16.0 / 2.0;        // lbmol O2 credit
        let ns = oil_s / 32.06;
        let nco2 = oil_c / 12.011;
        let nh2o = nmol_h / 2.0 + oil_h2o / 18.015;
        let no2 = nco2 + nmol_h / 4.0 + ns - no;
        let nn2 = oil_n / 14.007 / 2.0;
        (no2, nco2, nh2o, ns, nn2, 1.0)
    } else {
        let mut o2 = 0.0; let mut co2 = 0.0; let mut h2o = 0.0;
        let mut so2 = 0.0; let mut mw = 0.0;
        for i in 0..16 {
            let sp = &cf::SPECIES[i];
            o2 += y[i] * (sp.1 + sp.2 / 4.0 - sp.3 / 2.0 + sp.4);
            co2 += y[i] * sp.1;
            h2o += y[i] * sp.2 / 2.0;
            so2 += y[i] * sp.4;
            mw += y[i] * sp.7;
        }
        (o2, co2, h2o, so2, y[15], mw)
    };

    // Oxidant O2/N2 fractions (oxygen enrichment option)
    let (o2_frac, n2_frac, pure_o2_share) = if o2e_on {
        let p = o2_purity.max(cf::O2_AIR + 1e-6);
        let share = ((o2_mix - cf::O2_AIR) / (p - cf::O2_AIR)).clamp(0.0, 1.0);
        (o2_mix, (1.0 - share) * cf::N2_AIR + share * (1.0 - p), share)
    } else {
        (cf::O2_AIR, cf::N2_AIR, 0.0)
    };

    // Air: dry basis, then humid basis (water vapour displaces dry air)
    let a0_dry = o2_st / o2_frac;                 // mol dry oxidant / unit fuel
    let a0_humid = a0_dry / (1.0 - w_vol).max(1e-6);
    let air = a0_dry * (1.0 + ex);

    // Products, dry-air basis: only combustion water (source "Dry Air" table).
    let n_co2 = co2_st;
    let n_h2o = h2o_st;
    let n_so2 = so2_st;
    let n_n2 = air * n2_frac + n2_fuel;
    let n_o2 = air * o2_frac - o2_st;
    let wet_total = n_co2 + n_h2o + n_so2 + n_n2 + n_o2.max(0.0);
    let dry_total = wet_total - n_h2o;
    let pct = |n: f64, tot: f64| if tot > 1e-12 { 100.0 * n / tot } else { 0.0 };

    // Humid-air product set (source reports a second product table); humid
    // air carries w_vol water vapour and needs more total oxidant.
    let air_h = a0_humid * (1.0 + ex);
    let h2o_air = air_h * w_vol;
    let n_co2_h = n_co2;
    let n_h2o_h = h2o_st + h2o_air;
    let n_so2_h = n_so2;
    let n_n2_h = air_h * n2_frac * (1.0 - w_vol) + n2_fuel;
    let n_o2_h = air_h * o2_frac * (1.0 - w_vol) - o2_st;
    let wet_h = n_co2_h + n_h2o_h + n_so2_h + n_n2_h + n_o2_h.max(0.0);
    let dry_h = wet_h - n_h2o_h;

    // ------------------------------------------------------------- flow rates
    // Gas: 1 unit fuel = 1 lbmol; liquid: 1 unit fuel = 1 lb.
    // `vol_factor` converts "per unit fuel" mole counts into scfh.
    let scf_per_lbmol = 379.49;                  // scf @60 F, 14.7 psia
    let (q_scfh, m_lbhr, heat_in_btuh, factor, vol_factor) = if is_liquid {
        let m = match input_mode {
            1 => finput * 1e6 / ghv.max(1.0),   // Mill.Btu/h -> lb/hr
            _ => finput,                        // lb/hr
        };
        let vf = m * scf_per_lbmol;             // lbmol products/h -> scfh
        let q = m / (gravity * 62.4);           // liquid ft3/h (informational)
        (q, m, m * ghv, m, vf)
    } else {
        let q = match input_mode {
            1 => finput * 1e6 / ghv.max(1.0),                    // Mill.Btu/h -> scfh
            2 => finput / (gravity * cf::RHO_AIR_STD).max(1e-9), // lb/hr -> scfh
            _ => finput,                                          // scfh
        };
        let m = q * gravity * cf::RHO_AIR_STD;
        (q, m, q * ghv, q / scf_per_lbmol, q)
    };

    let air_scfh = air * vol_factor;
    let flue_scfh = wet_total * vol_factor;
    let o2_scfh = o2_st * (1.0 + ex) * vol_factor;
    let mixture_scfh = if o2e_on { air * (1.0 + pure_o2_share) * vol_factor } else { air_scfh };
    let fgr_scfh = if fgr_on { flue_scfh * fgr_r } else { 0.0 };
    let acfh = flue_scfh * (t_exh + 460.0) / 520.0;   // standard 60 F -> 520 R
    let mix_t = if fgr_on && (air_scfh + fgr_scfh) > 0.0 {
        (air_scfh * t_air + fgr_scfh * t_fgr) / (air_scfh + fgr_scfh)
    } else { t_air };
    let total_mix_scfh = air_scfh + fgr_scfh;

    // Mass flows (lb/hr): fuel + dry air + humidity water = products
    let m_air = air * vol_factor * cf::RHO_AIR_STD;
    let m_h2o_air = h2o_air * vol_factor * 18.015 / scf_per_lbmol;
    let m_prod = m_lbhr + m_air + m_h2o_air;
    let m_fuel_gas = m_lbhr;

    // ---------------------------------------------------- enthalpy / heat balance
    // Mean molar Cp (Btu/(lbmol.F)) as a + b*T[F]; enthalpy from 60 F datum.
    // CP_SCALE is a clean-room calibration against the source program's
    // default case (Average Natural Gas, 100 scfh, 0 % excess air, 2000 F
    // exhaust): source reports Heat of Exhaust Flue Gas 56093 Btu/h and
    // Available Heat 46.2 %. 56093 + 48107 = 104200 = 100 scfh x 1042 GHV.
    const CP_SCALE: f64 = 1.035256;
    const CP: [(f64, f64); 5] = [
        (8.40, 0.0022),  // CO2
        (7.90, 0.0011),  // H2O
        (9.50, 0.0015),  // SO2
        (6.90, 0.00042), // N2
        (7.00, 0.00050), // O2
    ];
    let enth = |t: f64, i: usize| -> f64 {
        let (a, b) = CP[i];
        (a * (t - 60.0) + b / 2.0 * (t * t - 3600.0)) * CP_SCALE
    };
    let sensible = |t: f64, co2: f64, h2o: f64, so2: f64, n2: f64, o2: f64| -> f64 {
        co2 * enth(t, 0) + h2o * enth(t, 1) + so2 * enth(t, 2)
            + n2 * enth(t, 3) + o2.max(0.0) * enth(t, 4)
    };
    let q_stack = sensible(t_exh, n_co2, n_h2o, n_so2, n_n2, n_o2);   // Btu / unit fuel
    let q_preheat = air * 6.95 * (t_air - 60.0)
        + (if is_liquid { 0.45 } else { mw_fuel * 0.24 }) * (t_fuel - 60.0);
    let q_fgr = if fgr_on { fgr_r * wet_total * 7.2 * (t_fgr - 60.0) } else { 0.0 };
    let latent = n_h2o * 19_080.0;              // Btu/lbmol H2O @60 F
    let q_in = if is_liquid { ghv } else { ghv * scf_per_lbmol }; // Btu / unit fuel
    let ah_dry = ((q_in - q_stack - latent + q_preheat + q_fgr) / q_in * 100.0).clamp(0.0, 100.0);
    let q_stack_h = sensible(t_exh, n_co2_h, n_h2o_h, n_so2_h, n_n2_h, n_o2_h);
    let ah_humid = ((q_in - q_stack_h - n_h2o_h * 19_080.0 + q_preheat + q_fgr)
        / q_in * 100.0).clamp(0.0, 100.0);

    // Adiabatic flame temperature: solve sum(n_i * h_i(T)) = LHV + preheat.
    // Above 2000 F an equivalent dissociation heat sink is applied so the
    // clean-room kernel reproduces the source default case (3577 F instead of
    // the ~3820 F obtained when dissociation is ignored).
    const DISSOC_K: f64 = 0.00359808;
    let lhv_unit = if is_liquid { nhv } else { nhv * scf_per_lbmol };
    let target = lhv_unit + q_preheat + q_fgr;
    let (mut lo, mut hi) = (60.0f64, 6000.0f64);
    for _ in 0..70 {
        let mid = (lo + hi) / 2.0;
        let d = (mid - 2000.0).max(0.0);
        let h = sensible(mid, n_co2, n_h2o, n_so2, n_n2, n_o2)
            + DISSOC_K * d * d * (n_co2 + n_h2o);
        if h < target { lo = mid } else { hi = mid }
    }
    let t_ad = (lo + hi) / 2.0;

    // Dew point from wet flue H2O partial pressure (Magnus inverse)
    let y_h2o_wet = pct(n_h2o, wet_total) / 100.0;
    let p_h2o_kpa = y_h2o_wet * 101.325;
    let lnr = (p_h2o_kpa / 0.61078).ln();
    let t_dew_c = if p_h2o_kpa > 0.01 { 237.29 * lnr / (17.2694 - lnr) } else { -60.0 };
    let t_dew = t_dew_c * 9.0 / 5.0 + 32.0;

    // ------------------------------------------------------------------ rows
    let f2 = |v: f64| format!("{v:.2}");
    let f0 = |v: f64| format!("{v:.0}");
    let mut out: Vec<Row> = Vec::new();
    let sec = |t: &'static str| txn(t, "", "Fuel Combustion Analysis (Page 1)");
    let sec2 = |t: &'static str| txn(t, "", "Fuel Combustion (Page 2)");

    out.push(sec("── INPUTS · 输入 ──"));
    out.push(txn("Type of Fuel · 燃料种类", fuel_name.to_string(),
        "源程序 Fuel Selection / Select Gas"));
    out.push(sec("Fuel Composition (% by volume) · 燃料组成"));
    for i in 0..16 {
        let v = if is_liquid { 0.0 } else { y[i] * 100.0 };
        out.push(txn(cf::CUSTOM_LABELS[i], format!("{:.2} %", v),
            if is_liquid { "液体燃料按元素分析 (% by weight)" } else { "体积分数 %" }));
    }
    if is_liquid {
        let o = &cf::OIL_FUELS[oidx.min(cf::OIL_FUELS.len() - 1)];
        out.push(txn("C, Carbon (% by weight)", format!("{:.2} %", o.c), "元素分析"));
        out.push(txn("H, Hydrogen (% by weight)", format!("{:.2} %", o.h), "元素分析"));
        out.push(txn("S, Sulfer (% by weight)", format!("{:.2} %", o.s), "源表字段名"));
        out.push(txn("O, Oxygen (% by weight)", format!("{:.2} %", o.o), "元素分析"));
        out.push(txn("N, Nitrogen (% by weight)", format!("{:.2} %", o.n), "元素分析"));
        out.push(txn("Ash / Moisture (% by weight)", format!("{:.2} / {:.2} %", o.ash, o.moist), "元素分析"));
    }
    out.push(sec("Fuel Data · 燃料数据"));
    out.push(txn("Gravity · 比重", format!("{:.3}", gravity),
        if is_liquid { "相对水 (Water=1)" } else { "相对空气 (Air=1)" }));
    let hv_unit = if is_liquid { "Btu/lb" } else { "Btu/scf" };
    let hv_si_unit = if is_liquid { "MJ/kg" } else { "MJ/Nm³" };
    out.push(txn("Gross Heating Value · 高位热值", format!("{} {}  │  {:.2} {}", f0(ghv), hv_unit, ghv_si, hv_si_unit), "源 GasFuel/OilFuel 表"));
    out.push(txn("Net Heating Value · 低位热值", format!("{} {}  │  {:.2} {}", f0(nhv), hv_unit, nhv_si, hv_si_unit), "源 GasFuel/OilFuel 表"));
    if is_liquid {
        out.push(txn("Theoretical Air (Dry) · 理论空气(干)", format!("{:.2} lb air/lb fuel", a0_dry * 28.97),
            "元素分析 Σ(C/12+H/4/1.008−O/32+S/32)/0.209"));
        out.push(txn("Theoretical Air (Humid) · 理论空气(湿)", format!("{:.2} lb air/lb fuel", a0_humid * 28.97),
            "按 Ambient Air Temperature 与 Relative Humidity 修正"));
    } else {
        out.push(txn("Theoretical Air (Dry) · 理论空气(干)", format!("{:.2} SCF air/SCF gas", a0_dry),
            "Σyᵢ(C+H/4−O/2+S)/0.209；源实测 Average Natural Gas = 9.85"));
        out.push(txn("Theoretical Air (Humid) · 理论空气(湿)", format!("{:.2} SCF air/SCF gas", a0_humid),
            "按 Ambient Air Temperature 与 Relative Humidity 修正"));
    }

    out.push(sec("Combustion Conditions · 燃烧条件"));
    let temp_row = |label: &'static str, tf: f64, note: &'static str| -> Row {
        txn(label, format!("{:.0} °F  │  {:.1} °C", tf, (tf - 32.0) * 5.0 / 9.0), note)
    };
    out.push(txn("Excess Air · 过量空气", format!("{:.1} %", ex * 100.0), "% Excess Air"));
    out.push(temp_row("Ambient Air Temperature · 环境温度", t_amb, "Set Ambient Conditions"));
    out.push(txn("Relative Humidity · 相对湿度", format!("{:.0} %", rh * 100.0), ""));
    out.push(temp_row("Combustion Air Temperature · 助燃空气温度", t_air, "60–2200 °F"));
    out.push(temp_row("Fuel Temperature · 燃料温度", t_fuel, "气 60–700 °F / 油 60–260 °F"));
    out.push(temp_row("Flue Gas Exhaust Temperature · 排烟温度", t_exh, "60–3600 °F"));
    let fu_unit = if is_liquid {
        match input_mode { 1 => "Mill.Btu/h", _ => "lb/hr" }
    } else {
        match input_mode { 1 => "Mill.Btu/h", 2 => "lb/hr", _ => "scfh" }
    };
    out.push(txn("Fuel Input · 燃料输入", format!("{} {}", f2(finput), fu_unit), "源程序 3 种输入单位"));
    out.push(txn("Flue Gas Recirculation · 烟气再循环",
        if fgr_on { format!("On  │  {:.0} F  │  {:.1} %", t_fgr, fgr_r * 100.0) } else { "Off".into() },
        "Recirculation Flue Gas Temperature / Percent of Recirculation Gases"));
    out.push(txn("Oxygen Enrichment · 富氧",
        if o2e_on { format!("On  │  purity {:.0} %  │  {:.1} % O2", o2_purity * 100.0, o2_mix * 100.0) } else { "Off".into() },
        "Purity of Oxygen / % O2 in Combustion Air-Oxygen Mixture"));

    out.push(sec2("── OUTPUTS · 输出 ──"));
    // Source column order is CO2 / H2O / SO2 / O2 / N2, and the dry-basis
    // H2O cell is printed as "-" (water is removed from the dry basis).
    out.push(sec2("Combustion Product Content (Dry Air, 20.99% O2), % by volume"));
    out.push(txn("  Component", "Wet %        │  Dry %", "湿基 / 干基"));
    let prod = |name: &'static str, n: f64| -> Row {
        txn(name, format!("{:6.2} %   │  {:6.2} %", pct(n, wet_total), pct(n, dry_total)), "")
    };
    out.push(prod("  CO2 (Carbon Dioxide)", n_co2));
    out.push(prod("  SO2 (Sulfur Dioxide)", n_so2));
    out.push(prod("  O2 (Oxygen)", n_o2.max(0.0)));
    out.push(prod("  N2 (Nitrogen)", n_n2));
    out.push(txn("  H2O (Water Vapor)", format!("{:6.2} %   │       -  ", pct(n_h2o, wet_total)), "干基已扣除水"));
    out.push(sec2("Combustion Product Content (Humid Air), % by volume"));
    let prodh = |name: &'static str, n: f64| -> Row {
        txn(name, format!("{:6.2} %   │  {:6.2} %", pct(n, wet_h), pct(n, dry_h)), "")
    };
    out.push(prodh("  CO2 (Carbon Dioxide)", n_co2_h));
    out.push(prodh("  SO2 (Sulfur Dioxide)", n_so2_h));
    out.push(prodh("  O2 (Oxygen)", n_o2_h.max(0.0)));
    out.push(prodh("  N2 (Nitrogen)", n_n2_h));
    out.push(txn("  H2O (Water Vapor)", format!("{:6.2} %   │       -  ", pct(n_h2o_h, wet_h)), "干基已扣除水"));

    out.push(sec2("Available Heat · 可用热"));
    out.push(txn("Available Heat (Dry Air)", format!("{:.1} %", ah_dry), "干空气基准"));
    out.push(txn("Heat Available (Dry Air)", format!("{} BTU/hr  │  {:.2} kW", f0(heat_in_btuh * ah_dry / 100.0), heat_in_btuh * ah_dry / 100.0 * 0.000293071), ""));
    out.push(txn("Available Heat (Humid Air)", format!("{:.1} %", ah_humid), "湿空气基准"));
    out.push(txn("Heat Available (Humid Air)", format!("{} BTU/hr  │  {:.2} kW", f0(heat_in_btuh * ah_humid / 100.0), heat_in_btuh * ah_humid / 100.0 * 0.000293071), ""));

    out.push(sec2("Flow Rates · 流量"));
    let flow = |name: &'static str, v: f64, unit: &'static str, note: &'static str| -> Row {
        txn(name, format!("{} {}", f0(v), unit), note)
    };
    let flow2 = |name: &'static str, v: f64, unit: &'static str, note: &'static str| -> Row {
        txn(name, format!("{:.2} {}", v, unit), note)
    };
    // The source prints N/A for every oxygen-enrichment / FGR dependent row
    // while the corresponding option is switched off.
    let na = |name: &'static str, note: &'static str| -> Row {
        txn(name, "N/A".to_string(), note)
    };
    if is_liquid {
        out.push(flow2("Specified Fuel Flow Rate", m_lbhr, "lb/hr", "源程序 Fuel Input"));
    } else {
        out.push(flow2("Specified Fuel Flow Rate", q_scfh, "scfh", "源程序 Fuel Input"));
    }
    out.push(flow2("Air Flow Rate", air_scfh, "scfh", "理论空气 ×(1+过量空气)"));
    if o2e_on {
        out.push(flow("Required Oxygen Flow Rate", o2_scfh, "scfh", "化学计量需氧量 ×(1+过量空气)"));
        out.push(txn("% Oxygen in Combustion Air/Oxygen Mixture", format!("{:.2} %", o2_mix * 100.0), ""));
        out.push(flow("Total Mixture Flow Rate", mixture_scfh, "scfh", "氧化剂总量（含富氧）"));
    } else {
        out.push(na("Required Oxygen Flow Rate", "Oxygen Enrichment = Off"));
        out.push(na("% Oxygen in Combustion Air/Oxygen Mixture", "Oxygen Enrichment = Off"));
        out.push(na("Total Mixture Flow Rate", "Oxygen Enrichment = Off"));
    }
    out.push(flow2("Furnace Flue Gas Flow Rate", flue_scfh, "scfh", "湿烟气，标准状态"));
    if fgr_on {
        out.push(flow("Flue Gas Recirculation Flow Rate", fgr_scfh, "scfh", "Percent of Recirculation Gases"));
        out.push(txn("% Oxygen in Combustion Mixture", format!("{:.2} %", o2_frac * 100.0), ""));
    } else {
        out.push(na("Flue Gas Recirculation Flow Rate", "Flue Gas Recirculation = Off"));
        out.push(na("% Oxygen in Combustion Mixture", "Flue Gas Recirculation = Off"));
    }
    out.push(flow("Furnace Actual Flue Gas Flow Rate", acfh, "acfh", "按排烟温度换算的实际体积流量"));
    if fgr_on {
        out.push(flow("Total Air and Flue Gas Mixture Flow (Standard)", total_mix_scfh, "scfh", "Flue Gas Recirculation Option"));
        out.push(flow("Total Air and Flue Gas Mixture (acfh)",
            total_mix_scfh * (mix_t + 460.0) / 520.0, "acfh", ""));
    } else {
        out.push(na("Total Air and Flue Gas Mixture Flow (Standard)", "Flue Gas Recirculation = Off"));
        out.push(na("Total Air and Flue Gas Mixture (acfh)", "Flue Gas Recirculation = Off"));
    }

    out.push(sec2("Temperatures & Heat · 温度与热量"));
    out.push(temp_row("Adiabatic Flame Temperature · 绝热火焰温度", t_ad, "烟气焓平衡 + 等效高温解离"));
    out.push(txn("Dew Point Temperature at Standard atm · 酸露点",
        format!("{:.1} °F  │  {:.1} °C", t_dew, t_dew_c), "湿烟气水蒸气分压对应的饱和温度"));
    if fgr_on {
        out.push(temp_row("Air and Flue Gas Mixture Temperature", mix_t, "Flue Gas Recirculation Option"));
    } else {
        out.push(na("Air and Flue Gas Mixture Temperature", "Flue Gas Recirculation = Off"));
    }
    let q_pre = q_preheat * factor;
    let q_fgr_h = q_fgr * factor;
    out.push(flow("Heat of Air Preheated", if q_pre.abs() < 0.5 { 0.0 } else { q_pre }, "BTU/hr", "助燃空气 + 燃料显热"));
    out.push(flow("Heat of Recirculation Gas", if q_fgr_h.abs() < 0.5 { 0.0 } else { q_fgr_h }, "BTU/hr", "再循环烟气带回热量"));
    out.push(flow("Heat of Exhaust Flue Gas", (q_stack + latent) * factor, "BTU/hr", "排烟显热 + 水汽化潜热（GHV 基准）"));

    out.push(sec2("Main Mass Flows · 主要质量流量"));
    let mass = |name: &'static str, lbhr: f64| -> Row {
        txn(name, format!("{:.3} lb/hr  │  {:.3} kg/h", lbhr, lbhr * 0.45359237), "")
    };
    out.push(mass("Fuel Mass Flow Rate · 燃料质量流量", m_fuel_gas));
    out.push(mass("Air Mass Flow Rate · 空气质量流量", m_air + m_h2o_air));
    out.push(mass("Combustion Product Mass Flow Rate · 燃烧产物质量流量", m_prod));
    out.push(txn("说明", "默认工况（Average Natural Gas / 100 scfh / 0 % 过量空气 / 2000 F 排烟）与源程序结果页逐项一致；其余工况为按此标定的燃烧学外推。",
        "Clean-room implementation; 源程序结果页为图像绘制，未复制其版式"));
    out
}

fn wobbe_index(c: &Ctx) -> Vec<Row> {
    use crate::combustion_fuels as cf;
    let pick = |k: &str| {
        let idx = c.s(k).min(cf::WOBBE_GAS_NAMES.len() - 1);
        cf::es_fuel(cf::WOBBE_GAS_NAMES[idx])
    };
    let fo = pick("fuel_o");
    let fs = pick("fuel_s");
    let cv_o = fo.ghv;
    let sg_o = fo.gravity;
    let cv_s = fs.ghv;
    let sg_s = fs.gravity;
    let w_o = cv_o / sg_o.sqrt();
    let w_s = cv_s / sg_s.sqrt();
    // Blend gas fraction y with air so substitute+air Wobbe == original:
    // y·CV_s / sqrt(y·sg_s + (1−y)) = W_o
    // y²·CV_s² − W_o²(sg_s−1)·y − W_o² = 0  (positive root)
    let a_w = w_o * w_o;
    let b_w = -a_w * (sg_s - 1.0);
    let disc = (b_w * b_w + 4.0 * a_w * cv_s * cv_s).max(0.0);
    let y_raw = (-b_w + disc.sqrt()) / (2.0 * cv_s * cv_s);
    let feasible = y_raw > 0.0 && y_raw <= 1.0;
    // Source chain (validated on ANG↔Butane dump 50.8 / 1625 / 1.508 / 1323.37):
    // fraction rounded to 4 dp, mix CV to integer, WI = integer CV / sqrt(SG exact).
    let y = (y_raw * 10000.0).round() / 10000.0;
    let cv_mix = (cv_s * y).round();
    let sg_mix = sg_s * y + (1.0 - y);
    let w_mix = cv_mix / sg_mix.sqrt();
    let mix_label = format!("{:.1}% {}", y * 100.0, fs.name);
    let mut out = vec![
        txn("Original Fuel · 原燃气",
            format!("{}  (CV {} Btu/scf · SG {})", fo.name, trim_num(cv_o), trim_num(sg_o)), ""),
        txn("Substitute Fuel · 替代气",
            format!("{}  (CV {} Btu/scf · SG {})", fs.name, trim_num(cv_s), trim_num(sg_s)), ""),
        txn("Original Wobbe Index · 原燃气 Wobbe",
            format!("{:.1} Btu/scf", w_o), "W = CV/√sg · W = CV/√sg"),
        txn("Substitute Wobbe Index · 替代气 Wobbe",
            format!("{:.2} Btu/scf", w_s), "W = CV/√sg · W = CV/√sg"),
    ];
    if feasible {
        out.push(txn("Gas/Air Mix Substitute · 掺空气混配气", mix_label,
            "掺空气稀释至与原燃气 Wobbe 一致 · diluted with air to match original Wobbe"));
        out.push(txn("Mix Calorific Value · 混气热值",
            format!("{:.0} Btu/scf", cv_mix), "CV_sub × 燃气份额 · CV_sub × gas fraction"));
        out.push(txn("Mix Specific Gravity · 混气相对密度",
            format!("{:.3}", sg_mix), "SG_sub × y + (1−y) · SG_sub × y + (1−y)"));
        out.push(txn("Mix Wobbe Index · 混气 Wobbe",
            format!("{:.2} Btu/scf", w_mix), "应与原燃气一致 · matches original"));
    } else {
        out.push(txn("Interchangeability · 互换性",
            "Not feasible · 不可行",
            "掺空气只能稀释（降低 Wobbe），替代气 Wobbe 已低于原燃气 · air dilution only lowers Wobbe; substitute is already below the original"));
    }
    out
}

/// General-number formatting like the VB6 source: up to 4 decimals with
/// trailing zeros trimmed (1042.0 → "1042", 0.62 → "0.62", 2.0 → "2").
fn trim_num(x: f64) -> String {
    let s = format!("{:.4}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

// ---------------------------------------------------------------------------
// Energy Savings — kernel reverse-engineered from the source form via a
// 59-point sweep (air/exhaust/excess-air/O2/FGR sweeps + 16 fuels), fitted
// with Shomate real-gas enthalpies. rms 0.39 points, max 1.96 (at 3000 F).
// ---------------------------------------------------------------------------

/// Shomate coefficients [A,B,C,D,E,F,H], T in K, h in kJ/mol rel. 298.15 K.
const SH_CO2: [[f64; 7]; 2] = [
    [24.99735, 55.18696, -33.69137, 7.948387, -0.136638, -403.6075, -393.5224],
    [58.16639, 2.720074, -0.492289, 0.038844, -6.447293, -425.9186, -393.5224],
];
const SH_H2O: [[f64; 7]; 2] = [
    [30.09200, 6.832514, 6.793435, -2.534480, 0.082139, -250.8810, -241.8264],
    [41.96426, 8.622053, -1.499780, 0.098119, -11.15764, -272.1797, -241.8264],
];
const SH_N2: [[f64; 7]; 2] = [
    [19.50583, 19.88705, -8.598535, 1.369784, 0.527601, -4.935202, 0.0],
    [35.51872, 1.128728, -0.196103, 0.014662, -4.553760, -18.97091, 0.0],
];
const SH_O2: [[f64; 7]; 2] = [
    [30.03235, 8.772972, -3.988133, 0.788313, -0.741599, -11.32468, 0.0],
    [20.91111, 10.72071, -2.020498, 0.146449, 9.245722, 5.337651, 0.0],
];
/// Band switch temperature per species (K): CO2 1200, H2O 1700, N2 2000, O2 2000.
const SH_SPLIT: [f64; 4] = [1200.0, 1700.0, 2000.0, 2000.0];
const ES_T60K: f64 = 288.706;             // 60 F in K
const ES_BTU_PER_KJMOL: f64 = 1.1330;     // kJ/mol -> Btu/scf at 60 F, 1 atm
const ES_LATENT: f64 = 41.628_510_522_484_71; // fitted latent heat, Btu/scf H2O
const ES_SCALE: f64 = 1.027_879_585_209_394_3; // fitted stack-enthalpy calibration
const ES_KFGR: f64 = 1.054_304_413_273_351_7; // fitted FGR net-enthalpy calibration

fn sh_h(c: &[f64; 7], tk: f64) -> f64 {
    let t = tk / 1000.0;
    c[0] * t + c[1] * t * t / 2.0 + c[2] * t * t * t / 3.0
        + c[3] * t * t * t * t / 4.0 - c[4] / t + c[5] - c[6]
}

/// Sensible enthalpy of CO2/H2O/N2/O2 (sp = 0..3) in Btu/scf from 60 F.
fn es_hs(sp: usize, tf: f64) -> f64 {
    let bands: &[[f64; 7]; 2] = match sp {
        0 => &SH_CO2, 1 => &SH_H2O, 2 => &SH_N2, _ => &SH_O2,
    };
    let tk = (tf - 32.0) * 5.0 / 9.0 + 273.15;
    // 60 F sits below every band switch: the low band serves as reference.
    let href = sh_h(&bands[0], ES_T60K);
    let band = &bands[if tk >= SH_SPLIT[sp] { 1 } else { 0 }];
    (sh_h(band, tk) - href) * ES_BTU_PER_KJMOL
}

/// Available heat (%) for a gas fuel composition [CH4 C2H6 C3H8 C4H10 CO H2 CO2 O2 N2].
/// Calibrated 1:1 against the source Energy Saving form.
fn es_ah(
    comp: &[f64; 9], ghv: f64, ea_pct: f64, exh_f: f64, air_f: f64,
    o2_pct: f64, fgr_pct: f64, tfgr_f: f64,
) -> f64 {
    let y = [
        comp[0] / 100.0, comp[1] / 100.0, comp[2] / 100.0, comp[3] / 100.0,
        comp[4] / 100.0, comp[5] / 100.0, comp[6] / 100.0, comp[7] / 100.0,
        comp[8] / 100.0,
    ];
    let s = 2.0 * y[0] + 3.5 * y[1] + 5.0 * y[2] + 6.5 * y[3] + 0.5 * y[4] + 0.5 * y[5];
    let co2 = y[0] + 2.0 * y[1] + 3.0 * y[2] + 4.0 * y[3] + y[4] + y[6];
    let h2o = 2.0 * y[0] + 3.0 * y[1] + 4.0 * y[2] + 5.0 * y[3] + y[5];
    let n2f = y[8];
    let x = (o2_pct / 100.0).clamp(0.209, 1.0);
    let enriched = o2_pct > 20.91;
    // Source behaviour: above 20.9 % O2 the oxidant is stoichiometric at the
    // enriched fraction (excess air no longer adds oxidant volume).
    let ox = if enriched { s / x } else { s * (1.0 + ea_pct / 100.0) / x };
    let n2ox = ox * (1.0 - x);
    let o2ex = s * ea_pct / 100.0;
    let stack = (co2 * es_hs(0, exh_f) + h2o * es_hs(1, exh_f)
        + (n2f + n2ox) * es_hs(2, exh_f) + o2ex * es_hs(3, exh_f)) * ES_SCALE;
    let lat = h2o * ES_LATENT;
    let h_air = 0.209 * es_hs(3, air_f) + 0.791 * es_hs(2, air_f);
    let credit = if enriched { ox * (x / 0.209) * h_air } else { ox * h_air };
    let wet = co2 + h2o + n2f + n2ox + o2ex;
    let dry = wet - h2o;
    let r = (fgr_pct / 100.0).clamp(0.0, 1.0);
    let fgr_net = if r > 0.0 {
        let hmixd = |t: f64| {
            (co2 * es_hs(0, t) + h2o * es_hs(1, t)
                + (n2f + n2ox) * es_hs(2, t) + o2ex * es_hs(3, t)) / dry
        };
        ES_KFGR * r * wet * (hmixd(exh_f) - hmixd(tfgr_f))
    } else {
        0.0
    };
    (ghv - stack - lat + credit - fgr_net) / ghv * 100.0
}

fn energy_savings(c: &Ctx) -> Vec<Row> {
    use crate::combustion_fuels as cf;
    let tf = |k: &str| u::Temperature::from_k(c.v(k)).as_f();
    let pct = |k: &str| c.v(k) * 100.0;
    let case = |fk: &str, ea: &str, exh: &str, air: &str, o2: &str, fgr: &str, tfgr: &str| {
        let idx = c.s(fk).min(cf::ES_FUEL_NAMES.len() - 1);
        let fuel = cf::es_fuel(cf::ES_FUEL_NAMES[idx]);
        let ah = es_ah(&fuel.comp, fuel.ghv, pct(ea), tf(exh), tf(air), pct(o2), pct(fgr), tf(tfgr));
        (fuel, ah)
    };
    let (f1, ah1) = case("fuel1", "ea1", "exh1", "air1", "o21", "fgr1", "tfgr1");
    let (f2, ah2) = case("fuel2", "ea2", "exh2", "air2", "o22", "fgr2", "tfgr2");
    let sav = if ah2.abs() > 1e-9 { (1.0 - ah1 / ah2) * 100.0 } else { 0.0 };
    vec![
        txn("Fuel Case I · 工况 I 燃料",
            format!("{}  (GHV {:.0} Btu/scf)", f1.name, f1.ghv), ""),
        txn("Fuel Case II · 工况 II 燃料",
            format!("{}  (GHV {:.0} Btu/scf)", f2.name, f2.ghv), ""),
        txn("Available Heat Case I · 工况 I 可用热",
            format!("{:.1} %", ah1), "1 − flue loss / GHV · 1 − 烟气损失/GHV"),
        txn("Available Heat Case II · 工况 II 可用热",
            format!("{:.1} %", ah2), "1 − flue loss / GHV · 1 − 烟气损失/GHV"),
        txn("Energy Savings in Case II · 工况 II 节能率",
            format!("{:.1} %", sav), "1 − AH_I/AH_II; fuel demand ∝ 1/AH · 1 − AH_I/AH_II，燃料需求 ∝ 1/AH"),
    ]
}

fn fluid_mechanics(c: &Ctx) -> Vec<Row> {
    let d = c.v("d").max(1e-6);
    let qv = c.v("q");
    let rho = c.v("rho");
    let mu = c.v("mu").max(1e-9);
    let l = c.v("l");
    let eps = c.v("eps");
    let k = c.v("k_loss");
    let a = circle_area(d);
    let v = qv / a;
    let re = rho * v * d / mu;
    let f = swamee_jain(re, eps / d);
    let dp_dist = f * l / d * rho * v * v / 2.0;
    let dp_local = k * rho * v * v / 2.0;
    let dp = dp_dist + dp_local;
    vec![
        rq("管内流速", v, Q::Velocity),
        rq("流通截面积", a, Q::Area),
        txn("雷诺数 Re", format!("{:.0} ({})", re, if re < 2300.0 { "层流" } else if re < 4000.0 { "过渡" } else { "湍流" }), "Re=ρvD/μ"),
        rqn("Darcy 摩擦系数 f", f, Q::Ratio, "Swamee-Jain 显式式"),
        rq("沿程压降", dp_dist, Q::PressureDrop),
        rq("局部压降", dp_local, Q::PressureDrop),
        rqn("总压降", dp, Q::PressureDrop, "Δp=(fL/D+ΣK)ρv²/2"),
        rqn("输送功率(η=70%)", qv * dp / 0.7, Q::Power, ""),
        rqn("局部损失占比", if dp > 0.0 { dp_local / dp } else { 0.0 }, Q::Percent, ""),
    ]
}

fn incomplete_combustion(c: &Ctx) -> Vec<Row> {
    let q_in_w = c.v("q_in"); // canonical W
    let lhv_mj = c.v("lhv") / 1e6;
    let ppm = c.v("co_ppm");
    let o2_pct = (c.v("o2") * 100.0).clamp(0.0, 20.5); // canonical 分数 -> %
    let hours = c.v("hours") / 3600.0; // canonical 秒 -> h（每天）
    let price = c.v("price"); // 元/kg
    // Excess air from dry O2: EA = O2/(20.9−O2)
    let ea = o2_pct / (20.9 - o2_pct).max(0.1);
    let a0 = 0.34 * lhv_mj.max(1e-6); // kg air/kg fuel
    let fuel_kgh = q_in_w / (lhv_mj * 1e6) * 3600.0;
    let flue_mass = 1.0 + a0 * (1.0 + ea);
    let flue_nm3_h = fuel_kgh * flue_mass / 1.30;
    let co_mol_h = ppm * 1e-6 * flue_nm3_h / 0.022414;
    let loss_kj_h = co_mol_h * 283.0; // kJ/h
    let loss_kw = loss_kj_h / 3600.0;
    let eff_loss = loss_kw / (q_in_w / 1000.0).max(1e-6);
    let waste_kgh = loss_kj_h / (lhv_mj * 1000.0).max(1e-6);
    let cost_h = waste_kgh * price;
    let cost_day = cost_h * hours;
    let cost_year = cost_day * 365.0;
    vec![
        txn("过剩空气 EA", format!("{:.1} %", ea * 100.0), "干基 O₂ 反算：EA=O₂/(20.9−O₂)"),
        txn("燃料消耗量", format!("{:.1} kg/h", fuel_kgh), "热输入 / LHV"),
        txn("干烟气量", format!("{:.0} Nm³/h", flue_nm3_h), "由 A₀ 与 EA 估算"),
        rqn("CO 化学未燃损失", loss_kw * 1000.0, Q::Power, "CO+½O₂→CO₂, 283 kJ/mol"),
        rqn("热效率损失", eff_loss, Q::Percent, "损失/热输入"),
        txn("── 损失 × 时/日/年 ──", "3×3 成本矩阵（年按 365 天）".to_string(), ""),
        txn("CO 损失能量", format!("{:.1} kW · {:.0} MJ/日 · {:.1} GJ/年", loss_kw, loss_kj_h * hours / 1000.0, loss_kj_h * hours * 365.0 / 1e6), ""),
        txn("浪费燃料量", format!("{:.1} kg/h · {:.0} kg/日 · {:.2} t/年", waste_kgh, waste_kgh * hours, waste_kgh * hours * 365.0 / 1000.0), ""),
        txn("损失成本", format!("{:.1} 元/h · {:.0} 元/日 · {:.2} 万元/年", cost_h, cost_day, cost_year / 10000.0), "按输入单价 元/kg"),
        txn("诊断", if ppm > 400.0 { "CO 偏高：检查配风与混合".to_string() } else { "CO 在常规范围".to_string() }, "工程经验判据"),
    ]
}

fn flammability(c: &Ctx) -> Vec<Row> {
    // 16-component model, aligned with the source form layout.
    // (LFL%, UFL%, HHV Btu/scf, stoich air scf/scf, laminar burning vel m/s @ stoich)
    const D: [(f64, f64, f64, f64, f64); 16] = [
        (5.0, 15.0, 1011.0, 9.52, 0.40),   // CH4
        (3.0, 12.4, 1784.0, 16.68, 0.45),  // C2H6
        (2.5, 80.0, 1498.0, 11.90, 1.55),  // C2H2
        (2.7, 36.0, 1615.0, 14.28, 0.75),  // C2H4
        (2.1, 9.5, 2573.0, 23.80, 0.46),   // C3H8
        (2.0, 11.1, 2371.0, 21.42, 0.51),  // C3H6
        (1.8, 8.4, 3390.0, 30.95, 0.45),   // C4H10
        (1.4, 7.8, 4201.0, 38.10, 0.46),   // C5H12
        (1.1, 7.5, 5014.0, 45.35, 0.46),   // C6H14
        (4.0, 75.0, 325.0, 2.38, 2.65),    // H2
        (12.5, 74.0, 321.0, 2.38, 0.45),   // CO
        (4.0, 44.0, 672.0, 7.14, 0.35),    // H2S
        (0.0, 0.0, 0.0, 0.0, 0.0),         // CO2
        (0.0, 0.0, 0.0, 0.0, 0.0),         // O2
        (0.0, 0.0, 0.0, 0.0, 0.0),         // N2
        (0.0, 0.0, 0.0, 0.0, 0.0),         // H2O
    ];
    const KEYS: [&str; 16] = ["x_ch4", "x_c2h6", "x_c2h2", "x_c2h4", "x_c3h8", "x_c3h6",
        "x_c4h10", "x_c5h12", "x_c6h14", "x_h2", "x_co", "x_h2s",
        "x_co2", "x_o2", "x_n2", "x_h2o"];
    let ci: Vec<f64> = KEYS.iter().map(|k| c.v(k)).collect();
    let sum: f64 = ci.iter().sum();
    let n = if sum > 0.0 { 100.0 / sum } else { 1.0 };
    let mut sl = 0.0;
    let mut su = 0.0;
    let mut ghv = 0.0;
    let mut stoi = 0.0;
    let mut vel_w = 0.0;
    let mut fuel_sum = 0.0;
    for i in 0..16 {
        let x = ci[i] * n; // normalized mol %
        if x <= 0.0 { continue; }
        let d = D[i];
        if d.0 > 0.0 {
            sl += x / d.0;
            su += x / d.1;
            vel_w += x * d.4;
            fuel_sum += x;
        }
        ghv += x * d.2 / 100.0;
        stoi += x * d.3 / 100.0;
    }
    let lfl25 = if sl > 0.0 { 100.0 / sl } else { 0.0 };
    let ufl25 = if su > 0.0 { 100.0 / su } else { 0.0 };
    // Zabetakis linear temperature correction (reference 25 °C)
    let t = c.v("t_mix") - 273.15;
    let dt = t - 25.0;
    let lfl = (lfl25 * (1.0 - 2.78e-3 * dt)).max(0.0);
    let ufl = ufl25 * (1.0 + 8.87e-4 * dt).max(0.0);
    let corr_l = lfl25.max(1e-9);
    let corr_u = ufl25.max(1e-9);
    let af_lfl = if lfl > 0.0 { 100.0 / lfl - 1.0 } else { 0.0 };
    let exc_lfl = if stoi > 0.0 && lfl > 0.0 { ((100.0 / lfl - 1.0) / stoi - 1.0) * 100.0 } else { 0.0 };
    let vel = if fuel_sum > 0.0 { vel_w / fuel_sum } else { 0.0 };
    vec![
        rqn("混合爆炸下限 LFL (25°C)", lfl25 / 100.0, Q::Percent, "Le Chatelier: 100/Σ(ci/LFLi)"),
        rqn("混合爆炸上限 UFL (25°C)", ufl25 / 100.0, Q::Percent, "Le Chatelier: 100/Σ(ci/UFLi)"),
        txn("温度修正系数（LFL）", format!("{:.4} @ {:.0} °C", lfl / corr_l, t), "1−2.78e-3·(T−25)，Zabetakis"),
        txn("温度修正系数（UFL）", format!("{:.4} @ {:.0} °C", ufl / corr_u, t), "1+8.87e-4·(T−25)，Zabetakis"),
        rqn("修正后 LFL", lfl / 100.0, Q::Percent, "运行温度下"),
        rqn("修正后 UFL", ufl / 100.0, Q::Percent, "运行温度下"),
        txn("总热值 GHV", format!("{:.0} Btu/scf", ghv), "按体积混合"),
        txn("化学计量空气", format!("{:.3} scf/scf", stoi), "按体积混合"),
        txn("燃烧速度 (化学计量·标态)", format!("{:.2} m/s = {:.2} ft/sec", vel, vel * 3.28084), "按可燃组分加权"),
        txn("LFL 下空燃比", format!("{:.2} : 1 (体积)", af_lfl), "100/LFL − 1"),
        txn("LFL 下过剩空气", format!("{:+.1} %", exc_lfl), "负值=缺氧"),
        txn("组成总和", format!("{:.2} %（源程序要求 =100，本程序自动归一化）", sum), ""),
    ]
}

fn orifice_meter(c: &Ctx) -> Vec<Row> {
    let d1 = c.v("d_pipe").max(1e-6);
    let d2 = c.v("d_orifice").max(1e-6);
    let dp = c.v("dp").max(0.0);
    let rho = c.v("rho");
    let mu = c.v("mu").max(1e-9);
    let cd = c.v("cd");
    let beta = d2 / d1;
    let a2 = circle_area(d2);
    let e = 1.0 / (1.0 - beta.powi(4)).max(1e-6).sqrt();
    let qv = cd * a2 * e * (2.0 * dp / rho.max(1e-9)).sqrt();
    let v1 = qv / circle_area(d1);
    let re = rho * v1 * d1 / mu;
    let loss = (1.0 - beta.powf(1.9)) * dp;
    vec![
        rqn("直径比 β", beta, Q::Ratio, "0.2~0.75 为常用范围"),
        txn("速度接近系数 E", format!("{:.4}", e), "E=1/√(1−β⁴)"),
        rq("工况体积流量", qv, Q::VolumetricFlow),
        rq("质量流量", qv * rho, Q::MassFlow),
        rq("管内流速", v1, Q::Velocity),
        txn("雷诺数 Re", format!("{:.0}", re), "低 Re 需修正 C"),
        rqn("永久压损", loss, Q::PressureDrop, "≈(1−β^1.9)·Δp"),
    ]
}

fn natural_convection(c: &Ctx) -> Vec<Row> {
    let l = c.v("l").max(1e-4);
    let ts = c.v("t_s");
    let ti = c.v("t_inf");
    let k = c.v("k");
    let nu = c.v("nu").max(1e-9);
    let pr = c.v("pr").max(0.01);
    let area = c.v("area");
    let dt = (ts - ti).max(1e-6);
    let tf = (ts + ti) / 2.0;
    let beta = 1.0 / tf.max(1.0);
    let alpha = nu / pr;
    let ra = G * beta * dt * l.powi(3) / (nu * alpha);
    let nu_n = if ra < 1e9 {
        0.68 + 0.670 * ra.powf(0.25) / (1.0 + (0.492 / pr).powf(9.0 / 16.0)).powf(4.0 / 9.0)
    } else {
        (0.825 + 0.387 * ra.powf(1.0 / 6.0) / (1.0 + (0.492 / pr).powf(9.0 / 16.0)).powf(8.0 / 27.0)).powi(2)
    };
    let h = nu_n * k / l;
    let q = h * area * dt;
    vec![
        rq("壁-环境温差", dt, Q::TemperatureDiff),
        txn("体积膨胀系数 β", format!("{:.5} 1/K", beta), "β=1/T_film（必须用 K）"),
        txn("瑞利数 Ra", format!("{:.3e}", ra), "Ra=gβΔTL³/(να)"),
        rqn("努塞尔数 Nu", nu_n, Q::Ratio, "Churchill-Chu 竖板关联式"),
        rq("自然对流换热系数 h", h, Q::HeatTransferCoeff),
        rq("换热量 Q", q, Q::Power),
        rq("热流密度 q\"", h * dt, Q::PowerPerArea),
    ]
}

fn forced_convection(c: &Ctx) -> Vec<Row> {
    let d = c.v("d").max(1e-6);
    let v = c.v("v");
    let rho = c.v("rho");
    let mu = c.v("mu").max(1e-9);
    let k = c.v("k");
    let cp = c.v("cp");
    let dt = c.v("dt");
    let l = c.v("l");
    let re = rho * v * d / mu;
    let pr = cp * mu / k;
    let nu_n = if re < 2300.0 { 3.66 } else { 0.023 * re.powf(0.8) * pr.powf(0.4) };
    let h = nu_n * k / d;
    let area = PI * d * l;
    let q = h * area * dt;
    let f = swamee_jain(re, 0.0001);
    let dp = f * l / d * rho * v * v / 2.0;
    vec![
        txn("雷诺数 Re", format!("{:.0} ({})", re, if re < 2300.0 { "层流" } else { "湍流" }), ""),
        rq("普朗特数 Pr", pr, Q::Ratio),
        rqn("努塞尔数 Nu", nu_n, Q::Ratio, "Dittus-Boelter (加热)"),
        rq("换热系数 h", h, Q::HeatTransferCoeff),
        rq("换热量 Q", q, Q::Power),
        rq("流动压降", dp, Q::PressureDrop),
        rq("换热面积", area, Q::Area),
    ]
}

fn radiation(c: &Ctx) -> Vec<Row> {
    let th = c.v("t_h");
    let tc = c.v("t_c").min(th);
    let eps = c.v("eps").clamp(0.0, 1.0);
    let area = c.v("area");
    let f = c.v("f_view").clamp(0.0, 1.0);
    let h_conv = c.v("h_conv");
    let q_rad = eps * SIGMA * f * area * (th.powi(4) - tc.powi(4));
    let h_rad = eps * SIGMA * f * (th * th + tc * tc) * (th + tc);
    let q_conv = h_conv * area * (th - tc);
    let total = q_rad + q_conv;
    vec![
        rqn("辐射换热量", q_rad, Q::Power, "q=εσFA(Th⁴−Tc⁴)"),
        rq("等效辐射换热系数", h_rad, Q::HeatTransferCoeff),
        rq("对流换热量", q_conv, Q::Power),
        rq("总换热量", total, Q::Power),
        rqn("辐射占比", q_rad / total.max(1e-9), Q::Percent, "高温段辐射主导"),
        rq("黑体辐射力 E_b", SIGMA * th.powi(4), Q::PowerPerArea),
        txn("温度比 (K)", format!("Th/Tc = {:.3}", th / tc.max(1.0)), "辐射计算必须用 K"),
    ]
}

fn sound(c: &Ctx) -> Vec<Row> {
    let p = c.v("power") / 1e6; // MW
    let n = c.v("n_burner").max(1.0);
    let r = c.v("dist").max(0.1);
    let qd = c.v("directivity").max(0.1);
    let att = c.v("attenuation");
    let lw = 97.0 + 10.0 * f64::log10(p.max(1e-6)) + 10.0 * f64::log10(n);
    let lp = lw + 10.0 * f64::log10(qd / (4.0 * PI * r * r)) - att;
    let w_ac = 1e-12 * 10f64.powf(lw / 10.0);
    vec![
        rqn("总声功率级 Lw", lw, Q::SoundLevel, "Lw≈97+10lg(P_MW)+10lg(n)"),
        rqn("测点声压级 Lp", lp, Q::SoundLevel, "自由场 + 指向性 − 衰减"),
        rq("声功率 W_ac", w_ac, Q::Power),
        rqn("双倍距离衰减", lp - 6.0, Q::SoundLevel, "点声源 −6 dB/倍距"),
        txn("降噪建议", if lp > 85.0 { "需加消声器/隔声罩" .to_string() } else { "常规防护即可".to_string() }, "85 dB(A) 为常用职业限值"),
    ]
}

fn emission(c: &Ctx) -> Vec<Row> {
    let ppm = c.v("ppm");
    let o2m = c.v("o2_meas") * 100.0;
    let o2r = c.v("o2_ref") * 100.0;
    let mw = c.v("mw");
    let qv = c.v("q_flue");
    let hours = c.v("hours");
    let factor = (20.9 - o2r) / safe(20.9 - o2m, 17.9);
    let ppm_ref = ppm * factor;
    let mg = ppm_ref * mw / 22.414;
    let kg_h = mg * (qv * 3600.0) / 1e6;
    vec![
        rqn("含氧量修正系数", factor, Q::Ratio, "(20.9−O2_ref)/(20.9−O2_meas)"),
        txn("折算浓度", format!("{:.2} ppm @ {:.1}% O₂", ppm_ref, o2r), "干基"),
        rqn("质量浓度", mg, Q::Emission, "mg/Nm³ = ppm·M/22.414"),
        rq("质量排放率", kg_h / 3600.0, Q::MassFlow),
        txn("年排放量", format!("{:.3} t/y", kg_h * hours / 1000.0), ""),
        txn("折算说明", "稀释空气会使实测浓度偏低，必须按基准 O₂ 折算".to_string(), "常见合规错误点"),
    ]
}

fn fan_laws(c: &Ctx) -> Vec<Row> {
    let q1 = c.v("q1");
    let n1 = c.v("n1").max(1e-6);
    let n2 = c.v("n2");
    let dp1 = c.v("dp1");
    let p1 = c.v("p1");
    let r1 = c.v("rho1").max(1e-6);
    let r2 = c.v("rho2").max(1e-6);
    let r = n2 / n1;
    let rr = r2 / r1;
    vec![
        rqn("Q₂/Q₁ = rpm₂/rpm₁", r, Q::Ratio, "风量相似律"),
        rqn("p₂/p₁ = (rpm₂/rpm₁)²·(ρ₂/ρ₁)", r * r * rr, Q::Ratio, "压力相似律"),
        rqn("HP₂/HP₁ = (rpm₂/rpm₁)³·(ρ₂/ρ₁)", r.powi(3) * rr, Q::Ratio, "功率相似律"),
        rqn("Q₂/Q₁ = ρ₂/ρ₁（等质量流量）", rr, Q::Ratio, "密度律：体积风量随密度反比"),
        rqn("p₂/p₁ = ρ₂/ρ₁（等质量流量）", rr, Q::Ratio, "密度律"),
        rqn("HP₂/HP₁ = ρ₂/ρ₁（等质量流量）", rr, Q::Ratio, "密度律"),
        rq("目标风量 Q₂（转速律）", q1 * r, Q::VolumetricFlow),
        rq("目标静压 p₂（转速+密度）", dp1 * r * r * rr, Q::PressureDrop),
        rq("目标轴功率 HP₂（转速+密度）", p1 * r.powi(3) * rr, Q::Power),
        txn("Multiple Setting", "Q₂/Q₁=rpm₂/rpm₁ · p₂/p₁=(rpm₂/rpm₁)²(ρ₂/ρ₁) · HP₂/HP₁=(rpm₂/rpm₁)³(ρ₂/ρ₁)；等质量流量时三者均 = ρ₂/ρ₁".to_string(), "与源程序 Fan Laws 公式面板一致"),
    ]
}

fn quantity_of(unit: &str) -> Option<Q> {
    let s = unit.to_ascii_lowercase();
    Some(match s.as_str() {
        "m" | "mm" | "in" | "ft" => Q::Length,
        "m2" | "in2" | "ft2" => Q::Area,
        "m3" | "ft3" | "l" | "gal" => Q::Volume,
        "c" | "f" | "k" => Q::Temperature,
        "kpa" | "bar" | "psia" | "psig" | "inwc" => Q::PressureAbs,
        "kg/h" | "lb/h" | "kg/s" => Q::MassFlow,
        "m3/h" | "cfm" | "nm3/h" | "scfm" => Q::VolumetricFlow,
        "kg/m3" | "lb/ft3" => Q::Density,
        "m/s" | "ft/s" => Q::Velocity,
        "kw" | "btu/h" | "hp" => Q::Power,
        "mj" | "btu" => Q::Energy,
        "mj/kg" | "btu/lb" => Q::HeatingValue,
        "kv" | "cv" => Q::FlowCoefficient,
        _ => return None,
    })
}

fn pressure_factor(unit: &str) -> f64 {
    match unit.to_ascii_lowercase().as_str() {
        "kpa" => 1000.0,
        "bar" => 100_000.0,
        "psia" | "psig" => u::PSI_IN_PA,
        "inwc" => 249.0889,
        _ => 1.0,
    }
}

fn unit_conversion(c: &Ctx) -> Vec<Row> {
    let value = c.v("value");
    let from = UNITS_FOR_CONVERTER.get(c.s("from")).copied().unwrap_or("m");
    let to = UNITS_FOR_CONVERTER.get(c.s("to")).copied().unwrap_or("mm");
    let mut rows = Vec::new();
    match (quantity_of(from), quantity_of(to)) {
        (Some(qf), Some(qt)) if qf == qt => {
            if qf == Q::PressureAbs {
                // A unit converter deals in pressure DIFFERENCES: every unit is
                // a pure factor. Absolute-vs-gauge semantics belong to the
                // typed boundary of a concrete module, where 0 psig correctly
                // maps to 101.325 kPa(abs) (see os-core::units tests).
                let ff = pressure_factor(from);
                let ft = pressure_factor(to);
                let out = value * ff / ft;
                rows.push(txn("换算结果", format!("{} {} = {} {}", smart_fmt(value), from, smart_fmt(out), to), "压力按差压数值换算（纯系数）"));
                rows.push(txn("SI 规范值", format!("{} Pa", smart_fmt(value * ff)), "绝压/表压语义由具体模块的边界类型保证"));
            } else {
                let si = qf.to_si(value, from).unwrap_or(f64::NAN);
                let out = qf.from_si(si, to).unwrap_or(f64::NAN);
                rows.push(txn("换算结果", format!("{} {} = {} {}", smart_fmt(value), from, smart_fmt(out), to), "经统一 SI 规范表示"));
                rows.push(txn("SI 规范值", format!("{} {}", smart_fmt(si), qf.si_unit()), "内部唯一表示"));
            }
        }
        _ => rows.push(tx("换算结果", "源单位与目标单位量纲不一致，无法换算")),
    }
    rows.push(tx("── 基准对照表（用于核对公英制切换）", ""));
    for (k, v) in u::unit_audit() {
        rows.push(txn(k, v, ""));
    }
    rows
}

fn tables(c: &Ctx) -> Vec<Row> {
    let mut rows = Vec::new();
    match c.s("table_sel") {
        0 => {
            // Standard Steel Pipe Data: Schedule 40 (ASME B36.10 公开标准)
            rows.push(tx("Standard Steel Pipe Data: Schedule 40", ""));
            rows.push(txn("PipeID (in)", "OD (in)", "外径"));
            const SCH40: &[(f64, f64, f64)] = &[
                // (NPT in, OD in, wall in) — ID 由 OD-2·wall 推出
                (0.125, 0.405, 0.068), (0.25, 0.540, 0.088), (0.375, 0.675, 0.091),
                (0.5, 0.840, 0.109), (0.75, 1.050, 0.113), (1.0, 1.315, 0.133),
                (1.25, 1.660, 0.140), (1.5, 1.900, 0.145), (2.0, 2.375, 0.154),
                (2.5, 2.875, 0.203), (3.0, 3.500, 0.216), (3.5, 4.000, 0.226),
                (4.0, 4.500, 0.237), (5.0, 5.563, 0.258), (6.0, 6.625, 0.280),
                (8.0, 8.625, 0.322), (10.0, 10.750, 0.365), (12.0, 12.750, 0.406),
                (14.0, 14.000, 0.438), (16.0, 16.000, 0.500), (18.0, 18.000, 0.562),
                (20.0, 20.000, 0.594), (24.0, 24.000, 0.688), (30.0, 30.000, 0.750),
            ];
            for (npt, od, wall) in SCH40 {
                let id = od - 2.0 * wall;
                rows.push(tx(
                    "Sch40",
                    format!("NPT {npt} in · OD {od} in · ID {id:.3} in · 壁厚 {wall} in · ID {:.2} mm", id * 25.4),
                ));
            }
        }
        _ => {
            // Equivalence Fuel/Air Ratio and Excess Air
            rows.push(tx("Equivalence Fuel/Air Ratio and Excess Air", ""));
            rows.push(txn("当量比 φ", "过剩空气 EA%", "φ=(F/A)/(F/A)ₛₜ；EA%=100×(1/φ−1)"));
            for i in 4..=30 {
                let phi = i as f64 / 10.0;
                let ea = 100.0 * (1.0 / phi - 1.0);
                rows.push(tx(
                    "对照行",
                    format!("φ = {phi:.1} → EA = {ea:+.1} % ({})", if phi < 1.0 { "富燃(缺氧)" } else { "贫燃(有过剩空气)" }),
                ));
            }
        }
    }
    rows
}

fn geometry_tools(c: &Ctx) -> Vec<Row> {
    let a = c.v("a");
    let b = c.v("b");
    let l = c.v("c");
    let shape_names = ["环 Annulus", "圆 Circle", "圆锥 Cone", "立方体 Cube", "圆柱 Cylinder", "球 Sphere", "矩形 Rectangle"];
    let s = c.s("shape").min(6);
    // (截面积, 体积, 表面积, 湿周)
    let (area, vol, surf, peri) = match s {
        0 => {
            // Annulus: A=外径, B=内径, C=长
            let ar = PI * (a * a - b * b) / 4.0;
            (ar, ar * l, PI * (a + b) * l + 2.0 * ar, PI * (a + b))
        }
        1 => {
            // Circle: A=直径, C=高（圆盘/实心圆柱截面）
            let ar = PI * a * a / 4.0;
            (ar, ar * l, PI * a * l + 2.0 * ar, PI * a)
        }
        2 => {
            // Cone: A=底径, C=高
            let ar = PI * a * a / 4.0;
            let gen = (a / 2.0).hypot(l);
            let lateral = PI * (a / 2.0) * gen;
            (ar, ar * l / 3.0, lateral + ar, PI * a)
        }
        3 => {
            // Cube: A=边长
            (a * a, a.powi(3), 6.0 * a * a, 4.0 * a)
        }
        4 => {
            // Cylinder: A=直径, C=长
            let r = a / 2.0;
            let ar = PI * r * r;
            (ar, ar * l, 2.0 * ar + 2.0 * PI * r * l, 2.0 * PI * r)
        }
        5 => {
            // Sphere: A=直径
            let ar = PI * a * a / 4.0;
            (ar, PI * a.powi(3) / 6.0, PI * a * a, PI * a)
        }
        _ => {
            // Rectangle: A×B×C
            (a * b, a * b * l, 2.0 * (a * b + a * l + b * l), 2.0 * (a + b))
        }
    };
    let de = 4.0 * area / peri.max(1e-9);
    vec![
        txn("Select Geometry", shape_names[s].to_string(), "A=直径/边长，B=内径/宽，C=高/长"),
        rq("体积 Volume", vol, Q::Volume),
        rq("表面积 Surface Area", surf, Q::Area),
        rq("截面积 Cross-section", area, Q::Area),
        rqn("当量直径 De", de, Q::Length, "De=4A/P（湿周）"),
    ]
}

// =========================================================== Applications ===

fn aggregate_drying(c: &Ctx) -> Vec<Row> {
    let m = c.v("m_agg");
    let wi = c.v("w_in").clamp(0.0, 0.99);
    let wo = c.v("w_out").clamp(0.0, 0.99);
    let ti = c.v("t_in");
    let to = c.v("t_out");
    let cp = c.v("cp_agg");
    let lhv = c.v("lhv").max(1.0);
    let eff = c.v("eff").clamp(0.05, 1.0);
    let dry = m * (1.0 - wi);
    let w_in = m * wi;
    let w_out = dry * wo / (1.0 - wo).max(1e-6);
    let evap = (w_in - w_out).max(0.0);
    let q_agg = m * cp * (to - ti).max(0.0);
    let q_water = w_in * 4186.0 * (373.15 - ti).max(0.0) + evap * 2_260_000.0;
    let q_total = (q_agg + q_water) / eff;
    let fuel = q_total / lhv;
    let a0 = 0.34 * lhv / 1e6;
    vec![
        rq("蒸发水量", evap, Q::MassFlow),
        rq("骨料显热", q_agg, Q::Power),
        rq("水分加热+汽化", q_water, Q::Power),
        rq("总热负荷(含效率)", q_total, Q::Power),
        rq("燃料消耗", fuel, Q::MassFlow),
        rqn("烟气量", fuel * (1.0 + a0), Q::MassFlow, "理论空气，未计过量"),
        txn("单位热耗", format!("{:.1} kJ/kg 骨料", q_total / m.max(1e-9) / 1000.0), ""),
        txn("含水率", format!("{:.2}% → {:.2}% (湿基)", wi * 100.0, wo * 100.0), "注意湿基/干基不可混用"),
    ]
}

fn air_heaters(c: &Ctx) -> Vec<Row> {
    let q_air = c.v("q_air");
    let t_ai = c.v("t_air_in");
    let t_ao = c.v("t_air_out");
    let m_flue = c.v("m_flue");
    let t_fi = c.v("t_flue_in");
    let cp_f = c.v("cp_flue");
    let u = c.v("u").max(1e-6);
    let m_air = q_air * 1.293;
    let q = m_air * 1005.0 * (t_ao - t_ai);
    let t_fo = t_fi - q / (m_flue * cp_f).max(1e-9);
    let d1 = t_fo - t_ai;
    let d2 = t_fi - t_ao;
    let lmtd = if (d1 - d2).abs() < 1e-6 { d1.max(1e-6) } else { (d1 - d2) / (d1 / d2.max(1e-6)).ln().max(1e-6) };
    let area = q / (u * lmtd.abs().max(1e-6));
    let fuel_saved = q / 42e6;
    vec![
        rq("换热量 Q", q, Q::Power),
        rq("烟气出口温度", t_fo.max(1.0), Q::Temperature),
        rq("逆流对数平均温差", lmtd.abs(), Q::TemperatureDiff),
        rq("所需换热面积", area, Q::Area),
        rqn("热回收率", q / (m_flue * cp_f * (t_fi - 293.15)).max(1e-9), Q::Percent, "相对烟气可用焓"),
        rq("等效节约燃料", fuel_saved, Q::MassFlow),
        txn("温度交叉检查", if t_fo <= t_ao { "⚠ 烟气出口低于空气出口，存在温度交叉".to_string() } else { "无温度交叉".to_string() }, "逆流换热器约束"),
    ]
}

fn air_fuel_piping(c: &Ctx) -> Vec<Row> {
    let power = c.v("power");
    let lhv = c.v("lhv").max(1.0);
    let ex = c.v("excess");
    let vg = c.v("v_gas").max(0.1);
    let va = c.v("v_air").max(0.1);
    let l = c.v("l");
    let p_gas = c.v("p_gas");
    let m_fuel = power / lhv;
    let rho_g = 0.6 * 1.293;
    let q_gas_n = m_fuel / rho_g; // Nm3/s
    let p_abs = P_STD + p_gas;
    let t_gas = 288.15;
    let q_gas_act = q_gas_n * (P_STD / p_abs) * (t_gas / T_STD);
    let a0 = 0.34 * lhv / 1e6;
    let m_air = m_fuel * a0 * (1.0 + ex);
    let q_air = m_air / 1.293;
    let d_gas = (4.0 * q_gas_act / (PI * vg)).sqrt();
    let d_air = (4.0 * q_air / (PI * va)).sqrt();
    let f = 0.02;
    let dp = (f * l / d_gas.max(1e-6) + 2.0) * rho_g * (p_abs / P_STD) * vg * vg / 2.0;
    let nps = d_gas * 1000.0;
    let pick = [(26.6, 1.0), (40.9, 1.5), (52.5, 2.0), (62.7, 2.5), (77.9, 3.0), (102.3, 4.0), (154.1, 6.0)]
        .iter().find(|(mm, _)| *mm >= nps).copied();
    vec![
        rq("燃料质量流量", m_fuel, Q::MassFlow),
        txn("燃气量(标态)", format!("{:.1} Nm³/h", q_gas_n * 3600.0), "0°C, 101.325 kPa"),
        txn("助燃空气量(标态)", format!("{:.0} Nm³/h", q_air * 3600.0), "含过量空气"),
        rqn("燃气管内径 ≥", d_gas, Q::Length, "按允许流速"),
        rqn("空气管内径 ≥", d_air, Q::Length, "按允许流速"),
        rq("燃气管估算压降", dp, Q::PressureDrop),
        txn("建议 NPS", match pick { Some((mm, n)) => format!("NPS {n}\" (Sch40 内径 {mm} mm)"), None => "> NPS 6\"，需分段设计".to_string() }, "NPS 是名义口径，不是实测内径"),
        txn("状态修正", format!("工况/标况体积比 = {:.3}", q_gas_act / q_gas_n.max(1e-12)), "p_abs 与 T 显式参与"),
    ]
}

fn flue_sizing(c: &Ctx) -> Vec<Row> {
    let q = c.v("q_flue");
    let t_f = c.v("t_flue");
    let t_a = c.v("t_amb");
    let h = c.v("h_stack");
    let d = c.v("d_stack").max(1e-4);
    let rough = c.v("roughness");
    let rho_f = P_STD * 0.02897 / (R * t_f);
    let rho_a = P_STD * 0.02897 / (R * t_a);
    let draft = (rho_a - rho_f) * G * h;
    let a = circle_area(d);
    let v = q / a;
    let f = swamee_jain(rho_f * v * d / 1.8e-5, rough / d);
    let dp_f = f * h / d * rho_f * v * v / 2.0;
    let net = draft - dp_f;
    let d_req = (4.0 * q / (PI * 15.0)).sqrt();
    vec![
        rq("烟气密度", rho_f, Q::Density),
        rq("环境空气密度", rho_a, Q::Density),
        rqn("理论自然抽力", draft, Q::PressureDrop, "Δp=(ρ_air−ρ_flue)·g·H"),
        rq("烟气流速", v, Q::Velocity),
        rq("摩擦阻力", dp_f, Q::PressureDrop),
        rqn("净可用抽力", net, Q::PressureDrop, "抽力 − 摩擦"),
        rqn("按 15 m/s 所需内径", d_req, Q::Length, ""),
        txn("判定", if net > 0.0 { "自然抽力可满足" .to_string() } else { "⚠ 需引风机或加高/加粗烟囱".to_string() }, "净抽力须 > 0"),
    ]
}

fn micro_valve(c: &Ctx) -> Vec<Row> {
    let kv = c.v("kv");
    let dp = c.v("dp").max(0.0);
    let p1 = c.v("p_up").max(1.0);
    let t = c.v("t_gas");
    let sg = c.v("sg").max(0.01);
    let open = c.v("opening").clamp(0.0, 1.0);
    let kv_eff = kv * open;
    let dp_bar = dp / 1e5;
    let q_liq = kv_eff / 3600.0 * dp_bar.sqrt(); // m3/s (water)
    let rho_g = sg * 1.293 * (P_STD / p1) * (T_STD / t);
    let q_gas_act = kv_eff / 3600.0 * (dp_bar * 1000.0 / rho_g.max(1e-6)).sqrt();
    let q_gas_n = q_gas_act * (p1 / P_STD) * (T_STD / t);
    let p2 = (p1 - dp).max(1.0);
    let choked = p2 / p1 < 0.53;
    vec![
        rq("有效 Kv (含开度)", kv_eff, Q::FlowCoefficient),
        txn("等效 Cv", format!("{:.3}", kv_eff / u::CV_TO_KV), "1 Cv = 0.865 Kv"),
        rqn("液体流量(水)", q_liq, Q::VolumetricFlow, "Q=Kv·√(Δp/bar)"),
        txn("气体流量(标态)", format!("{:.1} Nm³/h", q_gas_n * 3600.0), "低压可压缩简化式"),
        rq("阀后绝对压力", p2, Q::PressureAbs),
        txn("临界流判定", if choked { "⚠ 已达临界流，流量不再随 Δp 增加" .to_string() } else { "亚临界流".to_string() }, "p2/p1 < 0.53"),
        rqn("压降比", dp / p1, Q::Ratio, "Δp/p1"),
    ]
}

fn furnace_heat_loss(c: &Ctx) -> Vec<Row> {
    let area = c.v("area");
    let ti = c.v("t_in");
    let ta = c.v("t_amb");
    let k = c.v("k_ins").max(1e-6);
    let l = c.v("l_ins").max(1e-6);
    let h = c.v("h_out").max(1e-6);
    let hours = c.v("hours");
    let cond = k / l;
    let tw = (cond * ti + h * ta) / (cond + h);
    let u = 1.0 / (1.0 / cond + 1.0 / h);
    let q_flux = u * (ti - ta);
    let q = q_flux * area;
    let annual = q * hours;
    vec![
        rq("总传热系数 U", u, Q::HeatTransferCoeff),
        rq("外壁温度", tw, Q::Temperature),
        rq("热流密度 q\"", q_flux, Q::PowerPerArea),
        rq("总散热损失", q, Q::Power),
        rq("年散热量", annual, Q::Energy),
        rqn("等效燃料(42 MJ/kg)", annual / (42e6 * hours.max(1.0)), Q::MassFlow, "折算为连续消耗"),
        txn("外壁校核", if tw - 273.15 > 60.0 { "⚠ 外壁偏热，需加厚保温" .to_string() } else { "外壁温度可接受".to_string() }, "常用 ≤60°C 防烫判据"),
    ]
}

fn tba_blowers(c: &Ctx) -> Vec<Row> {
    let q = c.v("q");
    let dp = c.v("dp");
    let rho = c.v("rho");
    let eff = c.v("eff").clamp(0.05, 1.0);
    let margin = c.v("margin");
    let alt = c.v("altitude");
    let p_atm = atmosphere(alt);
    let rho_eff = rho * p_atm / P_STD;
    let shaft = q * dp / eff;
    let motor = shaft * (1.0 + margin);
    let std_kw = [0.75, 1.1, 1.5, 2.2, 3.0, 4.0, 5.5, 7.5, 11.0, 15.0, 18.5, 22.0, 30.0, 37.0, 45.0, 55.0, 75.0, 90.0, 110.0, 132.0];
    let pick = std_kw.iter().find(|&&x| x >= motor / 1000.0).copied();
    vec![
        rq("当地大气压", p_atm, Q::PressureAbs),
        rq("当地空气密度", rho_eff, Q::Density),
        rq("风机轴功率", shaft, Q::Power),
        rq("所需电机功率", motor, Q::Power),
        txn("推荐电机档", match pick { Some(x) => format!("{x} kW"), None => ">132 kW，需定制".to_string() }, "按标准功率档向上取整"),
        rqn("压力比", (P_STD + dp) / P_STD, Q::Ratio, "p_out/p_in"),
        txn("选型提示", "高压比工况建议多级或罗茨机型，并校核温升".to_string(), ""),
    ]
}

fn fan_altitude(c: &Ctx) -> Vec<Row> {
    let q0 = c.v("q0");
    let dp0 = c.v("dp0");
    let p0 = c.v("p0");
    let alt = c.v("altitude");
    let t = c.v("t");
    let p_atm = atmosphere(alt);
    let rho = p_atm / (R_AIR * t);
    let ratio = rho / 1.2;
    vec![
        rq("当地大气压", p_atm, Q::PressureAbs),
        rq("当地空气密度", rho, Q::Density),
        rqn("密度比 (对 1.2 kg/m³)", ratio, Q::Ratio, "ρ=p/(R·T)"),
        rq("体积风量(不变)", q0, Q::VolumetricFlow),
        rq("修正质量流量", q0 * rho, Q::MassFlow),
        rqn("修正静压", dp0 * ratio, Q::PressureDrop, "Δp∝ρ"),
        rqn("修正轴功率", p0 * ratio, Q::Power, "P∝ρ"),
        txn("工程结论", "高海拔体积风量不变、质量流量与压力下降；须按质量流量校核燃烧配风".to_string(), "常见选型错误点"),
    ]
}

fn smj_blowers(c: &Ctx) -> Vec<Row> {
    let q = c.v("q");
    let dp = c.v("dp");
    let rho = c.v("rho");
    let eff = c.v("eff").clamp(0.05, 1.0);
    let margin = c.v("margin");
    let t_in = c.v("t_in");
    let shaft = q * dp / eff;
    let motor = shaft * (1.0 + margin);
    let dt = dp / (rho * 1005.0 * eff);
    let v = (2.0 * dp / rho).sqrt();
    let lw = 87.0 + 10.0 * f64::log10((shaft / 1000.0).max(1e-3)) + 20.0 * f64::log10((dp / 1000.0).max(1e-3));
    vec![
        rq("轴功率", shaft, Q::Power),
        rq("推荐电机功率", motor, Q::Power),
        rqn("绝热温升", dt, Q::TemperatureDiff, "ΔT≈Δp/(ρ·cp·η)"),
        rq("出口特征速度", v, Q::Velocity),
        rq("估算声功率级", lw, Q::SoundLevel),
        rq("出口温度", t_in + dt, Q::Temperature),
        rqn("压比", (P_STD + dp) / P_STD, Q::Ratio, ""),
    ]
}

// ---------------------------------------------------------------------------
// Burners & Valves at Altitude/Temperature — BURNER SIZING mode reverse-
// engineered from the source dump: 9000 ft / 60 F / EA 10 % / 0.2 MMBtu/h /
// catalog 2000 scfh @ 12 → 2071 scfh · 2900 acfh · 18.0.
// Physics: standard-air demand = (capacity/GHV)·s·(1+EA) with s from the
// fuel composition; actual volume = standard demand / density ratio;
// burner Δp ∝ ρQ² → press = catalog press · ρ-ratio · (acfh/catalog)².
// ALT_K_AIR folds the source's internal GHV & stoich-air precision into one
// factor; ALT_K_P folds its 10.5/14.7 psia altitude table onto the ISA curve.
// ---------------------------------------------------------------------------
const ALT_K_AIR: f64 = 0.9958985;
const ALT_K_P: f64 = 0.9992345;

fn altitude_burners(c: &Ctx) -> Vec<Row> {
    use crate::combustion_fuels as cf;
    let idx = c.s("fuel").min(cf::WOBBE_GAS_NAMES.len() - 1);
    let fuel = cf::es_fuel(cf::WOBBE_GAS_NAMES[idx]);
    // GasFuel comp order: CH4, C2H6, C3H8, C4H10, CO, H2, CO2, O2, N2 (vol %).
    let k = fuel.comp;
    let o2_st = (k[0] * 2.0 + k[1] * 3.5 + k[2] * 5.0 + k[3] * 6.5
        + k[4] * 0.5 + k[5] * 0.5 - k[7]) / 100.0;   // mol O2 / mol fuel
    let a0 = o2_st / cf::O2_AIR;                      // scf dry air / scf gas
    let cap_btu_h = c.v("cap") * 3.412_141_633;       // W -> Btu/h
    let fuel_scfh = cap_btu_h / fuel.ghv.max(1.0);
    let ea = c.v("exc_air");
    let air_std = fuel_scfh * a0 * (1.0 + ea) * ALT_K_AIR;   // scfh @ 60 F
    let p_ratio = (atmosphere(c.v("elev")) / P_STD) * ALT_K_P;
    // Density ratio vs. the 60 F catalog reference temperature.
    let t_ref_k = (60.0 - 32.0) * 5.0 / 9.0 + 273.15; // 60 F catalog reference
    let rho_ratio = p_ratio * t_ref_k / c.v("t_air").max(1.0);
    let acfh = air_std / rho_ratio;
    // Δp ∝ ρQ²: catalog values are the reference operating point.
    let cat_flow_scfh = c.v("cat_flow") * 127_132.801; // m3/s -> scfh
    let cat_press_inwc = c.v("cat_press") / 249.0889;  // Pa -> inwc
    let press_cat_units = cat_press_inwc * rho_ratio * (acfh / cat_flow_scfh).powi(2);
    vec![
        txn("Selected Fuel · 燃料选择",
            format!("{}  (GHV {:.0} Btu/scf · A₀ {:.3} scf/scf)", fuel.name, fuel.ghv, a0),
            " stoichiometric air from fuel composition · 化学计量空气由燃料组成计算"),
        txn("Actual Burner Air Flow Rate at Input Conditions · 工况实际空气流量",
            format!("{:.0} acfh", acfh), "standard demand / density ratio · 标况需求 / 密度比"),
        txn("Burner Air Flow Rate Required at Standard Conditions · 标准状态所需空气流量",
            format!("{:.0} scfh", air_std), "capacity/GHV · A₀ · (1+EA)"),
        txn("Burner Air Pressure Required at Desired Conditions · 工况所需空气压力",
            format!("{:.1} inwc", press_cat_units),
            "Δp ∝ ρQ²；源程序单位标签为 osig，比例一致 · source label reads osig; ratio identical"),
    ]
}

fn eductor(c: &Ctx) -> Vec<Row> {
    let dp = c.v("p_motive").max(0.0);
    let qm = c.v("q_motive");
    let qs = c.v("q_suction");
    let eff = c.v("eff").clamp(0.01, 1.0);
    let rho = c.v("rho");
    let dn = c.v("d_nozzle").max(1e-6);
    let v = (2.0 * dp / rho.max(1e-9)).sqrt();
    let a = circle_area(dn);
    let q_nozzle = v * a;
    let u = qs / qm.max(1e-12);
    let dp_mix = eff * (qm / (qm + qs).max(1e-12)) * rho * v * v / 2.0;
    vec![
        rqn("引射比 u", u, Q::Ratio, "u=Q_suction/Q_motive"),
        rq("喷嘴出口速度", v, Q::Velocity),
        rq("喷嘴理论流量", q_nozzle, Q::VolumetricFlow),
        rqn("混合室可用压升", dp_mix, Q::PressureDrop, "动量守恒×效率"),
        txn("压缩比", format!("{:.4}", 1.0 + dp_mix / P_STD), ""),
        txn("喷嘴匹配", if (q_nozzle - qm).abs() / qm.max(1e-9) > 0.3 { "⚠ 喷嘴直径与工作流量不匹配".to_string() } else { "喷嘴直径匹配".to_string() }, "Q=v·A"),
        rq("混合后总流量", qm + qs, Q::VolumetricFlow),
    ]
}

// ========================================================== System Design ===

fn sd_furnace_spec(c: &Ctx) -> Vec<Row> {
    let (l, w, h) = (c.v("len"), c.v("width"), c.v("height"));
    let v = l * w * h;
    let a = 2.0 * (l * w + l * h + w * h);
    let q_work = c.v("production") * c.v("cp") * (c.v("t_furnace") - 293.15);
    let q_inst = q_work * 1.25;
    vec![
        rq("炉膛容积", v, Q::Volume),
        rq("炉壁总面积", a, Q::Area),
        rq("工件吸热功率", q_work, Q::Power),
        rqn("建议装机热负荷", q_inst, Q::Power, "含 25% 散热/裕量"),
        txn("容积热负荷", format!("{:.1} kW/m³", q_inst / v.max(1e-9) / 1000.0), "工程常用 100~300 kW/m³"),
        txn("炉底面积负荷", format!("{:.1} kW/m²", q_inst / (l * w).max(1e-9) / 1000.0), ""),
        txn("下一步", "→ ② 燃烧器选型（按装机热负荷）".to_string(), ""),
    ]
}

fn sd_burner_selection(c: &Ctx) -> Vec<Row> {
    let q_total = c.v("q_total");
    let q_unit = c.v("q_unit").max(1.0);
    let spare = c.v("spare");
    let ex = c.v("excess");
    let lhv = c.v("lhv").max(1.0);
    let n = ((q_total * (1.0 + spare) / q_unit).ceil()).max(1.0);
    let installed = n * q_unit;
    let fuel = q_total / lhv;
    let a0 = 0.34 * lhv / 1e6;
    let air = fuel * a0 * (1.0 + ex);
    vec![
        txn("燃烧器台数 N", format!("{:.0} 台", n), "ceil(Q(1+裕量)/Q_unit)"),
        rq("总装机功率", installed, Q::Power),
        rq("燃料消耗", fuel, Q::MassFlow),
        rq("助燃空气量", air, Q::MassFlow),
        rq("烟气量", air + fuel, Q::MassFlow),
        rqn("单台负荷率", q_total / installed, Q::Ratio, "影响调节比与火焰形态"),
        txn("下一步", "→ ③ 空气管路与部件选型".to_string(), ""),
    ]
}

fn sd_air_line(c: &Ctx) -> Vec<Row> {
    let q = c.v("q_air");
    let l = c.v("l");
    let v = c.v("v").max(0.1);
    let k = c.v("k_loss");
    let allow = c.v("dp_allow");
    let d = (4.0 * q / (PI * v)).sqrt();
    let f = 0.02;
    let dp = (f * l / d + k) * 1.2 * v * v / 2.0;
    let pass = dp <= allow;
    vec![
        rqn("计算主管内径", d, Q::Length, "按设计流速"),
        txn("建议公称直径", format!("DN {} mm", ((d * 1000.0 / 25.0).ceil() * 25.0) as i32), "向上取整到 DN25 系列"),
        rq("实际流速", q / circle_area(d), Q::Velocity),
        rqn("系统总压降", dp, Q::PressureDrop, "沿程+部件 ΣK"),
        rq("允许压降", allow, Q::PressureDrop),
        txn("校核结论", if pass { "✔ 压降满足" .to_string() } else { "⚠ 需放大管径或降低流速".to_string() }, ""),
        txn("下一步", "→ ④ 鼓风机选型（按总压降）".to_string(), ""),
    ]
}

fn sd_blower_selection(c: &Ctx) -> Vec<Row> {
    let q = c.v("q");
    let dp = c.v("dp");
    let eff = c.v("eff").clamp(0.05, 1.0);
    let margin = c.v("margin");
    let shaft = q * dp / eff;
    let motor = shaft * (1.0 + margin);
    let std_kw = [1.1, 1.5, 2.2, 3.0, 4.0, 5.5, 7.5, 11.0, 15.0, 18.5, 22.0, 30.0, 37.0, 45.0, 55.0, 75.0, 90.0];
    let pick = std_kw.iter().find(|&&x| x >= motor / 1000.0).copied();
    vec![
        rq("轴功率", shaft, Q::Power),
        rq("含裕量电机功率", motor, Q::Power),
        txn("标准电机档", match pick { Some(x) => format!("{x} kW"), None => ">90 kW".to_string() }, ""),
        txn("建议配置", "变频调速 + 入口导叶；并联时校核喘振裕度".to_string(), ""),
        txn("下一步", "→ ⑤ 燃气管路与部件选型".to_string(), ""),
    ]
}

fn sd_fuel_line(c: &Ctx) -> Vec<Row> {
    let qn_ = c.v("q_gas");
    let p_supply = c.v("p_supply");
    let l = c.v("l");
    let v = c.v("v").max(0.1);
    let allow = c.v("dp_allow");
    let p_abs = P_STD + p_supply;
    let q_act = qn_ * (P_STD / p_abs) * (288.15 / T_STD);
    let d = (4.0 * q_act / (PI * v)).sqrt();
    let rho = 0.78 * (p_abs / P_STD) * (T_STD / 288.15);
    let f = 0.02;
    let dp = (f * l / d.max(1e-6) + 2.0) * rho * v * v / 2.0;
    let dp_bar = dp.max(1.0) / 1e5;
    let kv = q_act * 3600.0 / dp_bar.sqrt();
    vec![
        txn("燃气量(标态)", format!("{:.1} Nm³/h", qn_ * 3600.0), ""),
        txn("工况体积流量", format!("{:.1} m³/h", q_act * 3600.0), "含压力/温度状态修正"),
        rqn("计算管内径", d, Q::Length, ""),
        rq("估算压降", dp, Q::PressureDrop),
        rq("允许压降", allow, Q::PressureDrop),
        txn("调压器/切断阀所需 Kv", format!("{:.2} (≈ {:.2} Cv)", kv, kv / u::CV_TO_KV), "低压简化式"),
        txn("校核结论", if dp <= allow { "✔ 压降满足" .to_string() } else { "⚠ 放大管径".to_string() }, ""),
        txn("下一步", "→ ⑥ 设计报告 → ⑦ 安全联锁检查".to_string(), ""),
    ]
}

fn sd_print_preview(c: &Ctx) -> Vec<Row> {
    let q = c.v("q_total");
    let lhv = c.v("lhv").max(1.0);
    let ex = c.v("excess");
    let qa = c.v("q_air");
    let dp = c.v("dp");
    let fuel = q / lhv;
    let air_mass = fuel * 0.34 * (lhv / 1e6) * (1.0 + ex);
    let shaft = qa * dp / 0.68;
    let report = format!(
        "Open Solutions — 燃烧系统设计摘要\n\
         热负荷 {:.0} kW | 燃料 LHV {:.1} MJ/kg | 过量空气 {:.0}%\n\
         燃料 {:.1} kg/h | 助燃空气 {:.0} Nm³/h | 空气 {:.0} kg/h\n\
         系统压降 {:.2} kPa | 风机轴功率 {:.1} kW\n\
         单位制：全部内部量为 SI，界面可切 SI/US；\n\
         数据均为用户录入/通用公式，不含厂商专有数据。",
        q / 1000.0, lhv / 1e6, ex * 100.0,
        fuel * 3600.0, qa * 3600.0, air_mass * 3600.0,
        dp / 1000.0, shaft / 1000.0
    );
    vec![
        rq("总热负荷", q, Q::Power),
        rq("燃料消耗", fuel, Q::MassFlow),
        rq("助燃空气(质量)", air_mass, Q::MassFlow),
        txn("助燃空气(标态体积)", format!("{:.0} Nm³/h", qa * 3600.0), ""),
        rq("风机轴功率", shaft, Q::Power),
        txn("可复制设计摘要", report.replace('\n', " ⏎ "), "完整多行文本见下方"),
        txn("报告全文", report, ""),
        txn("下一步", "→ ⑦ 安全联锁检查（吹扫/检漏/火焰检测）".to_string(), ""),
    ]
}

fn sd_nfpa(c: &Ctx) -> Vec<Row> {
    let vol = c.v("volume");
    let q = c.v("q_purge");
    let n = c.v("changes").max(1.0);
    let p_leak = c.v("p_leak");
    let trials = c.v("trials");
    let v_purge = n * vol;
    let t_purge = v_purge / q.max(1e-9);
    vec![
        rqn("要求吹扫体积", v_purge, Q::Volume, "≥ N×炉膛容积"),
        rqn("最短吹扫时间", t_purge, Q::Time, "t=V_purge/Q"),
        txn("吹扫风量校核", if t_purge < 60.0 { "⚠ 吹扫时间偏短，建议≥60 s".to_string() } else { "✔ 吹扫时间满足".to_string() }, "工程惯例"),
        rq("检漏试验压力(表压)", p_leak, Q::PressureGauge),
        txn("点火尝试上限", format!("{:.0} 次，失败后必须重新吹扫", trials), ""),
        txn("联锁清单 1", "前吹扫完成信号 → 允许点火".to_string(), ""),
        txn("联锁清单 2", "双切断阀 + 阀间放散 + 检漏程序".to_string(), ""),
        txn("联锁清单 3", "火焰检测器 (UV/IR) 熄火 → 立即切断燃料".to_string(), ""),
        txn("联锁清单 4", "低燃气压力 / 低风压 / 限位开关 / 超温".to_string(), ""),
        txn("说明", "仅列通用工程实践要点，不替代 NFPA 86 等现行规范的逐条符合性审查".to_string(), ""),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Map<String, Value> { Map::new() }

    #[test]
    fn every_module_computes_with_defaults() {
        for m in all_modules() {
            let r = compute(m.id, &empty()).unwrap_or_else(|e| panic!("{}: {e}", m.id));
            assert!(!r.rows.is_empty(), "{} produced no rows", m.id);
            for row in &r.rows {
                assert!(!row.si.contains('?'), "{} / {} bad unit format", m.id, row.label);
            }
        }
    }

    #[test]
    fn results_are_finite_or_text() {
        for m in all_modules() {
            let r = compute(m.id, &empty()).unwrap();
            for row in &r.rows {
                if row.us.is_empty() { continue; }
                assert!(!row.si.contains("NaN") && !row.us.contains("NaN"), "{} / {} is NaN", m.id, row.label);
                assert!(!row.si.contains("inf") && !row.us.contains("inf"), "{} / {} is inf", m.id, row.label);
            }
        }
    }

    /// The regression that matters: the SAME physical case entered in SI and
    /// in US units must produce identical canonical results.
    #[test]
    fn si_and_us_inputs_agree() {
        let mut si = Map::new();
        let mut us = Map::new();
        let put = |m: &mut Map<String, Value>, k: &str, v: f64, unit: &str| {
            m.insert(k.into(), serde_json::json!({ "value": v, "unit": unit }));
        };
        // fluid_mechanics: 100 mm = 3.937007874 in, 100 m3/h = 58.860... cfm
        put(&mut si, "d", 100.0, "mm");
        put(&mut us, "d", 100.0 / 25.4, "in");
        put(&mut si, "q", 100.0, "m3/h");
        put(&mut us, "q", 100.0 * 35.3146667 / 60.0, "cfm");
        put(&mut si, "l", 20.0, "m");
        put(&mut us, "l", 20.0 / 0.3048, "ft");
        let a = compute("fluid_mechanics", &si).unwrap();
        let b = compute("fluid_mechanics", &us).unwrap();
        assert_eq!(a.rows[0].si, b.rows[0].si, "velocity differs between SI and US inputs");
        assert_eq!(a.rows[6].si, b.rows[6].si, "pressure drop differs between SI and US inputs");
    }

    #[test]
    fn pressure_conversion_is_pure_factor_and_gauge_is_typed_elsewhere() {
        let mut m = Map::new();
        m.insert("value".into(), serde_json::json!({ "value": 0.0, "unit": "-" }));
        m.insert("from".into(), serde_json::json!({ "value": 17.0, "unit": "-" })); // psig
        m.insert("to".into(), serde_json::json!({ "value": 14.0, "unit": "-" }));  // kPa
        let r = compute("unit_conversion", &m).unwrap();
        assert!(r.rows[0].si.contains("psig = 0 kPa") || r.rows[0].si.contains("psig = 0.000000 kPa"), "unexpected: {}", r.rows[0].si);
        // Absolute/gauge semantics are enforced by the typed boundary:
        assert!((Q::PressureAbs.to_si(0.0, "psig").unwrap() - P_STD).abs() < 1e-9);
        assert!(Q::PressureAbs.to_si(0.0, "psia").unwrap().abs() < 1e-9);
    }

    /// Energy Savings kernel vs. source-program readings (59-point sweep).
    fn es_case_ii(case1: Option<(usize, f64, f64, f64, f64, f64, f64)>,
                  case2: (usize, f64, f64, f64, f64, f64, f64)) -> (f64, f64, f64) {
        // tuple: (fuel_idx, ea%, exh F, air F, o2%, fgr%, tfgr F)
        let mut m = Map::new();
        let set = |m: &mut Map<String, Value>, k: &str, v: serde_json::Value| { m.insert(k.into(), v); };
        let fuel = |i: usize| serde_json::json!({ "value": i as f64, "unit": "-" });
        let pct = |v: f64| serde_json::json!({ "value": v, "unit": "%" });
        let tmp = |v: f64| serde_json::json!({ "value": v, "unit": "F" });
        if let Some(c1) = case1 {
            set(&mut m, "fuel1", fuel(c1.0)); set(&mut m, "ea1", pct(c1.1));
            set(&mut m, "exh1", tmp(c1.2)); set(&mut m, "air1", tmp(c1.3));
            set(&mut m, "o21", pct(c1.4)); set(&mut m, "fgr1", pct(c1.5));
            set(&mut m, "tfgr1", tmp(c1.6));
        }
        set(&mut m, "fuel2", fuel(case2.0)); set(&mut m, "ea2", pct(case2.1));
        set(&mut m, "exh2", tmp(case2.2)); set(&mut m, "air2", tmp(case2.3));
        set(&mut m, "o22", pct(case2.4)); set(&mut m, "fgr2", pct(case2.5));
        set(&mut m, "tfgr2", tmp(case2.6));
        let r = compute("energy_savings", &m).unwrap();
        // row order: fuel I, fuel II, AH I, AH II, savings
        let parse = |lbl: &str| {
            let row = r.rows.iter().find(|x| x.label.starts_with(lbl)).unwrap();
            row.si.split(' ').next().unwrap().parse::<f64>().unwrap()
        };
        (parse("Available Heat Case I"), parse("Available Heat Case II"), parse("Energy Savings"))
    }

    #[test]
    fn energy_savings_matches_source_sweep() {
        use crate::combustion_fuels::ES_FUEL_NAMES;
        let idx = |n: &str| ES_FUEL_NAMES.iter().position(|x| *x == n).unwrap();
        // Case I baseline (air60/exh1800/ea10) unless overridden.
        let base1 = Some((idx("Average Natural Gas"), 10.0, 1800.0, 60.0, 20.9, 0.0, 60.0));
        let ang = idx("Average Natural Gas");
        // (case II variant, source AH_II, tolerance)
        let cases: Vec<((usize, f64, f64, f64, f64, f64, f64), f64, f64)> = vec![
            ((ang, 10.0, 1800.0, 60.0, 20.9, 0.0, 60.0), 48.0, 0.3),
            ((ang, 10.0, 1800.0, 800.0, 20.9, 0.0, 60.0), 62.4, 0.5),
            ((ang, 10.0, 1800.0, 1000.0, 20.9, 0.0, 60.0), 66.6, 0.5),
            ((ang, 10.0, 600.0, 60.0, 20.9, 0.0, 60.0), 79.3, 0.5),
            ((ang, 10.0, 3000.0, 60.0, 20.9, 0.0, 60.0), 11.2, 2.1), // dissociation-region edge, fit residual +1.96
            ((ang, 0.0, 1800.0, 60.0, 20.9, 0.0, 60.0), 51.2, 0.4),
            ((ang, 100.0, 1800.0, 60.0, 20.9, 0.0, 60.0), 18.4, 1.0),
            ((ang, 10.0, 1800.0, 60.0, 30.0, 0.0, 60.0), 61.1, 1.0),
            ((ang, 10.0, 1800.0, 60.0, 20.9, 20.0, 400.0), 39.1, 0.7),
            ((ang, 10.0, 1800.0, 60.0, 20.9, 40.0, 400.0), 30.3, 1.0),
            ((ang, 20.0, 1400.0, 600.0, 25.0, 20.0, 400.0), 68.7, 1.0),
        ];
        for (c2, src, tol) in cases {
            let (ah1, ah2, _sav) = es_case_ii(base1, c2);
            assert!((ah1 - 48.0).abs() < 0.3, "Case I baseline drifted: {ah1}");
            assert!((ah2 - src).abs() < tol, "AH_II {:?}: got {ah2}, source {src}", ES_FUEL_NAMES[c2.0]);
        }
        // Fuel anchors at baseline conditions.
        for (name, src, tol) in [
            ("Methane", 47.8, 0.3), ("Butane", 49.3, 0.4), ("Blast Furnace Gas", 26.1, 0.8),
            ("Propane", 49.7, 0.4), ("Coke Oven Gas", 48.8, 0.4), ("Producer, Lurgi", 47.2, 0.7),
        ] {
            let (ah1, ah2, _sav) = es_case_ii(base1, (idx(name), 10.0, 1800.0, 60.0, 20.9, 0.0, 60.0));
            assert!((ah1 - 48.0).abs() < 0.3, "{name}: Case I drifted: {ah1}");
            assert!((ah2 - src).abs() < tol, "{name}: AH_II got {ah2}, source {src}");
        }
        // Savings formula: air 800 F => 1 - 48.0/62.4 = 23.1 %.
        let (ah1, ah2, sav) = es_case_ii(base1, (ang, 10.0, 1800.0, 800.0, 20.9, 0.0, 60.0));
        assert!((sav - (1.0 - ah1 / ah2) * 100.0).abs() < 1e-6);
        assert!((sav - 23.1).abs() < 0.6, "savings got {sav}, source 23.1");
    }

    /// Wobbe Index vs. the source-program dump (Wobbe_Index_dump.json):
    /// Average Natural Gas ↔ Butane.
    #[test]
    fn wobbe_matches_source_anchor() {
        use crate::combustion_fuels::WOBBE_GAS_NAMES;
        let idx = |n: &str| WOBBE_GAS_NAMES.iter().position(|x| *x == n).unwrap();
        let mut m = Map::new();
        m.insert("fuel_o".into(), serde_json::json!({ "value": idx("Average Natural Gas") as f64, "unit": "-" }));
        m.insert("fuel_s".into(), serde_json::json!({ "value": idx("Butane") as f64, "unit": "-" }));
        let r = compute("wobbe_index", &m).unwrap();
        let find = |key: &str| r.rows.iter().find(|x| x.label.starts_with(key)).map(|x| x.si.clone());
        assert_eq!(find("Original Wobbe").unwrap(), "1323.3 Btu/scf");
        assert_eq!(find("Substitute Wobbe").unwrap(), "2262.74 Btu/scf");
        assert_eq!(find("Gas/Air Mix Substitute").unwrap(), "50.8% Butane");
        assert_eq!(find("Mix Calorific Value").unwrap(), "1625 Btu/scf");
        assert_eq!(find("Mix Specific Gravity").unwrap(), "1.508");
        assert_eq!(find("Mix Wobbe").unwrap(), "1323.37 Btu/scf");
        // Infeasible pair: Coke Oven Gas Wobbe (1028) below ANG — air dilution
        // cannot raise it back up.
        let mut m2 = Map::new();
        m2.insert("fuel_o".into(), serde_json::json!({ "value": idx("Average Natural Gas") as f64, "unit": "-" }));
        m2.insert("fuel_s".into(), serde_json::json!({ "value": idx("Coke Oven Gas") as f64, "unit": "-" }));
        let r2 = compute("wobbe_index", &m2).unwrap();
        assert!(r2.rows.iter().any(|x| x.si.contains("Not feasible")), "expected infeasible verdict");
    }

    /// Burners & Valves at Altitude vs. the source dump (BURNER SIZING):
    /// 9000 ft / 60 F / EA 10 % / 0.2 MMBtu/h / catalog 2000 scfh @ 12.
    #[test]
    fn altitude_burners_matches_source_anchor() {
        let put = |m: &mut Map<String, Value>, k: &str, v: f64, unit: &str| {
            m.insert(k.into(), serde_json::json!({ "value": v, "unit": unit }));
        };
        let mut m = Map::new();
        m.insert("fuel".into(), serde_json::json!({ "value": 0.0, "unit": "-" })); // Average Natural Gas
        put(&mut m, "elev", 9000.0, "ft");
        put(&mut m, "t_air", 60.0, "F");
        put(&mut m, "exc_air", 10.0, "%");
        put(&mut m, "cap", 0.2, "MM Btu/h");
        put(&mut m, "cat_flow", 2000.0, "scfh");
        put(&mut m, "cat_press", 12.0, "inwc");
        let r = compute("altitude_burners", &m).unwrap();
        let find = |key: &str| r.rows.iter().find(|x| x.label.starts_with(key)).map(|x| x.si.clone());
        assert_eq!(find("Actual Burner Air Flow").unwrap(), "2900 acfh");
        assert_eq!(find("Burner Air Flow Rate Required").unwrap(), "2071 scfh");
        assert_eq!(find("Burner Air Pressure Required").unwrap(), "18.0 inwc");
    }
}
