use std::env;
use microswitch::run;
use microswitch::error::{error_msgbox, AppRunError, SoundThreadError};

#[cfg(target_os = "windows")]
fn hide_console_window() {
    // This function hides the console window if the program was started using explorer, but it
    // still lets us print if the program was started by cmd.

    use std::ptr;
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::processthreadsapi::GetCurrentProcessId;
    use winapi::um::winuser::GetWindowThreadProcessId;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    let console_window = unsafe {GetConsoleWindow()};
    if console_window == ptr::null_mut() {
        return;
    }

    #[allow(unused_assignments)]
    let mut my_pid: u32 = 0;
    unsafe { my_pid = GetCurrentProcessId(); }

    let mut console_window_pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(console_window, &mut console_window_pid); }

    if my_pid == console_window_pid {
        unsafe { ShowWindow(console_window, SW_HIDE); }
    }
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window() {
    // noop
}


fn main() -> Result<(), AppRunError> {
    println!(concat!("μSwitch ", env!("CARGO_PKG_VERSION")));

    hide_console_window();

    let args = env::args();
    if let Err(err) = run(args) {
        match &err {
            AppRunError::Config { source } => {
                error_msgbox("The configuration is not valid", &source);
            }
            AppRunError::SoundThread { source: SoundThreadError::SampleLoad { source } } => {
                error_msgbox("The configuration is not valid", &source);
            },
            _ => {
                error_msgbox("Unexpected error", &err)
            },
        }

        Err(err)
    } else {
        Ok(())
    }
}
