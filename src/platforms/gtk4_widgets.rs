#![allow(unsafe_code)]

use super::gtk4_sys::{
    GTK_ORIENTATION_HORIZONTAL, GTK_ORIENTATION_VERTICAL, Gtk4Sys, GtkClickCallback, GtkHandle,
};
use std::ffi::{CString, c_int, c_void};

impl Gtk4Sys {
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

    pub fn configure_window_icon(&self, win: GtkHandle, app_id: &str, search_dir: &str) {
        if let Ok(c_dir) = CString::new(search_dir)
            && !self.icon_theme.is_null()
        {
            unsafe {
                (self.gtk_icon_theme_add_search_path)(self.icon_theme, c_dir.as_ptr());
            }
        }
        if let Ok(c_app) = CString::new(app_id) {
            unsafe {
                (self.g_set_prgname)(c_app.as_ptr());
                (self.g_set_application_name)(c_app.as_ptr());
                (self.gtk_window_set_default_icon_name)(c_app.as_ptr());
                (self.gtk_window_set_icon_name)(win.0, c_app.as_ptr());
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

    pub fn create_header_bar(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_header_bar_new)()) }
    }

    pub fn create_scrolled_window(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_scrolled_window_new)()) }
    }

    pub fn create_box(&self, spacing: i32) -> GtkHandle {
        unsafe {
            let bx = (self.gtk_box_new)(GTK_ORIENTATION_VERTICAL, spacing);
            self.set_uniform_margin(GtkHandle(bx), 20);
            GtkHandle(bx)
        }
    }

    pub fn create_grid(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_grid_new)()) }
    }

    pub fn create_separator(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_separator_new)(GTK_ORIENTATION_HORIZONTAL)) }
    }

    pub fn create_label(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_label_new)(c"".as_ptr())) }
    }

    pub fn create_button(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_button_new_with_label)(c"".as_ptr())) }
    }

    pub fn create_entry(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_entry_new)()) }
    }

    pub fn create_switch(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_switch_new)()) }
    }

    pub fn create_check_button(&self) -> GtkHandle {
        unsafe { GtkHandle((self.gtk_check_button_new_with_label)(c"".as_ptr())) }
    }

    pub fn create_slider(&self) -> GtkHandle {
        unsafe {
            GtkHandle((self.gtk_scale_new_with_range)(
                GTK_ORIENTATION_HORIZONTAL,
                0.0,
                100.0,
                1.0,
            ))
        }
    }

    pub fn create_drop_down(&self) -> GtkHandle {
        let null_list: [*const std::ffi::c_char; 1] = [std::ptr::null()];
        unsafe { GtkHandle((self.gtk_drop_down_new_from_strings)(null_list.as_ptr())) }
    }

    pub fn set_window_title(&self, win: GtkHandle, title: &str) {
        if let Ok(c_str) = CString::new(title) {
            unsafe {
                (self.gtk_window_set_title)(win.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_window_titlebar(&self, win: GtkHandle, header_bar: GtkHandle) {
        unsafe {
            (self.gtk_window_set_titlebar)(win.0, header_bar.0);
        }
    }

    pub fn set_header_bar_show_title_buttons(&self, hb: GtkHandle, show: bool) {
        unsafe {
            (self.gtk_header_bar_set_show_title_buttons)(hb.0, i32::from(show));
        }
    }

    pub fn header_bar_pack_end(&self, hb: GtkHandle, child: GtkHandle) {
        unsafe {
            (self.gtk_header_bar_pack_end)(hb.0, child.0);
        }
    }

    pub fn set_scrolled_child(&self, scroll: GtkHandle, child: GtkHandle) {
        unsafe {
            (self.gtk_scrolled_window_set_child)(scroll.0, child.0);
        }
    }

    pub fn set_scrolled_min_height(&self, scroll: GtkHandle, min_height: i32) {
        unsafe {
            (self.gtk_scrolled_window_set_min_content_height)(scroll.0, min_height);
        }
    }

    pub fn set_orientation(&self, widget: GtkHandle, vertical: bool) {
        let orient = if vertical {
            GTK_ORIENTATION_VERTICAL
        } else {
            GTK_ORIENTATION_HORIZONTAL
        };
        unsafe {
            (self.gtk_orientable_set_orientation)(widget.0, orient);
        }
    }

    pub fn set_box_spacing(&self, bx: GtkHandle, spacing: i32) {
        unsafe {
            (self.gtk_box_set_spacing)(bx.0, spacing);
        }
    }

    pub fn set_grid_row_spacing(&self, grid: GtkHandle, row_spacing: u32) {
        unsafe {
            (self.gtk_grid_set_row_spacing)(grid.0, row_spacing);
        }
    }

    pub fn set_grid_column_spacing(&self, grid: GtkHandle, col_spacing: u32) {
        unsafe {
            (self.gtk_grid_set_column_spacing)(grid.0, col_spacing);
        }
    }

    pub fn grid_attach(&self, grid: GtkHandle, child: GtkHandle, col: i32, row: i32) {
        unsafe {
            (self.gtk_grid_attach)(grid.0, child.0, col, row, 1, 1);
        }
    }

    pub fn set_hexpand(&self, widget: GtkHandle, expand: bool) {
        unsafe {
            (self.gtk_widget_set_hexpand)(widget.0, i32::from(expand));
        }
    }

    pub fn set_vexpand(&self, widget: GtkHandle, expand: bool) {
        unsafe {
            (self.gtk_widget_set_vexpand)(widget.0, i32::from(expand));
        }
    }

    fn parse_gtk_align(align: &str) -> c_int {
        match align.trim() {
            "start" => 1,
            "end" => 2,
            "center" => 3,
            "baseline" => 4,
            _ => 0,
        }
    }

    pub fn set_halign(&self, widget: GtkHandle, align: &str) {
        unsafe {
            (self.gtk_widget_set_halign)(widget.0, Self::parse_gtk_align(align));
        }
    }

    pub fn set_valign(&self, widget: GtkHandle, align: &str) {
        unsafe {
            (self.gtk_widget_set_valign)(widget.0, Self::parse_gtk_align(align));
        }
    }

    pub fn set_uniform_margin(&self, widget: GtkHandle, margin: i32) {
        unsafe {
            (self.gtk_widget_set_margin_top)(widget.0, margin);
            (self.gtk_widget_set_margin_bottom)(widget.0, margin);
            (self.gtk_widget_set_margin_start)(widget.0, margin);
            (self.gtk_widget_set_margin_end)(widget.0, margin);
        }
    }

    pub fn set_margin_side(&self, widget: GtkHandle, side: &str, margin: i32) {
        unsafe {
            match side {
                "margin-top" => (self.gtk_widget_set_margin_top)(widget.0, margin),
                "margin-bottom" => (self.gtk_widget_set_margin_bottom)(widget.0, margin),
                "margin-start" => (self.gtk_widget_set_margin_start)(widget.0, margin),
                "margin-end" => (self.gtk_widget_set_margin_end)(widget.0, margin),
                _ => {}
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

    pub fn set_entry_text(&self, entry: GtkHandle, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            unsafe {
                (self.gtk_editable_set_text)(entry.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_entry_placeholder(&self, entry: GtkHandle, placeholder: &str) {
        if let Ok(c_str) = CString::new(placeholder) {
            unsafe {
                (self.gtk_entry_set_placeholder_text)(entry.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_switch_active(&self, sw: GtkHandle, active: bool) {
        unsafe {
            (self.gtk_switch_set_active)(sw.0, i32::from(active));
        }
    }

    pub fn set_check_button_label(&self, chk: GtkHandle, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            unsafe {
                (self.gtk_check_button_set_label)(chk.0, c_str.as_ptr());
            }
        }
    }

    pub fn set_check_button_active(&self, chk: GtkHandle, active: bool) {
        unsafe {
            (self.gtk_check_button_set_active)(chk.0, i32::from(active));
        }
    }

    pub fn set_slider_value(&self, scale: GtkHandle, val: f64) {
        unsafe {
            (self.gtk_range_set_value)(scale.0, val);
        }
    }

    pub fn set_drop_down_selected(&self, dd: GtkHandle, idx: u32) {
        unsafe {
            (self.gtk_drop_down_set_selected)(dd.0, idx);
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

    pub fn box_insert_after(&self, bx: GtkHandle, child: GtkHandle, sibling: Option<GtkHandle>) {
        let sib_ptr = sibling.map_or(std::ptr::null_mut(), |h| h.0);
        unsafe {
            (self.gtk_box_insert_child_after)(bx.0, child.0, sib_ptr);
        }
    }

    pub fn box_remove(&self, bx: GtkHandle, child: GtkHandle) {
        unsafe {
            (self.gtk_box_remove)(bx.0, child.0);
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
