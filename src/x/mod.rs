
use std::ptr::null;
use std::ffi::CString;
use std::mem::zeroed;
use x11::xlib;



pub fn set_desktop_for_window(window: xlib::Window, desktop: i64) {
    unsafe {
        let display = xlib::XOpenDisplay(null());

        let mut wattr: xlib::XWindowAttributes = zeroed();
        xlib::XGetWindowAttributes(display, window, &mut wattr);

        let wm_desktop = {
            let wm_desktop_str = CString::new("_NET_WM_DESKTOP").unwrap();
            xlib::XInternAtom(display, wm_desktop_str.as_ptr(), xlib::False)
        };

        let root = xlib::XRootWindowOfScreen(wattr.screen);

        let data = {
            let mut data = xlib::ClientMessageData::new();
            data.set_long(0, desktop);
            data.set_long(1, 2);
            data
        };

        let ev = xlib::XClientMessageEvent {
            type_: xlib::ClientMessage,
            serial: 0,
            send_event: 0,
            display: display,
            window: window,
            message_type: wm_desktop,
            format: 32,
            data: data
        };

        xlib::XSendEvent(
            display,
            root,
            xlib::False,
            xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask,
            &mut xlib::XEvent::from(ev));

        xlib::XFlush(display);
        xlib::XCloseDisplay(display);
    }
}
