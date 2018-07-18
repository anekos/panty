
#![cfg_attr(feature = "cargo-clippy", allow(not_unsafe_ptr_arg_deref))]

use std::ffi::{CString, CStr};
use std::mem::zeroed;
use std::os::raw::c_void;
use std::slice;
use std::thread;
use std::time;
use std::time::Duration;

use x11::xlib::*;



#[macro_export]
macro_rules! with_display {
    { $display:ident => $y:expr }  => {
        {
            use std::ptr::null;
            use std::process::exit;
            use x11::xlib::{XOpenDisplay, XCloseDisplay, Display};
            use x::*;

            unsafe {
                let $display: *mut Display = XOpenDisplay(null());
                if $display.is_null() {
                    error!("Could not open display");
                    exit(1);
                }
                let result = {
                    $y
                };
                XCloseDisplay($display);
                result
            }

        }
    };
}


pub fn wait_for_kill(display: *mut Display, window: Window) {
    while window_exists(display, window) {
        thread::sleep(time::Duration::from_millis(10));
    }
}


pub fn fetch_all_windows(display: *mut Display) -> Vec<u64> {
    unsafe {
        let mut dummy: Window = zeroed();
        let root = XDefaultRootWindow(display);
        let mut p_children: *mut Window = zeroed();
        let mut n_children: u32 = 0;
        XQueryTree(display, root, &mut dummy, &mut dummy, &mut p_children, &mut n_children);

        if n_children == 0 {
            return vec![];
        }

        let children = Vec::from_raw_parts(p_children as *mut Window, n_children as usize, n_children as usize);

        XFree(p_children as *mut c_void);

        children
    }
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


pub fn get_text_property(display: *mut Display, window: Window, name: &str) -> Option<String> {
    unsafe {
        let mut actual_type: u64 = 0;
        let mut actual_format: i32 = 0;
        let mut n_items: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = zeroed();
        let string = intern_atom(display, "STRING");

        let name: u64 = intern_atom(display, name);
        XGetWindowProperty(
            display,
            window,
            name,
            0,
            !0,
            False,
            string,
            &mut actual_type,
            &mut actual_format,
            &mut n_items,
            &mut bytes_after,
            &mut prop);

        if prop.is_null() {
            None
        } else {
            from_cstring(prop as *mut i8)
        }
    }
}


pub fn get_window_class(display: *mut Display, window: Window) -> Option<String> {
    unsafe {
        let mut class_hint: XClassHint = zeroed();

        if XGetClassHint(display, window, &mut class_hint) == 0 {
            return None;
        }

        tap!({
            if class_hint.res_class.is_null() {
                None
            } else {
                from_cstring(class_hint.res_class)
            }
        } => {
            XFree(class_hint.res_name as *mut c_void);
            XFree(class_hint.res_class as *mut c_void);
        })
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

        if n_children == 0 {
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


pub fn get_window_role(display: *mut Display, window: Window) -> Option<String> {
    get_text_property(display, window, "WM_WINDOW_ROLE")
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


pub fn wait_for_visible(display: *mut Display, window: Window, max_tries: u64) -> bool {
    let mut tried = 0;
    while !is_window_visible(display, window) {
        tried += 1;
        thread::sleep(Duration::from_millis(1));
        if max_tries < tried {
            return false;
        }
    }
    true
}


fn intern_atom(display: *mut Display, name: &str) -> u64 {
    unsafe {
        let cstr = CString::new(name).unwrap();
        XInternAtom(display, cstr.as_ptr(), False)
    }
}


fn from_cstring(ptr: *mut i8) -> Option<String> {
    unsafe {
        CStr::from_ptr(ptr).to_str().map(|it| it.to_string()).ok()
    }
}
