-- Open Solutions / SQLite schema v2
-- Clean-room reimplementation: this file contains STRUCTURE ONLY.
-- Engineering data is imported separately by tools/import_mdb.py, which also
-- records provenance (source file, SHA256, original table/column names).

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO meta(key, value) VALUES
    ('schema_version', '2'),
    ('application',    'Open Solutions'),
    ('data_origin',    'see source_files / source_tables'),
    ('notice',         'Schema only; data provenance is recorded per row in the source column.');

-- Provenance for data imported from an installed E-Solutions .mdb set.
-- Populated by tools/import_mdb.py; DDL lives here so a fresh `init-db`
-- produces the same structure. No vendor data is embedded in the source tree.
CREATE TABLE IF NOT EXISTS source_files (
    id            INTEGER PRIMARY KEY,
    mdb           TEXT NOT NULL UNIQUE,
    bytes         INTEGER NOT NULL,
    sha256        TEXT NOT NULL,
    jet_version   INTEGER,
    table_count   INTEGER NOT NULL,
    row_count     INTEGER NOT NULL,
    imported_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS source_tables (
    id            INTEGER PRIMARY KEY,
    mdb           TEXT NOT NULL,
    source_table  TEXT NOT NULL,
    sqlite_table  TEXT NOT NULL UNIQUE,
    row_count     INTEGER NOT NULL,
    column_count  INTEGER NOT NULL,
    columns_json  TEXT NOT NULL,
    UNIQUE (mdb, source_table)
);

-- Fuel.mdb
CREATE TABLE IF NOT EXISTS fuels (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    code           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    lhv_mj_kg      REAL CHECK (lhv_mj_kg IS NULL OR lhv_mj_kg > 0),
    hhv_mj_kg      REAL CHECK (hhv_mj_kg IS NULL OR hhv_mj_kg > 0),
    density_kg_nm3 REAL CHECK (density_kg_nm3 IS NULL OR density_kg_nm3 > 0),
    source         TEXT NOT NULL DEFAULT 'user-supplied',
    note           TEXT,
    updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Fuel.mdb / GasFuel — gas composition (volume %), imported from the source set.
-- `code` mirrors fuels.code for the gas members; rows come from import_mdb.py.
CREATE TABLE IF NOT EXISTS fuel_gas_components (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    code                TEXT NOT NULL UNIQUE,
    name                TEXT NOT NULL,
    ch4                 REAL,
    c2h6                REAL,
    c3h8                REAL,
    c4h10               REAL,
    co                  REAL,
    h2                  REAL,
    co2                 REAL,
    o2                  REAL,
    n2                  REAL,
    h2o                 REAL,
    gravity             REAL,
    gross_heat_btu_scf  REAL,
    net_heat_btu_scf    REAL,
    cp                  REAL,
    gross_heat_si       REAL,
    net_heat_si         REAL,
    source              TEXT NOT NULL DEFAULT 'user-supplied',
    note                TEXT,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Gasbrn / GasbrnConvert / Combburn2 / AsphaltBrn1
CREATE TABLE IF NOT EXISTS burners (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    fuel_code       TEXT,
    capacity_kw_min REAL CHECK (capacity_kw_min IS NULL OR capacity_kw_min >= 0),
    capacity_kw_max REAL CHECK (capacity_kw_max IS NULL OR capacity_kw_max >= 0),
    turndown        REAL CHECK (turndown IS NULL OR turndown >= 1),
    excess_air_pct  REAL,
    source          TEXT NOT NULL DEFAULT 'user-supplied',
    note            TEXT,
    FOREIGN KEY (fuel_code) REFERENCES fuels(code)
);

-- Blower1 / EclipseBlower1
CREATE TABLE IF NOT EXISTS blowers (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    code             TEXT NOT NULL UNIQUE,
    name             TEXT NOT NULL,
    flow_m3h_max     REAL CHECK (flow_m3h_max IS NULL OR flow_m3h_max > 0),
    pressure_kpa_max REAL CHECK (pressure_kpa_max IS NULL OR pressure_kpa_max > 0),
    power_kw         REAL CHECK (power_kw IS NULL OR power_kw > 0),
    source           TEXT NOT NULL DEFAULT 'user-supplied',
    note             TEXT
);

-- Valve1 / Valve1Convert
CREATE TABLE IF NOT EXISTS valves (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    code              TEXT NOT NULL UNIQUE,
    name              TEXT NOT NULL,
    dn_mm             REAL CHECK (dn_mm IS NULL OR dn_mm > 0),
    cv                REAL CHECK (cv IS NULL OR cv > 0),
    max_pressure_kpa  REAL CHECK (max_pressure_kpa IS NULL OR max_pressure_kpa > 0),
    source            TEXT NOT NULL DEFAULT 'user-supplied',
    note              TEXT
);

-- Orifice / Orifice1
CREATE TABLE IF NOT EXISTS orifices (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    code         TEXT NOT NULL UNIQUE,
    name         TEXT NOT NULL,
    diameter_mm  REAL CHECK (diameter_mm IS NULL OR diameter_mm > 0),
    cd           REAL CHECK (cd IS NULL OR (cd > 0 AND cd <= 1)),
    source       TEXT NOT NULL DEFAULT 'user-supplied',
    note         TEXT
);

-- Pipe1 / PipeSchedule40
CREATE TABLE IF NOT EXISTS pipes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL,
    schedule    TEXT NOT NULL,
    nps_inch    REAL NOT NULL CHECK (nps_inch > 0),
    od_mm       REAL CHECK (od_mm IS NULL OR od_mm > 0),
    wall_mm     REAL CHECK (wall_mm IS NULL OR wall_mm > 0),
    inner_d_mm  REAL CHECK (inner_d_mm IS NULL OR inner_d_mm > 0),
    source      TEXT NOT NULL DEFAULT 'user-supplied',
    note        TEXT,
    UNIQUE (schedule, nps_inch)
);

-- Fitting1
CREATE TABLE IF NOT EXISTS fittings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    code       TEXT NOT NULL UNIQUE,
    name       TEXT NOT NULL,
    dn_mm      REAL CHECK (dn_mm IS NULL OR dn_mm > 0),
    k_factor   REAL CHECK (k_factor IS NULL OR k_factor >= 0),
    source     TEXT NOT NULL DEFAULT 'user-supplied',
    note       TEXT
);

-- Material / Material1
CREATE TABLE IF NOT EXISTS materials (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    code            TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    max_temp_c      REAL,
    density_kg_m3   REAL CHECK (density_kg_m3 IS NULL OR density_kg_m3 > 0),
    source          TEXT NOT NULL DEFAULT 'user-supplied',
    note            TEXT
);

-- GasPressRegulator
CREATE TABLE IF NOT EXISTS regulators (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    code             TEXT NOT NULL UNIQUE,
    name             TEXT NOT NULL,
    inlet_max_kpa    REAL CHECK (inlet_max_kpa IS NULL OR inlet_max_kpa > 0),
    outlet_min_kpa   REAL CHECK (outlet_min_kpa IS NULL OR outlet_min_kpa >= 0),
    outlet_max_kpa   REAL CHECK (outlet_max_kpa IS NULL OR outlet_max_kpa > 0),
    source           TEXT NOT NULL DEFAULT 'user-supplied',
    note             TEXT
);

-- User-owned references to their own vendor literature (no vendor data stored).
CREATE TABLE IF NOT EXISTS catalog_refs (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    vendor     TEXT NOT NULL,
    document   TEXT NOT NULL,
    url        TEXT,
    added_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS calc_runs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    module      TEXT NOT NULL,
    input_json  TEXT NOT NULL DEFAULT '{}',
    result_json TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_burners_fuel    ON burners(fuel_code);
CREATE INDEX IF NOT EXISTS idx_pipes_schedule  ON pipes(schedule, nps_inch);
CREATE INDEX IF NOT EXISTS idx_calc_runs_proj  ON calc_runs(project_id);

-- Explicit unit conventions: every numeric column declares its canonical unit.
-- The application reads this at startup, so units are auditable data rather
-- than an implicit assumption buried in code. This is part of the unit bug fix.
CREATE TABLE IF NOT EXISTS unit_conventions (
    table_name     TEXT NOT NULL,
    column_name    TEXT NOT NULL,
    canonical_unit TEXT NOT NULL,
    note           TEXT,
    PRIMARY KEY (table_name, column_name)
);

INSERT OR IGNORE INTO unit_conventions(table_name, column_name, canonical_unit, note) VALUES
 ('fuels','lhv_mj_kg','MJ/kg (LHV)','lower heating value; never substitute HHV'),
 ('fuels','hhv_mj_kg','MJ/kg (HHV)','higher heating value'),
 ('fuels','density_kg_nm3','kg/Nm3 @ 0 C / 101.325 kPa','standard-state density, NOT actual density'),
 ('burners','capacity_kw_min','kW',NULL),
 ('burners','capacity_kw_max','kW',NULL),
 ('burners','excess_air_pct','% of stoichiometric air',NULL),
 ('blowers','flow_m3h_max','m3/h ACTUAL','not Nm3/h'),
 ('blowers','pressure_kpa_max','kPa GAUGE','gauge unless explicitly marked absolute'),
 ('blowers','power_kw','kW',NULL),
 ('valves','dn_mm','mm',NULL),
 ('valves','cv','US gpm @ 1 psi','Kv = 0.865 * Cv'),
 ('orifices','diameter_mm','mm',NULL),
 ('orifices','cd','dimensionless 0..1',NULL),
 ('pipes','nps_inch','inch NOMINAL','NPS is a nominal size, NOT a measured OD; never multiply by 25.4 to obtain OD'),
 ('pipes','od_mm','mm',NULL),
 ('pipes','wall_mm','mm',NULL),
 ('pipes','inner_d_mm','mm',NULL),
 ('fittings','dn_mm','mm',NULL),
 ('fittings','k_factor','dimensionless',NULL),
 ('materials','max_temp_c','deg C','affine scale; ratios must be formed in kelvin'),
 ('materials','density_kg_m3','kg/m3',NULL),
 ('regulators','inlet_max_kpa','kPa',NULL),
 ('regulators','outlet_min_kpa','kPa',NULL),
 ('regulators','outlet_max_kpa','kPa',NULL);

INSERT OR IGNORE INTO meta(key, value) VALUES
 ('unit_convention', 'SI canonical; conversions only at the I/O boundary, exactly once'),
 ('unit_convention_table', 'unit_conventions');
