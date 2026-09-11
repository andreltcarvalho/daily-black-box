use crate::sessions::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VdiIdentity {
    pub executable: String,
    pub window_class: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub paused: bool,
    pub configured: bool,
    pub idle_minutes: u32,
    pub vdi: Option<VdiIdentity>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            paused: true,
            configured: false,
            idle_minutes: 5,
            vdi: None,
        }
    }
}

pub struct Store {
    pub conn: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Self::initialize(conn)
    }
    pub fn initialize(conn: Connection) -> Result<Self> {
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(|e| e.to_string())?;
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version > 3 {
            return Err("Banco de uma versão mais recente. Nenhum dado foi alterado.".into());
        }
        if version == 0 {
            conn.execute_batch("BEGIN IMMEDIATE;")
                .map_err(|e| e.to_string())?;
            if let Err(error) = conn.execute_batch(include_str!("../migrations/001_initial.sql")) {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(error.to_string());
            }
            conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;
        }
        if version <= 1 {
            conn.execute_batch("BEGIN IMMEDIATE;")
                .map_err(|e| e.to_string())?;
            if let Err(error) = conn.execute_batch(include_str!("../migrations/002_app_names.sql"))
            {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(error.to_string());
            }
            conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;
        }
        if version <= 2 {
            conn.execute_batch("BEGIN IMMEDIATE;")
                .map_err(|e| e.to_string())?;
            if let Err(error) =
                conn.execute_batch(include_str!("../migrations/003_app_categories.sql"))
            {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(error.to_string());
            }
            conn.execute_batch("COMMIT;").map_err(|e| e.to_string())?;
        }
        Ok(Self { conn })
    }
    pub fn settings(&self) -> Result<Settings> {
        let value: Option<String> = self
            .conn
            .query_row("SELECT value FROM settings WHERE key='app'", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        value
            .map(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
            .unwrap_or_else(|| Ok(Settings::default()))
    }
    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let data = serde_json::to_string(settings).map_err(|e| e.to_string())?;
        self.conn.execute("INSERT INTO settings VALUES('app',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[data]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn categories(&self) -> Result<Vec<Category>> {
        let mut q = self
            .conn
            .prepare("SELECT id,name,distraction FROM categories ORDER BY position")
            .map_err(|e| e.to_string())?;
        let rows = q
            .query_map([], |r| {
                Ok(Category {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    distraction: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
    pub fn classify(&self, hostname: &str, category: &str) -> Result<()> {
        if !valid_hostname(hostname) {
            return Err("Domínio inválido.".into());
        }
        if category == "vdi" {
            return Err("A categoria VDI é reservada à janela remota.".into());
        }
        self.conn.execute("INSERT INTO domain_categories VALUES(?1,?2,?3) ON CONFLICT(hostname) DO UPDATE SET category=excluded.category,updated_at=excluded.updated_at",params![hostname,category,chrono::Utc::now().timestamp_millis()]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn classify_app(&self, app_name: &str, category: &str) -> Result<()> {
        if app_name.is_empty() || app_name.len() > 255 || app_name.chars().any(char::is_control) {
            return Err("Aplicativo inválido.".into());
        }
        if category == "vdi" {
            return Err("A categoria VDI é reservada à janela remota.".into());
        }
        self.conn.execute("INSERT INTO app_categories VALUES(?1,?2,?3) ON CONFLICT(app_name) DO UPDATE SET category=excluded.category,updated_at=excluded.updated_at",params![app_name,category,chrono::Utc::now().timestamp_millis()]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn set_distraction(&self, id: &str, enabled: bool) -> Result<()> {
        if id == "vdi" {
            return Err("VDI não é uma categoria de distração.".into());
        }
        let count = self
            .conn
            .execute(
                "UPDATE categories SET distraction=?2 WHERE id=?1",
                params![id, enabled],
            )
            .map_err(|e| e.to_string())?;
        if count != 1 {
            return Err("Categoria não encontrada.".into());
        }
        Ok(())
    }
    pub fn recover(&self) -> Result<Option<i64>> {
        self.conn
            .execute("UPDATE sessions SET is_open=0 WHERE is_open=1", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .query_row(
                "SELECT confirmed_utc FROM tracker_checkpoint WHERE id=1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }
    pub fn persist(&mut self, segment: &mut Segment, access: Option<(&Stamp, &str)>) -> Result<()> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        if segment.id == 0 {
            tx.execute("INSERT INTO sessions(run_id,start_utc,end_utc,duration_ms,local_date,offset_seconds,source,hostname,app_name,reason,is_open) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![segment.run_id,segment.start_utc,segment.end_utc,segment.duration_ms,segment.local_date,segment.offset_seconds,segment.activity.source,segment.activity.hostname,segment.activity.app_name,segment.activity.reason,segment.is_open]).map_err(|e|e.to_string())?;
        } else {
            tx.execute(
                "UPDATE sessions SET end_utc=?2,duration_ms=?3,is_open=?4 WHERE id=?1",
                params![
                    segment.id,
                    segment.end_utc,
                    segment.duration_ms,
                    segment.is_open
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        let id = if segment.id == 0 {
            tx.last_insert_rowid()
        } else {
            segment.id
        };
        if let Some((at, tab_key)) = access {
            if let Some(host) = &segment.activity.hostname {
                tx.execute("INSERT INTO accesses(run_id,at_utc,local_date,hostname,tab_key) VALUES(?1,?2,?3,?4,?5)",params![segment.run_id,at.utc,at.date(),host,tab_key]).map_err(|e|e.to_string())?;
            }
        }
        tx.execute("INSERT INTO tracker_checkpoint VALUES(1,?1,0) ON CONFLICT(id) DO UPDATE SET confirmed_utc=excluded.confirmed_utc,clean_exit=0",[segment.end_utc]).map_err(|e|e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        segment.id = id;
        Ok(())
    }
    pub fn clean_exit(&self) -> Result<()> {
        self.conn
            .execute("UPDATE tracker_checkpoint SET clean_exit=1 WHERE id=1", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn delete(&mut self, day: Option<&str>) -> Result<()> {
        if day.is_some_and(|d| !valid_day(d)) {
            return Err("Data inválida.".into());
        }
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for table in ["sessions", "accesses"] {
            // Table names are compile-time constants; user input is always bound.
            tx.execute(
                &format!("DELETE FROM {table} WHERE ?1 IS NULL OR local_date=?1"),
                [day],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.execute("DELETE FROM tracker_checkpoint", [])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn report(&self, day: &str) -> Result<DayReport> {
        if !valid_day(day) {
            return Err("Data inválida.".into());
        }
        let mut q = self.conn.prepare("SELECT id,run_id,start_utc,end_utc,duration_ms,local_date,offset_seconds,source,hostname,app_name,reason,is_open FROM sessions WHERE local_date=?1 ORDER BY id").map_err(|e|e.to_string())?;
        let sessions = q
            .query_map([day], |r| {
                Ok(Segment {
                    id: r.get(0)?,
                    run_id: r.get(1)?,
                    start_utc: r.get(2)?,
                    end_utc: r.get(3)?,
                    duration_ms: r.get(4)?,
                    local_date: r.get(5)?,
                    offset_seconds: r.get(6)?,
                    activity: Activity {
                        source: r.get(7)?,
                        hostname: r.get(8)?,
                        app_name: r.get(9)?,
                        reason: r.get(10)?,
                    },
                    is_open: r.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut q = self
            .conn
            .prepare("SELECT at_utc,hostname,tab_key FROM accesses WHERE local_date=?1 ORDER BY id")
            .map_err(|e| e.to_string())?;
        let accesses = q
            .query_map([day], |r| {
                Ok(Access {
                    at_utc: r.get(0)?,
                    hostname: r.get(1)?,
                    tab_key: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut q = self
            .conn
            .prepare("SELECT hostname,category FROM domain_categories")
            .map_err(|e| e.to_string())?;
        let domain_mappings = q
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<BTreeMap<_, _>, _>>()
            .map_err(|e| e.to_string())?;
        let mut q = self
            .conn
            .prepare("SELECT app_name,category FROM app_categories")
            .map_err(|e| e.to_string())?;
        let app_mappings = q
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<std::result::Result<BTreeMap<_, _>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(report(
            day,
            sessions,
            accesses,
            self.categories()?,
            domain_mappings,
            app_mappings,
        ))
    }
}

pub struct Recorder {
    pub store: Store,
    pub run_id: String,
    pub current: Option<Segment>,
    last: Option<Stamp>,
    last_flush: u64,
    tab_key: Option<String>,
}
impl Recorder {
    pub fn new(store: Store, at: &Stamp) -> Result<Self> {
        let recovered = store.recover()?;
        let mut this = Self {
            store,
            run_id: format!("{}-{}", at.utc, std::process::id()),
            current: None,
            last: None,
            last_flush: 0,
            tab_key: None,
        };
        if let Some(end) = recovered.filter(|t| *t < at.utc) {
            let start = Stamp {
                utc: end,
                mono: 0,
                offset: at.offset,
            };
            this.add_gap(&start, (at.utc - end) as u64, "app_not_running")?;
        }
        Ok(this)
    }
    fn add_gap(&mut self, start: &Stamp, duration: u64, reason: &str) -> Result<()> {
        let mut at = start.clone();
        let mut left = duration;
        while left > 0 {
            let amount = until_midnight(&at).min(left);
            let mut s = Segment::new(&self.run_id, &at, Activity::new("unobserved", reason));
            s.is_open = false;
            s.end_utc += amount as i64;
            s.duration_ms = amount;
            self.store.persist(&mut s, None)?;
            at = at.advance(amount);
            left -= amount;
        }
        Ok(())
    }
    pub fn observe(&mut self, at: Stamp, activity: Activity, tab: Option<String>) -> Result<()> {
        if let Some(last) = self.last.clone() {
            let elapsed = at.mono.saturating_sub(last.mono);
            let civil = at.utc - last.utc;
            if elapsed > 15_000 || civil > 15_000 && elapsed < 2_000 {
                self.close()?;
                if civil > 0 {
                    self.add_gap(&last, civil as u64, "observation_gap")?;
                }
            } else if (civil - elapsed as i64).abs() > 1_000 || at.offset != last.offset {
                if let Some(current) = &mut self.current {
                    current.duration_ms += elapsed;
                    current.end_utc = last.utc + elapsed as i64;
                }
                self.close()?;
                let mut adjusted = activity.clone();
                adjusted.reason = "clock_adjusted".into();
                self.current = Some(Segment::new(&self.run_id, &at, adjusted));
            } else if let Some(current) = &mut self.current {
                let mut cursor = last;
                let mut left = elapsed;
                while left > 0 {
                    let amount = left.min(until_midnight(&cursor));
                    current.duration_ms += amount;
                    current.end_utc = cursor.utc + amount as i64;
                    cursor = cursor.advance(amount);
                    left -= amount;
                    if cursor.date() != current.local_date {
                        current.is_open = false;
                        self.store.persist(current, None)?;
                        *current = Segment::new(&self.run_id, &cursor, current.activity.clone());
                    }
                }
            }
        }
        let changed = self.current.as_ref().is_none_or(|c| {
            c.activity.source != activity.source
                || c.activity.hostname != activity.hostname
                || c.activity.app_name != activity.app_name
                || (c.activity.reason != activity.reason && c.activity.reason != "clock_adjusted")
        });
        if changed {
            self.close()?;
            self.current = Some(Segment::new(&self.run_id, &at, activity));
        }
        let new_access = tab.is_some() && (changed || self.tab_key != tab);
        self.tab_key = tab;
        self.last = Some(at.clone());
        if changed || new_access || at.mono.saturating_sub(self.last_flush) >= 5_000 {
            if let Some(current) = &mut self.current {
                self.store.persist(
                    current,
                    if new_access {
                        self.tab_key.as_deref().map(|key| (&at, key))
                    } else {
                        None
                    },
                )?;
            }
            self.last_flush = at.mono;
        }
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        if let Some(current) = &mut self.current {
            current.is_open = false;
            self.store.persist(current, None)?;
        }
        self.current = None;
        self.last = None;
        self.tab_key = None;
        Ok(())
    }
    pub fn flush(&mut self) -> Result<()> {
        if let Some(current) = &mut self.current {
            self.store.persist(current, None)?;
        }
        Ok(())
    }
    pub fn truncate_browser(&mut self, at: &Stamp) -> Result<()> {
        if self
            .current
            .as_ref()
            .is_none_or(|s| s.activity.source != "browser")
        {
            return Ok(());
        }
        self.close()?;
        let tx = self.store.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("UPDATE sessions SET duration_ms=MAX(0,duration_ms-(end_utc-?2)),end_utc=?2 WHERE run_id=?1 AND source='browser' AND start_utc<?2 AND end_utc>?2",params![self.run_id,at.utc]).map_err(|e|e.to_string())?;
        tx.execute(
            "DELETE FROM sessions WHERE run_id=?1 AND source='browser' AND start_utc>=?2",
            params![self.run_id, at.utc],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        self.observe(
            at.clone(),
            Activity::app("Google Chrome".into(), "extension_unavailable"),
            None,
        )
    }
}
