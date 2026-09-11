use caixa_preta::{commands::write_export, sessions::*, store::Store};
use rusqlite::Connection;
#[test]
fn export_round_trip_and_existing_temp_file_are_safe() {
    let root = std::env::temp_dir().join(format!("caixa-preta-export-test-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("day.json");
    let store = Store::initialize(Connection::open_in_memory().unwrap()).unwrap();
    let report = store.report("2026-09-10").unwrap();
    write_export(&report, &path).unwrap();
    let exported: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(exported["schema_version"], 3);
    assert_eq!(exported["date"], "2026-09-10");
    assert!(exported["sessions"].as_array().unwrap().is_empty());
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temp, b"existing data").unwrap();
    assert!(write_export(&report, &path).is_err());
    assert_eq!(std::fs::read(&temp).unwrap(), b"existing data");
    std::fs::remove_file(&temp).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&root).unwrap();
}
#[test]
fn invalid_export_extension_does_not_touch_destination() {
    let report = report(
        "2026-09-10",
        vec![],
        vec![],
        vec![],
        Default::default(),
        Default::default(),
    );
    assert!(write_export(&report, std::path::Path::new("protected.sqlite3")).is_err());
}
