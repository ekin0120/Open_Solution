//! Domain models. Field names are generic engineering attributes only —
//! no vendor schema, column names or proprietary data are reproduced.

#[derive(Debug, Clone, PartialEq)]
pub struct Fuel {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub lhv_mj_kg: Option<f64>,
    pub hhv_mj_kg: Option<f64>,
    pub density_kg_nm3: Option<f64>,
    pub source: String,
    pub note: Option<String>,
}

impl Fuel {
    /// A record with no vendor data: the caller fills in its own measurements.
    pub fn user_supplied(code: &str, name: &str) -> Self {
        Self {
            id: 0,
            code: code.to_string(),
            name: name.to_string(),
            lhv_mj_kg: None,
            hhv_mj_kg: None,
            density_kg_nm3: None,
            source: "user-supplied".to_string(),
            note: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Burner {
    pub code: String,
    pub name: String,
    pub fuel_code: Option<String>,
    pub capacity_kw_min: Option<f64>,
    pub capacity_kw_max: Option<f64>,
    pub turndown: Option<f64>,
    pub excess_air_pct: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blower {
    pub code: String,
    pub name: String,
    pub flow_m3h_max: Option<f64>,
    pub pressure_kpa_max: Option<f64>,
    pub power_kw: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Valve {
    pub code: String,
    pub name: String,
    pub dn_mm: Option<f64>,
    pub cv: Option<f64>,
    pub max_pressure_kpa: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Orifice {
    pub code: String,
    pub name: String,
    pub diameter_mm: Option<f64>,
    pub cd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipe {
    pub schedule: String,
    pub nps_inch: f64,
    pub od_mm: Option<f64>,
    pub wall_mm: Option<f64>,
    pub inner_d_mm: Option<f64>,
}

impl Pipe {
    pub fn inner_diameter_mm(&self) -> Option<f64> {
        if let Some(d) = self.inner_d_mm {
            return Some(d);
        }
        match (self.od_mm, self.wall_mm) {
            (Some(od), Some(wall)) => Some(od - 2.0 * wall),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fitting {
    pub code: String,
    pub name: String,
    pub dn_mm: Option<f64>,
    pub k_factor: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub code: String,
    pub name: String,
    pub max_temp_c: Option<f64>,
    pub density_kg_m3: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Regulator {
    pub code: String,
    pub name: String,
    pub inlet_max_kpa: Option<f64>,
    pub outlet_min_kpa: Option<f64>,
    pub outlet_max_kpa: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_inner_diameter() {
        let p = Pipe {
            schedule: "SCH40".into(),
            nps_inch: 2.0,
            od_mm: Some(60.3),
            wall_mm: Some(3.91),
            inner_d_mm: None,
        };
        assert!((p.inner_diameter_mm().unwrap() - 52.48).abs() < 1e-9);

        let explicit = Pipe { inner_d_mm: Some(50.0), ..p };
        assert_eq!(explicit.inner_diameter_mm(), Some(50.0));
    }

    #[test]
    fn fuel_default_has_no_vendor_values() {
        let f = Fuel::user_supplied("NG-1", "My natural gas");
        assert_eq!(f.source, "user-supplied");
        assert!(f.lhv_mj_kg.is_none());
    }
}
