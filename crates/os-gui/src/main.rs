//! Open Solutions desktop GUI backend.
//!
//! A zero-GUI-dependency design: this binary serves the single-file HTML UI on
//! a loopback-only HTTP port and opens it in an application-mode browser window
//! (no address bar/tabs), so the result looks and behaves like a desktop app.
//! All engineering math and unit conversion stay in Rust (`os-core`,
//! `os-modules`); the browser is only a renderer.

use os_core::model::Fuel;
use os_core::storage::{Database, BUSINESS_TABLES};
use os_core::units::UnitSystem;
use os_modules::quantity::Quantity;
use os_modules::{catalog, compute};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};
use tiny_http::{Header, Method, Response, Server};

const UI_HTML: &str = include_str!("ui.html");

fn json_response(body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap(),
    )
}

fn html_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(
        Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
    )
}

fn db_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let dir = exe.parent().unwrap_or(&exe).join("data");
    let _ = fs::create_dir_all(&dir);
    dir.join("open_solutions.db")
}

fn system_of(s: &str) -> UnitSystem {
    if s.eq_ignore_ascii_case("us") || s.eq_ignore_ascii_case("imperial") {
        UnitSystem::Imperial
    } else {
        UnitSystem::Si
    }
}

fn parse_qty(name: &str) -> Result<Quantity, String> {
    serde_json::from_value::<Quantity>(Value::String(name.to_string()))
        .map_err(|e| format!("unknown quantity {name}: {e}"))
}

// ---------------------------------------------------------------- API body --

#[derive(Deserialize)]
struct ComputeReq {
    id: String,
    inputs: serde_json::Map<String, Value>,
}

#[derive(Deserialize)]
struct ConvertField {
    qty: String,
    value: f64,
    unit: String,
}

#[derive(Deserialize)]
struct ConvertReq {
    fields: Vec<ConvertField>,
    to_system: String,
}

#[derive(Deserialize)]
struct FuelReq {
    code: String,
    name: String,
    lhv: Option<f64>,
    hhv: Option<f64>,
    density: Option<f64>,
}

fn db_state(db: &Database) -> Value {
    // The application catalog is 14 business tables (+ `meta`). Imported MDB
    // tables (`src_*`, ~500 of them) live in the same file but are reported
    // through `source` instead of flooding this list.
    let tables: Vec<Value> = db
        .table_names()
        .unwrap_or_default()
        .iter()
        .filter(|t| BUSINESS_TABLES.contains(&t.as_str()) || t.as_str() == "meta")
        .map(|t| {
            json!({
                "name": t,
                "rows": db.count_rows(t).unwrap_or(Some(0)).unwrap_or(0),
                "business": BUSINESS_TABLES.contains(&t.as_str()),
            })
        })
        .collect();
    let source = db
        .connection()
        .query_row(
            "SELECT (SELECT COUNT(*) FROM source_files), \
                    (SELECT COUNT(*) FROM source_tables), \
                    (SELECT COALESCE(SUM(row_count),0) FROM source_tables)",
            [],
            |r| {
                Ok(json!({
                    "files": r.get::<_, i64>(0)?,
                    "tables": r.get::<_, i64>(1)?,
                    "rows": r.get::<_, i64>(2)?,
                }))
            },
        )
        .unwrap_or_else(|_| json!({"files": 0, "tables": 0, "rows": 0}));
    let conventions: Vec<Value> = db
        .unit_conventions()
        .unwrap_or_default()
        .iter()
        .map(|(t, c, u, n)| json!({"table": t, "column": c, "unit": u, "note": n}))
        .collect();
    let fuels: Vec<Value> = db
        .list_fuels()
        .unwrap_or_default()
        .iter()
        .map(|f| json!({"code": f.code, "name": f.name, "lhv": f.lhv_mj_kg, "hhv": f.hhv_mj_kg, "density": f.density_kg_nm3, "source": f.source}))
        .collect();
    let gas_fuels: Vec<Value> = db
        .list_gas_fuels()
        .unwrap_or_default()
        .iter()
        .map(|g| json!({
            "code": g.code,
            "name": g.name,
            "ch4": g.ch4, "c2h6": g.c2h6, "c3h8": g.c3h8, "c4h10": g.c4h10,
            "co": g.co, "h2": g.h2, "co2": g.co2, "o2": g.o2, "n2": g.n2, "h2o": g.h2o,
            "gravity": g.gravity,
            "gross_heat_btu_scf": g.gross_heat_btu_scf,
            "net_heat_btu_scf": g.net_heat_btu_scf,
            "cp": g.cp,
            "gross_heat_si": g.gross_heat_si,
            "net_heat_si": g.net_heat_si,
            "source": g.source,
        }))
        .collect();

    let history: Vec<Value> = db
        .connection()
        .prepare("SELECT module, created_at, substr(result_json, 1, 90) FROM calc_runs ORDER BY id DESC LIMIT 12")
        .and_then(|mut stmt| {
            let rows = stmt.query_map([], |r| {
                Ok(json!({"module": r.get::<_, String>(0)?, "at": r.get::<_, String>(1)?, "preview": r.get::<_, String>(2)?}))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();
    json!({
        "schema_version": db.schema_version().unwrap_or_else(|_| "?".into()),
        "tables": tables,
        "conventions": conventions,
        "fuels": fuels,
        "gas_fuels": gas_fuels,
        "history": history,
        "source": source,
    })
}

fn handle_api(db: &Database, method: &Method, path: &str, body: &str) -> Result<(String, bool), String> {
    let is = |m: &Method, p: &str| *m == Method::Get && path.starts_with(p);
    if is(method, "/api/modules") { return Ok((serde_json::to_string(&catalog()).unwrap(), false)); }
    if is(method, "/api/audit") {
        let rows: Vec<Value> = os_core::units::unit_audit()
            .iter()
            .map(|(k, v)| json!({"item": k, "value": v}))
            .collect();
        return Ok((serde_json::to_string(&rows).unwrap(), false));
    }
    if is(method, "/api/db") { return Ok((db_state(db).to_string(), false)); }
    let post = |p: &str| *method == Method::Post && path.starts_with(p);
    if post("/api/compute") {
        let req: ComputeReq = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let result = compute(&req.id, &req.inputs)?;
        let out = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        let _ = db.connection().execute(
            "INSERT INTO calc_runs (project_id, module, input_json, result_json) VALUES (1, ?1, ?2, ?3)",
            rusqlite::params![&req.id, body, out],
        );
        return Ok((out, false));
    }
    if post("/api/convert-fields") {
        let req: ConvertReq = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let sys = system_of(&req.to_system);
        let mut values = Vec::new();
        let mut units = Vec::new();
        for f in &req.fields {
            let q = parse_qty(&f.qty)?;
            let si = q.to_si(f.value, &f.unit).unwrap_or(f.value);
            let to = q.display_unit(sys);
            values.push(q.from_si(si, to).unwrap_or(si));
            units.push(to);
        }
        return Ok((json!({"values": values, "units": units}).to_string(), false));
    }
    if post("/api/fuel") {
        let req: FuelReq = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let mut fuel = Fuel::user_supplied(req.code.trim(), req.name.trim());
        fuel.lhv_mj_kg = req.lhv;
        fuel.hhv_mj_kg = req.hhv;
        fuel.density_kg_nm3 = req.density;
        db.insert_fuel(&fuel).map_err(|e| format!("写入失败: {e}"))?;
        return Ok((db_state(db).to_string(), false));
    }
    if post("/api/shutdown") {
        return Ok((json!({"ok": true}).to_string(), true));
    }
    Err(format!("unknown endpoint: {method} {path}"))
}

// ------------------------------------------------------------- app window --

fn open_app_window(url: &str, profile: &PathBuf) -> Option<Child> {
    // Pre-seed a minimal Chromium profile so Edge never shows its
    // first-run / implicit sign-in / sync-promotion dialogs over the app.
    let default_dir = profile.join("Default");
    let _ = fs::create_dir_all(&default_dir);
    let prefs = default_dir.join("Preferences");
    let _ = fs::write(
            &prefs,
            r#"{"browser":{"has_seen_welcome_page":true,"check_default_browser":false},"signin":{"allowed":false},"sync":{"requested":false,"keep_everything_synced":false},"fre":{"has_seen_fre":true},"profile":{"name":"Open Solutions","exit_type":"Normal","exited_cleanly":true},"edge":{"services":{"implicit_signin_enabled":false}}}"#,
        );
    let local_state = profile.join("Local State");
    let _ = fs::write(
            &local_state,
            r#"{"browser":{"enabled_labs_experiments":[]},"profile":{"last_used":"Default","info_cache":{"Default":{"name":"Open Solutions"}}},"user_experience_metrics":{"reporting_enabled":false}}"#,
        );
    let edge = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ]
    .iter()
    .find(|p| PathBuf::from(p).exists())
    .copied();
    if let Some(exe) = edge {
        let mut launch: Vec<String> = vec![
            format!("--app={url}"),
            "--window-size=1500,980".into(),
            "--window-position=100,50".into(),
            format!("--user-data-dir={}", profile.display()),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
            "--disable-sync".into(),
            "--disable-default-apps".into(),
            "--disable-background-networking".into(),
            "--disable-component-update".into(),
            "--disable-features=msEdgeSidebarV2,msUndersideButton,msImplicitSignin,msEdgeIdentityFRE,msEdgeWelcomePage,msEdgeSyncPromotion".into(),
        ];
        // Opt-in only: a fixed debugging port would collide with other Chromium
        // instances and exposes a local control channel. Set OS_CDP_PORT=9222
        // when browser-level verification is actually needed.
        if let Ok(p) = std::env::var("OS_CDP_PORT") {
            if !p.is_empty() {
                launch.push(format!("--remote-debugging-port={p}"));
            }
        }
        Command::new(exe).args(&launch).spawn().ok()
    } else {
        let _ = Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let no_browser = args.iter().any(|a| a == "--no-browser");

    let path = db_path();
    let db = Database::open(&path)?;
    db.migrate()?;
    let _ = db.connection().execute(
        "INSERT OR IGNORE INTO projects (id, code, name, description) \
         VALUES (1, 'DEFAULT', '默认项目', 'GUI 计算历史自动归档')",
        [],
    );

    // Reserve a free loopback port, then let tiny_http bind it.
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let url = format!("http://127.0.0.1:{port}/");
    let server = Server::http(format!("127.0.0.1:{port}"))?;

    println!("Open Solutions GUI");
    println!("  database : {}", path.display());
    println!("  url      : {url}");
    println!("  schema   : v{}", db.schema_version()?);
    println!("  modules  : {}", catalog().len());
    println!("  关闭窗口或点击界面右上角「退出」即可结束程序。\n");

    let child = if !no_browser {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let profile = exe_dir.join("edge-profile");
        let _ = fs::create_dir_all(&profile);
        open_app_window(&url, &profile)
    } else {
        None
    };
    // A short grace period: Chromium may fork and return immediately.
    if let Some(c) = child {
        std::thread::spawn(move || {
            let start = Instant::now();
            let mut c = c;
            loop {
                std::thread::sleep(Duration::from_millis(700));
                match c.try_wait() {
                    Ok(Some(_)) if start.elapsed() > Duration::from_secs(6) => {
                        std::process::exit(0);
                    }
                    Ok(Some(_)) => break,
                    _ => {}
                }
            }
        });
    }

    for mut request in server.incoming_requests() {
        let method = request.method().clone();
        let path = request.url().to_string();
        let mut body = String::new();
        let _ = request.as_reader().read_to_string(&mut body);

        let response = if method == Method::Get && (path == "/" || path.starts_with("/?")) {
            html_response(UI_HTML)
        } else {
            match handle_api(&db, &method, &path, &body) {
                Ok((text, shutdown)) => {
                    let r = json_response(text);
                    if shutdown {
                        let _ = request.respond(r);
                        std::thread::sleep(Duration::from_millis(150));
                        std::process::exit(0);
                    }
                    r
                }
                Err(e) => json_response(json!({"error": e}).to_string()),
            }
        };
        let _ = request.respond(response);
    }
    Ok(())
}
