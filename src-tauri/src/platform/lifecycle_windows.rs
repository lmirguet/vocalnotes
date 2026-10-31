//! Hidden native receiver, not an indicator. Requires real Windows 11 recipe.
use super::*;
use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::*},
    UI::WindowsAndMessaging::*,
};

struct Callback(Arc<dyn Fn() + Send + Sync>);

unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // The pointer is installed at creation and owned until DestroyWindow returns.
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(lp as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let callback = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Callback;
        if !callback.is_null()
            && ((message == WM_WTSSESSION_CHANGE
                && matches!(
                    wp as u32,
                    WTS_SESSION_LOCK
                        | WTS_SESSION_LOGOFF
                        | WTS_CONSOLE_DISCONNECT
                        | WTS_REMOTE_DISCONNECT
                ))
                || (message == WM_POWERBROADCAST && wp as u32 == PBT_APMSUSPEND)
                || message == WM_ENDSESSION
                || message == WM_DESTROY)
        {
            ((*callback).0)();
        }
        DefWindowProcW(hwnd, message, wp, lp)
    }
}

fn unlocked() -> Result<(), &'static str> {
    let mut buffer = null_mut();
    let mut bytes = 0u32;
    // WTS allocates this buffer; inspect only documented fields then always release it.
    unsafe {
        if WTSQuerySessionInformationW(
            null_mut(),
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &mut buffer,
            &mut bytes,
        ) == 0
        {
            return Err("Surveillance WTS indisponible : vocal refusé.");
        }
        let valid = if !buffer.is_null() && bytes as usize >= size_of::<WTSINFOEXW>() {
            let info = &*(buffer as *const WTSINFOEXW);
            info.Level == 1
                && info.Data.WTSInfoExLevel1.SessionFlags as u32 == WTS_SESSIONSTATE_UNLOCK
                && info.Data.WTSInfoExLevel1.SessionState == WTSActive
        } else {
            false
        };
        WTSFreeMemory(buffer as *mut c_void);
        if !valid {
            return Err("Session verrouillée/déconnectée ou état WTS inconnu : vocal refusé.");
        }
    }
    Ok(())
}

pub(super) fn monitor(
    stop: Arc<AtomicBool>,
    callback: Arc<dyn Fn() + Send + Sync>,
    ready: &mpsc::SyncSender<Result<(), &'static str>>,
) -> Result<(), &'static str> {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let name: Vec<u16> = format!(
        "VocalnotesLifecycle{}\0",
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
    .encode_utf16()
    .collect();
    let mut context = Box::new(Callback(callback));
    // Entire HWND and message pump live on one thread. Never show this window.
    unsafe {
        let instance = GetModuleHandleW(null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            ..zeroed()
        };
        if RegisterClassW(&class) == 0 {
            return Err("Création surveillance Windows impossible.");
        }
        // A hidden top-level window receives power broadcasts; HWND_MESSAGE would not.
        let hwnd = CreateWindowExW(
            0,
            name.as_ptr(),
            name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            (&mut *context as *mut Callback).cast(),
        );
        if hwnd.is_null() {
            UnregisterClassW(name.as_ptr(), instance);
            return Err("Fenêtre de surveillance Windows indisponible.");
        }
        let registered = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) != 0;
        let result = (|| {
            if !registered {
                return Err("Abonnement WTS impossible : vocal refusé.");
            }
            unlocked()?; // subscribe before initial state
            ready
                .send(Ok(()))
                .map_err(|_| "Initialisation Windows annulée.")?;
            while !stop.load(Ordering::Acquire) {
                let mut message: MSG = zeroed();
                while PeekMessageW(&mut message, hwnd, 0, 0, PM_REMOVE) != 0 {
                    if message.message == WM_QUIT {
                        return Err("Surveillance Windows interrompue.");
                    }
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
                unlocked()?; // also fails closed on WTS service loss
                thread::sleep(Duration::from_millis(100));
            }
            Ok(())
        })();
        if registered {
            WTSUnRegisterSessionNotification(hwnd);
        }
        // Planned teardown must not emit a false lock/stop to a newer session.
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        DestroyWindow(hwnd);
        UnregisterClassW(name.as_ptr(), instance);
        result
    }
}
