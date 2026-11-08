#![allow(unsafe_code)]

use crate::engine::{EventPayload, NativeEvent, NodeId};
use std::cell::RefCell;
use std::ffi::c_void;

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<NativeEvent>> = const { RefCell::new(Vec::new()) };
}

pub const fn pack_node_id(id: NodeId) -> usize {
    (id.index << 32) | (id.generation as usize)
}

#[allow(clippy::cast_possible_truncation)]
pub const fn unpack_node_id(packed: usize) -> NodeId {
    NodeId {
        index: packed >> 32,
        generation: (packed & 0xFFFF_FFFF) as u32,
    }
}

fn enqueue_event(packed: usize, event_name: &str) {
    let node = unpack_node_id(packed);
    PENDING_EVENTS.with(|queue| {
        queue.borrow_mut().push(NativeEvent {
            node,
            event_name: String::from(event_name),
            payload: EventPayload::None,
        });
    });
}

pub fn drain_pending_events() -> Vec<NativeEvent> {
    PENDING_EVENTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

pub(super) unsafe extern "C" fn on_gtk_button_clicked(_btn: *mut c_void, user_data: *mut c_void) {
    enqueue_event(user_data as usize, "clicked");
}

pub(super) unsafe extern "C" fn on_gtk_entry_changed(_entry: *mut c_void, user_data: *mut c_void) {
    enqueue_event(user_data as usize, "input");
}

pub(super) unsafe extern "C" fn on_gtk_check_toggled(_chk: *mut c_void, user_data: *mut c_void) {
    enqueue_event(user_data as usize, "toggled");
}

pub(super) unsafe extern "C" fn on_gtk_scale_value_changed(
    _scale: *mut c_void,
    user_data: *mut c_void,
) {
    enqueue_event(user_data as usize, "value-changed");
}

pub(super) unsafe extern "C" fn on_gtk_switch_active_notify(
    _sw: *mut c_void,
    _pspec: *mut c_void,
    user_data: *mut c_void,
) {
    enqueue_event(user_data as usize, "toggled");
}

pub(super) unsafe extern "C" fn on_gtk_drop_down_selected_notify(
    _dd: *mut c_void,
    _pspec: *mut c_void,
    user_data: *mut c_void,
) {
    enqueue_event(user_data as usize, "value-changed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_unpacks_and_drains_native_gtk_callback_events() {
        let _ = drain_pending_events();
        let original = NodeId {
            index: 42,
            generation: 7,
        };
        let packed = pack_node_id(original);
        assert_eq!(unpack_node_id(packed), original);

        unsafe {
            on_gtk_button_clicked(std::ptr::null_mut(), packed as *mut c_void);
            on_gtk_entry_changed(std::ptr::null_mut(), packed as *mut c_void);
            on_gtk_switch_active_notify(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                packed as *mut c_void,
            );
        }

        let events = drain_pending_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].node, original);
        assert_eq!(events[0].event_name, "clicked");
        assert_eq!(events[1].event_name, "input");
        assert_eq!(events[2].event_name, "toggled");
        assert_eq!(drain_pending_events(), Vec::<NativeEvent>::new());
    }
}
