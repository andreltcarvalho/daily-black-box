use crate::{
    sessions::*,
    store::{Result, VdiIdentity},
};
use std::{
    mem::size_of,
    ptr::{null, null_mut},
    sync::{mpsc::Sender, OnceLock},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::*, RemoteDesktop::*, SystemInformation::*, Threading::*},
    UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

#[derive(Clone, Debug)]
pub struct Foreground {
    pub hwnd: usize,
    pub identity: Option<VdiIdentity>,
    pub app_name: Option<String>,
    pub idle_ms: Option<u64>,
}
#[derive(Clone, Copy, Debug)]
pub enum PlatformEvent {
    Focus,
    Locked(bool),
    Suspended(bool),
    Failure,
}
static EVENTS: OnceLock<Sender<PlatformEvent>> = OnceLock::new();
pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
pub fn sample() -> Foreground {
    // Read only metadata and the timestamp of last input. Never install keyboard hooks.
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut input = LASTINPUTINFO {
            cbSize: size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let idle = if GetLastInputInfo(&mut input) != 0 {
            let duration = GetTickCount().wrapping_sub(input.dwTime) as u64;
            (duration < 7 * 24 * 60 * 60 * 1000).then_some(duration)
        } else {
            None
        };
        let metadata = metadata(hwnd);
        Foreground {
            hwnd: hwnd as usize,
            identity: metadata.as_ref().map(|(identity, _)| identity.clone()),
            app_name: metadata.map(|(_, app_name)| app_name),
            idle_ms: idle,
        }
    }
}
fn friendly_app_name(executable: &str) -> String {
    match executable.to_ascii_lowercase().as_str() {
        "leagueclient.exe" | "leagueclientux.exe" | "league of legends.exe" => {
            "League of Legends".into()
        }
        "chatgpt.exe" => "ChatGPT".into(),
        "chrome.exe" => "Google Chrome".into(),
        "msrdc.exe" => "Windows App".into(),
        "caixa-preta-do-dia.exe" => "Caixa Preta do Dia".into(),
        _ => executable
            .strip_suffix(".exe")
            .or_else(|| executable.strip_suffix(".EXE"))
            .unwrap_or(executable)
            .replace(['_', '-'], " "),
    }
}
unsafe fn metadata(hwnd: HWND) -> Option<(VdiIdentity, String)> {
    if hwnd.is_null() {
        return None;
    }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process.is_null() {
        return None;
    }
    let mut path = vec![0u16; 32768];
    let mut length = path.len() as u32;
    let ok = QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length);
    CloseHandle(process);
    if ok == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&path[..length as usize]);
    let executable_name = path.rsplit('\\').next()?;
    let exe = executable_name.to_ascii_lowercase();
    let mut class = [0u16; 256];
    let count = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
    if count <= 0 {
        return None;
    }
    let window_class = String::from_utf16_lossy(&class[..count as usize]);
    // Stable package family marker, not the installed version/PID or full path.
    let executable = if exe == "msrdc.exe"
        && path
            .to_ascii_lowercase()
            .contains("microsoftcorporationii.windows365_")
    {
        "WindowsApp/msrdc.exe".into()
    } else {
        exe
    };
    Some((
        VdiIdentity {
            executable,
            window_class,
        },
        friendly_app_name(executable_name),
    ))
}
pub fn capture_vdi() -> Result<VdiIdentity> {
    let candidate = sample()
        .identity
        .ok_or("Não foi possível identificar a janela em foco.")?;
    if candidate.executable != "WindowsApp/msrdc.exe"
        || candidate.window_class != "TscShellContainerClass"
    {
        return Err("Coloque a janela da sessão remota do Windows App em foco, não a tela de conexões, e tente novamente.".into());
    }
    Ok(candidate)
}
pub fn resolve(
    fg: &Foreground,
    vdi: Option<&VdiIdentity>,
    paused: bool,
    locked: bool,
    suspended: bool,
    idle_limit: u64,
) -> Activity {
    if paused {
        return Activity::new("paused", "manual_pause");
    }
    if suspended {
        return Activity::new("unobserved", "suspended");
    }
    if locked {
        return Activity::new("idle", "session_locked");
    }
    match fg.idle_ms {
        None => return Activity::new("unknown", "input_unavailable"),
        Some(idle) if idle >= idle_limit => return Activity::new("idle", "no_input"),
        _ => {}
    }
    match &fg.identity {
        Some(identity) if Some(identity) == vdi => Activity::new("vdi", ""),
        Some(identity) if identity.executable == "caixa-preta-do-dia.exe" => {
            let mut activity = Activity::new("system", "app");
            activity.app_name = fg.app_name.clone();
            activity
        }
        Some(_) => fg
            .app_name
            .clone()
            .map(|name| Activity::app(name, "foreground_app"))
            .unwrap_or_else(|| Activity::new("unknown", "unidentified_window")),
        None => Activity::new("unknown", "unidentified_window"),
    }
}
unsafe extern "system" fn on_focus(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if let Some(tx) = EVENTS.get() {
        let _ = tx.send(PlatformEvent::Focus);
    }
}
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if let Some(tx) = EVENTS.get() {
        if msg == WM_WTSSESSION_CHANGE {
            if w as u32 == WTS_SESSION_LOCK {
                let _ = tx.send(PlatformEvent::Locked(true));
            }
            if w as u32 == WTS_SESSION_UNLOCK {
                let _ = tx.send(PlatformEvent::Locked(false));
            }
        }
        if msg == WM_POWERBROADCAST {
            if w as u32 == PBT_APMSUSPEND {
                let _ = tx.send(PlatformEvent::Suspended(true));
            }
            if matches!(w as u32, PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND) {
                let _ = tx.send(PlatformEvent::Suspended(false));
            }
        }
    }
    DefWindowProcW(hwnd, msg, w, l)
}
pub fn start_hooks(tx: Sender<PlatformEvent>) {
    let _ = EVENTS.set(tx.clone());
    std::thread::spawn(move || unsafe {
        let instance = GetModuleHandleW(null());
        let class = wide("CaixaPretaObserver");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..std::mem::zeroed()
        };
        if RegisterClassW(&wc) == 0 {
            let _ = tx.send(PlatformEvent::Failure);
            return;
        }
        // Hidden top-level window receives power broadcasts; a message-only window does not.
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            null_mut(),
            Some(on_focus),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if hwnd.is_null()
            || hook.is_null()
            || WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0
        {
            let _ = tx.send(PlatformEvent::Failure);
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if !hook.is_null() {
            UnhookWinEvent(hook);
        }
        if !hwnd.is_null() {
            WTSUnRegisterSessionNotification(hwnd);
            DestroyWindow(hwnd);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fg(exe: &str, idle: Option<u64>) -> Foreground {
        Foreground {
            hwnd: 1,
            identity: Some(VdiIdentity {
                executable: exe.into(),
                window_class: "session".into(),
            }),
            app_name: Some(friendly_app_name(exe)),
            idle_ms: idle,
        }
    }
    #[test]
    fn focus_and_idle_have_explicit_precedence() {
        let remote = fg("WindowsApp/msrdc.exe", Some(0));
        let vdi = remote.identity.as_ref();
        assert_eq!(
            resolve(&remote, vdi, false, false, false, 300_000).source,
            "vdi"
        );
        assert_eq!(
            resolve(
                &fg("chrome.exe", Some(0)),
                vdi,
                false,
                false,
                false,
                300_000
            )
            .app_name
            .as_deref(),
            Some("Google Chrome")
        );
        assert_eq!(
            resolve(&remote, vdi, true, true, true, 300_000).source,
            "paused"
        );
        assert_eq!(
            resolve(&remote, vdi, false, true, false, 300_000).source,
            "idle"
        );
        assert_eq!(
            resolve(
                &fg("WindowsApp/msrdc.exe", Some(300_000)),
                vdi,
                false,
                false,
                false,
                300_000
            )
            .source,
            "idle"
        );
        assert_eq!(
            resolve(
                &fg("WindowsApp/msrdc.exe", None),
                vdi,
                false,
                false,
                false,
                300_000
            )
            .source,
            "unknown"
        );
    }
    #[test]
    fn same_executable_different_class_is_not_vdi() {
        let remote = fg("WindowsApp/msrdc.exe", Some(0));
        let mut launcher = remote.clone();
        launcher.identity.as_mut().unwrap().window_class = "launcher".into();
        assert_ne!(
            resolve(
                &launcher,
                remote.identity.as_ref(),
                false,
                false,
                false,
                300_000
            )
            .source,
            "vdi"
        );
    }
    #[test]
    fn local_apps_use_a_readable_name_without_window_content() {
        assert_eq!(
            resolve(
                &fg("LeagueClientUx.exe", Some(0)),
                None,
                false,
                false,
                false,
                300_000
            ),
            Activity::app("League of Legends".into(), "foreground_app")
        );
        assert_eq!(
            resolve(
                &fg("ChatGPT.exe", Some(0)),
                None,
                false,
                false,
                false,
                300_000
            )
            .app_name
            .as_deref(),
            Some("ChatGPT")
        );
    }
}
