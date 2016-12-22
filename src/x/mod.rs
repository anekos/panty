
use std::ffi::CString;
use std::mem::zeroed;
use std::os::raw::c_void;
use std::slice;
use x11::xlib::*;



#[macro_export]
macro_rules! with_display {
    { $display:ident => $y:expr }  => {
        {
            use std::ptr::null;
            use x11::xlib::{XOpenDisplay, XCloseDisplay, Display};
            use x::*;

            unsafe {
                let $display: *mut Display = XOpenDisplay(null());
                let result = {
                    $y
                };
                XCloseDisplay($display);
                result
            }

        }
    };
}


pub fn set_text_property(display: *mut Display, window: Window, name: &str, value: &str) {
    unsafe {
        XChangeProperty(
            display,
            window,
            intern_atom(display, name),
            intern_atom(display, "STRING"),
            8,
            PropModeReplace,
            value.as_ptr(),
            value.len() as i32);
    }
}


pub fn restore_window(display: *mut Display, window: Window) {
    unsafe {
        XMapWindow(display, window);
        XRaiseWindow(display, window);
        XDeleteProperty(display, window, intern_atom(display, "_NET_WM_STATE"));
        XFlush(display);
    }
}


pub fn kill_window(display: *mut Display, window: Window) {
    unsafe {
        XKillClient(display, window);
    }
}


pub fn window_exists(display: *mut Display, window: Window) -> bool {
    unsafe {
        let mut dummy: Window = zeroed();
        let mut p_children: *mut Window = zeroed();
        let mut n_children: u32 = 0;
        let root = XDefaultRootWindow(display);

        XQueryTree(display, root, &mut dummy, &mut dummy, &mut p_children, &mut n_children);

        if n_children <= 0 {
            return false
        }

        let children = slice::from_raw_parts(p_children as *const Window, n_children as usize);

        XFree(p_children as *mut c_void);

        for child in children {
            if *child == window {
                return true
            }
        }

        false
    }
}

pub fn is_window_visible(display: *mut Display, window: Window) -> bool {
    unsafe {
        let mut wattr: XWindowAttributes = zeroed();
        XGetWindowAttributes(display, window, &mut wattr);
        wattr.map_state == IsViewable
    }
}


pub fn unmap_window(display: *mut Display, window: Window) {
    unsafe {
        XUnmapWindow(display, window);
        XFlush(display);
    }
}


pub fn set_window_role(display: *mut Display, window: Window, role: &str) {
    set_text_property(display, window, "WM_WINDOW_ROLE", role);
}


pub fn set_desktop_for_window(display: *mut Display, window: Window, desktop: i64) {
    unsafe {
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
    }
}


pub fn get_current_desktop(display: *mut Display) ->  i64 {
    unsafe {
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
