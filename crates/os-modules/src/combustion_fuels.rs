// AUTO-GENERATED from the imported source Fuel.mdb tables
// (src_Fuel__GasFuel / OilFuel / WasteOil in open_solutions.db).
// Units are the source US units: gas heating values Btu/scf @60 F,
// liquid heating values Btu/lb; Gravity = specific gravity vs air (gas)
// or vs water (oil). SI columns are the source GrossHeatSI/NetHeatSI.
#![allow(non_upper_case_globals)]

/// Standard gaseous fuel: 9 volumetric constituents (%), plus the source
/// fuel-property row. All values are from the source GasFuel table.
pub struct GasFuel {
    pub name: &'static str,
    /// CH4, C2H6, C3H8, C4H10, CO, H2, CO2, O2, N2 (vol %)
    pub comp: [f64; 9],
    pub gravity: f64,
    pub ghv: f64,          // Btu/scf, gross (HHV)
    pub nhv: f64,          // Btu/scf, net (LHV)
    pub cp: f64,           // Btu/(scf.F), source Cp column
    pub ghv_si: f64,       // MJ/Nm3 @0 C
    pub nhv_si: f64,       // MJ/Nm3 @0 C
}

/// Liquid fuel ultimate analysis (wt %) and heating values.
pub struct OilFuel {
    pub name: &'static str,
    pub c: f64, pub h: f64, pub s: f64, pub o: f64, pub n: f64,
    pub ash: f64, pub moist: f64,
    pub gravity: f64,      // vs water
    pub ghv: f64,          // Btu/lb
    pub nhv: f64,          // Btu/lb
    pub ghv_si: f64,       // MJ/kg
    pub nhv_si: f64,       // MJ/kg
}

pub const GAS_FUELS: &[GasFuel] = &[
    GasFuel { name: "Average Natural Gas", comp: [89.45, 5.08, 1.44, 0.31, 0.0, 0.0, 0.73, 0.0, 2.99], gravity: 0.62, ghv: 1042.0, nhv: 965.0, cp: 0.025, ghv_si: 41.03, nhv_si: 38.0 },
    GasFuel { name: "Blast Furnace Gas", comp: [0.0, 0.0, 0.0, 0.0, 25.1, 1.65, 15.4, 0.35, 57.5], gravity: 1.04, ghv: 86.5, nhv: 86.0, cp: 0.021, ghv_si: 3.4, nhv_si: 3.38 },
    GasFuel { name: "Butane", comp: [0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0], gravity: 2.0, ghv: 3200.0, nhv: 3018.0, cp: 0.07, ghv_si: 126.0, nhv_si: 118.83 },
    GasFuel { name: "Coke Oven Gas", comp: [30.3, 0.0, 0.0, 3.3, 4.5, 57.9, 1.8, 0.2, 2.2], gravity: 0.36, ghv: 617.0, nhv: 560.0, cp: 0.0362, ghv_si: 24.294, nhv_si: 22.05 },
    GasFuel { name: "Erdgas L (Low Calorific)", comp: [81.3, 2.85, 0.42, 0.23, 0.0, 0.0, 0.89, 0.01, 14.3], gravity: 0.6488, ghv: 891.0, nhv: 803.0, cp: 0.025, ghv_si: 35.089, nhv_si: 31.64 },
    GasFuel { name: "Erdgas H (High Calorific)", comp: [93.3, 3.38, 0.94, 0.71, 0.0, 0.0, 0.89, 0.0, 0.78], gravity: 0.6076, ghv: 1050.0, nhv: 948.0, cp: 0.025, ghv_si: 41.36, nhv_si: 37.33 },
    GasFuel { name: "N.Gas Birmingham", comp: [93.14, 2.5, 0.67, 0.32, 0.0, 0.0, 1.06, 0.0, 2.14], gravity: 0.599, ghv: 1024.0, nhv: 923.0, cp: 0.023, ghv_si: 40.32, nhv_si: 36.34 },
    GasFuel { name: "N.Gas East Ohio", comp: [94.25, 3.98, 0.57, 0.16, 0.0, 0.01, 0.68, 0.1, 0.3], gravity: 0.591, ghv: 1031.0, nhv: 935.0, cp: 0.0236, ghv_si: 40.59, nhv_si: 36.81 },
    GasFuel { name: "N.Gas Pittsburgh", comp: [94.03, 3.58, 0.79, 0.28, 0.0, 0.0, 0.8, 0.01, 0.4], gravity: 0.595, ghv: 1120.0, nhv: 1025.0, cp: 0.025, ghv_si: 40.16, nhv_si: 40.36 },
    GasFuel { name: "UGI Gas", comp: [95.736, 2.242, 0.51, 0.41, 0.0, 0.0, 0.0, 0.0, 1.102], gravity: 0.586, ghv: 1034.0, nhv: 933.0, cp: 0.0254, ghv_si: 40.71, nhv_si: 36.737 },
    GasFuel { name: "Landfill, Cagistrio 81", comp: [53.4, 0.0, 0.0, 0.0, 0.0, 0.0, 34.3, 0.05, 12.25], gravity: 0.88, ghv: 540.0, nhv: 495.0, cp: 0.0275, ghv_si: 21.26, nhv_si: 19.49 },
    GasFuel { name: "Methane", comp: [100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], gravity: 0.56, ghv: 1012.0, nhv: 911.0, cp: 0.0225, ghv_si: 39.87, nhv_si: 35.87 },
    GasFuel { name: "Producer, Koppers-Totzek", comp: [0.0, 0.0, 0.0, 0.0, 58.7, 32.9, 7.0, 0.0, 1.4], gravity: 0.71, ghv: 290.0, nhv: 275.0, cp: 0.0194, ghv_si: 11.42, nhv_si: 10.83 },
    GasFuel { name: "Producer, Lurgi", comp: [10.2, 0.0, 0.0, 0.0, 17.1, 40.2, 31.4, 0.0, 1.1], gravity: 0.74, ghv: 304.0, nhv: 275.0, cp: 0.024, ghv_si: 11.97, nhv_si: 10.83 },
    GasFuel { name: "Propane", comp: [0.0, 0.0, 100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], gravity: 1.52, ghv: 2500.0, nhv: 2316.0, cp: 0.045, ghv_si: 98.437, nhv_si: 91.19 },
    GasFuel { name: "Propane, Commercial", comp: [0.0, 2.2, 97.3, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0], gravity: 1.52, ghv: 2499.0, nhv: 2316.0, cp: 0.045, ghv_si: 98.4, nhv_si: 91.19 },
    GasFuel { name: "N.Gas UGI", comp: [95.68, 2.44, 0.38, 0.07, 0.0, 0.0, 0.92, 0.1, 0.41], gravity: 0.577, ghv: 1024.0, nhv: 922.0, cp: 0.023, ghv_si: 40.32, nhv_si: 36.3 },
];

pub const OIL_FUELS: &[OilFuel] = &[
    OilFuel { name: "Oil #1", c: 86.6, h: 13.3, s: 0.14, o: 0.0, n: 0.0, ash: 0.0, moist: 0.0, gravity: 0.8301, ghv: 19715.0, nhv: 18498.0, ghv_si: 45.758, nhv_si: 42.935 },
    OilFuel { name: "Oil #2", c: 87.3, h: 12.5, s: 0.21, o: 0.0, n: 0.0, ash: 0.0, moist: 0.0, gravity: 0.8703, ghv: 19445.0, nhv: 18278.0, ghv_si: 45.1318, nhv_si: 42.424 },
    OilFuel { name: "Oil #4", c: 86.4, h: 11.6, s: 1.99, o: 0.0, n: 0.0, ash: 0.018, moist: 0.2, gravity: 0.9161, ghv: 19147.0, nhv: 18042.0, ghv_si: 44.44, nhv_si: 41.876 },
    OilFuel { name: "Oil #5", c: 88.7, h: 10.7, s: 0.57, o: 0.0, n: 0.0, ash: 0.02, moist: 0.4, gravity: 0.9594, ghv: 18837.0, nhv: 17872.0, ghv_si: 43.72, nhv_si: 41.48 },
    OilFuel { name: "Oil #6", c: 88.3, h: 9.3, s: 0.85, o: 0.7, n: 0.3, ash: 0.04, moist: 0.2, gravity: 1.0184, ghv: 18495.0, nhv: 17426.0, ghv_si: 42.927, nhv_si: 40.446 },
    OilFuel { name: "Average Waste Oil", c: 85.4, h: 12.8, s: 0.51, o: 0.0, n: 0.0, ash: 0.87, moist: 0.42, gravity: 0.9, ghv: 19200.0, nhv: 18200.0, ghv_si: 44.66, nhv_si: 42.34 },
];

/// Drop-down captions: standard gas names, then "Customized Gas Fuel".
pub const GAS_FUEL_NAMES: &[&'static str] = &[
    "Average Natural Gas",
    "Blast Furnace Gas",
    "Butane",
    "Coke Oven Gas",
    "Erdgas L (Low Calorific)",
    "Erdgas H (High Calorific)",
    "N.Gas Birmingham",
    "N.Gas East Ohio",
    "N.Gas Pittsburgh",
    "UGI Gas",
    "Landfill, Cagistrio 81",
    "Methane",
    "Producer, Koppers-Totzek",
    "Producer, Lurgi",
    "Propane",
    "Propane, Commercial",
    "N.Gas UGI",
    "Customized Gas Fuel",
];

pub const OIL_FUEL_NAMES: &[&'static str] = &[
    "Oil #1",
    "Oil #2",
    "Oil #4",
    "Oil #5",
    "Oil #6",
    "Average Waste Oil",
    "Customized Oil Mixture",
];

/// Energy Savings module fuel list: the source form's 25 listbox entries in
/// source order. Verified against the running source: every entry past the 16
/// standard gas fuels (Customized Gas Mixture, Oil #1–#6, Customized Oil
/// Mixture, Custom Oil, Electricity) falls back to Average Natural Gas data.
pub const ES_FUEL_NAMES: &[&'static str] = &[
    "Average Natural Gas",
    "Blast Furnace Gas",
    "Butane",
    "Coke Oven Gas",
    "N.Gas Birmingham",
    "N.Gas East Ohio",
    "N.Gas Pittsburgh",
    "UGI Gas",
    "Erdgas L (Low Calorific)",
    "Erdgas H (High Calorific)",
    "Landfill, Cagistrio 81",
    "Methane",
    "Producer, Koppers-Totzek",
    "Producer, Lurgi",
    "Propane",
    "Propane, Commercial",
    "Customized Gas Mixture",
    "Oil #1",
    "Oil #2",
    "Oil #4",
    "Oil #5",
    "Oil #6",
    "Customized Oil Mixture",
    "Custom Oil",
    "Electricity",
];

/// Resolve an Energy Savings fuel entry. Gas names map to the GasFuel table;
/// oil / electricity / custom entries return Average Natural Gas, matching the
/// verified source behaviour.
pub fn es_fuel(name: &str) -> &'static GasFuel {
    GAS_FUELS
        .iter()
        .find(|g| g.name == name)
        .unwrap_or(&GAS_FUELS[0])
}

/// Wobbe Index form: the two list boxes carry the 16 standard gases only
/// (source Wobbe_Index form dumps show no "Customized Gas Mixture" entry).
pub const WOBBE_GAS_NAMES: &[&str] = &[
    "Average Natural Gas",
    "Blast Furnace Gas",
    "Butane",
    "Coke Oven Gas",
    "Erdgas L (Low Calorific)",
    "Erdgas H (High Calorific)",
    "N.Gas Birmingham",
    "N.Gas East Ohio",
    "N.Gas Pittsburgh",
    "UGI Gas",
    "Landfill, Cagistrio 81",
    "Methane",
    "Producer, Koppers-Totzek",
    "Producer, Lurgi",
    "Propane",
    "Propane, Commercial",
];

/// 16 constituents on the source report (Fuel Composition, % by volume).
pub const CUSTOM_KEYS: &[(&'static str, &'static str, &'static str)] = &[
    ("y_ch4", "CH4", "Methane"),
    ("y_c2h2", "C2H2", "Acetylene"),
    ("y_c2h4", "C2H4", "Ethylene"),
    ("y_c2h6", "C2H6", "Ethane"),
    ("y_c3h6", "C3H6", "Propylene"),
    ("y_c3h8", "C3H8", "Propane"),
    ("y_c4h10", "C4H10", "Butane"),
    ("y_c5h12", "C5H12", "N-Pentane"),
    ("y_c6h14", "C6H14", "N-Hexane"),
    ("y_co", "CO", "Carbon Monoxide"),
    ("y_h2", "H2", "Hydrogen"),
    ("y_h2s", "H2S", "Hydrogen Sulfide"),
    ("y_co2", "CO2", "Carbon Dioxide"),
    ("y_o2", "O2", "Oxygen"),
    ("y_h2o", "H2O", "Water Vapor"),
    ("y_n2", "N2", "Nitrogen"),
];

/// Static row labels for the composition table (avoids per-call allocation).
pub const CUSTOM_LABELS: &[&'static str] = &[
    "CH4 (Methane)",
    "C2H2 (Acetylene)",
    "C2H4 (Ethylene)",
    "C2H6 (Ethane)",
    "C3H6 (Propylene)",
    "C3H8 (Propane)",
    "C4H10 (Butane)",
    "C5H12 (N-Pentane)",
    "C6H14 (N-Hexane)",
    "CO (Carbon Monoxide)",
    "H2 (Hydrogen)",
    "H2S (Hydrogen Sulfide)",
    "CO2 (Carbon Dioxide)",
    "O2 (Oxygen)",
    "H2O (Water Vapor)",
    "N2 (Nitrogen)",
];

/// Stoichiometric O2 (mol O2 per mol species) and CO2/H2O yields.
/// Formula CxHyOz: O2 = x + y/4 - z/2.
/// Tuple: (C, H, O, S, HHV Btu/scf, LHV Btu/scf, MW lb/lbmol).
pub const SPECIES: &[(&'static str, f64, f64, f64, f64, f64, f64, f64)] = &[
    ("y_ch4", 1.0, 4.0, 0.0, 0.0, 1010.0, 910.0, 16.043),
    ("y_c2h2", 2.0, 2.0, 0.0, 0.0, 1440.0, 1390.0, 26.038),
    ("y_c2h4", 2.0, 4.0, 0.0, 0.0, 1600.0, 1500.0, 28.054),
    ("y_c2h6", 2.0, 6.0, 0.0, 0.0, 1770.0, 1619.0, 30.07),
    ("y_c3h6", 3.0, 6.0, 0.0, 0.0, 2330.0, 2180.0, 42.081),
    ("y_c3h8", 3.0, 8.0, 0.0, 0.0, 2516.0, 2315.0, 44.097),
    ("y_c4h10", 4.0, 10.0, 0.0, 0.0, 3262.0, 3012.0, 58.124),
    ("y_c5h12", 5.0, 12.0, 0.0, 0.0, 4000.0, 3700.0, 72.151),
    ("y_c6h14", 6.0, 14.0, 0.0, 0.0, 4750.0, 4400.0, 86.178),
    ("y_co", 1.0, 0.0, 1.0, 0.0, 321.0, 321.0, 28.01),
    ("y_h2", 0.0, 2.0, 0.0, 0.0, 325.0, 275.0, 2.016),
    ("y_h2s", 0.0, 2.0, 0.0, 1.0, 672.0, 619.0, 34.082),
    ("y_co2", 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 44.01),
    ("y_o2", 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 31.999),
    ("y_h2o", 0.0, 2.0, 1.0, 0.0, 0.0, 0.0, 18.015),
    ("y_n2", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 28.013),
];

/// Dry air O2 fraction used by the source program (report header 20.99% O2;
/// the theoretical-air row 9.85 SCF/SCF for Average Natural Gas implies 0.209).
pub const O2_AIR: f64 = 0.209;
pub const N2_AIR: f64 = 0.791;
/// Air density at 60 F, 14.7 psia, lb/scf (source Gas & Liquid Properties).
pub const RHO_AIR_STD: f64 = 0.076335;
/// scf -> Nm3 @0 C and Btu/scf -> MJ/Nm3 conversion used by the source SI columns.
pub const BTU_SCF_TO_MJ_NM3: f64 = 0.0393822;
