pub mod bridge;
pub mod commands;
pub mod runtime;
pub mod sessions;
pub mod store;
pub mod windows_tracker;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
pub struct StartupIssue {
    pub message: String,
    pub path: String,
}
pub fn run() {
    let app=tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app,_,_|{if let Some(window)=app.get_webview_window("main"){let _=window.show();let _=window.set_focus();}}))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .setup(|app|{
            let directory=app.path().app_local_data_dir()?;
            #[cfg(debug_assertions)]
            let directory=std::env::var_os("CPD_TEST_DATA_DIR").map(std::path::PathBuf::from).unwrap_or(directory);
            let path=directory.join("data.sqlite3");
            let runtime=std::fs::create_dir_all(&directory).map_err(|e|e.to_string()).and_then(|_|runtime::Runtime::new(path.clone()));
            let shared=match runtime {
                Ok(state)=>{let shared=Arc::new(Mutex::new(state));app.manage(shared.clone());Some(shared)},
                Err(error)=>{app.manage(StartupIssue {message:format!("Não foi possível abrir o banco local. Os arquivos existentes foram preservados. {error}"),path:path.to_string_lossy().into()});None}
            };
            let open=MenuItem::with_id(app,"open","Abrir Caixa Preta do Dia",true,None::<&str>)?;
            let pause=MenuItem::with_id(app,"pause","Pausar / retomar rastreamento",true,None::<&str>)?;
            let quit=MenuItem::with_id(app,"quit","Sair e encerrar coleta",true,None::<&str>)?;
            let menu=Menu::with_items(app,&[&open,&pause,&quit])?;
            TrayIconBuilder::with_id("main").icon(app.default_window_icon().unwrap().clone()).menu(&menu).tooltip("Caixa Preta do Dia — Pausado")
                .on_menu_event(|app,event|{
                    match event.id.as_ref(){
                        "open"=>{if let Some(w)=app.get_webview_window("main"){let _=w.show();let _=w.set_focus();}},
                        "pause"=>{if let Some(shared)=app.try_state::<runtime::Shared>(){if let Ok(mut state)=shared.lock(){let paused=state.settings.paused;let failed=state.pause(!paused).is_err();drop(state);if failed{if let Some(w)=app.get_webview_window("main"){let _=w.show();let _=w.set_focus();}}};}},
                        "quit"=>app.exit(0),_=>{}
                    }
                }).build(app)?;
            if let Some(shared)=shared {runtime::start(shared.clone(),app.handle().clone());bridge::start(shared);}
            Ok(())
        })
        .on_window_event(|window,event|if let tauri::WindowEvent::CloseRequested{api,..}=event{api.prevent_close();let _=window.hide();})
        .invoke_handler(tauri::generate_handler![commands::get_status,commands::get_day,commands::set_paused,commands::capture_vdi,commands::set_idle_minutes,commands::classify_domain,commands::classify_app,commands::set_distraction,commands::delete_data,commands::export_day])
        .build(tauri::generate_context!())
        .expect("Não foi possível abrir o aplicativo. Os dados existentes foram preservados.");
    app.run(|app, event| {
        if let tauri::RunEvent::Exit = event {
            if let Some(state) = app.try_state::<runtime::Shared>() {
                if let Ok(mut state) = state.lock() {
                    state.shutdown();
                }
            }
        }
    });
}
