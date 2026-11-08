#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_ulong, c_void};

/// Opaque native GTK4 widget/object pointer encapsulated inside the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GtkHandle(*mut c_void);

#[cfg(target_os = "linux")]
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_LAZY: c_int = 1;
const GTK_ORIENTATION_VERTICAL: c_int = 1;
const GTK_STYLE_PROVIDER_PRIORITY_APPLICATION: c_uint = 600;

pub type GtkClickCallback = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// Dynamically loaded `GTK4` / `GLib` / `GObject` / `GIO` function table for Linux.
/// Isolates all C FFI calls (`unsafe`) behind a safe Rust API (Single Responsibility Principle).
pub struct Gtk4Sys {
    gtk_css_provider_load_from_string: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_window_new: unsafe extern "C" fn() -> *mut c_void,
    gtk_window_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_window_set_default_size: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    gtk_window_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_window_present: unsafe extern "C" fn(*mut c_void),
    gtk_window_destroy: unsafe extern "C" fn(*mut c_void),
    gtk_window_get_toplevels: unsafe extern "C" fn() -> *mut c_void,
    gtk_box_new: unsafe extern "C" fn(c_int, c_int) -> *mut c_void,
    gtk_box_append: unsafe extern "C" fn(*mut c_void, *mut c_void),
    gtk_label_new: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_label_set_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_button_new_with_label: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    gtk_button_set_label: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_widget_add_css_class: unsafe extern "C" fn(*mut c_void, *const c_char),
    gtk_widget_set_margin_top: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_bottom: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_start: unsafe extern "C" fn(*mut c_void, c_int),
    gtk_widget_set_margin_end: unsafe extern "C" fn(*mut c_void, c_int),
    g_main_context_iteration: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    g_signal_connect_data: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        GtkClickCallback,
        *mut c_void,
        *mut c_void,
        c_uint,
    ) -> c_ulong,
    g_list_model_get_n_items: unsafe extern "C" fn(*mut c_void) -> c_uint,
    css_provider: *mut c_void,
}

impl Gtk4Sys {
    #[cfg(target_os = "linux")]
    pub fn try_load() -> Option<Self> {
        unsafe {
            let gtk_lib = Self::open_lib(c"libgtk-4.so.1")?;
            let gobject_lib = Self::open_lib(c"libgobject-2.0.so.0")?;
            let gio_lib = Self::open_lib(c"libgio-2.0.so.0")?;
            let glib_lib = Self::open_lib(c"libglib-2.0.so.0")?;

            let gtk_init_check: unsafe extern "C" fn() -> c_int =
                Self::load_fn(gtk_lib, c"gtk_init_check")?;
            if gtk_init_check() == 0 {
                return None;
            }

            let css_provider = Self::init_css_provider(gtk_lib)?;

            Some(Self {
                gtk_css_provider_load_from_string: Self::load_fn(
                    gtk_lib,
                    c"gtk_css_provider_load_from_string",
                )?,
                gtk_window_new: Self::load_fn(gtk_lib, c"gtk_window_new")?,
                gtk_window_set_title: Self::load_fn(gtk_lib, c"gtk_window_set_title")?,
                gtk_window_set_default_size: Self::load_fn(
                    gtk_lib,
                    c"gtk_window_set_default_size",
                )?,
                gtk_window_set_child: Self::load_fn(gtk_lib, c"gtk_window_set_child")?,
                gtk_window_present: Self::load_fn(gtk_lib, c"gtk_window_present")?,
                gtk_window_destroy: Self::load_fn(gtk_lib, c"gtk_window_destroy")?,
                gtk_window_get_toplevels: Self::load_fn(gtk_lib, c"gtk_window_get_toplevels")?,
                gtk_box_new: Self::load_fn(gtk_lib, c"gtk_box_new")?,
                gtk_box_append: Self::load_fn(gtk_lib, c"gtk_box_append")?,
                gtk_label_new: Self::load_fn(gtk_lib, c"gtk_label_new")?,
                gtk_label_set_text: Self::load_fn(gtk_lib, c"gtk_label_set_text")?,
                gtk_button_new_with_label: Self::load_fn(gtk_lib, c"gtk_button_new_with_label")?,
                gtk_button_set_label: Self::load_fn(gtk_lib, c"gtk_button_set_label")?,
                gtk_widget_add_css_class: Self::load_fn(gtk_lib, c"gtk_widget_add_css_class")?,
                gtk_widget_set_margin_top: Self::load_fn(gtk_lib, c"gtk_widget_set_margin_top")?,
                gtk_widget_set_margin_bottom: Self::load_fn(
                    gtk_lib,
                    c"gtk_widget_set_margin_bottom",
                )?,
                gtk_widget_set_margin_start: Self::load_fn(
                    gtk_lib,
                    c"gtk_widget_set_margin_start",
                )?,
                gtk_widget_set_margin_end: Self::load_fn(gtk_lib, c"gtk_widget_set_margin_end")?,
                g_main_context_iteration: Self::load_fn(glib_lib, c"g_main_context_iteration")?,
                g_signal_connect_data: Self::load_fn(gobject_lib, c"g_signal_connect_data")?,
                g_list_model_get_n_items: Self::load_fn(gio_lib, c"g_list_model_get_n_items")?,
                css_provider,
            })
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn try_load() -> Option<Self> {
        None
    }

    #[cfg(target_os = "linux")]
    unsafe fn init_css_provider(gtk_lib: *mut c_void) -> Option<*mut c_void> {
        unsafe {
            let gdk_display_get_default: unsafe extern "C" fn() -> *mut c_void =
                Self::load_fn(gtk_lib, c"gdk_display_get_default")?;
            let gtk_css_provider_new: unsafe extern "C" fn() -> *mut c_void =
                Self::load_fn(gtk_lib, c"gtk_css_provider_new")?;
            let gtk_style_context_add_provider_for_display: unsafe extern "C" fn(
                *mut c_void,
                *mut c_void,
                c_uint,
            ) = Self::load_fn(gtk_lib, c"gtk_style_context_add_provider_for_display")?;

            let display = gdk_display_get_default();
            let css_provider = gtk_css_provider_new();
            if !display.is_null() && !css_provider.is_null() {
                gtk_style_context_add_provider_for_display(
                    display,
                    css_provider,
                    GTK_STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
            Some(css_provider)
        }
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

    pub fn load_css(&self, css: &str) {
        if self.css_provider.is_null() {
            return;
        }
        if let Ok(c_css) = CString::new(css) {
            unsafe {
                (self.gtk_css_provider_load_from_string)(self.css_provider, c_css.as_ptr());
            }
        }
    }

    pub fn create_window(&self) -> GtkHandle {
        unsafe {
            let win = (self.gtk_window_new)();
            (self.gtk_window_set_default_size)(win, 460, 280);
            GtkHandle(win)
        }
    }

    pub fn create_box(&self, spacing: i32) -> GtkHandle {
        unsafe {
            let bx = (self.gtk_box_new)(GTK_ORIENTATION_VERTICAL, spacing);
            (self.gtk_widget_set_margin_top)(bx, 20);
            (self.gtk_widget_set_margin_bottom)(bx, 20);
            (self.gtk_widget_set_margin_start)(bx, 20);
            (self.gtk_widget_set_margin_end)(bx, 20);
            GtkHandle(bx)
        }
    }

    pub fn create_label(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_label_new)(c"".as_ptr())) }
    }

    pub fn create_button(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_button_new_with_label)(c"".as_ptr())) }
    }

    pub fn set_window_title(&self, win: GtkHandle, title: &str) {
        if let Ok(c_str) = CString::new(title) {
            unsafe {
                (self.gtk_window_set_title)(win.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_label_text(&self, lbl: GtkHandle, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            unsafe {
                (self.gtk_label_set_text)(lbl.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_button_label(&self, btn: GtkHandle, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            unsafe {
                (self.gtk_button_set_label)(btn.0, c_str.as_ptr());
            }
        }
    }

    pub fn add_css_class(&self, widget: GtkHandle, class_name: &str) {
        if let Ok(c_str) = CString::new(class_name) {
            unsafe {
                (self.gtk_widget_add_css_class)(widget.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_window_child(&self, win: GtkHandle, child: GtkHandle) {
        unsafe {
            (self.gtk_window_set_child)(win.0, child.0);
            (self.gtk_window_present)(win.0);
        }
    }

    pub fn box_append(&self, bx: GtkHandle, child: GtkHandle) {
        unsafe {
            (self.gtk_box_append)(bx.0, child.0);
        }
    }

    pub fn destroy_window(&self, win: GtkHandle) {
        unsafe {
            (self.gtk_window_destroy)(win.0);
        }
    }

    pub fn connect_clicked(
        &self,
        btn: GtkHandle,
        callback: GtkClickCallback,
        packed_node_id: usize,
    ) {
        unsafe {
            (self.g_signal_connect_data)(
                btn.0,
                c"clicked".as_ptr(),
                callback,
                packed_node_id as *mut c_void,
                std::ptr::null_mut(),
                0,
            );
        }
    }

    pub fn open_window_count(&self) -> u32 {
        unsafe {
            let list = (self.gtk_window_get_toplevels)();
            if list.is_null() {
                0
            } else {
                (self.g_list_model_get_n_items)(list)
            }
        }
    }

    pub fn step_main_context(&self, may_block: bool) {
        unsafe {
            (self.g_main_context_iteration)(std::ptr::null_mut(), i32::from(may_block));
        }
    }
}
