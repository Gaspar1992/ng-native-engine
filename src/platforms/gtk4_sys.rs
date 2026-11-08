#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_int, c_uint, c_ulong, c_void};

/// Opaque native GTK4 widget/object pointer encapsulated inside the FFI boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GtkHandle(pub(super) *mut c_void);

#[cfg(target_os = "linux")]
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

const RTLD_LAZY: c_int = 1;
pub const GTK_ORIENTATION_HORIZONTAL: c_int = 0;
pub const GTK_ORIENTATION_VERTICAL: c_int = 1;
const GTK_STYLE_PROVIDER_PRIORITY_APPLICATION: c_uint = 600;

pub type GtkClickCallback = unsafe extern "C" fn(*mut c_void, *mut c_void);

/// Dynamically loaded `GTK4` / `GLib` / `GObject` / `GIO` function table for Linux.
/// Isolates all C FFI symbol loading (`dlopen` / `dlsym`) from widget operations.
pub struct Gtk4Sys {
    pub(super) gtk_css_provider_load_from_string: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_icon_theme_add_search_path: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_window_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_window_set_title: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_window_set_titlebar: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_window_set_icon_name: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_window_set_default_icon_name: unsafe extern "C" fn(*const c_char),
    pub(super) gtk_window_set_default_size: unsafe extern "C" fn(*mut c_void, c_int, c_int),
    pub(super) gtk_window_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_window_present: unsafe extern "C" fn(*mut c_void),
    pub(super) gtk_window_destroy: unsafe extern "C" fn(*mut c_void),
    pub(super) gtk_window_get_toplevels: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_header_bar_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_header_bar_set_show_title_buttons: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_header_bar_pack_end: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_scrolled_window_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_scrolled_window_set_child: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_scrolled_window_set_min_content_height: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_box_new: unsafe extern "C" fn(c_int, c_int) -> *mut c_void,
    pub(super) gtk_box_append: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_box_remove: unsafe extern "C" fn(*mut c_void, *mut c_void),
    pub(super) gtk_box_insert_child_after:
        unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    pub(super) gtk_box_set_spacing: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_grid_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_grid_attach:
        unsafe extern "C" fn(*mut c_void, *mut c_void, c_int, c_int, c_int, c_int),
    pub(super) gtk_grid_set_row_spacing: unsafe extern "C" fn(*mut c_void, c_uint),
    pub(super) gtk_grid_set_column_spacing: unsafe extern "C" fn(*mut c_void, c_uint),
    pub(super) gtk_separator_new: unsafe extern "C" fn(c_int) -> *mut c_void,
    pub(super) gtk_orientable_set_orientation: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_label_new: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub(super) gtk_label_set_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_button_new_with_label: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub(super) gtk_button_set_label: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_entry_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_entry_set_placeholder_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_editable_set_text: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_switch_new: unsafe extern "C" fn() -> *mut c_void,
    pub(super) gtk_switch_set_active: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_check_button_new_with_label: unsafe extern "C" fn(*const c_char) -> *mut c_void,
    pub(super) gtk_check_button_set_label: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_check_button_set_active: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_scale_new_with_range: unsafe extern "C" fn(c_int, f64, f64, f64) -> *mut c_void,
    pub(super) gtk_range_set_value: unsafe extern "C" fn(*mut c_void, f64),
    pub(super) gtk_drop_down_new_from_strings:
        unsafe extern "C" fn(*const *const c_char) -> *mut c_void,
    pub(super) gtk_drop_down_set_selected: unsafe extern "C" fn(*mut c_void, c_uint),
    pub(super) gtk_widget_add_css_class: unsafe extern "C" fn(*mut c_void, *const c_char),
    pub(super) gtk_widget_set_hexpand: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_vexpand: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_halign: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_valign: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_margin_top: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_margin_bottom: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_margin_start: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) gtk_widget_set_margin_end: unsafe extern "C" fn(*mut c_void, c_int),
    pub(super) g_main_context_iteration: unsafe extern "C" fn(*mut c_void, c_int) -> c_int,
    pub(super) g_set_prgname: unsafe extern "C" fn(*const c_char),
    pub(super) g_set_application_name: unsafe extern "C" fn(*const c_char),
    pub(super) g_signal_connect_data: unsafe extern "C" fn(
        *mut c_void,
        *const c_char,
        GtkClickCallback,
        *mut c_void,
        *mut c_void,
        c_uint,
    ) -> c_ulong,
    pub(super) g_list_model_get_n_items: unsafe extern "C" fn(*mut c_void) -> c_uint,
    pub(super) css_provider: *mut c_void,
    pub(super) icon_theme: *mut c_void,
}

impl Gtk4Sys {
    #[cfg(target_os = "linux")]
    #[allow(clippy::too_many_lines)]
    pub fn try_load() -> Option<Self> {
        unsafe {
            let gtk_lib = Self::open_lib(c"libgtk-4.so.1")?;
            let gobject_lib = Self::open_lib(c"libgobject-2.0.so.0")?;
            let gio_lib = Self::open_lib(c"libgio-2.0.so.0")?;
            let glib_lib = Self::open_lib(c"libglib-2.0.so.0")?;

            let g_set_prgname: unsafe extern "C" fn(*const c_char) =
                Self::load_fn(glib_lib, c"g_set_prgname")?;
            g_set_prgname(c"ng-linux-counter".as_ptr());

            let gtk_init_check: unsafe extern "C" fn() -> c_int =
                Self::load_fn(gtk_lib, c"gtk_init_check")?;
            if gtk_init_check() == 0 {
                return None;
            }

            let (css_provider, icon_theme) = Self::init_display_resources(gtk_lib)?;

            Some(Self {
                gtk_css_provider_load_from_string: Self::load_fn(
                    gtk_lib,
                    c"gtk_css_provider_load_from_string",
                )?,
                gtk_icon_theme_add_search_path: Self::load_fn(
                    gtk_lib,
                    c"gtk_icon_theme_add_search_path",
                )?,
                gtk_window_new: Self::load_fn(gtk_lib, c"gtk_window_new")?,
                gtk_window_set_title: Self::load_fn(gtk_lib, c"gtk_window_set_title")?,
                gtk_window_set_titlebar: Self::load_fn(gtk_lib, c"gtk_window_set_titlebar")?,
                gtk_window_set_icon_name: Self::load_fn(gtk_lib, c"gtk_window_set_icon_name")?,
                gtk_window_set_default_icon_name: Self::load_fn(
                    gtk_lib,
                    c"gtk_window_set_default_icon_name",
                )?,
                gtk_window_set_default_size: Self::load_fn(
                    gtk_lib,
                    c"gtk_window_set_default_size",
                )?,
                gtk_window_set_child: Self::load_fn(gtk_lib, c"gtk_window_set_child")?,
                gtk_window_present: Self::load_fn(gtk_lib, c"gtk_window_present")?,
                gtk_window_destroy: Self::load_fn(gtk_lib, c"gtk_window_destroy")?,
                gtk_window_get_toplevels: Self::load_fn(gtk_lib, c"gtk_window_get_toplevels")?,
                gtk_header_bar_new: Self::load_fn(gtk_lib, c"gtk_header_bar_new")?,
                gtk_header_bar_set_show_title_buttons: Self::load_fn(
                    gtk_lib,
                    c"gtk_header_bar_set_show_title_buttons",
                )?,
                gtk_header_bar_pack_end: Self::load_fn(gtk_lib, c"gtk_header_bar_pack_end")?,
                gtk_scrolled_window_new: Self::load_fn(gtk_lib, c"gtk_scrolled_window_new")?,
                gtk_scrolled_window_set_child: Self::load_fn(
                    gtk_lib,
                    c"gtk_scrolled_window_set_child",
                )?,
                gtk_scrolled_window_set_min_content_height: Self::load_fn(
                    gtk_lib,
                    c"gtk_scrolled_window_set_min_content_height",
                )?,
                gtk_box_new: Self::load_fn(gtk_lib, c"gtk_box_new")?,
                gtk_box_append: Self::load_fn(gtk_lib, c"gtk_box_append")?,
                gtk_box_remove: Self::load_fn(gtk_lib, c"gtk_box_remove")?,
                gtk_box_insert_child_after: Self::load_fn(gtk_lib, c"gtk_box_insert_child_after")?,
                gtk_box_set_spacing: Self::load_fn(gtk_lib, c"gtk_box_set_spacing")?,
                gtk_grid_new: Self::load_fn(gtk_lib, c"gtk_grid_new")?,
                gtk_grid_attach: Self::load_fn(gtk_lib, c"gtk_grid_attach")?,
                gtk_grid_set_row_spacing: Self::load_fn(gtk_lib, c"gtk_grid_set_row_spacing")?,
                gtk_grid_set_column_spacing: Self::load_fn(
                    gtk_lib,
                    c"gtk_grid_set_column_spacing",
                )?,
                gtk_separator_new: Self::load_fn(gtk_lib, c"gtk_separator_new")?,
                gtk_orientable_set_orientation: Self::load_fn(
                    gtk_lib,
                    c"gtk_orientable_set_orientation",
                )?,
                gtk_label_new: Self::load_fn(gtk_lib, c"gtk_label_new")?,
                gtk_label_set_text: Self::load_fn(gtk_lib, c"gtk_label_set_text")?,
                gtk_button_new_with_label: Self::load_fn(gtk_lib, c"gtk_button_new_with_label")?,
                gtk_button_set_label: Self::load_fn(gtk_lib, c"gtk_button_set_label")?,
                gtk_entry_new: Self::load_fn(gtk_lib, c"gtk_entry_new")?,
                gtk_entry_set_placeholder_text: Self::load_fn(
                    gtk_lib,
                    c"gtk_entry_set_placeholder_text",
                )?,
                gtk_editable_set_text: Self::load_fn(gtk_lib, c"gtk_editable_set_text")?,
                gtk_switch_new: Self::load_fn(gtk_lib, c"gtk_switch_new")?,
                gtk_switch_set_active: Self::load_fn(gtk_lib, c"gtk_switch_set_active")?,
                gtk_check_button_new_with_label: Self::load_fn(
                    gtk_lib,
                    c"gtk_check_button_new_with_label",
                )?,
                gtk_check_button_set_label: Self::load_fn(gtk_lib, c"gtk_check_button_set_label")?,
                gtk_check_button_set_active: Self::load_fn(
                    gtk_lib,
                    c"gtk_check_button_set_active",
                )?,
                gtk_scale_new_with_range: Self::load_fn(gtk_lib, c"gtk_scale_new_with_range")?,
                gtk_range_set_value: Self::load_fn(gtk_lib, c"gtk_range_set_value")?,
                gtk_drop_down_new_from_strings: Self::load_fn(
                    gtk_lib,
                    c"gtk_drop_down_new_from_strings",
                )?,
                gtk_drop_down_set_selected: Self::load_fn(gtk_lib, c"gtk_drop_down_set_selected")?,
                gtk_widget_add_css_class: Self::load_fn(gtk_lib, c"gtk_widget_add_css_class")?,
                gtk_widget_set_hexpand: Self::load_fn(gtk_lib, c"gtk_widget_set_hexpand")?,
                gtk_widget_set_vexpand: Self::load_fn(gtk_lib, c"gtk_widget_set_vexpand")?,
                gtk_widget_set_halign: Self::load_fn(gtk_lib, c"gtk_widget_set_halign")?,
                gtk_widget_set_valign: Self::load_fn(gtk_lib, c"gtk_widget_set_valign")?,
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
                g_set_prgname,
                g_set_application_name: Self::load_fn(glib_lib, c"g_set_application_name")?,
                g_signal_connect_data: Self::load_fn(gobject_lib, c"g_signal_connect_data")?,
                g_list_model_get_n_items: Self::load_fn(gio_lib, c"g_list_model_get_n_items")?,
                css_provider,
                icon_theme,
            })
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn try_load() -> Option<Self> {
        None
    }

    #[cfg(target_os = "linux")]
    unsafe fn init_display_resources(gtk_lib: *mut c_void) -> Option<(*mut c_void, *mut c_void)> {
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
            let gtk_icon_theme_get_for_display: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
                Self::load_fn(gtk_lib, c"gtk_icon_theme_get_for_display")?;

            let display = gdk_display_get_default();
            let css_provider = gtk_css_provider_new();
            let icon_theme = if display.is_null() {
                std::ptr::null_mut()
            } else {
                if !css_provider.is_null() {
                    gtk_style_context_add_provider_for_display(
                        display,
                        css_provider,
                        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                }
                gtk_icon_theme_get_for_display(display)
            };
            Some((css_provider, icon_theme))
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
}
