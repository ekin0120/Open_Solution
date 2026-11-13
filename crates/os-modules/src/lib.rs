//! Data-driven module registry and engineering kernels for Open Solutions.
//!
//! The GUI (and any other client) only needs three calls:
//!   * `catalog()`          — module metadata + field definitions for rendering
//!   * `compute(id, inputs)`— SI-in / SI-out calculation
//!   * `quantity::*`        — explicit boundary unit conversion

pub mod combustion_fuels;
pub mod defs;
pub mod engine;
pub mod fluids_data;
pub mod materials;
pub mod quantity;

pub use defs::{all_modules, tool_pages, Group, ModuleDef, ModuleJson};
pub use engine::{compute, ComputeResult, RowJson};

/// Complete UI catalog: 37 numerical modules + 4 utility pages.
pub fn catalog() -> Vec<ModuleJson> {
    let mut out: Vec<ModuleJson> = all_modules().iter().map(|m| m.json()).collect();
    out.extend(tool_pages());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_every_visible_group() {
        let c = catalog();
        assert_eq!(c.len(), 41, "37 modules + 4 tool pages");
        let count = |g: &str| c.iter().filter(|m| m.group_key == g).count();
        assert_eq!(count("Reference"), 19);
        assert_eq!(count("Applications"), 11);
        assert_eq!(count("SystemDesign"), 7);
        assert_eq!(count("Tools"), 4);
        for m in &c {
            if m.group_key == "Tools" { continue; }
            assert!(!m.fields.is_empty(), "{} has no inputs", m.id);
        }
    }
}
