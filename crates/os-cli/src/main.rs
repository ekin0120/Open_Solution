use std::env;
use std::error::Error;
use std::path::PathBuf;

use os_core::storage::BUSINESS_TABLES;
use os_core::units::{
    parse_length, parse_temperature, unit_audit, Density, FlowBasis, HeatingBasis,
    HeatingValue, Length, MassFlow, Pressure, Temperature, UnitSystem, VolumetricFlow,
};
use os_core::{calc, Database, Fuel};

const MODULES: &[(&str, &str)] = &[
    ("fuels", "fuel properties / heating values"),
    ("burners", "gas & combustion burners"),
    ("blowers", "combustion air blowers"),
    ("valves", "shut-off and control valves"),
    ("orifices", "metering orifices"),
    ("pipes", "pipe schedules and dimensions"),
    ("fittings", "fittings and loss coefficients"),
    ("materials", "material limits and density"),
    ("regulators", "gas pressure regulators"),
];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "-h" | "--help" | "help") {
        usage();
        return;
    }
    if let Err(e) = run(&args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn default_db() -> PathBuf {
    match env::var("OPEN_SOLUTIONS_DB") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from("data/open_solutions.db"),
    }
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn num(args: &[String], key: &str) -> Result<f64, Box<dyn Error>> {
    let s = arg(args, key).ok_or_else(|| format!("missing {key}"))?;
    Ok(s.parse()?)
}

fn opt_num(args: &[String], key: &str) -> Result<Option<f64>, Box<dyn Error>> {
    match arg(args, key) {
        Some(s) => Ok(Some(s.parse()?)),
        None => Ok(None),
    }
}

fn has(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Opens (creating if needed) and migrates the SQLite database.
fn open_db(args: &[String]) -> Result<(Database, PathBuf), Box<dyn Error>> {
    let path = arg(args, "--db").map(PathBuf::from).unwrap_or_else(default_db);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let db = Database::open(&path)?;
    db.migrate()?;
    Ok((db, path))
}

fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    match args[0].as_str() {
        "init-db" => {
            let (db, path) = open_db(args)?;
            println!("database : {}", path.display());
            println!("schema   : v{}", db.schema_version()?);
            println!("tables   : {}", db.table_names()?.join(", "));
            println!("unit rules: {} declared columns", db.unit_convention_count()?);
        }
        "tables" => {
            let (db, path) = open_db(args)?;
            println!("{}", path.display());
            for t in BUSINESS_TABLES {
                if let Some(n) = db.count_rows(t)? {
                    println!("  {t:<18} {n} rows");
                }
            }
        }
        "modules" => {
            for (m, desc) in MODULES {
                println!("  {m:<14} {desc}");
            }
        }
        "unit-rules" => {
            let (db, _) = open_db(args)?;
            println!("Canonical units stored in the database:");
            for (table, column, unit, note) in db.unit_conventions()? {
                let name = format!("{table}.{column}");
                match note {
                    Some(n) => println!("  {name:<32} {unit:<38} ! {n}"),
                    None => println!("  {name:<32} {unit}"),
                }
            }
        }
        "list-fuels" => {
            let (db, _) = open_db(args)?;
            let fuels = db.list_fuels()?;
            if fuels.is_empty() {
                println!("(no fuels yet - use `add-fuel` or insert your own rows)");
            }
            for f in fuels {
                println!(
                    "  {:<16} {:<28} LHV={:?} HHV={:?} source={}",
                    f.code, f.name, f.lhv_mj_kg, f.hhv_mj_kg, f.source
                );
            }
        }
        "add-fuel" => {
            let (db, _) = open_db(args)?;
            let code = arg(args, "--code").ok_or_else(|| "missing --code".to_string())?;
            let name = arg(args, "--name").ok_or_else(|| "missing --name".to_string())?;
            let mut fuel = Fuel::user_supplied(&code, &name);
            fuel.lhv_mj_kg = opt_num(args, "--lhv")?;
            fuel.hhv_mj_kg = opt_num(args, "--hhv")?;
            fuel.density_kg_nm3 = opt_num(args, "--density")?;
            fuel.note = arg(args, "--note");
            if let Some(s) = arg(args, "--source") {
                fuel.source = s;
            }
            let id = db.insert_fuel(&fuel)?;
            println!("inserted fuel id={id} code={}", fuel.code);
        }

        // ---- Unit bug fix: reference table, self-check, dual output ----

        "verify-units" => {
            // The very same physics entered in both unit systems must converge
            // on one SI result. This is the executable regression test for the
            // legacy "wrong output after switching units" bug.
            let d_si = Length::from_mm(100.0);
            let d_im = Length::from_inch(d_si.as_inch());
            let q_si = VolumetricFlow::from_m3_h(36.0, FlowBasis::Actual);
            let q_im = VolumetricFlow::from_cfm(q_si.as_cfm(), FlowBasis::Actual);
            let p_si = Pressure::from_kpa_abs(101.325);
            let p_im = Pressure::from_psia(p_si.as_psia());
            let t_si = Temperature::from_c(0.0);
            let t_im = Temperature::from_f(t_si.as_f());
            let m_si = MassFlow::from_kg_h(1000.0);
            let m_im = MassFlow::from_lb_h(m_si.as_lb_h());
            let hv_si = HeatingValue::from_mj_kg(50.0, HeatingBasis::Lhv);
            let hv_im = HeatingValue::from_btu_lb(hv_si.as_btu_lb(), HeatingBasis::Lhv);

            let rows = [
                ("length (mm)", d_si.as_mm(), d_im.as_mm()),
                ("temperature (K)", t_si.as_k(), t_im.as_k()),
                ("pressure (kPa abs)", p_si.as_kpa_abs(), p_im.as_kpa_abs()),
                (
                    "gas density (kg/m3)",
                    calc::ideal_gas_density(p_si, t_si, 28.97).as_kg_m3(),
                    calc::ideal_gas_density(p_im, t_im, 28.97).as_kg_m3(),
                ),
                (
                    "pipe velocity (m/s)",
                    calc::pipe_velocity(q_si, d_si).as_m_s(),
                    calc::pipe_velocity(q_im, d_im).as_m_s(),
                ),
                (
                    "heat release (MW)",
                    calc::heat_release(m_si, hv_si).as_mw(),
                    calc::heat_release(m_im, hv_im).as_mw(),
                ),
            ];

            println!("End-to-end unit consistency (SI input vs Imperial input):");
            let mut bad = 0usize;
            for (label, si, im) in &rows {
                let delta = (*si - *im).abs();
                println!("  {label:<24} SI={:<16.9} Imperial={:<16.9} delta={delta:.3e}", si, im);
                if delta > 1e-6 {
                    bad += 1;
                }
            }
            if bad > 0 {
                return Err(format!("{bad} unit-consistency check(s) FAILED").into());
            }
            println!("  PASS - imperial and metric inputs converge on the same SI value.");
        }
        "unit-check" => {
            println!("Reference conversions (the legacy failure points):");
            for (label, value) in unit_audit() {
                println!("  {label:<28} {value}");
            }
            println!("\nRule: SI internally, conversion only at the I/O boundary, exactly once.");
        }
        "convert" => {
            let system = if has(args, "--imperial") {
                UnitSystem::Imperial
            } else {
                UnitSystem::Si
            };
            if let Some(s) = arg(args, "--length") {
                let l = parse_length(&s, system)?;
                println!("length      : {:.6} mm | {:.6} in | {:.6} m | {:.6} ft",
                    l.as_mm(), l.as_inch(), l.as_m(), l.as_ft());
            } else if let Some(s) = arg(args, "--temp") {
                let t = parse_temperature(&s, system)?;
                println!("temperature : {:.4} C | {:.4} F | {:.4} K",
                    t.as_c(), t.as_f(), t.as_k());
            } else {
                return Err("convert needs --length \"<n> <unit>\" or --temp \"<n> <unit>\"".into());
            }
        }
        "calc-gas" => {
            let m = num(args, "--m")?;
            let p = if let Some(v) = opt_num(args, "--psia")? {
                Pressure::from_psia(v)
            } else if let Some(v) = opt_num(args, "--psig")? {
                Pressure::from_psig(v)
            } else if let Some(v) = opt_num(args, "--p")? {
                Pressure::from_kpa_abs(v)
            } else {
                Pressure::from_kpa_abs(101.325)
            };
            let t = if let Some(v) = opt_num(args, "--tf")? {
                Temperature::from_f(v)
            } else if let Some(v) = opt_num(args, "--tc")? {
                Temperature::from_c(v)
            } else if let Some(v) = opt_num(args, "--t")? {
                Temperature::from_k(v)
            } else {
                Temperature::from_c(0.0)
            };
            let rho = calc::ideal_gas_density(p, t, m);
            println!("density = {:.6} kg/m3 | {:.6} lb/ft3", rho.as_kg_m3(), rho.as_lb_ft3());
            println!("state   = {:.3} kPa(abs) / {:.3} psia | {:.2} C / {:.2} F",
                p.as_kpa_abs(), p.as_psia(), t.as_c(), t.as_f());
        }
        "calc-pipe" => {
            let d = if let Some(s) = arg(args, "--d") {
                parse_length(&s, UnitSystem::Si)?
            } else if let Some(v) = opt_num(args, "--din")? {
                Length::from_inch(v)
            } else {
                Length::from_mm(num(args, "--dmm")?)
            };
            let q = if let Some(v) = opt_num(args, "--cfm")? {
                VolumetricFlow::from_cfm(v, FlowBasis::Actual)
            } else if let Some(v) = opt_num(args, "--flow")? {
                VolumetricFlow::from_m3_h(v, FlowBasis::Actual)
            } else {
                return Err("calc-pipe needs --flow <m3/h> or --cfm <ft3/min>".into());
            };
            let v = calc::pipe_velocity_checked(q, d)?;
            println!("velocity = {:.4} m/s | {:.4} ft/s | {:.4} km/h",
                v.as_m_s(), v.as_ft_s(), v.as_km_h());
            println!("bore     = {:.4} mm | {:.4} in", d.as_mm(), d.as_inch());
            println!("flow     = {:.4} m3/h (ACTUAL) | {:.4} CFM",
                q.as_m3_h(), q.as_cfm());
            if let (Some(rho), Some(mu)) = (opt_num(args, "--rho")?, opt_num(args, "--mu")?) {
                println!("Re       = {:.0}", calc::reynolds(Density::from_kg_m3(rho), v, d, mu));
            }
        }
        "calc-heat" => {
            let mdot = if let Some(v) = opt_num(args, "--lbh")? {
                MassFlow::from_lb_h(v)
            } else if let Some(v) = opt_num(args, "--mdot")? {
                MassFlow::from_kg_h(v)
            } else {
                return Err("calc-heat needs --mdot <kg/h> or --lbh <lb/h>".into());
            };
            let basis = if has(args, "--hhv") { HeatingBasis::Hhv } else { HeatingBasis::Lhv };
            let hv = if let Some(v) = opt_num(args, "--btulb")? {
                HeatingValue::from_btu_lb(v, basis)
            } else if let Some(v) = opt_num(args, "--lhv")? {
                HeatingValue::from_mj_kg(v, basis)
            } else {
                return Err("calc-heat needs --lhv <MJ/kg> or --btulb <BTU/lb>".into());
            };
            let p = calc::heat_release(mdot, hv);
            println!("heat release = {:.4} kW | {:.2} BTU/h | {:.6} MW | {:.4} hp",
                p.as_kw(), p.as_btu_h(), p.as_mw(), p.as_hp());
            println!("fuel basis   = {:?} (never silently substituted)", hv.basis());
        }
        other => {
            eprintln!("unknown command: {other}");
            usage();
        }
    }
    Ok(())
}

fn usage() {
    println!(
        "Open Solutions 0.2.0 - independent combustion equipment toolkit\n\
         \n\
         Usage: open-solutions <command> [options]   (db commands accept --db <path>)\n\
         \n\
         Database:\n\
           init-db / tables / modules / unit-rules / list-fuels\n\
           add-fuel --code C --name N [--lhv --hhv --density --source --note]\n\
         \n\
         Units (imperial/metric bug fix):\n\
           verify-units                      end-to-end SI vs Imperial consistency\n\
           unit-check                        reference conversion table\n\
           unit-rules                        canonical units of every DB column\n\
           convert --length \"4 in\"           explicit length, dual output\n\
           convert --temp \"212 F\"             explicit temperature, dual output\n\
         \n\
         Calculations (metric OR imperial input, always dual output):\n\
           calc-gas  --m g/mol [--p kPa(abs) | --psia | --psig] [--t K | --tc | --tf]\n\
           calc-pipe --d \"100 mm\" (or \"4 in\") --flow m3/h | --cfm  [--rho --mu]\n\
           calc-heat --mdot kg/h | --lbh  --lhv MJ/kg | --btulb  [--hhv]\n\
         \n\
         Database path: --db <path> or env OPEN_SOLUTIONS_DB\n\
         Default: ./data/open_solutions.db"
    );
}
