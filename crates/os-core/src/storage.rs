//! SQLite persistence layer.
//!
//! The schema lives in `app/schema.sql` and is embedded at compile time, so a
//! fresh database is one `migrate()` call away. `bundled` SQLite means no
//! system sqlite3.dll is needed.
//!
//! Unit conventions are stored as data (`unit_conventions`), not assumptions
//! in code — part of the imperial/metric bug fix.

use rusqlite::{params, Connection, Result};

use crate::model::Fuel;

/// A gas fuel from the imported `GasFuel` source table: 10 component volume
/// fractions (%) plus the published specific gravity and heating values.
#[derive(Debug, Clone)]
pub struct GasFuel {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub ch4: f64, pub c2h6: f64, pub c3h8: f64, pub c4h10: f64,
    pub co: f64, pub h2: f64, pub co2: f64, pub o2: f64, pub n2: f64, pub h2o: f64,
    pub gravity: Option<f64>,
    pub gross_heat_btu_scf: Option<f64>,
    pub net_heat_btu_scf: Option<f64>,
    pub cp: Option<f64>,
    pub gross_heat_si: Option<f64>,
    pub net_heat_si: Option<f64>,
    pub source: String,
}

/// Embedded DDL; the file sits at `app/schema.sql` (three levels up from `src/`).
pub const SCHEMA_SQL: &str = include_str!("../../../schema.sql");

/// Tables that hold engineering data (used for whitelisting).
pub const BUSINESS_TABLES: &[&str] = &[
    "fuels",
    "fuel_gas_components",
    "burners",
    "blowers",
    "valves",
    "orifices",
    "pipes",
    "fittings",
    "materials",
    "regulators",
    "catalog_refs",
    "projects",
    "calc_runs",
    "unit_conventions",
];

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Ok(Self { conn })
    }

    /// Idempotent: all DDL uses IF NOT EXISTS / INSERT OR IGNORE.
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA_SQL)
    }

    pub fn schema_version(&self) -> Result<String> {
        self.conn.query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
    }

    pub fn table_names(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    /// Row count for a whitelisted table. Dynamic identifiers cannot be bound
    /// as parameters, hence the explicit allow-list.
    pub fn count_rows(&self, table: &str) -> Result<Option<i64>> {
        if !BUSINESS_TABLES.contains(&table) {
            return Ok(None);
        }
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let n = self.conn.query_row(&sql, [], |row| row.get::<_, i64>(0))?;
        Ok(Some(n))
    }

    /// Declared canonical unit for every numeric column, with warnings.
    pub fn unit_conventions(&self) -> Result<Vec<(String, String, String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT table_name, column_name, canonical_unit, note \
             FROM unit_conventions ORDER BY table_name, column_name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;
        rows.collect()
    }

    pub fn unit_convention_count(&self) -> Result<i64> {
        self.conn.query_row("SELECT COUNT(*) FROM unit_conventions", [], |row| row.get(0))
    }

    pub fn insert_fuel(&self, fuel: &Fuel) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO fuels (code, name, lhv_mj_kg, hhv_mj_kg, density_kg_nm3, source, note) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &fuel.code,
                &fuel.name,
                fuel.lhv_mj_kg,
                fuel.hhv_mj_kg,
                fuel.density_kg_nm3,
                &fuel.source,
                &fuel.note,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_fuels(&self) -> Result<Vec<Fuel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, code, name, lhv_mj_kg, hhv_mj_kg, density_kg_nm3, source, note \
             FROM fuels ORDER BY code",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Fuel {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                lhv_mj_kg: row.get(3)?,
                hhv_mj_kg: row.get(4)?,
                density_kg_nm3: row.get(5)?,
                source: row.get(6)?,
                note: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn list_gas_fuels(&self) -> Result<Vec<GasFuel>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, code, name, ch4, c2h6, c3h8, c4h10, co, h2, co2, o2, n2, h2o, \
                    gravity, gross_heat_btu_scf, net_heat_btu_scf, cp, gross_heat_si, net_heat_si, source \
             FROM fuel_gas_components ORDER BY code",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GasFuel {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                ch4: row.get(3)?,
                c2h6: row.get(4)?,
                c3h8: row.get(5)?,
                c4h10: row.get(6)?,
                co: row.get(7)?,
                h2: row.get(8)?,
                co2: row.get(9)?,
                o2: row.get(10)?,
                n2: row.get(11)?,
                h2o: row.get(12)?,
                gravity: row.get(13)?,
                gross_heat_btu_scf: row.get(14)?,
                net_heat_btu_scf: row.get(15)?,
                cp: row.get(16)?,
                gross_heat_si: row.get(17)?,
                net_heat_si: row.get(18)?,
                source: row.get(19)?,
            })
        })?;
        rows.collect()
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_builds_and_is_idempotent() {
        let db = Database::in_memory().unwrap();
        db.migrate().unwrap();
        db.migrate().unwrap();
        assert_eq!(db.schema_version().unwrap(), "2");
        let tables = db.table_names().unwrap();
        for expected in BUSINESS_TABLES {
            assert!(tables.contains(&expected.to_string()), "missing table {expected}");
        }
        assert_eq!(db.count_rows("fuels").unwrap(), Some(0));
        assert!(matches!(db.count_rows("not_a_table"), Ok(None)));
    }

    #[test]
    fn unit_conventions_are_declared_data() {
        let db = Database::in_memory().unwrap();
        db.migrate().unwrap();
        let rules = db.unit_conventions().unwrap();
        assert!(rules.len() >= 20, "only {} rules", rules.len());
        assert!(rules.iter().any(|(t, c, u, _)| {
            t.as_str() == "pipes" && c.as_str() == "nps_inch" && u.contains("NOMINAL")
        }));
        assert!(rules.iter().any(|(t, c, _, n)| {
            t.as_str() == "fuels"
                && c.as_str() == "lhv_mj_kg"
                && n.as_deref().unwrap_or("").contains("never substitute HHV")
        }));
    }

    #[test]
    fn fuel_crud_roundtrip() {
        let db = Database::in_memory().unwrap();
        db.migrate().unwrap();
        let mut fuel = Fuel::user_supplied("NG-TEST", "Test gas");
        fuel.lhv_mj_kg = Some(48.0);
        fuel.hhv_mj_kg = Some(53.1);
        fuel.density_kg_nm3 = Some(0.78);
        let id = db.insert_fuel(&fuel).unwrap();
        assert!(id > 0);

        let fuels = db.list_fuels().unwrap();
        assert_eq!(fuels.len(), 1);
        assert_eq!(fuels[0].code, "NG-TEST");
        assert_eq!(fuels[0].lhv_mj_kg, Some(48.0));
        assert_eq!(fuels[0].source, "user-supplied");
    }

    #[test]
    fn foreign_key_is_enforced() {
        let db = Database::in_memory().unwrap();
        db.migrate().unwrap();
        let bad = db.connection().execute(
            "INSERT INTO burners (code, name, fuel_code) VALUES ('B1', 'b', 'NO-SUCH-FUEL')",
            [],
        );
        assert!(bad.is_err());
    }
}
