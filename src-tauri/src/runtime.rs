use crate::{
    sessions::*,
    store::*,
    windows_tracker::{self, Foreground, PlatformEvent},
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant},
};

pub type Shared = Arc<Mutex<Runtime>>;
#[derive(Clone)]
pub struct BrowserState {
    pub generation: u64,
    pub sequence: u64,
    pub focused: bool,
    pub hostname: Option<String>,
    pub tab_key: Option<String>,
    pub system: bool,
    pub confirmed: Stamp,
}
#[derive(Serialize)]
pub struct Status {
    pub settings: Settings,
    pub activity: Activity,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub extension_connected: bool,
    pub vdi_in_focus: bool,
    pub last_saved_utc: Option<i64>,
    pub data_path: String,
}
pub struct Runtime {
    pub recorder: Recorder,
    pub settings: Settings,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub browser: Option<BrowserState>,
    pub generation: u64,
    pub last_sequence: u64,
    pub last_poll: Option<Instant>,
    pub foreground: Foreground,
    pub locked: bool,
    pub suspended: bool,
    pub start: Instant,
    pub data_path: PathBuf,
    pub shutting_down: bool,
}
impl Runtime {
    pub fn new(path: PathBuf) -> Result<Self> {
        let store = Store::open(&path)?;
        let settings = store.settings()?;
        let at = Stamp::now(0);
        Ok(Self {
            recorder: Recorder::new(store, &at)?,
            settings,
            error: None,
            warning: None,
            browser: None,
            generation: 1,
            last_sequence: 0,
            last_poll: None,
            foreground: windows_tracker::sample(),
            locked: false,
            suspended: false,
            start: Instant::now(),
            data_path: path,
            shutting_down: false,
        })
    }
    pub fn stamp(&self) -> Stamp {
        Stamp::now(self.start.elapsed().as_millis() as u64)
    }
    pub fn invalidate_browser(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.browser = None;
        self.last_sequence = 0;
    }
    pub fn status(&self) -> Status {
        Status {
            settings: self.settings.clone(),
            activity: self
                .recorder
                .current
                .as_ref()
                .map(|s| s.activity.clone())
                .unwrap_or_else(|| Activity::new("paused", "not_started")),
            error: self.error.clone(),
            warning: self.warning.clone(),
            extension_connected: self
                .last_poll
                .is_some_and(|t| t.elapsed() < Duration::from_secs(10)),
            vdi_in_focus: self.settings.vdi.is_some()
                && self.foreground.identity == self.settings.vdi,
            last_saved_utc: self
                .recorder
                .store
                .conn
                .query_row(
                    "SELECT confirmed_utc FROM tracker_checkpoint WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .ok(),
            data_path: self.data_path.to_string_lossy().into(),
        }
    }
    pub fn tick(&mut self, event: Option<PlatformEvent>) {
        if self.shutting_down {
            return;
        }
        if let Some(e) = event {
            match e {
            PlatformEvent::Locked(value)=>{self.locked=value;self.invalidate_browser();},
            PlatformEvent::Suspended(value)=>{self.suspended=value;self.invalidate_browser();},
            PlatformEvent::Failure=>self.warning=Some("Notificações do Windows indisponíveis. Verifique foco, bloqueio e suspensão antes de confiar na coleta.".into()),
            PlatformEvent::Focus=>{},
        }
        }
        let fresh = windows_tracker::sample();
        if fresh.hwnd != self.foreground.hwnd {
            self.invalidate_browser();
        }
        self.foreground = fresh;
        if self.error.is_some() {
            return;
        }
        if let Err(error) = self.record() {
            self.error = Some(format!("Não foi possível gravar a atividade: {error}"));
            self.invalidate_browser();
        }
    }
    fn record(&mut self) -> Result<()> {
        if self.settings.paused && self.recorder.current.is_none() {
            return Ok(());
        }
        let at = self.stamp();
        let limit = self.settings.idle_minutes as u64 * 60_000;
        let mut activity = windows_tracker::resolve(
            &self.foreground,
            self.settings.vdi.as_ref(),
            self.settings.paused,
            self.locked,
            self.suspended,
            limit,
        );
        let mut tab = None;
        let browser_in_focus = self
            .foreground
            .identity
            .as_ref()
            .is_some_and(|i| i.executable == "chrome.exe");
        if browser_in_focus && activity.source == "app" {
            if let Some(browser) = &self.browser {
                if browser.generation == self.generation
                    && at.mono.saturating_sub(browser.confirmed.mono) <= 10_000
                    && browser.focused
                {
                    if let Some(host) = &browser.hostname {
                        activity = Activity::browser(host.clone());
                        tab = browser.tab_key.clone();
                    } else if browser.system {
                        activity = Activity::new("system", "browser_internal");
                        activity.app_name = Some("Google Chrome".into());
                    }
                }
            }
        }
        // Stop assigning a dead browser connection at its last confirmed observation.
        if self
            .browser
            .as_ref()
            .is_some_and(|b| at.mono.saturating_sub(b.confirmed.mono) > 10_000)
        {
            let confirmed = self.browser.as_ref().unwrap().confirmed.clone();
            self.recorder.truncate_browser(&confirmed)?;
            self.invalidate_browser();
        }
        // Record the threshold boundary, not the next polling tick, without retroactive idle.
        if activity.source == "idle"
            && activity.reason == "no_input"
            && self
                .recorder
                .current
                .as_ref()
                .is_some_and(|s| s.activity.source != "idle")
        {
            let excess = self
                .foreground
                .idle_ms
                .unwrap_or(limit)
                .saturating_sub(limit)
                .min(1000);
            let boundary = Stamp {
                utc: at.utc - excess as i64,
                mono: at.mono.saturating_sub(excess),
                offset: at.offset,
            };
            self.recorder.observe(boundary, activity.clone(), None)?;
        }
        self.recorder.observe(at, activity, tab)
    }
    pub fn pause(&mut self, paused: bool) -> Result<()> {
        if !paused && self.settings.vdi.is_none() {
            return Err("Configure a janela da VDI antes de iniciar.".into());
        }
        if self.error.is_some() {
            return Err("Resolva o erro de gravação antes de retomar. Feche e reabra o aplicativo para tentar novamente.".into());
        }
        let mut settings = self.settings.clone();
        settings.paused = paused;
        settings.configured = true;
        self.recorder.store.save_settings(&settings)?;
        self.settings = settings;
        self.invalidate_browser();
        self.tick(None);
        self.error.clone().map_or(Ok(()), Err)
    }
    pub fn delete(&mut self, day: Option<&str>) -> Result<()> {
        self.pause(true)?;
        self.recorder.close()?;
        self.invalidate_browser();
        self.recorder.store.delete(day)?;
        // Remain paused and do not immediately recreate an erased day.
        self.recorder.current = None;
        Ok(())
    }
    pub fn shutdown(&mut self) {
        let _ = self.recorder.close();
        let _ = self.recorder.store.clean_exit();
        self.shutting_down = true;
    }
}
pub fn start(shared: Shared, app: tauri::AppHandle) {
    let (tx, rx) = mpsc::channel();
    windows_tracker::start_hooks(tx);
    std::thread::spawn(move || loop {
        let event = rx.recv_timeout(Duration::from_secs(1)).ok();
        let label = match shared.lock() {
            Ok(mut state) => {
                if state.shutting_down {
                    break;
                }
                state.tick(event);
                if state.error.is_some() {
                    "Erro de gravação"
                } else if state.settings.paused {
                    "Pausado"
                } else {
                    "Rastreamento ativo"
                }
            }
            Err(_) => break,
        };
        // Tauri may dispatch tray operations to the UI thread. Release the runtime
        // lock first so an IPC command on that thread cannot deadlock with this loop.
        if let Some(tray) = app.tray_by_id("main") {
            let _ = tray.set_tooltip(Some(format!("Caixa Preta do Dia — {label}")));
        }
    });
}
