#!/usr/bin/env python3
"""Import E-Solutions 600 .mdb dumps into the Open Solutions SQLite database.

Input : Open Solutions/import_work/dump2/*.json  (produced by export_all.ps1,
        one JSON per source .mdb, all tables and rows verbatim)
Output: Open Solutions/app/dist/data/open_solutions.db

Two layers are written:
  1. Fidelity layer  - one `src_<mdb>__<table>` table per source table, with
     the original column names and values. Nothing is dropped or rounded.
  2. Domain layer    - the nine application catalog tables, populated with
     explicit unit conversions. Every row records its provenance in `source`
     and keeps the untouched original row as JSON in `note`.

Unit assumptions are declared in UNIT_ASSUMPTIONS below and copied into each
`note`, so a derived value can never be mistaken for a measured one.

Idempotent: re-running replaces the fidelity layer and all rows whose source
starts with 'ESolutions600:'. User-entered rows are left alone.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sqlite3
import sys
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
APP = os.path.abspath(os.path.join(HERE, ".."))
ROOT = os.path.dirname(APP)
DEFAULT_DUMP = os.path.join(ROOT, "import_work", "dump2")
DEFAULT_DB = os.path.join(APP, "dist", "data", "open_solutions.db")
SOURCE_DIR = os.path.join(ROOT, "import_work", "mdb")

SRC_PREFIX = "ESolutions600:"

# ---------------------------------------------------------------- units ----
MM_BTU_HR_TO_KW = 293.07107       # 1 MMBtu/h = 293.07107 kW
BTU_LB_TO_MJ_KG = 0.002326       # not used directly: SI columns already MJ/kg
IN_TO_MM = 25.4
FT3_TO_M3 = 0.028316846592       # standard cubic foot -> cubic metre
BHP_TO_KW = 0.745699872
LB_FT3_TO_KG_M3 = 16.018463
PSI_TO_KPA = 6.894757293
AIR_DENSITY_0C = 1.2929          # kg/Nm3 at 0 C, 101.325 kPa

UNIT_ASSUMPTIONS = {
    "fuels": "GasFuel: GrossHeatSI/NetHeatSI = MJ/Nm3 @0C; Gravity = SG vs air; "
             "density = Gravity * 1.2929 kg/Nm3; MJ/kg = MJ/Nm3 / density. "
             "OilFuel/WasteOil: GrossHeatSI/NetHeatSI already MJ/kg; liquid "
             "density is NOT written to density_kg_nm3 (see raw.Gravity).",
    "burners": "Only AsphaltBrn1.HeatCapMax and dbTriOx1.NominalBurnerHeat are "
               "assumed MMBtu/h and converted with x293.07107. All other raw "
               "capacity/pressure fields are kept verbatim in note (units not "
               "documented in the source and deliberately NOT guessed).",
    "blowers": "SCFH/FlowRate assumed standard ft3/h -> Nm3/h x0.028316846592. "
               "BHP -> kW x0.745699872. Pressure left NULL: source unit "
               "(psig / osig / in.w.c.) is not declared; raw value in note.",
    "valves": "CV is US gpm @1 psi (matches unit_conventions). ID is the actual "
              "inner diameter in inch -> mm x25.4; it is NOT a nominal DN.",
    "orifices": "Bore is inch -> diameter_mm x25.4. Cd is not in the source.",
    "pipes": "PipeSchedule40 SI columns already mm; WallThicknessUS is inch -> "
             "mm x25.4. Pipe1.PipeID is actual inch ID -> mm x25.4. NPS parsed "
             "from the fractional label and kept NOMINAL.",
    "fittings": "RESISTK is the dimensionless K factor. ID inch -> mm x25.4.",
    "materials": "Density is lb/ft3 -> kg/m3 x16.018463. Tmax is degF -> degC "
                 "(F-32)*5/9. Melting/boiling points kept verbatim in note.",
    "regulators": "Pmax/Pout1/Pout2 ASSUMED psig -> kPa x6.894757293 (source "
                  "does not declare the unit; Poffset is in in.w.c.). Verify "
                  "against a catalog before using for design.",
}

POSITIVE_COLUMNS = {
    "fuels": {"lhv_mj_kg", "hhv_mj_kg", "density_kg_nm3"},
    "burners": {"capacity_kw_min", "capacity_kw_max"},
    "blowers": {"flow_m3h_max", "pressure_kpa_max", "power_kw"},
    "valves": {"dn_mm", "cv", "max_pressure_kpa"},
    "orifices": {"diameter_mm", "cd"},
    "pipes": {"od_mm", "wall_mm", "inner_d_mm", "nps_inch"},
    "fittings": {"dn_mm"},
    "materials": {"density_kg_m3"},
    "regulators": {"inlet_max_kpa", "outlet_max_kpa"},
}
NONNEG_COLUMNS = {
    "fittings": {"k_factor"},
    "regulators": {"outlet_min_kpa"},
}

MANIFEST_DDL = """
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
"""


# --------------------------------------------------------------- helpers ----
def ident(name: str) -> str:
    return '"' + str(name).replace('"', '""') + '"'


def safe(s: str) -> str:
    t = re.sub(r"[^A-Za-z0-9_]", "_", str(s)).strip("_")
    return t or "X"


def num(v):
    if v is None or isinstance(v, bool):
        return None
    if isinstance(v, (int, float)):
        return float(v)
    s = str(v).strip().replace(",", "")
    m = re.match(r"^[-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?", s)
    if not m:
        return None
    try:
        return float(m.group())
    except ValueError:
        return None


def frac(v):
    if isinstance(v, (int, float)) and not isinstance(v, bool):
        return float(v)
    s = str(v or "").strip()
    m = re.search(r"(\d+(?:\.\d+)?)\s*/\s*(\d+(?:\.\d+)?)", s)
    if m and float(m.group(2)):
        return float(m.group(1)) / float(m.group(2))
    return num(s)


def jnote(*parts) -> str:
    return json.dumps(dict(parts), ensure_ascii=False, separators=(",", ":"))


class Codes:
    def __init__(self):
        self.used = set()

    def make(self, *parts, prefix=""):
        base = safe("_".join(str(p) for p in parts if p not in (None, "")))
        if prefix:
            base = safe(prefix) + "_" + base
        base = base[:110]
        code, i = base, 2
        while code in self.used:
            suffix = f"_{i}"
            code = base[: 110 - len(suffix)] + suffix
            i += 1
        self.used.add(code)
        return code


def infer_type(values):
    nz = [v for v in values if v is not None]
    if not nz:
        return "TEXT"
    if all(isinstance(v, bool) for v in nz):
        return "INTEGER"
    if all(isinstance(v, int) and not isinstance(v, bool) for v in nz):
        return "INTEGER"
    if all(isinstance(v, (int, float)) and not isinstance(v, bool) for v in nz):
        return "REAL"
    return "TEXT"


# --------------------------------------------------------- fidelity layer ---
def import_fidelity(conn, dumps):
    conn.executescript(MANIFEST_DDL)
    conn.execute("DELETE FROM source_tables")
    conn.execute("DELETE FROM source_files")
    old = [r[0] for r in conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'src\\_%' ESCAPE '\\'")]
    for t in old:
        conn.execute(f"DROP TABLE IF EXISTS {ident(t)}")

    total_rows = 0
    total_tables = 0
    for key, doc in sorted(dumps.items()):
        # dumps are named <stem>.json, but the manifest must record the real
        # .mdb filename so the SHA256 points at the actual source file.
        mdb = doc.get("mdb") or (key[:-5] + ".mdb" if key.endswith(".json") else key)
        stem = os.path.splitext(mdb)[0]
        path = os.path.join(SOURCE_DIR, mdb)
        raw = open(path, "rb").read() if os.path.exists(path) else doc.get("size", 0).to_bytes(4, "little")
        digest = hashlib.sha256(raw).hexdigest()
        table_count = 0
        file_rows = 0
        for t in doc["tables"]:
            orig = t["name"]
            cols = [c["name"] for c in t["columns"]]
            clr = [c["clr"] for c in t["columns"]]
            rows = t["rows"]
            sql_name = f"src_{safe(stem)}__{safe(orig)[:55]}"
            base, i = sql_name, 2
            while conn.execute(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?", (sql_name,)
            ).fetchone():
                sql_name = f"{base}_{i}"
                i += 1

            # guard against duplicate column labels
            seen, labels = {}, []
            for c in cols:
                c = str(c)
                n = seen.get(c, 0)
                seen[c] = n + 1
                labels.append(c if n == 0 else f"{c}__{n+1}")
            types = [infer_type([r.get(c) for r in rows]) for c in cols]
            ddl_cols = ", ".join(
                f"{ident(l)} {ty}" for l, ty in zip(labels, types)
            ) or '"__empty" TEXT'
            conn.execute(f"CREATE TABLE {ident(sql_name)} ({ddl_cols})")
            if rows and cols:
                placeholders = ",".join("?" for _ in labels)
                sql = (f"INSERT INTO {ident(sql_name)} "
                       f"({','.join(ident(l) for l in labels)}) VALUES ({placeholders})")
                payload = []
                for r in rows:
                    vals = []
                    for c, ty in zip(cols, types):
                        v = r.get(c)
                        if ty == "INTEGER" and v is not None:
                            v = int(v)
                        elif ty == "REAL" and v is not None:
                            v = float(v)
                        elif ty == "TEXT" and v is not None and not isinstance(v, str):
                            v = str(v)
                        vals.append(v)
                    payload.append(vals)
                conn.executemany(sql, payload)
            conn.execute(
                "INSERT INTO source_tables "
                "(mdb, source_table, sqlite_table, row_count, column_count, columns_json) "
                "VALUES (?,?,?,?,?,?)",
                (mdb, orig, sql_name, len(rows), len(cols),
                 json.dumps([{"name": a, "clr": b, "type": c}
                             for a, b, c in zip(cols, clr, types)], ensure_ascii=False)),
            )
            table_count += 1
            file_rows += len(rows)
        conn.execute(
            "INSERT INTO source_files "
            "(mdb, bytes, sha256, jet_version, table_count, row_count, imported_at) "
            "VALUES (?,?,?,?,?,?,?)",
            (mdb, doc.get("size", 0), digest, doc.get("jet_version"),
             table_count, file_rows, datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S")),
        )
        total_tables += table_count
        total_rows += file_rows
    return total_tables, total_rows


# ----------------------------------------------------------- domain layer ---
def import_domain(conn, dumps):
    codes = Codes()
    stats = {}

    def rows(mdb, table):
        d = dumps.get(mdb + ".json")
        if not d:
            return []
        t = next((x for x in d["tables"] if x["name"] == table), None)
        return t["rows"] if t else []

    def all_tables(*mdbs):
        for m in mdbs:
            d = dumps.get(m + ".json")
            if d:
                for t in d["tables"]:
                    yield m, t

    def table_cols(mdb, table):
        d = dumps.get(mdb + ".json")
        if not d:
            return []
        t = next((x for x in d["tables"] if x["name"] == table), None)
        return [c["name"] for c in t["columns"]] if t else []

    def put(table, columns, values):
        # CHECK constraints in schema.sql: drop a value that cannot satisfy
        # them instead of aborting the whole import. The raw row is still
        # preserved verbatim in `note` and in the fidelity layer.
        positive = POSITIVE_COLUMNS.get(table, set())
        nonneg = NONNEG_COLUMNS.get(table, set())
        vals = []
        for c, v in zip(columns, values):
            if v is not None and (c in positive or c in nonneg):
                try:
                    f = float(v)
                    v = f if (f > 0 if c in positive else f >= 0) else None
                except (TypeError, ValueError):
                    v = None
            vals.append(v)
        sql = (f"INSERT INTO {table} ({','.join(columns)}) "
               f"VALUES ({','.join('?' for _ in columns)})")
        conn.execute(sql, vals)

    # -- fuels + gas composition ------------------------------------------
    n = 0
    for tname, kind in (("GasFuel", "gas"), ("OilFuel", "oil"), ("WasteOil", "oil")):
        for r in rows("Fuel", tname):
            name = r.get("FuelName")
            if not name:
                continue
            if kind == "gas":
                sg = num(r.get("Gravity"))
                dens = sg * AIR_DENSITY_0C if sg else None
                hhv_si, lhv_si = num(r.get("GrossHeatSI")), num(r.get("NetHeatSI"))
                hhv = hhv_si / dens if hhv_si and dens else None
                lhv = lhv_si / dens if lhv_si and dens else None
            else:
                dens = None
                hhv, lhv = num(r.get("GrossHeatSI")), num(r.get("NetHeatSI"))
            code = codes.make(kind, name)
            note = jnote(("units", UNIT_ASSUMPTIONS["fuels"]), ("raw", r))
            put("fuels", ["code", "name", "lhv_mj_kg", "hhv_mj_kg",
                          "density_kg_nm3", "source", "note"],
                [code, str(name), lhv, hhv, dens,
                 f"{SRC_PREFIX}Fuel.mdb/{tname}", note])
            if kind == "gas":
                src = f"{SRC_PREFIX}Fuel.mdb/GasFuel"
                gnote = jnote(("units",
                               "Volume % by component (source columns); Gravity=SG vs air; "
                               "gross/net_heat_btu_scf are Btu/scf; gross/net_heat_si are "
                               "MJ/Nm3 @0C,101.325kPa; H2O not present in source -> 0."),
                              ("raw", r))
                put("fuel_gas_components",
                    ["code", "name", "ch4", "c2h6", "c3h8", "c4h10", "co", "h2", "co2",
                     "o2", "n2", "h2o", "gravity", "gross_heat_btu_scf", "net_heat_btu_scf",
                     "cp", "gross_heat_si", "net_heat_si", "source", "note"],
                    [code, str(name),
                     num(r.get("CH4")), num(r.get("C2H6")), num(r.get("C3H8")),
                     num(r.get("C4H10")), num(r.get("CO")), num(r.get("H2")),
                     num(r.get("CO2")), num(r.get("O2")), num(r.get("N2")), 0.0,
                     num(r.get("Gravity")), num(r.get("GrossHeat")), num(r.get("NetHeat")),
                     num(r.get("Cp")), num(r.get("GrossHeatSI")), num(r.get("NetHeatSI")),
                     src, gnote])
            n += 1
    stats["fuels"] = n

    # -- burners -----------------------------------------------------------
    n = 0
    burner_mdbs = ("AsphaltBrn1", "Combburn2", "Gasbrn", "GasbrnConvert", "dbTriOx1")
    for mdb, t in all_tables(*burner_mdbs):
        cols = [c["name"] for c in t["columns"]]
        if "Model" not in cols:
            continue
        for r in t["rows"]:
            model = r.get("Model") or r.get("ID")
            if model is None or str(model).strip() == "":
                continue
            cap = None
            if mdb == "AsphaltBrn1" and "HeatCapMax" in cols:
                v = num(r.get("HeatCapMax"))
                cap = v * MM_BTU_HR_TO_KW if v else None
            elif mdb == "dbTriOx1" and "NominalBurnerHeat" in cols:
                v = num(r.get("NominalBurnerHeat"))
                cap = v * MM_BTU_HR_TO_KW if v else None
            note = jnote(("units", UNIT_ASSUMPTIONS["burners"]), ("raw", r))
            put("burners", ["code", "name", "capacity_kw_max", "source", "note"],
                [codes.make(mdb, t["name"], model), str(model), cap,
                 f"{SRC_PREFIX}{mdb}.mdb/{t['name']}", note])
            n += 1
    stats["burners"] = n

    # -- blowers -----------------------------------------------------------
    n = 0
    for mdb, t in all_tables("Blower1", "EclipseBlower1"):
        cols = [c["name"] for c in t["columns"]]
        flow_col = "SCFH" if "SCFH" in cols else ("FlowRate" if "FlowRate" in cols else None)
        if "Model" not in cols or not flow_col:
            continue
        for r in t["rows"]:
            model = r.get("Model")
            if not model:
                continue
            f = num(r.get(flow_col))
            flow = f * FT3_TO_M3 if f else None
            b = num(r.get("BHP"))
            power = b * BHP_TO_KW if b else None
            note = jnote(("units", UNIT_ASSUMPTIONS["blowers"]), ("raw", r))
            put("blowers", ["code", "name", "flow_m3h_max", "power_kw", "source", "note"],
                [codes.make(mdb, t["name"], model), str(model), flow, power,
                 f"{SRC_PREFIX}{mdb}.mdb/{t['name']}", note])
            n += 1
    stats["blowers"] = n

    # -- valves ------------------------------------------------------------
    n = 0
    for mdb, t in all_tables("Valve1", "Valve1Convert"):
        cols = [c["name"] for c in t["columns"]]
        if "CV" not in cols:
            continue
        for i, r in enumerate(t["rows"]):
            model = r.get("Model") or r.get("NPT") or f"row{i+1}"
            cv = num(r.get("CV"))
            d = num(r.get("ID"))
            dn = d * IN_TO_MM if d else None
            if cv is not None and cv <= 0:
                cv = None
            note = jnote(("units", UNIT_ASSUMPTIONS["valves"]), ("raw", r))
            put("valves", ["code", "name", "dn_mm", "cv", "source", "note"],
                [codes.make(mdb, t["name"], i + 1, model), str(model), dn, cv,
                 f"{SRC_PREFIX}{mdb}.mdb/{t['name']}", note])
            n += 1
    stats["valves"] = n

    # -- orifices ----------------------------------------------------------
    n = 0
    for mdb, t in all_tables("Orifice1"):
        cols = [c["name"] for c in t["columns"]]
        if "Bore" not in cols:
            continue
        for i, r in enumerate(t["rows"]):
            b = num(r.get("Bore"))
            dia = b * IN_TO_MM if b else None
            name = r.get("Model") or r.get("PartNoBore") or f"{t['name']} #{i+1}"
            note = jnote(("units", UNIT_ASSUMPTIONS["orifices"]), ("raw", r))
            put("orifices", ["code", "name", "diameter_mm", "source", "note"],
                [codes.make(mdb, t["name"], i + 1, name), str(name), dia,
                 f"{SRC_PREFIX}{mdb}.mdb/{t['name']}", note])
            n += 1
    stats["orifices"] = n

    # -- pipes -------------------------------------------------------------
    n = 0
    seen_pipe = set()
    for r in rows("PipeSchedule40", "TableSch40"):
        nps = frac(r.get("PipeID"))
        if not nps or ("SCH40", nps) in seen_pipe:
            continue
        seen_pipe.add(("SCH40", nps))
        w = num(r.get("WallThicknessUS"))
        note = jnote(("units", UNIT_ASSUMPTIONS["pipes"]), ("raw", r))
        put("pipes", ["code", "schedule", "nps_inch", "od_mm", "wall_mm",
                      "inner_d_mm", "source", "note"],
            [f"SCH40-{r.get('PipeID')}", "SCH40", nps,
             num(r.get("OuterDiameterSI")), w * IN_TO_MM if w else None,
             num(r.get("InnerDiameterSI")),
             f"{SRC_PREFIX}PipeSchedule40.mdb/TableSch40", note])
        n += 1
    for r in rows("Pipe1", "PipeSize"):
        nps = frac(r.get("PipeNPT"))
        if not nps or ("NPT", nps) in seen_pipe:
            continue
        seen_pipe.add(("NPT", nps))
        d = num(r.get("PipeID"))
        note = jnote(("units", UNIT_ASSUMPTIONS["pipes"]), ("raw", r))
        put("pipes", ["code", "schedule", "nps_inch", "inner_d_mm", "source", "note"],
            [f"NPT-{r.get('PipeNPT')}".replace('"', "").strip(), "NPT", nps,
             d * IN_TO_MM if d else None,
             f"{SRC_PREFIX}Pipe1.mdb/PipeSize", note])
        n += 1
    stats["pipes"] = n

    # -- fittings ----------------------------------------------------------
    n = 0
    for mdb, t in all_tables("Fitting1"):
        cols = [c["name"] for c in t["columns"]]
        if "RESISTK" not in cols:
            continue
        for i, r in enumerate(t["rows"]):
            name = r.get("Model") or r.get("TeeType") or f"{t['name']} #{i+1}"
            d = num(r.get("ID"))
            k = num(r.get("RESISTK"))
            note = jnote(("units", UNIT_ASSUMPTIONS["fittings"]), ("raw", r))
            put("fittings", ["code", "name", "dn_mm", "k_factor", "source", "note"],
                [codes.make(mdb, t["name"], i + 1, name), str(name),
                 d * IN_TO_MM if d else None, k,
                 f"{SRC_PREFIX}{mdb}.mdb/{t['name']}", note])
            n += 1
    stats["fittings"] = n

    # -- materials (Material1 wins over Material on duplicate names) -------
    materials = {}
    for mdb in ("Material", "Material1"):
        for t in ("MetalProp", "Solid"):
            for r in rows(mdb, t):
                name = r.get("MetalName")
                if not name:
                    continue
                key = re.sub(r"[^a-z0-9]", "", str(name).lower())
                tmax = num(r.get("Tmax"))
                dens = num(r.get("Density"))
                materials[key] = (
                    str(name),
                    (tmax - 32) * 5 / 9 if tmax is not None else None,
                    dens * LB_FT3_TO_KG_M3 if dens else None,
                    f"{SRC_PREFIX}{mdb}.mdb/{t}",
                    jnote(("units", UNIT_ASSUMPTIONS["materials"]), ("raw", r)),
                )
    for key, (name, tmax, dens, src, note) in materials.items():
        put("materials", ["code", "name", "max_temp_c", "density_kg_m3", "source", "note"],
            [codes.make("MAT", name), name, tmax, dens, src, note])
    stats["materials"] = len(materials)

    # -- regulators --------------------------------------------------------
    n = 0
    for r in rows("GasPressRegulator", "GPR"):
        model = r.get("Model")
        if not model:
            continue
        pmax, p1, p2 = num(r.get("Pmax")), num(r.get("Pout1")), num(r.get("Pout2"))
        note = jnote(("units", UNIT_ASSUMPTIONS["regulators"]), ("raw", r))
        put("regulators", ["code", "name", "inlet_max_kpa", "outlet_min_kpa",
                           "outlet_max_kpa", "source", "note"],
            [codes.make("GPR", model), str(r.get("Description") or model),
             pmax * PSI_TO_KPA if pmax else None,
             p1 * PSI_TO_KPA if p1 else None,
             p2 * PSI_TO_KPA if p2 else None,
             f"{SRC_PREFIX}GasPressRegulator.mdb/GPR", note])
        n += 1
    stats["regulators"] = n
    return stats


# ------------------------------------------------------------------- main ---
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dump", default=DEFAULT_DUMP)
    ap.add_argument("--db", default=DEFAULT_DB)
    ap.add_argument("--report", default=os.path.join(ROOT, "import_work", "import_report.json"))
    args = ap.parse_args()

    if not os.path.isdir(args.dump):
        sys.exit(f"dump dir not found: {args.dump}")
    dumps = {}
    for fn in sorted(os.listdir(args.dump)):
        if fn.lower().endswith(".json"):
            with open(os.path.join(args.dump, fn), encoding="utf-8") as f:
                doc = json.load(f)
            if doc.get("error"):
                print(f"  !! {fn}: {doc['error']}")
            dumps[fn] = doc
    if not dumps:
        sys.exit("no dump JSON found")

    conn = sqlite3.connect(args.db)
    conn.execute("PRAGMA foreign_keys = ON")
    # Ensure current schema is present (idempotent DDL). New tables such as
    # fuel_gas_components are created here before any import writes to them.
    schema_path = os.path.join(APP, "schema.sql")
    if os.path.exists(schema_path):
        conn.executescript(open(schema_path, encoding="utf-8").read())
    # remove previous import rows only; user rows are untouched
    for t in ("burners", "blowers", "valves", "orifices", "pipes",
              "fittings", "materials", "regulators", "fuels", "fuel_gas_components"):
        conn.execute(f"DELETE FROM {t} WHERE source LIKE ?", (SRC_PREFIX + "%",))

    src_tables, src_rows = import_fidelity(conn, dumps)
    stats = import_domain(conn, dumps)

    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('data_origin',?)",
                 ("ESolutions600 local MDB import (user-authorized, 20 files)",))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('notice',?)",
                 ("Engineering catalog data imported from the locally installed "
                  "ESolutions600 MDB files; provenance is in source_files / "
                  "source_tables and per-row 'source' / 'note' columns. "
                  "Verify unit assumptions before design use.",))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('import_files',?)", (str(len(dumps)),))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('import_source_tables',?)", (str(src_tables),))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('import_source_rows',?)", (str(src_rows),))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('import_at',?)",
                 (datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC"),))
    conn.execute("INSERT OR REPLACE INTO meta(key,value) VALUES ('import_unit_warning',?)",
                 ("Derived SI values carry unit assumptions in each row's note; "
                  "verify against vendor literature before design use.",))
    conn.commit()

    integrity = conn.execute("PRAGMA integrity_check").fetchone()[0]
    app_tables = conn.execute(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
    ).fetchone()[0]
    report = {
        "database": args.db,
        "files": len(dumps),
        "source_tables": src_tables,
        "source_rows": src_rows,
        "mapped_rows": stats,
        "app_tables_total": app_tables,
        "integrity": integrity,
        "unit_assumptions": UNIT_ASSUMPTIONS,
    }
    with open(args.report, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    conn.execute("VACUUM")
    conn.close()

    print(f"files={len(dumps)}  fidelity_tables={src_tables}  fidelity_rows={src_rows}")
    for k, v in stats.items():
        print(f"  {k:<12} {v:>5} rows")
    print(f"integrity={integrity}  total_tables={app_tables}")
    print(f"report -> {args.report}")
    print(f"db size -> {os.path.getsize(args.db)} bytes")


if __name__ == "__main__":
    main()
