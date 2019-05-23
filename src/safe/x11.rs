use std::ffi::CStr;

pub(crate) struct DisplayWrapper {
    display: *mut x11::xlib::Display,
}

pub(crate) struct FailedToOpenXDisplay;

impl Drop for DisplayWrapper {
    fn drop(&mut self) {
        unsafe { x11::xlib::XCloseDisplay(self.display) };
    }
}

pub(crate) fn open_display(display: &CStr) -> Result<DisplayWrapper, FailedToOpenXDisplay> {
    let connection = unsafe { x11::xlib::XOpenDisplay(display.as_ptr()) };

    if connection.is_null() {
        Err(FailedToOpenXDisplay)
    } else {
        Ok(DisplayWrapper {
            display: connection,
        })
    }
}

pub(crate) fn start_x_server(display: &str, vt: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new("X").arg(display).arg(vt).spawn()
}

pub(crate) fn poll_for_x_available(
    x: &mut std::process::Child,
    display: &CStr,
) -> Result<bool, crate::ProgramError> {
    if let Some(_status) = x.try_wait().map_err(crate::ProgramError::Io)? {
        Err(crate::ProgramError::XServerQuit)
    } else {
        // Process has not exited at least, maybe it worked or will work in the future
        Ok(open_display(display).is_ok())
    }
}
