
use std::ptr::null;
use std::ops::Deref;
use x11::xlib::*;



pub struct DisplayContainer {
    display: *mut Display
}

unsafe impl Sync for DisplayContainer {
}


impl Deref for DisplayContainer {
 type Target = *mut Display;

 fn deref(&self) -> &Self::Target {
     &self.display
 }
}


lazy_static! {
    pub static ref DISPLAY: DisplayContainer = unsafe {
        DisplayContainer { display: XOpenDisplay(null()) }
    };
}



// pub fn with_display<T>(body: &Fn(*mut Display) -> T) -> T {
//     unsafe {
//         let display = XOpenDisplay(null());
//         let result = body(display);
//         XCloseDisplay(display);
//         result
//     }
// }
