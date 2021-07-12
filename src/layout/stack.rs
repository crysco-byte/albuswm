use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowGeometry, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct StackLayout {
    name: String,
}

impl StackLayout {
    pub fn new<S: Into<String>>(name: S) -> StackLayout {
        StackLayout { name: name.into() }
    }
}

impl Layout for StackLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(
        &self,
        connection: &Connection,
        viewport: &Viewport,
        stack: &Stack<WindowId>,
        _master: &Option<WindowId>,
    ) {
        if stack.is_empty() {
            return;
        }

        // A non-empty `Stack` is guaranteed to have something focused.
        let focused_id = stack.focused().unwrap();
        let geometry = WindowGeometry::default(viewport);

        for window_id in stack.iter() {
            if focused_id == window_id {
                continue;
            }
            connection.disable_window_tracking(window_id);
            connection.unmap_window(window_id);
            connection.enable_window_tracking(window_id);
        }
        connection.configure_window(focused_id, &geometry);
    }

    fn resize_right(&mut self, _viewport: &Viewport, _resize_amount: i16) {
        return;
    }

    fn resize_left(&mut self, _viewport: &Viewport, _resize_amount: i16) {
        return;
    }
}
