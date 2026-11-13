//! Type-safe units — this module *is* the fix for the imperial/metric bug.
//!
//! Root cause in the legacy program: bare `f64` values carrying an implicit
//! unit, converted ad hoc at every step. Symptoms: double conversion, °C/°F
//! treated as a pure scale factor, gauge vs absolute pressure mixed, standard
//! vs actual volumetric flow mixed, LHV/HHV silently swapped, and
//! squared/cubed conversions done linearly.
//!
//! Rule here: **one canonical SI representation per quantity**; conversion
//! happens ONLY at the I/O boundary and exactly once. Quantities are distinct
//! types, so passing a length where a pressure is expected will not compile.

use std::fmt;

// ---- Constants (exact where an exact definition exists) -------------------

pub const T_ZERO_C_IN_K: f64 = 273.15;
pub const R_UNIVERSAL_J_PER_MOL_K: f64 = 8.314_462_618;
pub const P_STD_PA: f64 = 101_325.0;
pub const P_STD_KPA: f64 = 101.325;
pub const T_STD_K: f64 = 273.15;

pub const INCH_IN_M: f64 = 0.0254;          // exact
pub const FT_IN_M: f64 = 0.3048;            // exact
pub const LB_IN_KG: f64 = 0.453_592_37;     // exact
pub const BTU_IN_J: f64 = 1055.055_852_62;  // IT Btu
pub const HP_IN_W: f64 = 745.699_871_582_270_2;
pub const PSI_IN_PA: f64 = 6894.757_293_168;
pub const FT3_IN_M3: f64 = 0.028_316_846_592;
pub const CFM_IN_M3S: f64 = FT3_IN_M3 / 60.0;
pub const CV_TO_KV: f64 = 0.865;            // Cv (US gpm/psi) -> Kv (m3/h/bar)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem { Si, Imperial }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureBasis { Absolute, Gauge }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowBasis { Standard, Actual }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatingBasis { Lhv, Hhv }

#[derive(Debug, Clone, PartialEq)]
pub struct UnitError(pub String);

impl fmt::Display for UnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for UnitError {}

// ---- Length / Area / Volume ----------------------------------------------

/// Canonical: metre.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length(f64);

impl Length {
    pub fn from_m(v: f64) -> Self { Self(v) }
    pub fn from_mm(v: f64) -> Self { Self(v * 1e-3) }
    pub fn from_cm(v: f64) -> Self { Self(v * 1e-2) }
    pub fn from_inch(v: f64) -> Self { Self(v * INCH_IN_M) }
    pub fn from_ft(v: f64) -> Self { Self(v * FT_IN_M) }
    pub fn as_m(self) -> f64 { self.0 }
    pub fn as_mm(self) -> f64 { self.0 * 1e3 }
    pub fn as_cm(self) -> f64 { self.0 * 1e2 }
    pub fn as_inch(self) -> f64 { self.0 / INCH_IN_M }
    pub fn as_ft(self) -> f64 { self.0 / FT_IN_M }
    pub fn squared(self) -> Area { Area::from_m2(self.0 * self.0) }
    pub fn cubed(self) -> Volume { Volume::from_m3(self.0 * self.0 * self.0) }
    /// Area of a circle whose diameter is this length (pipe bore).
    pub fn circle_area(self) -> Area {
        Area::from_m2(std::f64::consts::PI * self.0 * self.0 / 4.0)
    }
}

/// Canonical: m². Squared conversions are inherently non-linear.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Area(f64);

impl Area {
    pub fn from_m2(v: f64) -> Self { Self(v) }
    pub fn from_mm2(v: f64) -> Self { Self(v * 1e-6) }
    pub fn from_in2(v: f64) -> Self { Self(v * INCH_IN_M * INCH_IN_M) }
    pub fn as_m2(self) -> f64 { self.0 }
    pub fn as_mm2(self) -> f64 { self.0 * 1e6 }
    pub fn as_in2(self) -> f64 { self.0 / (INCH_IN_M * INCH_IN_M) }
}

/// Canonical: m³. Cubed conversions are inherently non-linear.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume(f64);

impl Volume {
    pub fn from_m3(v: f64) -> Self { Self(v) }
    pub fn from_ft3(v: f64) -> Self { Self(v * FT3_IN_M3) }
    pub fn as_m3(self) -> f64 { self.0 }
    pub fn as_ft3(self) -> f64 { self.0 / FT3_IN_M3 }
}

// ---- Temperature (AFFINE, not a scale factor) -----------------------------

/// Canonical: kelvin. Never multiply a °C/°F value as if it were a ratio.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f64);

impl Temperature {
    pub fn from_k(v: f64) -> Self { Self(v) }
    pub fn from_c(v: f64) -> Self { Self(v + T_ZERO_C_IN_K) }
    pub fn from_f(v: f64) -> Self { Self((v - 32.0) * 5.0 / 9.0 + T_ZERO_C_IN_K) }
    pub fn as_k(self) -> f64 { self.0 }
    pub fn as_c(self) -> f64 { self.0 - T_ZERO_C_IN_K }
    pub fn as_f(self) -> f64 { (self.0 - T_ZERO_C_IN_K) * 9.0 / 5.0 + 32.0 }
}

// ---- Pressure (absolute internally; gauge only at the boundary) ----------

/// Canonical: Pa **absolute**. `from_psig(0)` is 101325 Pa, not 0 Pa.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pressure(f64);

impl Pressure {
    pub fn from_pa_abs(v: f64) -> Self { Self(v) }
    pub fn from_kpa_abs(v: f64) -> Self { Self(v * 1000.0) }
    pub fn from_bar_abs(v: f64) -> Self { Self(v * 100_000.0) }
    pub fn from_psia(v: f64) -> Self { Self(v * PSI_IN_PA) }
    pub fn from_kpa_gauge(v: f64) -> Self { Self(P_STD_PA + v * 1000.0) }
    pub fn from_bar_gauge(v: f64) -> Self { Self(P_STD_PA + v * 100_000.0) }
    pub fn from_psig(v: f64) -> Self { Self(P_STD_PA + v * PSI_IN_PA) }

    /// Interpret a reading with an explicit basis and unit system.
    pub fn from_basis(v: f64, basis: PressureBasis, system: UnitSystem) -> Self {
        let si = match system {
            UnitSystem::Si => v * 1000.0,        // kPa
            UnitSystem::Imperial => v * PSI_IN_PA, // psi
        };
        match basis {
            PressureBasis::Absolute => Self(si),
            PressureBasis::Gauge => Self(P_STD_PA + si),
        }
    }

    pub fn as_pa_abs(self) -> f64 { self.0 }
    pub fn as_kpa_abs(self) -> f64 { self.0 / 1000.0 }
    pub fn as_bara(self) -> f64 { self.0 / 100_000.0 }
    pub fn as_psia(self) -> f64 { self.0 / PSI_IN_PA }
    pub fn as_kpa_gauge(self) -> f64 { (self.0 - P_STD_PA) / 1000.0 }
    pub fn as_bar_gauge(self) -> f64 { (self.0 - P_STD_PA) / 100_000.0 }
    pub fn as_psig(self) -> f64 { (self.0 - P_STD_PA) / PSI_IN_PA }
}

// ---- Flows ----------------------------------------------------------------

/// Canonical: kg/s.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MassFlow(f64);

impl MassFlow {
    pub fn from_kg_s(v: f64) -> Self { Self(v) }
    pub fn from_kg_h(v: f64) -> Self { Self(v / 3600.0) }
    pub fn from_lb_s(v: f64) -> Self { Self(v * LB_IN_KG) }
    pub fn from_lb_h(v: f64) -> Self { Self(v * LB_IN_KG / 3600.0) }
    pub fn as_kg_s(self) -> f64 { self.0 }
    pub fn as_kg_h(self) -> f64 { self.0 * 3600.0 }
    pub fn as_lb_s(self) -> f64 { self.0 / LB_IN_KG }
    pub fn as_lb_h(self) -> f64 { self.0 / LB_IN_KG * 3600.0 }
}

/// Canonical: m³/s, with an explicit Standard/Actual tag.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VolumetricFlow { m3_s: f64, basis: FlowBasis }

impl VolumetricFlow {
    pub fn from_m3_s(v: f64, basis: FlowBasis) -> Self { Self { m3_s: v, basis } }
    pub fn from_m3_h(v: f64, basis: FlowBasis) -> Self { Self::from_m3_s(v / 3600.0, basis) }
    pub fn from_cfm(v: f64, basis: FlowBasis) -> Self { Self::from_m3_s(v * CFM_IN_M3S, basis) }
    pub fn from_nm3_h(v: f64) -> Self { Self::from_m3_h(v, FlowBasis::Standard) } // 0 °C, 101.325 kPa
    pub fn from_scfm(v: f64) -> Self { Self::from_cfm(v, FlowBasis::Standard) }   // 60 °F, 14.696 psia

    pub fn basis(self) -> FlowBasis { self.basis }
    pub fn as_m3_s(self) -> f64 { self.m3_s }
    pub fn as_m3_h(self) -> f64 { self.m3_s * 3600.0 }
    pub fn as_cfm(self) -> f64 { self.m3_s / CFM_IN_M3S }

    /// Ideal-gas state correction: V ∝ T/P. `from` and `to` states are explicit,
    /// so SCFM (60 °F) and Nm³ (0 °C) can never be confused silently.
    pub fn rebase(self, t_from: Temperature, p_from: Pressure,
                  t_to: Temperature, p_to: Pressure) -> Self {
        let q = self.m3_s
            * (t_to.as_k() / t_from.as_k())
            * (p_from.as_pa_abs() / p_to.as_pa_abs());
        Self { m3_s: q, basis: self.basis }
    }

    /// Nm³/h (0 °C, 101.325 kPa abs) -> actual state.
    pub fn standard_to_actual(self, t_actual: Temperature, p_actual: Pressure) -> Self {
        let q = self.rebase(
            Temperature::from_c(0.0),
            Pressure::from_kpa_abs(101.325),
            t_actual,
            p_actual,
        );
        Self { m3_s: q.as_m3_s(), basis: FlowBasis::Actual }
    }
}

/// Canonical: kg/m³.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Density(f64);

impl Density {
    pub fn from_kg_m3(v: f64) -> Self { Self(v) }
    pub fn from_lb_ft3(v: f64) -> Self { Self(v * LB_IN_KG / FT3_IN_M3) }
    pub fn as_kg_m3(self) -> f64 { self.0 }
    pub fn as_lb_ft3(self) -> f64 { self.0 * FT3_IN_M3 / LB_IN_KG }
}

/// Canonical: m/s.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Velocity(f64);

impl Velocity {
    pub fn from_m_s(v: f64) -> Self { Self(v) }
    pub fn as_m_s(self) -> f64 { self.0 }
    pub fn as_ft_s(self) -> f64 { self.0 / FT_IN_M }
    pub fn as_km_h(self) -> f64 { self.0 * 3.6 }
    pub fn as_mph(self) -> f64 { self.0 * 3.6 * 1000.0 / 1609.344 }
}

// ---- Power / heating value ------------------------------------------------

/// Canonical: watt.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl Power {
    pub fn from_w(v: f64) -> Self { Self(v) }
    pub fn from_kw(v: f64) -> Self { Self(v * 1000.0) }
    pub fn from_mw(v: f64) -> Self { Self(v * 1e6) }
    pub fn from_btu_h(v: f64) -> Self { Self(v * BTU_IN_J / 3600.0) }
    pub fn from_hp(v: f64) -> Self { Self(v * HP_IN_W) }
    pub fn as_w(self) -> f64 { self.0 }
    pub fn as_kw(self) -> f64 { self.0 / 1000.0 }
    pub fn as_mw(self) -> f64 { self.0 / 1e6 }
    pub fn as_btu_h(self) -> f64 { self.0 / BTU_IN_J * 3600.0 }
    pub fn as_hp(self) -> f64 { self.0 / HP_IN_W }
}

/// Canonical: J/kg, with an explicit LHV/HHV tag — never swapped implicitly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatingValue { j_kg: f64, basis: HeatingBasis }

impl HeatingValue {
    pub fn from_j_kg(v: f64, basis: HeatingBasis) -> Self { Self { j_kg: v, basis } }
    pub fn from_mj_kg(v: f64, basis: HeatingBasis) -> Self { Self::from_j_kg(v * 1e6, basis) }
    pub fn from_kj_kg(v: f64, basis: HeatingBasis) -> Self { Self::from_j_kg(v * 1e3, basis) }
    pub fn from_btu_lb(v: f64, basis: HeatingBasis) -> Self {
        Self::from_j_kg(v * BTU_IN_J / LB_IN_KG, basis)
    }
    pub fn basis(self) -> HeatingBasis { self.basis }
    pub fn as_j_kg(self) -> f64 { self.j_kg }
    pub fn as_mj_kg(self) -> f64 { self.j_kg / 1e6 }
    pub fn as_btu_lb(self) -> f64 { self.j_kg * LB_IN_KG / BTU_IN_J }
    /// Fails loudly rather than silently using the wrong heating value.
    pub fn require_basis(self, expected: HeatingBasis) -> Result<Self, UnitError> {
        if self.basis == expected {
            Ok(self)
        } else {
            Err(UnitError(format!(
                "heating value basis mismatch: have {:?}, need {:?}",
                self.basis, expected
            )))
        }
    }
}

// ---- Boundary parsing & audit --------------------------------------------

/// Parse `"2 in"`, `"50.8 mm"`, `"0.5 ft"`. An explicit suffix ALWAYS wins;
/// a bare number is interpreted in the caller's current `UnitSystem`.
pub fn parse_length(s: &str, system: UnitSystem) -> Result<Length, UnitError> {
    let mut parts = s.trim().split_whitespace();
    let number = parts.next().ok_or_else(|| UnitError("empty length".into()))?;
    let v: f64 = number.parse().map_err(|_| UnitError(format!("bad number: {number}")))?;
    let unit = parts.next().unwrap_or("").to_ascii_lowercase();
    match unit.as_str() {
        "" => Ok(match system {
            UnitSystem::Si => Length::from_mm(v),
            UnitSystem::Imperial => Length::from_inch(v),
        }),
        "mm" => Ok(Length::from_mm(v)),
        "cm" => Ok(Length::from_cm(v)),
        "m" => Ok(Length::from_m(v)),
        "in" | "inch" | "inches" | "\"" => Ok(Length::from_inch(v)),
        "ft" | "feet" | "foot" | "'" => Ok(Length::from_ft(v)),
        _ => Err(UnitError(format!("unknown length unit: {unit}"))),
    }
}

/// Parse `"100 C"`, `"212 F"`, `"373.15 K"`. °C/°F are affine conversions.
pub fn parse_temperature(s: &str, system: UnitSystem) -> Result<Temperature, UnitError> {
    let mut parts = s.trim().split_whitespace();
    let number = parts.next().ok_or_else(|| UnitError("empty temperature".into()))?;
    let v: f64 = number.parse().map_err(|_| UnitError(format!("bad number: {number}")))?;
    let unit = parts.next().unwrap_or("").to_ascii_lowercase();
    match unit.as_str() {
        "" => Ok(match system {
            UnitSystem::Si => Temperature::from_c(v),
            UnitSystem::Imperial => Temperature::from_f(v),
        }),
        "c" | "°c" | "degc" => Ok(Temperature::from_c(v)),
        "f" | "°f" | "degf" => Ok(Temperature::from_f(v)),
        "k" | "kelvin" => Ok(Temperature::from_k(v)),
        _ => Err(UnitError(format!("unknown temperature unit: {unit}"))),
    }
}

pub fn cv_to_kv(cv: f64) -> f64 { cv * CV_TO_KV }
pub fn kv_to_cv(kv: f64) -> f64 { kv / CV_TO_KV }

/// Reference table the user can eyeball to catch a wrong conversion.
pub fn unit_audit() -> Vec<(&'static str, String)> {
    let scfm = VolumetricFlow::from_scfm(1.0);
    let scfm_as_nm3h = scfm.rebase(
        Temperature::from_f(60.0),
        Pressure::from_psia(14.696),
        Temperature::from_c(0.0),
        Pressure::from_kpa_abs(101.325),
    ).as_m3_h();

    vec![
        ("1 in        -> mm",       format!("{:.6}", Length::from_inch(1.0).as_mm())),
        ("1 ft        -> m",        format!("{:.6}", Length::from_ft(1.0).as_m())),
        ("1 in^2      -> mm^2",     format!("{:.6}", Length::from_inch(1.0).squared().as_mm2())),
        ("1 ft^3      -> m^3",      format!("{:.9}", Volume::from_ft3(1.0).as_m3())),
        ("0 C         -> F",        format!("{:.4}", Temperature::from_c(0.0).as_f())),
        ("100 C       -> F",        format!("{:.4}", Temperature::from_c(100.0).as_f())),
        ("212 F       -> C",        format!("{:.4}", Temperature::from_f(212.0).as_c())),
        ("0 psig      -> kPa(abs)", format!("{:.4}", Pressure::from_psig(0.0).as_kpa_abs())),
        ("0 psia      -> kPa(abs)", format!("{:.4}", Pressure::from_psia(0.0).as_kpa_abs())),
        ("14.696 psia -> kPa(abs)", format!("{:.4}", Pressure::from_psia(14.696).as_kpa_abs())),
        ("1 CFM       -> m3/h",     format!("{:.6}", VolumetricFlow::from_cfm(1.0, FlowBasis::Actual).as_m3_h())),
        ("1 SCFM@60F  -> Nm3/h@0C", format!("{:.6}", scfm_as_nm3h)),
        ("1 lb/ft3    -> kg/m3",    format!("{:.6}", Density::from_lb_ft3(1.0).as_kg_m3())),
        ("1 BTU/h     -> W",        format!("{:.6}", Power::from_btu_h(1.0).as_w())),
        ("1 kW        -> BTU/h",    format!("{:.4}", Power::from_kw(1.0).as_btu_h())),
        ("1 BTU/lb    -> kJ/kg",    format!("{:.6}", HeatingValue::from_btu_lb(1.0, HeatingBasis::Lhv).as_j_kg() / 1000.0)),
        ("1 Cv        -> Kv",       format!("{:.6}", cv_to_kv(1.0))),
    ]
}

// ---- Thin compatibility helpers (prefer the typed API above) --------------

pub fn c_to_k(c: f64) -> f64 { Temperature::from_c(c).as_k() }
pub fn k_to_c(k: f64) -> f64 { Temperature::from_k(k).as_c() }
pub fn f_to_c(f: f64) -> f64 { Temperature::from_f(f).as_c() }
pub fn c_to_f(c: f64) -> f64 { Temperature::from_c(c).as_f() }
pub fn inch_to_mm(v: f64) -> f64 { Length::from_inch(v).as_mm() }
pub fn mm_to_inch(v: f64) -> f64 { Length::from_mm(v).as_inch() }
pub fn bar_to_kpa(v: f64) -> f64 { v * 100.0 }
pub fn kpa_to_bar(v: f64) -> f64 { v / 100.0 }
pub fn kpa_to_pa(v: f64) -> f64 { v * 1000.0 }
pub fn pa_to_kpa(v: f64) -> f64 { v / 1000.0 }
pub fn m3h_to_m3s(v: f64) -> f64 { v / 3600.0 }
pub fn m3s_to_m3h(v: f64) -> f64 { v * 3600.0 }
pub fn kw_to_mw(v: f64) -> f64 { v / 1000.0 }
pub fn mw_to_kw(v: f64) -> f64 { v * 1000.0 }

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    #[test]
    fn length_is_exact_and_reversible() {
        assert!(close(Length::from_inch(1.0).as_mm(), 25.4, 1e-12));
        assert!(close(Length::from_mm(50.8).as_inch(), 2.0, 1e-12));
        assert!(close(Length::from_ft(1.0).as_m(), 0.3048, 1e-12));
    }

    #[test]
    fn squared_and_cubed_are_not_linear() {
        // Classic bug: multiplying by 25.4 instead of 25.4^2 / 304.8^3.
        assert!(close(Length::from_inch(1.0).squared().as_mm2(), 645.16, 1e-9));
        assert!(close(Volume::from_ft3(1.0).as_m3(), 0.028_316_846_592, 1e-12));
        assert!(close(Length::from_ft(1.0).cubed().as_m3(), FT3_IN_M3, 1e-15));
    }

    #[test]
    fn temperature_is_affine_not_a_scale_factor() {
        assert!(close(Temperature::from_c(0.0).as_f(), 32.0, 1e-9));
        assert!(close(Temperature::from_c(100.0).as_f(), 212.0, 1e-9));
        assert!(close(Temperature::from_f(212.0).as_c(), 100.0, 1e-9));
        assert!(close(Temperature::from_f(32.0).as_c(), 0.0, 1e-9));
        assert!(close(Temperature::from_c(0.0).as_k(), 273.15, 1e-9));
        // A ratio MUST be formed in kelvin, never in °C.
        let ratio_k = Temperature::from_c(200.0).as_k() / Temperature::from_c(0.0).as_k();
        assert!(close(ratio_k, 473.15 / 273.15, 1e-12));
        assert!(!close(ratio_k, 200.0 / 0.0, 1e-12));
    }

    #[test]
    fn gauge_zero_is_not_absolute_zero() {
        assert!(close(Pressure::from_psig(0.0).as_kpa_abs(), 101.325, 1e-9));
        assert!(close(Pressure::from_psia(0.0).as_kpa_abs(), 0.0, 1e-9));
        assert!(close(Pressure::from_kpa_gauge(0.0).as_psig(), 0.0, 1e-9));
        assert!(close(Pressure::from_basis(0.0, PressureBasis::Gauge, UnitSystem::Imperial).as_kpa_abs(),
                      101.325, 1e-9));
        assert!(close(Pressure::from_basis(14.696, PressureBasis::Absolute, UnitSystem::Imperial).as_kpa_abs(),
                      Pressure::from_psia(14.696).as_kpa_abs(), 1e-9));
    }

    #[test]
    fn standard_flow_rebases_by_state_not_by_unit() {
        let q = VolumetricFlow::from_nm3_h(1000.0);
        let hot = q.rebase(
            Temperature::from_c(0.0), Pressure::from_kpa_abs(101.325),
            Temperature::from_c(200.0), Pressure::from_kpa_abs(101.325),
        );
        assert!(close(hot.as_m3_h(), 1000.0 * 473.15 / 273.15, 1e-6));
        // SCFM (60 F) and Nm3 (0 C) are different reference states.
        let nm3_per_scfm = VolumetricFlow::from_scfm(1.0).rebase(
            Temperature::from_f(60.0), Pressure::from_psia(14.696),
            Temperature::from_c(0.0), Pressure::from_kpa_abs(101.325),
        ).as_m3_h();
        assert!(nm3_per_scfm > 1.60 && nm3_per_scfm < 1.62, "got {nm3_per_scfm}");
    }

    #[test]
    fn actual_flow_units_agree() {
        let q_si = VolumetricFlow::from_m3_h(3600.0, FlowBasis::Actual);
        let q_im = VolumetricFlow::from_cfm(2118.879_972_76, FlowBasis::Actual);
        assert!(close(q_si.as_m3_s(), 1.0, 1e-9));
        assert!(close(q_si.as_m3_s(), q_im.as_m3_s(), 1e-4));
        assert_eq!(q_si.basis(), FlowBasis::Actual);
    }

    #[test]
    fn heating_value_basis_is_never_swapped() {
        let lhv = HeatingValue::from_mj_kg(48.0, HeatingBasis::Lhv);
        let hhv = HeatingValue::from_mj_kg(53.1, HeatingBasis::Hhv);
        assert_ne!(lhv.as_mj_kg(), hhv.as_mj_kg());
        assert_eq!(hhv.basis(), HeatingBasis::Hhv);
        assert!(hhv.require_basis(HeatingBasis::Lhv).is_err());
        assert!(lhv.require_basis(HeatingBasis::Lhv).is_ok());
        assert!(close(HeatingValue::from_btu_lb(1.0, HeatingBasis::Lhv).as_j_kg() / 1000.0,
                      2.326, 0.001));
    }

    #[test]
    fn power_and_density_factors() {
        assert!(close(Power::from_btu_h(1.0).as_w(), 0.293_071_07, 1e-7));
        assert!(close(Power::from_kw(1.0).as_btu_h(), 3412.142, 0.01));
        assert!(close(Density::from_lb_ft3(1.0).as_kg_m3(), 16.018_463, 1e-5));
        assert!(close(MassFlow::from_lb_h(3600.0 / LB_IN_KG).as_kg_s(), 1.0, 1e-9));
    }

    #[test]
    fn cv_kv_factor() {
        assert!(close(cv_to_kv(1.0), 0.865, 1e-12));
        assert!(close(kv_to_cv(cv_to_kv(12.3)), 12.3, 1e-12));
    }

    #[test]
    fn explicit_unit_suffix_always_wins() {
        assert!(close(parse_length("2 in", UnitSystem::Si).unwrap().as_mm(), 50.8, 1e-9));
        assert!(close(parse_length("2 in", UnitSystem::Imperial).unwrap().as_mm(), 50.8, 1e-9));
        assert!(close(parse_length("50.8 mm", UnitSystem::Imperial).unwrap().as_inch(), 2.0, 1e-9));
        assert!(close(parse_length("2", UnitSystem::Si).unwrap().as_mm(), 2.0, 1e-9));
        assert!(close(parse_length("2", UnitSystem::Imperial).unwrap().as_inch(), 2.0, 1e-9));
        assert!(parse_length("2 furlong", UnitSystem::Si).is_err());
        assert!(close(parse_temperature("212 F", UnitSystem::Si).unwrap().as_c(), 100.0, 1e-9));
        assert!(close(parse_temperature("100 C", UnitSystem::Imperial).unwrap().as_f(), 212.0, 1e-9));
        assert!(parse_temperature("100 bogus", UnitSystem::Si).is_err());
    }

    #[test]
    fn no_double_conversion_roundtrip() {
        // 2 in -> SI -> imperial -> SI must be stable to machine precision.
        let l = Length::from_inch(2.0);
        let back = Length::from_inch(l.as_inch());
        assert!(close(back.as_m(), l.as_m(), 1e-15));
        let t = Temperature::from_f(212.0);
        assert!(close(Temperature::from_k(t.as_k()).as_c(), 100.0, 1e-9));
        let p = Pressure::from_psig(100.0);
        assert!(close(Pressure::from_psia(p.as_psia()).as_psig(), 100.0, 1e-9));
    }
}
