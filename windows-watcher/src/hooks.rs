use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};

use once_cell::sync::{Lazy, OnceCell};
use windows::core::{Result, PWSTR};
use windows::Win32::Foundation::{CloseHandle, LPARAM, LRESULT, MAX_PATH, POINT, WPARAM};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER, CLSCTX_LOCAL_SERVER};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetWindowThreadProcessId, PostQuitMessage,
    SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, HHOOK,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN, WM_SYSKEYDOWN,
};

use crate::logging::log_message;

static MOUSE_HOOK: Lazy<Mutex<Option<HHOOK>>> = Lazy::new(|| Mutex::new(None));
static KEYBOARD_HOOK: Lazy<Mutex<Option<HHOOK>>> = Lazy::new(|| Mutex::new(None));

/// Shared flag for graceful shutdown.
pub static RUNNING: OnceCell<std::sync::Arc<AtomicBool>> = OnceCell::new();

/// Installs the global low-level keyboard and mouse hooks.
pub fn install_hooks() -> Result<()> {
    let module_handle = unsafe { GetModuleHandleW(None)? };

    let mouse_hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), module_handle, 0)?
    };
    *MOUSE_HOOK.lock().unwrap() = Some(mouse_hook);

    let keyboard_hook = unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), module_handle, 0)?
    };
    *KEYBOARD_HOOK.lock().unwrap() = Some(keyboard_hook);

    Ok(())
}

/// Removes installed keyboard and mouse hooks (optional cleanup API).
pub fn uninstall_hooks() {
    if let Some(h) = MOUSE_HOOK.lock().unwrap().take() {
        unsafe { let _ = UnhookWindowsHookEx(h); }
    }
    if let Some(h) = KEYBOARD_HOOK.lock().unwrap().take() {
        unsafe { let _ = UnhookWindowsHookEx(h); }
    }
}

unsafe extern "system" fn low_level_keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && matches!(w_param.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN) {
        let kbd = *(l_param.0 as *const KBDLLHOOKSTRUCT);
        handle_keyboard_event(kbd.vkCode);
    }

    let hook = KEYBOARD_HOOK.lock().unwrap();
    CallNextHookEx(hook.unwrap_or_default(), n_code, w_param, l_param)
}

fn handle_keyboard_event(vk_code: u32) {
    if vk_code == 0x1B {
        log_message("ESC key pressed — triggering shutdown.");
        trigger_shutdown();
        return;
    }

    let ctrl_pressed = (unsafe { GetAsyncKeyState(0x11) } & 0x8000u16 as i16) != 0;
    if vk_code == 0x43 && ctrl_pressed {
        log_message("Ctrl+C keyboard combo detected — triggering shutdown.");
        trigger_shutdown();
        return;
    }

    let (pid, app_name) = get_foreground_app_info();
    log_message(&format!(
        "Key Down: App='{}' (PID={}), vkCode: {}",
        app_name, pid, vk_code
    ));
}

unsafe extern "system" fn low_level_mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && w_param.0 as u32 == WM_LBUTTONDOWN {
        let mouse = *(l_param.0 as *const MSLLHOOKSTRUCT);
        handle_mouse_click(mouse.pt.x, mouse.pt.y);
    }

    let hook = MOUSE_HOOK.lock().unwrap();
    CallNextHookEx(hook.unwrap_or_default(), n_code, w_param, l_param)
}

fn handle_mouse_click(x: i32, y: i32) {
    log_message(&format!("Mouse Down: Pos=({}, {}) - Starting UIA lookup...", x, y));

    let result: Result<()> = (move || {
        let automation: IUIAutomation = unsafe {
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER | CLSCTX_LOCAL_SERVER)?
        };

        let (name_bstr, automation_id_bstr, pid) = unsafe {
            let element = automation.ElementFromPoint(POINT { x, y })?;
            (
                element.CurrentName()?,
                element.CurrentAutomationId()?,
                element.CurrentProcessId()? as u32,
            )
        };

        let app_name = if pid != 0 {
            get_app_name_from_pid(pid).unwrap_or("<PID Access Denied?>".to_string())
        } else {
            "<PID Err>".to_string()
        };

        log_message(&format!(
            "  Element: App='{}' (PID={}), Name='{}', AutomationID='{}'",
            app_name,
            pid,
            name_bstr.to_string(),
            automation_id_bstr.to_string(),
        ));

        Ok(())
    })();

    if let Err(e) = result {
        log_message(&format!("  UIA Error: {:?}", e));
    }
}

fn trigger_shutdown() {
    if let Some(r) = RUNNING.get() {
        r.store(false, Ordering::SeqCst);
    }
    unsafe { PostQuitMessage(0) };
}

fn get_foreground_app_info() -> (u32, String) {
    let hwnd = unsafe { GetForegroundWindow() };
    let mut pid: u32 = 0;

    let app_name = if hwnd.0 != 0 {
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid != 0 {
            get_app_name_from_pid(pid).unwrap_or("<PID Access Denied?>".to_string())
        } else {
            "<PID Err>".to_string()
        }
    } else {
        "<Unknown>".to_string()
    };

    (pid, app_name)
}

#[cfg(target_os = "windows")]
fn get_app_name_from_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut buffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut size = MAX_PATH;

    let result = unsafe {
        QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(buffer.as_mut_ptr()), &mut size)
    };
    unsafe { let _ = CloseHandle(handle); };

    if result.is_err() {
        return None;
    }

    let os_str = OsString::from_wide(&buffer[..size as usize]);
    Path::new(&os_str)
        .file_name()
        .and_then(OsStr::to_str)
        .map(String::from)
}

#[cfg(not(target_os = "windows"))]
fn get_app_name_from_pid(_pid: u32) -> Option<String> {
    None
}
