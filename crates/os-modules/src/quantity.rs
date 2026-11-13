//! Boundary unit conversion for the data-driven module engine.
//!
//! Every GUI field carries an explicit `Quantity` and an explicit unit string.
//! Values are converted to the canonical SI representation EXACTLY ONCE here,
//! at the input boundary; results are converted back only for display. This is
//! the GUI-level enforcement of the same rule `os-core::units` implements with
//! distinct types. Affine quantities (temperature, gauge pressure) and
//! non-linear ones (area, volume, flow coefficients) are NOT simple factors.

use os_core::units as u;
use os_core::units::UnitSystem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quantity {
    Dimensionless,
    Ratio,
    Percent,
    Angle,
    Length,
    Area,
    Volume,
    Temperature,
    TemperatureDiff,
    PressureAbs,
    PressureGauge,
    PressureDrop,
    MassFlow,
    VolumetricFlow,
    Density,
    Velocity,
    Power,
    PowerPerArea,
    Energy,
    HeatingValue,
    SpecificHeat,
    ThermalConductivity,
    HeatTransferCoeff,
    DynamicViscosity,
    Mass,
    Force,
    Time,
    FlowCoefficient,
    MolecularWeight,
    Emission,
    AngleDeg,
    SoundLevel,
    Frequency,
}

impl Quantity {
    pub fn si_unit(self) -> &'static str {
        match self {
            Q::Dimensionless | Q::Ratio => "-",
            Q::Percent => "frac",
            Q::Angle | Q::AngleDeg => "deg",
            Q::Length => "m",
            Q::Area => "m2",
            Q::Volume => "m3",
            Q::Temperature | Q::TemperatureDiff => "K",
            Q::PressureAbs | Q::PressureGauge | Q::PressureDrop => "Pa",
            Q::MassFlow => "kg/s",
            Q::VolumetricFlow => "m3/s",
            Q::Density => "kg/m3",
            Q::Velocity => "m/s",
            Q::Power => "W",
            Q::PowerPerArea => "W/m2",
            Q::Energy => "J",
            Q::HeatingValue => "J/kg",
            Q::SpecificHeat => "J/(kg.K)",
            Q::ThermalConductivity => "W/(m.K)",
            Q::HeatTransferCoeff => "W/(m2.K)",
            Q::DynamicViscosity => "Pa.s",
            Q::Mass => "kg",
            Q::Force => "N",
            Q::Time => "s",
            Q::FlowCoefficient => "Kv",
            Q::MolecularWeight => "kg/kmol",
            Q::Emission => "mg/Nm3",
            Q::SoundLevel => "dB",
            Q::Frequency => "Hz",
        }
    }

    /// Unit shown in the GUI for the selected system. Inputs are always parsed
    /// with THIS exact string, so a user can never be surprised by an implicit
    /// conversion. A suffix typed or selected in the other system still works.
    pub fn display_unit(self, system: UnitSystem) -> &'static str {
        match system {
            UnitSystem::Si => self.si_display(),
            UnitSystem::Imperial => self.us_display(),
        }
    }

    fn si_display(self) -> &'static str {
        match self {
            Q::Dimensionless | Q::Ratio => "-",
            Q::Percent => "%",
            Q::Angle | Q::AngleDeg => "deg",
            Q::Length => "mm",
            Q::Area => "m2",
            Q::Volume => "m3",
            Q::Temperature | Q::TemperatureDiff => "C",
            Q::PressureAbs | Q::PressureGauge | Q::PressureDrop => "kPa",
            Q::MassFlow => "kg/h",
            Q::VolumetricFlow => "m3/h",
            Q::Density => "kg/m3",
            Q::Velocity => "m/s",
            Q::Power => "kW",
            Q::PowerPerArea => "W/m2",
            Q::Energy => "MJ",
            Q::HeatingValue => "MJ/kg",
            Q::SpecificHeat => "kJ/(kg.K)",
            Q::ThermalConductivity => "W/(m.K)",
            Q::HeatTransferCoeff => "W/(m2.K)",
            Q::DynamicViscosity => "cP",
            Q::Mass => "kg",
            Q::Force => "N",
            Q::Time => "h",
            Q::FlowCoefficient => "Kv",
            Q::MolecularWeight => "kg/kmol",
            Q::Emission => "mg/Nm3",
            Q::SoundLevel => "dB",
            Q::Frequency => "Hz",
        }
    }

    fn us_display(self) -> &'static str {
        match self {
            Q::Dimensionless | Q::Ratio => "-",
            Q::Percent => "%",
            Q::Angle | Q::AngleDeg => "deg",
            Q::Length => "in",
            Q::Area => "ft2",
            Q::Volume => "ft3",
            Q::Temperature | Q::TemperatureDiff => "F",
            Q::PressureAbs => "psia",
            Q::PressureGauge | Q::PressureDrop => "psi",
            Q::MassFlow => "lb/h",
            Q::VolumetricFlow => "cfm",
            Q::Density => "lb/ft3",
            Q::Velocity => "ft/s",
            Q::Power => "BTU/h",
            Q::PowerPerArea => "BTU/(h.ft2)",
            Q::Energy => "BTU",
            Q::HeatingValue => "BTU/lb",
            Q::SpecificHeat => "BTU/(lb.F)",
            Q::ThermalConductivity => "BTU/(h.ft.F)",
            Q::HeatTransferCoeff => "BTU/(h.ft2.F)",
            Q::DynamicViscosity => "cP",
            Q::Mass => "lb",
            Q::Force => "lbf",
            Q::Time => "h",
            Q::FlowCoefficient => "Cv",
            Q::MolecularWeight => "lb/lbmol",
            Q::Emission => "mg/Nm3",
            Q::SoundLevel => "dB",
            Q::Frequency => "Hz",
        }
    }

    /// Convert a user-entered value into the canonical SI representation.
    pub fn to_si(self, v: f64, unit: &str) -> Result<f64, String> {
        let s = norm(unit);
        let m = s.as_str();
        let ok = |x: f64| Ok(x);
        match self {
            Q::Dimensionless | Q::Ratio | Q::SoundLevel => ok(v),
            Q::Percent => match m {
                "%" | "pct" => ok(v / 100.0),
                "frac" | "-" | "" => ok(v),
                _ => unknown(unit),
            },
            Q::Angle | Q::AngleDeg => match m {
                "deg" | "°" | "degree" => ok(v),
                "rad" => ok(v.to_degrees()),
                _ => unknown(unit),
            },
            Q::Length => match m {
                "m" => ok(u::Length::from_m(v).as_m()),
                "mm" => ok(u::Length::from_mm(v).as_m()),
                "cm" => ok(u::Length::from_cm(v).as_m()),
                "in" | "inch" | "\"" => ok(u::Length::from_inch(v).as_m()),
                "ft" | "feet" | "'" => ok(u::Length::from_ft(v).as_m()),
                _ => unknown(unit),
            },
            Q::Area => match m {
                "m2" => ok(v),
                "mm2" => ok(u::Area::from_mm2(v).as_m2()),
                "cm2" => ok(v * 1e-4),
                "in2" => ok(u::Area::from_in2(v).as_m2()),
                "ft2" => ok(v * u::FT_IN_M * u::FT_IN_M),
                _ => unknown(unit),
            },
            Q::Volume => match m {
                "m3" => ok(v),
                "l" | "liter" | "litre" => ok(v * 1e-3),
                "ft3" => ok(u::Volume::from_ft3(v).as_m3()),
                "in3" => ok(v * u::INCH_IN_M.powi(3)),
                "gal" | "usgal" => ok(v * 0.003_785_411_784),
                _ => unknown(unit),
            },
            Q::Temperature => match m {
                "k" | "kelvin" => ok(v),
                "c" | "°c" | "degc" => ok(u::Temperature::from_c(v).as_k()),
                "f" | "°f" | "degf" => ok(u::Temperature::from_f(v).as_k()),
                _ => unknown(unit),
            },
            Q::TemperatureDiff => match m {
                "k" | "c" | "°c" | "degc" | "dc" | "deltac" => ok(v),
                "f" | "°f" | "degf" | "df" | "deltaf" => ok(v * 5.0 / 9.0),
                _ => unknown(unit),
            },
            Q::PressureAbs => match m {
                "pa" => ok(v),
                "kpa" => ok(u::Pressure::from_kpa_abs(v).as_pa_abs()),
                "mpa" => ok(v * 1e6),
                "bar" | "bara" => ok(u::Pressure::from_bar_abs(v).as_pa_abs()),
                "psia" => ok(u::Pressure::from_psia(v).as_pa_abs()),
                "psig" => ok(u::Pressure::from_psig(v).as_pa_abs()),
                "inhg" => ok(v * 3386.389),
                "mmh2o" => ok(v * 9.80665),
                _ => unknown(unit),
            },
            Q::PressureGauge | Q::PressureDrop => match m {
                "pa" => ok(v),
                "kpa" | "kpag" => ok(v * 1000.0),
                "bar" | "barg" => ok(v * 1e5),
                "mpa" => ok(v * 1e6),
                "psi" | "psig" | "psid" => ok(v * u::PSI_IN_PA),
                "psia" => ok(v * u::PSI_IN_PA - u::P_STD_PA),
                "inwc" | "inh2o" => ok(v * 249.0889),
                "mmh2o" | "mmwc" => ok(v * 9.80665),
                _ => unknown(unit),
            },
            Q::MassFlow => match m {
                "kg/s" => ok(v),
                "kg/h" => ok(u::MassFlow::from_kg_h(v).as_kg_s()),
                "kg/min" => ok(v / 60.0),
                "t/h" => ok(v * 1000.0 / 3600.0),
                "lb/h" => ok(u::MassFlow::from_lb_h(v).as_kg_s()),
                "lb/s" => ok(u::MassFlow::from_lb_s(v).as_kg_s()),
                _ => unknown(unit),
            },
            Q::VolumetricFlow => match m {
                "m3/s" => ok(v),
                "m3/h" | "nm3/h" | "sm3/h" => ok(v / 3600.0),
                "l/s" => ok(v * 1e-3),
                "l/min" => ok(v * 1e-3 / 60.0),
                "cfm" | "ft3/min" | "scfm" => ok(v * u::CFM_IN_M3S),
                "cfh" | "ft3/h" | "scfh" => ok(v * u::FT3_IN_M3 / 3600.0),
                _ => unknown(unit),
            },
            Q::Density => match m {
                "kg/m3" => ok(v),
                "g/cm3" => ok(v * 1000.0),
                "lb/ft3" => ok(u::Density::from_lb_ft3(v).as_kg_m3()),
                _ => unknown(unit),
            },
            Q::Velocity => match m {
                "m/s" => ok(v),
                "m/h" => ok(v / 3600.0),
                "km/h" => ok(v / 3.6),
                "ft/s" => ok(v * u::FT_IN_M),
                "ft/min" => ok(v * u::FT_IN_M / 60.0),
                "mph" => ok(v * 1609.344 / 3600.0),
                _ => unknown(unit),
            },
            Q::Power => match m {
                "w" => ok(v),
                "kw" => ok(u::Power::from_kw(v).as_w()),
                "mw" => ok(u::Power::from_mw(v).as_w()),
                "btu/h" => ok(u::Power::from_btu_h(v).as_w()),
                "mm btu/h" | "mmbtu/h" | "mm btu/hr" => ok(u::Power::from_btu_h(v * 1_000_000.0).as_w()),
                "hp" => ok(u::Power::from_hp(v).as_w()),
                _ => unknown(unit),
            },
            Q::PowerPerArea => match m {
                "w/m2" => ok(v),
                "kw/m2" => ok(v * 1000.0),
                "btu/(h.ft2)" | "btu/h.ft2" => ok(v * u::BTU_IN_J / 3600.0 / (u::FT_IN_M * u::FT_IN_M)),
                _ => unknown(unit),
            },
            Q::Energy => match m {
                "j" => ok(v),
                "kj" => ok(v * 1e3),
                "mj" => ok(v * 1e6),
                "gj" => ok(v * 1e9),
                "kwh" => ok(v * 3.6e6),
                "btu" => ok(v * u::BTU_IN_J),
                _ => unknown(unit),
            },
            Q::HeatingValue => match m {
                "j/kg" => ok(v),
                "kj/kg" => ok(v * 1e3),
                "mj/kg" => ok(u::HeatingValue::from_mj_kg(v, u::HeatingBasis::Lhv).as_j_kg()),
                "btu/lb" => ok(u::HeatingValue::from_btu_lb(v, u::HeatingBasis::Lhv).as_j_kg()),
                "mj/nm3" | "mj/m3" => ok(v * 1e6),
                "btu/ft3" | "btu/scf" => ok(v * u::BTU_IN_J / u::FT3_IN_M3),
                _ => unknown(unit),
            },
            Q::SpecificHeat => match m {
                "j/(kg.k)" => ok(v),
                "kj/(kg.k)" | "kj/kg.k" => ok(v * 1e3),
                "btu/(lb.f)" | "btu/lb.f" => ok(v * u::BTU_IN_J / (u::LB_IN_KG * 5.0 / 9.0)),
                _ => unknown(unit),
            },
            Q::ThermalConductivity => match m {
                "w/(m.k)" | "w/m.k" => ok(v),
                "btu/(h.ft.f)" | "btu/h.ft.f" => ok(v * u::BTU_IN_J / 3600.0 / u::FT_IN_M / (5.0 / 9.0)),
                _ => unknown(unit),
            },
            Q::HeatTransferCoeff => match m {
                "w/(m2.k)" | "w/m2.k" => ok(v),
                "btu/(h.ft2.f)" | "btu/h.ft2.f" => {
                    ok(v * u::BTU_IN_J / 3600.0 / (u::FT_IN_M * u::FT_IN_M) / (5.0 / 9.0))
                }
                _ => unknown(unit),
            },
            Q::DynamicViscosity => match m {
                "pa.s" => ok(v),
                "cp" | "mpa.s" => ok(v * 1e-3),
                "lb/(ft.h)" => ok(v * u::LB_IN_KG / (u::FT_IN_M * 3600.0)),
                _ => unknown(unit),
            },
            Q::Mass => match m {
                "kg" => ok(v),
                "g" => ok(v * 1e-3),
                "t" | "tonne" => ok(v * 1000.0),
                "lb" => ok(v * u::LB_IN_KG),
                _ => unknown(unit),
            },
            Q::Force => match m {
                "n" => ok(v),
                "kn" => ok(v * 1000.0),
                "lbf" => ok(v * 4.448_221_615_260_5),
                _ => unknown(unit),
            },
            Q::Time => match m {
                "s" => ok(v),
                "min" => ok(v * 60.0),
                "h" | "hr" => ok(v * 3600.0),
                _ => unknown(unit),
            },
            Q::FlowCoefficient => match m {
                "kv" => ok(v),
                "cv" => ok(u::cv_to_kv(v)),
                _ => unknown(unit),
            },
            Q::MolecularWeight => ok(v),
            Q::Emission => match m {
                "mg/nm3" | "mg/m3" => ok(v),
                "g/m3" => ok(v * 1000.0),
                "ug/m3" | "µg/m3" => ok(v * 1e-3),
                _ => unknown(unit),
            },
            Q::Frequency => match m {
                "hz" => ok(v),
                "rpm" => ok(v / 60.0),
                _ => unknown(unit),
            },
        }
    }

    /// Convert a canonical SI value into a display unit (inverse of `to_si`).
    pub fn from_si(self, v: f64, unit: &str) -> Result<f64, String> {
        // A single affine/scale inverse is enough for every unit here:
        // si = a*v + b  =>  v = (si - b)/a.
        let a = self.to_si(1.0, unit)?;
        let b = self.to_si(0.0, unit)?;
        let scale = a - b;
        if scale == 0.0 {
            return Err(format!("unit {unit} has no inverse scale"));
        }
        Ok((v - b) / scale)
    }

    /// Nicely formatted value in a given unit.
    pub fn format(self, v_si: f64, unit: &str) -> String {
        match self.to_si(v_si, unit) {
            Err(_) => format!("{v_si:.6} ?"),
            Ok(_) => match self.from_si(v_si, unit) {
                Err(_) => format!("{v_si:.6} ?"),
                Ok(x) => format!("{} {unit}", smart_fmt(x)),
            },
        }
    }

    /// All units the user may pick for this quantity (mixed systems allowed).
    pub fn available_units(self) -> &'static [&'static str] {
        match self {
            Q::Dimensionless | Q::Ratio => &["-"],
            Q::Percent => &["%", "frac"],
            Q::Angle | Q::AngleDeg => &["deg", "rad"],
            Q::Length => &["m", "mm", "cm", "in", "ft"],
            Q::Area => &["m2", "mm2", "cm2", "in2", "ft2"],
            Q::Volume => &["m3", "L", "ft3", "in3", "gal"],
            Q::Temperature => &["C", "F", "K"],
            Q::TemperatureDiff => &["K", "C", "F"],
            Q::PressureAbs => &["kPa", "Pa", "bar", "MPa", "psia", "psig", "inHg", "mmH2O"],
            Q::PressureGauge | Q::PressureDrop => {
                &["kPa", "Pa", "bar", "psi", "inWC", "mmH2O", "psia"]
            }
            Q::MassFlow => &["kg/s", "kg/h", "t/h", "lb/h", "lb/s"],
            Q::VolumetricFlow => &["m3/s", "m3/h", "Nm3/h", "L/s", "cfm", "SCFM", "cfh"],
            Q::Density => &["kg/m3", "g/cm3", "lb/ft3"],
            Q::Velocity => &["m/s", "m/h", "km/h", "ft/s", "ft/min", "mph"],
            Q::Power => &["W", "kW", "MW", "BTU/h", "hp"],
            Q::PowerPerArea => &["W/m2", "kW/m2", "BTU/(h.ft2)"],
            Q::Energy => &["J", "kJ", "MJ", "GJ", "kWh", "BTU"],
            Q::HeatingValue => &["J/kg", "kJ/kg", "MJ/kg", "BTU/lb"],
            Q::SpecificHeat => &["J/(kg.K)", "kJ/(kg.K)", "BTU/(lb.F)"],
            Q::ThermalConductivity => &["W/(m.K)", "BTU/(h.ft.F)"],
            Q::HeatTransferCoeff => &["W/(m2.K)", "BTU/(h.ft2.F)"],
            Q::DynamicViscosity => &["Pa.s", "cP", "lb/(ft.h)"],
            Q::Mass => &["kg", "g", "t", "lb"],
            Q::Force => &["N", "kN", "lbf"],
            Q::Time => &["s", "min", "h"],
            Q::FlowCoefficient => &["Kv", "Cv"],
            Q::MolecularWeight => &["kg/kmol", "g/mol", "lb/lbmol"],
            Q::Emission => &["mg/Nm3", "g/m3", "ug/m3"],
            Q::SoundLevel => &["dB"],
            Q::Frequency => &["Hz", "rpm"],
        }
    }
}

// Short alias used throughout the implementation above.
use Quantity as Q;

fn norm(unit: &str) -> String {
    unit.trim()
        .replace('°', "")
        .replace(' ', "")
        .replace('\u{00B5}', "u")
        .to_ascii_lowercase()
}

fn unknown<T>(unit: &str) -> Result<T, String> {
    Err(format!("未知单位 / unknown unit: {unit}"))
}

/// 4-6 significant digits, fixed notation for everyday magnitudes.
pub fn smart_fmt(x: f64) -> String {
    if !x.is_finite() {
        return "—".to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let a = x.abs();
    if a != 0.0 && (a < 1e-4 || a >= 1e7) {
        format!("{x:.4e}")
    } else if a >= 1000.0 {
        format!("{x:.2}")
    } else if a >= 10.0 {
        format!("{x:.3}")
    } else if a >= 0.1 {
        format!("{x:.5}")
    } else {
        format!("{x:.6}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn affine_temperature_roundtrip() {
        let k = Q::Temperature.to_si(212.0, "F").unwrap();
        assert!(close(k, 373.15));
        assert!(close(Q::Temperature.from_si(k, "C").unwrap(), 100.0));
        assert!(close(Q::Temperature.from_si(k, "F").unwrap(), 212.0));
    }

    #[test]
    fn gauge_zero_is_atmospheric() {
        assert!(close(Q::PressureAbs.to_si(0.0, "psig").unwrap(), u::P_STD_PA));
        assert!(close(Q::PressureAbs.to_si(0.0, "psia").unwrap(), 0.0));
        assert!((Q::PressureAbs.to_si(14.696, "psia").unwrap() / 1000.0 - 101.325).abs() < 0.001);
    }

    #[test]
    fn squared_and_cubed_units() {
        assert!(close(Q::Area.to_si(1.0, "in2").unwrap() * 1e6, 645.16));
        assert!(close(Q::Volume.to_si(1.0, "ft3").unwrap(), u::FT3_IN_M3));
    }

    #[test]
    fn flow_coefficient_uses_cv_kv_not_linear() {
        assert!(close(Q::FlowCoefficient.to_si(1.0, "Cv").unwrap(), 0.865));
        assert!(close(
            Q::FlowCoefficient.from_si(Q::FlowCoefficient.to_si(12.3, "Cv").unwrap(), "Cv").unwrap(),
            12.3
        ));
    }

    #[test]
    fn si_and_us_inputs_agree() {
        let si = Q::MassFlow.to_si(1000.0, "kg/h").unwrap();
        let us = Q::MassFlow.to_si(1000.0 / u::LB_IN_KG, "lb/h").unwrap();
        assert!(close(si, us));
        let p_si = Q::HeatingValue.to_si(50.0, "MJ/kg").unwrap();
        let p_us = Q::HeatingValue
            .to_si(50e6 * u::LB_IN_KG / u::BTU_IN_J, "BTU/lb")
            .unwrap();
        assert!((p_si - p_us).abs() / p_si < 1e-12);
    }

    #[test]
    fn every_display_unit_is_parseable() {
        use UnitSystem::{Imperial, Si};
        for q in [
            Q::Dimensionless, Q::Percent, Q::Length, Q::Area, Q::Volume,
            Q::Temperature, Q::PressureAbs, Q::PressureGauge, Q::PressureDrop,
            Q::MassFlow, Q::VolumetricFlow, Q::Density, Q::Velocity, Q::Power,
            Q::Energy, Q::HeatingValue, Q::SpecificHeat, Q::ThermalConductivity,
            Q::HeatTransferCoeff, Q::DynamicViscosity, Q::Mass, Q::Force,
            Q::Time, Q::FlowCoefficient, Q::Emission, Q::PowerPerArea,
        ] {
            for s in [Si, Imperial] {
                let unit = q.display_unit(s);
                assert!(q.to_si(1.0, unit).is_ok(), "{q:?} {unit}");
                let si = q.to_si(42.0, unit).unwrap();
                assert!(close(q.from_si(si, unit).unwrap(), 42.0), "{q:?} {unit}");
            }
            for unit in q.available_units() {
                assert!(q.to_si(1.0, unit).is_ok(), "{q:?} {unit}");
            }
        }
    }
}
