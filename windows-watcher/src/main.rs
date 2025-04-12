// #![windows_subsystem = "windows"] // Optional: Prevents console window

use windows::core::{Result};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize,
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
};
use windows::Win32::Foundation::HWND;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod timestamp;
mod logging;
mod hooks;
mod network;

use logging::{log_message, prepare_log_file};
use hooks::{install_hooks, uninstall_hooks, RUNNING};
use network::{start_network_monitor};

fn main() -> Result<()> {
    // Initialize logging
    prepare_log_file()?;

    let running = Arc::new(AtomicBool::new(true));
    RUNNING.set(running.clone()).expect("RUNNING already set");

    log_message("Starting Windows Watcher...");

    // Initialize COM
    let hr = unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
    };
    if hr.is_err() {
        return Err(hr.into());
    }

    // Install hooks (mouse + keyboard)
    install_hooks()?;
    start_network_monitor();
    log_message("Hooks installed. Running message loop...");

    // Message loop
    let mut msg: MSG = MSG::default();
    unsafe {
        while running.load(Ordering::SeqCst) {
            if GetMessageW(&mut msg, HWND(0), 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                break;
            }
        }
    }

    log_message("Exiting message loop. Cleaning up...");
    uninstall_hooks();
    unsafe { CoUninitialize(); }
    log_message("Windows Watcher finished.");

    Ok(())
}
