
use std::ptr::null;
use std::ffi::CString;
use std::mem::zeroed;
use x11::xlib::*;



pub fn set_desktop_for_window(window: Window, desktop: i64) {
    unsafe {
        let display: *mut Display = XOpenDisplay(null());

        let mut wattr: XWindowAttributes = zeroed();
        XGetWindowAttributes(display, window, &mut wattr);

        let wm_desktop = intern_atom(display, "_NET_WM_DESKTOP");

        let root = XRootWindowOfScreen(wattr.screen);

        let data = {
            let mut data = ClientMessageData::new();
            data.set_long(0, desktop);
            data.set_long(1, 2);
            data
        };

        let ev = XClientMessageEvent {
            type_: ClientMessage,
            serial: 0,
            send_event: 0,
            display: display,
            window: window,
            message_type: wm_desktop,
            format: 32,
            data: data
        };

        XSendEvent(
            display,
            root,
            False,
            SubstructureNotifyMask | SubstructureRedirectMask,
            &mut XEvent::from(ev));

        XFlush(display);
        XCloseDisplay(display);
    }
}


pub fn get_current_desktop() ->  i64 {
    unsafe {
        let display = XOpenDisplay(null());
        let root = XDefaultRootWindow(display);

        let mut actual_type: u64 = 0;
        let mut actual_format: i32 = 0;
        let mut n_items: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = zeroed();

        let current_desktop: u64 = intern_atom(display, "_NET_CURRENT_DESKTOP");
        XGetWindowProperty(
            display,
            root,
            current_desktop,
            0,
            !0,
            False,
            AnyPropertyType as u64,
            &mut actual_type,
            &mut actual_format,
            &mut n_items,
            &mut bytes_after,
            &mut prop);

        if n_items > 0 {
            *(prop as *mut i64)
        } else {
            panic!("Fail: _NET_CURRENT_DESKTOP")
        }
    }
}


fn intern_atom(display: *mut Display, name: &str) -> u64 {
    unsafe {
        let cstr = CString::new(name).unwrap();
        XInternAtom(display, cstr.as_ptr(), False)
    }
}
