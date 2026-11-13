//! Engineering calculations with typed quantities.
//!
//! Every function takes/returns a unit-carrying type, so an imperial input
//! cannot silently be interpreted as metric. All formulas are textbook
//! relations; no vendor coefficients or lookup tables are used. Validate
//! against your own reference data before using results in design work.

use crate::units::{
    Density, FlowBasis, HeatingBasis, HeatingValue, Length, MassFlow, Power, Pressure,
    R_UNIVERSAL_J_PER_MOL_K, Temperature, UnitError, Velocity, VolumetricFlow,
};

/// Ideal-gas density. Pressure is absolute, temperature is in kelvin internally.
pub fn ideal_gas_density(p: Pressure, t: Temperature, molar_mass_g_mol: f64) -> Density {
    let m_kg_per_mol = molar_mass_g_mol / 1000.0;
    Density::from_kg_m3(p.as_pa_abs() * m_kg_per_mol / (R_UNIVERSAL_J_PER_MOL_K * t.as_k()))
}

/// Fuel heat release. The basis travels with `HeatingValue`, so LHV and HHV
/// can never be substituted silently.
pub fn heat_release(m_dot: MassFlow, hv: HeatingValue) -> Power {
    Power::from_w(m_dot.as_kg_s() * hv.as_j_kg())
}

/// Heat release that insists on a declared basis (fails loudly otherwise).
pub fn heat_release_with_basis(
    m_dot: MassFlow,
    hv: HeatingValue,
    expected: HeatingBasis,
) -> Result<Power, UnitError> {
    hv.require_basis(expected)?;
    Ok(heat_release(m_dot, hv))
}

/// Mean pipe velocity. Diameter is a `Length`, flow has an explicit basis.
pub fn pipe_velocity(q: VolumetricFlow, d: Length) -> Velocity {
    Velocity::from_m_s(q.as_m3_s() / d.circle_area().as_m2())
}

/// Pipe velocity that refuses a standard-state flow until it is rebased to the
/// actual state — a common source of wrong outputs in the legacy program.
pub fn pipe_velocity_checked(q: VolumetricFlow, d: Length) -> Result<Velocity, UnitError> {
    if q.basis() == FlowBasis::Standard {
        return Err(UnitError(
            "standard-state flow given to pipe_velocity; rebase to actual T/P first".into(),
        ));
    }
    Ok(pipe_velocity(q, d))
}

pub fn reynolds(rho: Density, v: Velocity, d: Length, dyn_visc_pa_s: f64) -> f64 {
    rho.as_kg_m3() * v.as_m_s() * d.as_m() / dyn_visc_pa_s
}

/// Incompressible orifice estimate. `dp_pa` is a differential pressure in Pa.
pub fn orifice_flow(cd: f64, d: Length, dp_pa: f64, rho: Density) -> VolumetricFlow {
    let q = cd * d.circle_area().as_m2() * (2.0 * dp_pa / rho.as_kg_m3()).sqrt();
    VolumetricFlow::from_m3_s(q, FlowBasis::Actual)
}

pub fn excess_air_pct(actual_air: MassFlow, stoich_air: MassFlow) -> f64 {
    (actual_air.as_kg_s() / stoich_air.as_kg_s() - 1.0) * 100.0
}

pub fn thermal_efficiency_pct(useful: Power, fuel_input: Power) -> f64 {
    useful.as_w() / fuel_input.as_w() * 100.0
}

/// One place that formats a result in both systems — conversion happens once.
pub fn dual_power(p: Power) -> (f64, f64) {
    (p.as_kw(), p.as_btu_h())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool { (a - b).abs() < tol }

    #[test]
    fn air_density_at_standard_conditions() {
        let rho = ideal_gas_density(
            Pressure::from_kpa_abs(101.325),
            Temperature::from_c(0.0),
            28.97,
        );
        assert!(close(rho.as_kg_m3(), 1.293, 0.01), "rho = {}", rho.as_kg_m3());
    }

    /// THE regression: metric and imperial inputs must give the same SI answer.
    #[test]
    fn si_and_imperial_inputs_agree() {
        let p_si = Pressure::from_kpa_abs(101.325);
        let p_imp = Pressure::from_psia(14.696);
        let t_si = Temperature::from_c(0.0);
        let t_imp = Temperature::from_f(32.0);
        let a = ideal_gas_density(p_si, t_si, 28.97).as_kg_m3();
        let b = ideal_gas_density(p_imp, t_imp, 28.97).as_kg_m3();
        assert!(close(a, b, 1e-3), "si={a} imperial={b}");

        let d_si = Length::from_mm(100.0);
        let d_imp = Length::from_inch(100.0 / 25.4);
        let q_si = VolumetricFlow::from_m3_h(36.0, FlowBasis::Actual);
        let q_imp = VolumetricFlow::from_cfm(21.188_799_727_6, FlowBasis::Actual);
        let v_si = pipe_velocity(q_si, d_si).as_m_s();
        let v_imp = pipe_velocity(q_imp, d_imp).as_m_s();
        assert!(close(v_si, v_imp, 1e-6), "v_si={v_si} v_imp={v_imp}");
        assert!(close(v_si, 1.273_239_5, 1e-5));
    }

    #[test]
    fn heat_release_and_basis_guard() {
        let m = MassFlow::from_kg_s(0.5);
        let lhv = HeatingValue::from_mj_kg(50.0, HeatingBasis::Lhv);
        let hhv = HeatingValue::from_mj_kg(55.0, HeatingBasis::Hhv);
        assert!(close(heat_release(m, lhv).as_mw(), 25.0, 1e-9));
        assert!(heat_release_with_basis(m, hhv, HeatingBasis::Lhv).is_err());
        assert!(heat_release_with_basis(m, lhv, HeatingBasis::Lhv).is_ok());
        let (kw, btu_h) = dual_power(heat_release(m, lhv));
        assert!(close(kw, 25_000.0, 1e-6));
        assert!(btu_h > 0.0);
    }

    #[test]
    fn imperial_fuel_inputs_agree() {
        let m_si = MassFlow::from_kg_h(1000.0);
        let m_imp = MassFlow::from_lb_h(1000.0 / 0.453_592_37);
        let hv_si = HeatingValue::from_mj_kg(50.0, HeatingBasis::Lhv);
        let hv_imp = HeatingValue::from_btu_lb(50.0 * 1e6 * 0.453_592_37 / 1055.055_852_62, HeatingBasis::Lhv);
        assert!(close(heat_release(m_si, hv_si).as_w(), heat_release(m_imp, hv_imp).as_w(), 1e-6));
    }

    #[test]
    fn standard_flow_is_refused_until_rebased() {
        let q = VolumetricFlow::from_nm3_h(1000.0);
        let d = Length::from_mm(100.0);
        assert!(pipe_velocity_checked(q, d).is_err());
        let actual = q.standard_to_actual(Temperature::from_c(20.0), Pressure::from_kpa_abs(101.325));
        assert!(pipe_velocity_checked(actual, d).is_ok());
    }

    #[test]
    fn reynolds_and_orifice() {
        let v = pipe_velocity(
            VolumetricFlow::from_m3_h(36.0, FlowBasis::Actual),
            Length::from_mm(100.0),
        );
        let re = reynolds(Density::from_kg_m3(1.2), v, Length::from_mm(100.0), 1.8e-5);
        assert!(re > 0.0 && re.is_finite());
        let q = orifice_flow(0.62, Length::from_mm(25.0), 5000.0, Density::from_kg_m3(1.2));
        assert!(q.as_m3_s() > 0.0 && q.as_m3_s().is_finite());
        assert_eq!(q.basis(), FlowBasis::Actual);
    }

    #[test]
    fn ratios_are_dimensionless() {
        assert!(close(excess_air_pct(MassFlow::from_kg_s(12.0), MassFlow::from_kg_s(10.0)), 20.0, 1e-9));
        assert!(close(thermal_efficiency_pct(Power::from_kw(20.0), Power::from_kw(25.0)), 80.0, 1e-9));
        assert!(close(
            excess_air_pct(MassFlow::from_lb_h(12.0), MassFlow::from_lb_h(10.0)),
            20.0, 1e-9,
        ));
    }
}
