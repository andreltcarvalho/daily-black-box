use crate::{
    runtime::{BrowserState, Shared},
    sessions::valid_hostname,
    windows_tracker::wide,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read, Write},
    mem::size_of,
    os::windows::io::FromRawHandle,
    ptr::{null, null_mut},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::{Authorization::*, *},
    Storage::FileSystem::*,
    System::{Pipes::*, RemoteDesktop::ProcessIdToSessionId, Threading::*},
};

pub const HOST_NAME: &str = "local.caixapreta.dodia";
pub const EXTENSION_ID: &str = include_str!("../../native-host/extension-id.txt");
pub const MAX_FRAME: usize = 16_384;
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Hello {
        version: u32,
    },
    Poll {},
    BrowserState {
        generation: u64,
        sequence: u64,
        focused: bool,
        hostname: Option<String>,
        tab_id: Option<i64>,
        window_id: Option<i64>,
        system: bool,
    },
}
#[derive(Serialize)]
pub struct Reply {
    pub kind: &'static str,
    pub version: u32,
    pub generation: u64,
    pub paused: bool,
    pub browser_in_focus: bool,
    pub error: Option<String>,
}
pub fn read_frame(input: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut length = [0u8; 4];
    input.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Tamanho de mensagem inválido",
        ));
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes)?;
    Ok(bytes)
}
pub fn write_frame(output: &mut impl Write, data: &[u8]) -> io::Result<()> {
    if data.is_empty() || data.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Tamanho de mensagem inválido",
        ));
    }
    output.write_all(&(data.len() as u32).to_le_bytes())?;
    output.write_all(data)?;
    Ok(())
}
pub fn handle(shared: &Shared, message: Message) -> Reply {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    let mut error = None;
    match message {
        Message::Hello { version } => {
            if version != 1 {
                error = Some("Versão de protocolo incompatível".into());
            } else {
                state.invalidate_browser();
                state.last_poll = Some(Instant::now());
            }
        }
        Message::Poll {} => {
            state.last_poll = Some(Instant::now());
        }
        Message::BrowserState {
            generation,
            sequence,
            focused,
            hostname,
            tab_id,
            window_id,
            system,
        } => {
            if hostname.as_ref().is_some_and(|h| !valid_hostname(h))
                || hostname.is_some()
                    && (!focused || system || tab_id.is_none() || window_id.is_none())
            {
                error = Some("Metadados inválidos".into());
            } else if !state.settings.paused
                && state.error.is_none()
                && generation == state.generation
                && sequence > state.last_sequence
            {
                state.last_sequence = sequence;
                state.browser = Some(BrowserState {
                    generation,
                    sequence,
                    focused,
                    hostname,
                    tab_key: tab_id
                        .zip(window_id)
                        .map(|(t, w)| format!("{generation}:{w}:{t}")),
                    system,
                    confirmed: state.stamp(),
                });
                state.last_poll = Some(Instant::now());
                state.tick(None);
            }
        }
    }
    Reply {
        kind: "tracking_state",
        version: 1,
        generation: state.generation,
        paused: state.settings.paused || state.error.is_some(),
        browser_in_focus: state
            .foreground
            .identity
            .as_ref()
            .is_some_and(|i| i.executable == "chrome.exe"),
        error: error.or_else(|| state.error.clone()),
    }
}
pub fn user_sid() -> io::Result<String> {
    unsafe {
        let mut token = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut size = 0;
        GetTokenInformation(token, TokenUser, null_mut(), 0, &mut size);
        // usize storage ensures TOKEN_USER pointer alignment.
        let mut storage = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
        let ok = GetTokenInformation(
            token,
            TokenUser,
            storage.as_mut_ptr().cast(),
            size,
            &mut size,
        );
        CloseHandle(token);
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        let user = &*(storage.as_ptr().cast::<TOKEN_USER>());
        let mut text = null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut text) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0;
        while *text.add(len) != 0 {
            len += 1;
        }
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(text, len));
        LocalFree(text.cast());
        Ok(sid)
    }
}
pub fn session_id() -> io::Result<u32> {
    unsafe {
        let mut id = 0;
        if ProcessIdToSessionId(GetCurrentProcessId(), &mut id) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(id)
    }
}
pub fn pipe_name() -> io::Result<String> {
    Ok(format!(
        r"\\.\pipe\CaixaPretaDoDia-{}-{}",
        user_sid()?,
        session_id()?
    ))
}
fn server_pipe() -> io::Result<File> {
    unsafe {
        let sddl = wide(&format!("D:P(A;;GA;;;{})", user_sid()?));
        let mut descriptor = null_mut();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            null_mut(),
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let security = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let name = wide(&pipe_name()?);
        let handle = CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            MAX_FRAME as u32,
            MAX_FRAME as u32,
            1000,
            &security,
        );
        LocalFree(descriptor);
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let file = File::from_raw_handle(handle);
        if ConnectNamedPipe(handle, null_mut()) == 0 && GetLastError() != ERROR_PIPE_CONNECTED {
            return Err(io::Error::last_os_error());
        }
        let mut client_session = 0;
        if GetNamedPipeClientSessionId(handle, &mut client_session) == 0
            || client_session != session_id()?
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Sessão Windows não autorizada",
            ));
        }
        Ok(file)
    }
}
pub fn connect_client() -> io::Result<File> {
    unsafe {
        let handle = CreateFileW(
            wide(&pipe_name()?).as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            null(),
            OPEN_EXISTING,
            SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
            null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(File::from_raw_handle(handle))
    }
}
pub fn start(shared: Shared) {
    std::thread::spawn(move || loop {
        match server_pipe() {
            Ok(mut file) => {
                let mut hello = false;
                while let Ok(frame) = read_frame(&mut file) {
                    let Ok(message) = serde_json::from_slice::<Message>(&frame) else {
                        break;
                    };
                    if !hello {
                        if !matches!(message, Message::Hello { version: 1 }) {
                            break;
                        }
                        hello = true;
                    }
                    let reply = handle(&shared, message);
                    if write_frame(&mut file, &serde_json::to_vec(&reply).unwrap()).is_err() {
                        break;
                    }
                }
                if let Ok(mut state) = shared.lock() {
                    if let Some(browser) = state.browser.clone() {
                        let _ = state.recorder.truncate_browser(&browser.confirmed);
                    }
                    state.invalidate_browser();
                    state.last_poll = None;
                }
            }
            Err(error) => {
                if let Ok(mut state) = shared.lock() {
                    state.warning = Some(format!("Ponte do navegador indisponível: {error}"));
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
        if shared.lock().map_or(true, |s| s.shutting_down) {
            break;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frames_validate_lengths_and_partial_input() {
        assert!(read_frame(&mut &((MAX_FRAME + 1) as u32).to_le_bytes()[..]).is_err());
        assert!(read_frame(&mut &[4, 0, 0, 0, 1][..]).is_err());
        let mut bytes = vec![];
        write_frame(&mut bytes, b"{\"kind\":\"poll\"}").unwrap();
        assert_eq!(
            read_frame(&mut bytes.as_slice()).unwrap(),
            b"{\"kind\":\"poll\"}"
        );
    }
    #[test]
    fn protocol_rejects_full_urls_and_unrecognized_fields() {
        assert!(serde_json::from_str::<Message>(
            r#"{"kind":"poll","url":"https://secret.example"}"#
        )
        .is_err());
        assert!(!valid_hostname("example.com/path?token=secret"));
    }
}
