use chrono::{DateTime, FixedOffset, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Activity {
    pub source: String,
    pub hostname: Option<String>,
    pub app_name: Option<String>,
    pub reason: String,
}
impl Activity {
    pub fn new(source: &str, reason: &str) -> Self {
        Self {
            source: source.into(),
            hostname: None,
            app_name: None,
            reason: reason.into(),
        }
    }
    pub fn browser(hostname: String) -> Self {
        Self {
            source: "browser".into(),
            hostname: Some(hostname),
            app_name: Some("Google Chrome".into()),
            reason: String::new(),
        }
    }
    pub fn app(name: String, reason: &str) -> Self {
        Self {
            source: "app".into(),
            hostname: None,
            app_name: Some(name),
            reason: reason.into(),
        }
    }
    pub fn context(&self) -> Option<String> {
        match self.source.as_str() {
            "vdi" | "system" => Some(self.source.clone()),
            "browser" => self.hostname.clone(),
            "app" => self.app_name.clone(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stamp {
    pub utc: i64,
    pub mono: u64,
    pub offset: i32,
}
impl Stamp {
    pub fn now(mono: u64) -> Self {
        let now = Local::now();
        Self {
            utc: now.timestamp_millis(),
            mono,
            offset: now.offset().local_minus_utc(),
        }
    }
    pub fn date(&self) -> String {
        local_datetime(self.utc, self.offset)
            .format("%Y-%m-%d")
            .to_string()
    }
    pub fn advance(&self, ms: u64) -> Self {
        Self {
            utc: self.utc + ms as i64,
            mono: self.mono + ms,
            offset: self.offset,
        }
    }
}
pub fn local_datetime(utc: i64, offset: i32) -> DateTime<FixedOffset> {
    DateTime::<Utc>::from_timestamp_millis(utc)
        .unwrap_or_default()
        .with_timezone(
            &FixedOffset::east_opt(offset).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap()),
        )
}
pub fn valid_day(day: &str) -> bool {
    day.len() == 10 && chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").is_ok()
}
pub fn valid_hostname(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && !host.ends_with('.')
        && host
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b".-:[]".contains(&c))
        && !host.contains("..")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub id: i64,
    pub run_id: String,
    pub start_utc: i64,
    pub end_utc: i64,
    pub duration_ms: u64,
    pub local_date: String,
    pub offset_seconds: i32,
    #[serde(flatten)]
    pub activity: Activity,
    pub is_open: bool,
}
impl Segment {
    pub fn new(run: &str, at: &Stamp, activity: Activity) -> Self {
        Self {
            id: 0,
            run_id: run.into(),
            start_utc: at.utc,
            end_utc: at.utc,
            duration_ms: 0,
            local_date: at.date(),
            offset_seconds: at.offset,
            activity,
            is_open: true,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub distraction: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Access {
    pub at_utc: i64,
    pub hostname: String,
    pub tab_key: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct DomainTotal {
    pub hostname: String,
    pub category: String,
    pub duration_ms: u64,
    pub accesses: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct AppTotal {
    pub label: String,
    pub category: String,
    pub duration_ms: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Bucket {
    pub label: String,
    pub duration_ms: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct DayReport {
    pub schema_version: u32,
    pub date: String,
    pub sessions: Vec<Segment>,
    pub accesses: Vec<Access>,
    pub categories: Vec<Category>,
    pub domains: Vec<DomainTotal>,
    pub app_totals: Vec<AppTotal>,
    pub category_totals: Vec<Bucket>,
    pub hourly: Vec<Bucket>,
    pub vdi_ms: u64,
    pub idle_ms: u64,
    pub unknown_ms: u64,
    pub unclassified_ms: u64,
    pub unidentified_ms: u64,
    pub coverage_ms: u64,
    pub distraction_ms: Option<u64>,
    pub largest_distraction: Option<String>,
    pub longest_focus_ms: u64,
    pub context_switches: u64,
}

pub fn report(
    day: &str,
    sessions: Vec<Segment>,
    accesses: Vec<Access>,
    categories: Vec<Category>,
    domain_mappings: BTreeMap<String, String>,
    app_mappings: BTreeMap<String, String>,
) -> DayReport {
    let configured = categories.iter().any(|c| c.distraction);
    let mut result = DayReport {
        schema_version: 3,
        date: day.into(),
        sessions,
        accesses,
        categories,
        domains: vec![],
        app_totals: vec![],
        category_totals: vec![],
        hourly: vec![],
        vdi_ms: 0,
        idle_ms: 0,
        unknown_ms: 0,
        unclassified_ms: 0,
        unidentified_ms: 0,
        coverage_ms: 0,
        distraction_ms: configured.then_some(0),
        largest_distraction: None,
        longest_focus_ms: 0,
        context_switches: 0,
    };
    let mut domains: BTreeMap<String, DomainTotal> = BTreeMap::new();
    let mut app_totals: BTreeMap<String, AppTotal> = BTreeMap::new();
    let mut cat_totals: BTreeMap<String, u64> = BTreeMap::new();
    let mut hours: BTreeMap<String, u64> = BTreeMap::new();
    let mut previous: Option<(&Segment, String)> = None;
    let mut focus = 0;
    for s in &result.sessions {
        let ms = s.duration_ms;
        if ms == 0 {
            continue;
        }
        let category = match s.activity.source.as_str() {
            "browser" => domain_mappings
                .get(s.activity.hostname.as_deref().unwrap_or(""))
                .map(String::as_str)
                .unwrap_or("unknown"),
            "vdi" => "vdi",
            "app" => app_mappings
                .get(s.activity.app_name.as_deref().unwrap_or(""))
                .map(String::as_str)
                .unwrap_or("local"),
            "system" => "system",
            _ => "unknown",
        };
        let measured = !matches!(s.activity.source.as_str(), "paused" | "unobserved");
        if measured {
            result.coverage_ms += ms;
        }
        if s.activity.source == "idle" {
            result.idle_ms += ms;
        }
        if measured && s.activity.source != "idle" {
            *cat_totals.entry(category.into()).or_default() += ms;
            if category == "unknown" {
                result.unknown_ms += ms;
                if s.activity.source == "browser" {
                    result.unclassified_ms += ms;
                } else {
                    result.unidentified_ms += ms;
                }
            }
        }
        if measured && s.activity.source == "app" {
            if let Some(app_name) = &s.activity.app_name {
                let row = app_totals
                    .entry(app_name.clone())
                    .or_insert_with(|| AppTotal {
                        label: app_name.clone(),
                        category: category.into(),
                        duration_ms: 0,
                    });
                row.duration_ms += ms;
            }
        }
        let connected = previous.as_ref().is_some_and(|(p, _)| {
            p.run_id == s.run_id
                && p.end_utc == s.start_utc
                && s.activity.reason != "clock_adjusted"
        });
        if s.activity.source == "vdi" {
            result.vdi_ms += ms;
            if connected && previous.as_ref().is_some_and(|(_, c)| c == "vdi") {
                focus += ms;
            } else {
                focus = ms;
            }
            result.longest_focus_ms = result.longest_focus_ms.max(focus);
        } else {
            focus = 0;
        }
        if let Some(context) = s.activity.context() {
            if connected && previous.as_ref().is_some_and(|(_, p)| p != &context) {
                result.context_switches += 1;
            }
            previous = Some((s, context));
        } else {
            previous = None;
        }
        if result
            .categories
            .iter()
            .any(|c| c.id == category && c.distraction)
        {
            if let Some(total) = &mut result.distraction_ms {
                *total += ms;
            }
        }
        if let Some(host) = &s.activity.hostname {
            let row = domains.entry(host.clone()).or_insert_with(|| DomainTotal {
                hostname: host.clone(),
                category: category.into(),
                duration_ms: 0,
                accesses: 0,
            });
            row.duration_ms += ms;
        }
        if measured {
            let mut elapsed = 0u64;
            while elapsed < ms {
                let instant = local_datetime(s.start_utc + elapsed as i64, s.offset_seconds);
                let hour = instant.format("%Hh %:z").to_string();
                let local_ms = (s.start_utc + elapsed as i64 + s.offset_seconds as i64 * 1000)
                    .rem_euclid(3_600_000);
                let amount = (3_600_000 - local_ms as u64).min(ms - elapsed);
                *hours.entry(hour).or_default() += amount;
                elapsed += amount;
            }
        }
    }
    for access in &result.accesses {
        let category = domain_mappings
            .get(&access.hostname)
            .cloned()
            .unwrap_or_else(|| "unknown".into());
        domains
            .entry(access.hostname.clone())
            .or_insert_with(|| DomainTotal {
                hostname: access.hostname.clone(),
                category,
                duration_ms: 0,
                accesses: 0,
            })
            .accesses += 1;
    }
    result.domains = domains.into_values().collect();
    result.domains.sort_by(|a, b| {
        b.duration_ms
            .cmp(&a.duration_ms)
            .then(a.hostname.cmp(&b.hostname))
    });
    result.app_totals = app_totals.into_values().collect();
    result.app_totals.sort_by(|a, b| {
        b.duration_ms
            .cmp(&a.duration_ms)
            .then(a.label.cmp(&b.label))
    });
    result.largest_distraction = result
        .domains
        .iter()
        .map(|d| (d.hostname.as_str(), d.category.as_str(), d.duration_ms))
        .chain(
            result
                .app_totals
                .iter()
                .map(|a| (a.label.as_str(), a.category.as_str(), a.duration_ms)),
        )
        .filter(|(_, category, duration_ms)| {
            *duration_ms > 0
                && result
                    .categories
                    .iter()
                    .any(|c| c.id == *category && c.distraction)
        })
        .max_by(|a, b| a.2.cmp(&b.2).then_with(|| b.0.cmp(a.0)))
        .map(|(label, _, _)| label.into());
    result.category_totals = cat_totals
        .into_iter()
        .map(|(label, duration_ms)| Bucket { label, duration_ms })
        .collect();
    result.hourly = hours
        .into_iter()
        .map(|(label, duration_ms)| Bucket { label, duration_ms })
        .collect();
    result
}

// Split at local midnight using the recorded fixed offset. Offset changes create a new anchor.
pub fn until_midnight(at: &Stamp) -> u64 {
    let local = local_datetime(at.utc, at.offset);
    let next = local
        .date_naive()
        .succ_opt()
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    (local
        .offset()
        .from_local_datetime(&next)
        .single()
        .unwrap()
        .timestamp_millis()
        - at.utc) as u64
}
