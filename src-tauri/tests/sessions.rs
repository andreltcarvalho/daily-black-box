use caixa_preta::{sessions::*, store::*};
use rusqlite::Connection;

fn stamp(seconds: u64) -> Stamp {
    Stamp {
        utc: 1_788_956_400_000 + seconds as i64 * 1000,
        mono: seconds * 1000,
        offset: -10_800,
    }
}
fn recorder() -> Recorder {
    Recorder::new(
        Store::initialize(Connection::open_in_memory().unwrap()).unwrap(),
        &stamp(0),
    )
    .unwrap()
}
fn observe(rec: &mut Recorder, start: u64, end: u64, act: Activity, tab: Option<&str>) {
    for t in start..=end {
        rec.observe(stamp(t), act.clone(), tab.map(str::to_string))
            .unwrap();
    }
}

#[test]
fn known_day_and_reclassification_share_the_same_totals() {
    let mut rec = recorder();
    observe(&mut rec, 0, 1800, Activity::new("vdi", ""), None);
    observe(
        &mut rec,
        1800,
        2400,
        Activity::browser("youtube.com".into()),
        Some("1:2"),
    );
    observe(
        &mut rec,
        2400,
        3000,
        Activity::new("idle", "no_input"),
        None,
    );
    observe(&mut rec, 3000, 3600, Activity::new("vdi", ""), None);
    rec.close().unwrap();
    let day = stamp(0).date();
    let report = rec.store.report(&day).unwrap();
    assert_eq!(
        (report.vdi_ms, report.idle_ms, report.unknown_ms),
        (2_400_000, 600_000, 600_000)
    );
    assert_eq!(report.coverage_ms, 3_600_000);
    assert_eq!(report.longest_focus_ms, 1_800_000);
    assert_eq!(report.context_switches, 1);
    assert_eq!(report.domains[0].accesses, 1);
    assert_eq!(report.distraction_ms, None);
    rec.store.classify("youtube.com", "video").unwrap();
    rec.store.set_distraction("video", true).unwrap();
    let report = rec.store.report(&day).unwrap();
    assert_eq!(report.distraction_ms, Some(600_000));
    assert_eq!(report.unknown_ms, 0);
    assert_eq!(report.largest_distraction.as_deref(), Some("youtube.com"));
}
#[test]
fn repeated_snapshots_do_not_create_accesses_but_switching_tabs_does() {
    let mut rec = recorder();
    let site = Activity::browser("example.org".into());
    observe(&mut rec, 0, 10, site.clone(), Some("1:1"));
    observe(&mut rec, 10, 20, site, Some("1:2"));
    rec.close().unwrap();
    let r = rec.store.report(&stamp(0).date()).unwrap();
    assert_eq!(r.domains[0].accesses, 2);
    assert_eq!(r.domains[0].duration_ms, 20_000);
    assert_eq!(r.context_switches, 0);
}
#[test]
fn crash_recovery_never_fills_the_offline_period_with_focus() {
    let mut rec = recorder();
    observe(&mut rec, 0, 7, Activity::new("vdi", ""), None);
    let mut recovered = Recorder::new(rec.store, &stamp(100)).unwrap();
    observe(&mut recovered, 100, 110, Activity::new("vdi", ""), None);
    recovered.close().unwrap();
    let r = recovered.store.report(&stamp(0).date()).unwrap();
    assert_eq!(r.vdi_ms, 15_000);
    assert_eq!(
        r.sessions
            .iter()
            .filter(|s| s.activity.source == "unobserved")
            .map(|s| s.duration_ms)
            .sum::<u64>(),
        95_000
    );
}
#[test]
fn observation_gap_is_not_focus() {
    let mut rec = recorder();
    observe(&mut rec, 0, 5, Activity::new("vdi", ""), None);
    observe(&mut rec, 100, 105, Activity::new("vdi", ""), None);
    rec.close().unwrap();
    assert_eq!(rec.store.report(&stamp(0).date()).unwrap().vdi_ms, 10_000);
}
#[test]
fn midnight_splits_sessions_without_counting_a_second_access() {
    let mut rec = recorder();
    let mut at = stamp(0);
    at.utc = chrono::DateTime::parse_from_rfc3339("2026-09-10T23:59:55-03:00")
        .unwrap()
        .timestamp_millis();
    for i in 0..=10 {
        rec.observe(
            at.advance(i * 1000),
            Activity::browser("example.com".into()),
            Some("1:1".into()),
        )
        .unwrap();
    }
    rec.close().unwrap();
    let a = rec.store.report("2026-09-10").unwrap();
    let b = rec.store.report("2026-09-11").unwrap();
    assert_eq!((a.coverage_ms, b.coverage_ms), (5_000, 5_000));
    assert_eq!(a.domains[0].accesses, 1);
    assert_eq!(b.domains[0].accesses, 0);
}
#[test]
fn clock_backwards_does_not_create_negative_or_duplicated_duration() {
    let mut rec = recorder();
    observe(&mut rec, 0, 5, Activity::new("vdi", ""), None);
    let mut adjusted = stamp(6);
    adjusted.utc -= 3_600_000;
    rec.observe(adjusted.clone(), Activity::new("vdi", ""), None)
        .unwrap();
    rec.observe(adjusted.advance(1000), Activity::new("vdi", ""), None)
        .unwrap();
    rec.close().unwrap();
    let report = rec.store.report(&stamp(0).date()).unwrap();
    assert_eq!(report.vdi_ms, 7_000);
    assert!(report
        .sessions
        .iter()
        .any(|s| s.activity.reason == "clock_adjusted"));
}
#[test]
fn deleting_one_day_preserves_another_and_classification() {
    let mut rec = recorder();
    observe(
        &mut rec,
        0,
        5,
        Activity::browser("example.com".into()),
        Some("x"),
    );
    rec.close().unwrap();
    rec.store.classify("example.com", "research").unwrap();
    observe(
        &mut rec,
        86_400,
        86_405,
        Activity::browser("example.com".into()),
        Some("y"),
    );
    rec.close().unwrap();
    rec.store.delete(Some(&stamp(0).date())).unwrap();
    assert_eq!(rec.store.report(&stamp(0).date()).unwrap().coverage_ms, 0);
    let next = rec.store.report(&stamp(86_400).date()).unwrap();
    assert_eq!(next.coverage_ms, 5000);
    assert_eq!(next.domains[0].category, "research");
}
#[test]
fn rejected_input_and_transaction_failure_leave_database_intact() {
    let mut rec = recorder();
    assert!(rec
        .store
        .classify("https://example.org/private?q=secret", "video")
        .is_err());
    assert!(rec.store.classify("example.org", "invented").is_err());
    assert!(rec.store.report("2026-99-99").is_err());
    rec.store.conn.execute_batch("CREATE TRIGGER fail_write BEFORE INSERT ON sessions BEGIN SELECT RAISE(ABORT,'disk failure'); END;").unwrap();
    assert!(rec
        .observe(
            stamp(0),
            Activity::browser("example.org".into()),
            Some("1".into())
        )
        .is_err());
    let count: i64 = rec
        .store
        .conn
        .query_row("SELECT COUNT(*) FROM accesses", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn local_apps_are_named_grouped_and_counted_as_contexts() {
    let mut rec = recorder();
    observe(
        &mut rec,
        0,
        60,
        Activity::app("League of Legends".into(), "foreground_app"),
        None,
    );
    observe(
        &mut rec,
        60,
        120,
        Activity::app("ChatGPT".into(), "foreground_app"),
        None,
    );
    rec.close().unwrap();
    let report = rec.store.report(&stamp(0).date()).unwrap();
    assert_eq!(report.app_totals.len(), 2);
    assert_eq!(report.app_totals[0].label, "ChatGPT");
    assert_eq!(report.app_totals[0].duration_ms, 60_000);
    assert_eq!(report.app_totals[0].category, "local");
    assert_eq!(report.context_switches, 1);
    assert_eq!(report.unidentified_ms, 0);
    rec.store
        .classify_app("League of Legends", "entertainment")
        .unwrap();
    rec.store.set_distraction("entertainment", true).unwrap();
    let report = rec.store.report(&stamp(0).date()).unwrap();
    assert_eq!(report.app_totals[1].category, "entertainment");
    assert_eq!(report.distraction_ms, Some(60_000));
    assert_eq!(
        report.largest_distraction.as_deref(),
        Some("League of Legends")
    );
}

#[test]
fn version_one_database_keeps_sessions_when_app_names_are_added() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO sessions(run_id,start_utc,end_utc,duration_ms,local_date,offset_seconds,source,hostname,reason,is_open) VALUES('old',0,1000,1000,'2026-09-10',0,'vdi',NULL,'',0)",
        [],
    )
    .unwrap();
    let store = Store::initialize(conn).unwrap();
    let version: u32 = store
        .conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    let report = store.report("2026-09-10").unwrap();
    assert_eq!(version, 3);
    assert_eq!(report.sessions.len(), 1);
    assert_eq!(report.sessions[0].activity.app_name, None);
}
