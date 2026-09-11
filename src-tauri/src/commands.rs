use crate::{
    runtime::{Shared, Status},
    sessions::DayReport,
    store::Result,
    windows_tracker,
};
use std::{io::Write, path::PathBuf};
use tauri::State;
#[tauri::command]
pub fn get_status(app: tauri::AppHandle) -> Result<Status> {
    use tauri::Manager;
    if let Some(state) = app.try_state::<Shared>() {
        return Ok(state.lock().map_err(|e| e.to_string())?.status());
    }
    let issue = app.state::<crate::StartupIssue>();
    Ok(Status {
        settings: crate::store::Settings::default(),
        activity: crate::sessions::Activity::new("paused", "storage_unavailable"),
        error: Some(issue.message.clone()),
        warning: None,
        extension_connected: false,
        vdi_in_focus: false,
        last_saved_utc: None,
        data_path: issue.path.clone(),
    })
}
#[tauri::command]
pub fn get_day(day: String, state: State<'_, Shared>) -> Result<DayReport> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.recorder.store.report(&day)
}
#[tauri::command]
pub fn set_paused(paused: bool, state: State<'_, Shared>) -> Result<()> {
    state.lock().map_err(|e| e.to_string())?.pause(paused)
}
#[tauri::command]
pub async fn capture_vdi(state: State<'_, Shared>) -> Result<()> {
    let shared = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let identity = windows_tracker::capture_vdi()?;
        let mut state = shared.lock().map_err(|e| e.to_string())?;
        let mut settings = state.settings.clone();
        settings.vdi = Some(identity);
        state.recorder.store.save_settings(&settings)?;
        state.settings = settings;
        state.invalidate_browser();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn set_idle_minutes(minutes: u32, state: State<'_, Shared>) -> Result<()> {
    if !(1..=60).contains(&minutes) {
        return Err("Escolha entre 1 e 60 minutos.".into());
    }
    let mut state = state.lock().map_err(|e| e.to_string())?;
    let mut settings = state.settings.clone();
    settings.idle_minutes = minutes;
    state.recorder.store.save_settings(&settings)?;
    state.recorder.close()?;
    state.settings = settings;
    state.invalidate_browser();
    Ok(())
}
#[tauri::command]
pub fn classify_domain(hostname: String, category: String, state: State<'_, Shared>) -> Result<()> {
    state
        .lock()
        .map_err(|e| e.to_string())?
        .recorder
        .store
        .classify(&hostname, &category)
}
#[tauri::command]
pub fn classify_app(app_name: String, category: String, state: State<'_, Shared>) -> Result<()> {
    state
        .lock()
        .map_err(|e| e.to_string())?
        .recorder
        .store
        .classify_app(&app_name, &category)
}
#[tauri::command]
pub fn set_distraction(category: String, enabled: bool, state: State<'_, Shared>) -> Result<()> {
    state
        .lock()
        .map_err(|e| e.to_string())?
        .recorder
        .store
        .set_distraction(&category, enabled)
}
#[tauri::command]
pub fn delete_data(day: Option<String>, state: State<'_, Shared>) -> Result<()> {
    state
        .lock()
        .map_err(|e| e.to_string())?
        .delete(day.as_deref())
}

pub fn write_export(report: &DayReport, path: &std::path::Path) -> Result<()> {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_none_or(|s| !s.eq_ignore_ascii_case("json"))
    {
        return Err("Escolha um arquivo .json.".into());
    }
    let data = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&data).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        std::fs::rename(&temporary, path).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
#[tauri::command]
pub fn export_day(day: String, path: String, state: State<'_, Shared>) -> Result<()> {
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err("Selecione um caminho absoluto para exportar.".into());
    }
    let mut state = state.lock().map_err(|e| e.to_string())?;
    if state.error.is_none() {
        state.recorder.flush()?;
    }
    let report = state.recorder.store.report(&day)?;
    write_export(&report, &path)
}
