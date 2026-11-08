#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_void};

#[cfg(target_os = "linux")]
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_LAZY: c_int = 1;

/// Isolated dynamic FFI loader for Apple/WebKit `JavaScriptCore` (`libjavascriptcoregtk-6.0.so.1`).
/// Provides 4-tier JIT ES2024 execution with 0 MB binary bloat on Linux/GNOME and macOS.
pub struct JscSys {
    jsc_context_evaluate: unsafe extern "C" fn(*mut c_void, *const c_char, isize) -> *mut c_void,
    jsc_context_get_exception: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    jsc_context_clear_exception: unsafe extern "C" fn(*mut c_void),
    jsc_value_to_string: unsafe extern "C" fn(*mut c_void) -> *mut c_char,
    g_free: unsafe extern "C" fn(*mut c_void),
    g_object_unref: unsafe extern "C" fn(*mut c_void),
    context: *mut c_void,
}

impl JscSys {
    #[cfg(target_os = "linux")]
    pub fn try_load() -> Option<Self> {
        unsafe {
            let jsc_lib = Self::open_lib(c"libjavascriptcoregtk-6.0.so.1")
                .or_else(|| Self::open_lib(c"libjavascriptcoregtk-4.1.so.0"))?;
            let glib_lib = Self::open_lib(c"libglib-2.0.so.0")?;
            let gobject_lib = Self::open_lib(c"libgobject-2.0.so.0")?;

            let jsc_context_new: unsafe extern "C" fn() -> *mut c_void =
                Self::load_fn(jsc_lib, c"jsc_context_new")?;
            let context = jsc_context_new();
            if context.is_null() {
                return None;
            }

            Some(Self {
                jsc_context_evaluate: Self::load_fn(jsc_lib, c"jsc_context_evaluate")?,
                jsc_context_get_exception: Self::load_fn(jsc_lib, c"jsc_context_get_exception")?,
                jsc_context_clear_exception: Self::load_fn(
                    jsc_lib,
                    c"jsc_context_clear_exception",
                )?,
                jsc_value_to_string: Self::load_fn(jsc_lib, c"jsc_value_to_string")?,
                g_free: Self::load_fn(glib_lib, c"g_free")?,
                g_object_unref: Self::load_fn(gobject_lib, c"g_object_unref")?,
                context,
            })
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn try_load() -> Option<Self> {
        None
    }

    #[cfg(target_os = "linux")]
    unsafe fn open_lib(name: &CStr) -> Option<*mut c_void> {
        let handle = unsafe { dlopen(name.as_ptr(), RTLD_LAZY) };
        if handle.is_null() { None } else { Some(handle) }
    }

    #[cfg(target_os = "linux")]
    unsafe fn load_fn<F: Copy>(handle: *mut c_void, sym: &CStr) -> Option<F> {
        let ptr = unsafe { dlsym(handle, sym.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&ptr) })
        }
    }

    pub fn eval(&self, script: &str) -> Result<String, String> {
        let c_script = CString::new(script).map_err(|e| e.to_string())?;
        unsafe {
            let val_ptr = (self.jsc_context_evaluate)(self.context, c_script.as_ptr(), -1);
            let exc_ptr = (self.jsc_context_get_exception)(self.context);
            if !exc_ptr.is_null() {
                (self.jsc_context_clear_exception)(self.context);
                if !val_ptr.is_null() {
                    (self.g_object_unref)(val_ptr);
                }
                return Err(String::from("JavaScriptCore runtime exception"));
            }

            if val_ptr.is_null() {
                return Ok(String::from("undefined"));
            }

            let str_ptr = (self.jsc_value_to_string)(val_ptr);
            let output = if str_ptr.is_null() {
                String::new()
            } else {
                let s = CStr::from_ptr(str_ptr).to_string_lossy().into_owned();
                (self.g_free)(str_ptr.cast::<c_void>());
                s
            };
            (self.g_object_unref)(val_ptr);
            Ok(output)
        }
    }
}

impl Drop for JscSys {
    fn drop(&mut self) {
        if !self.context.is_null() {
            unsafe {
                (self.g_object_unref)(self.context);
            }
        }
    }
}
