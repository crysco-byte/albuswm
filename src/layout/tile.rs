use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowGeometry, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct TileLayout {
    name: String,
}

impl Layout for TileLayout {
    fn name(&self) -> &str {
        &self.name
    }
    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.is_empty() {
            return;
        }
        let focused_id = stack.focused().unwrap();
        let mut accumulator : u32 = 0;
        if stack.len() < 2 {
            // Maybe this panics when changing groups because the window is not focused
            Self::configure_single_window(connection, viewport, focused_id);
        } else {
            Self::configure_focused_window(connection, viewport, focused_id);
            for window_id in stack.iter() {
                if window_id != focused_id {
                    Self::configure_unfocused_window(accumulator, connection, stack, viewport, window_id);
                    accumulator += 1;
                }
            }
        }
    }
}

impl TileLayout {
    pub fn new<S: Into<String>>(name: S) -> TileLayout {
        Self {
            name: name.into(),
        }
    }

    fn configure_unfocused_window(
        i: u32,
        connection: &Connection,
        stack: &Stack<WindowId>,
        viewport: &Viewport,
        window_id: &WindowId,
    ) {
        let window_height = viewport.height / (stack.len() - 1) as u32;
        let unfocused_geometry = WindowGeometry {
            x: viewport.width / 2,
            y: i as u32 * window_height,
            width: viewport.width / 2,
            height: window_height,
        };
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &unfocused_geometry);
        connection.enable_window_tracking(window_id);
    }

    fn configure_focused_window(
        connection: &Connection,
        viewport: &Viewport,
        window_id: &WindowId,
    ) {
        let focused_geometry = WindowGeometry {
            x: viewport.x,
            y: viewport.y,
            width: viewport.width / 2,
            height: viewport.height,
        };
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &focused_geometry);
        connection.enable_window_tracking(window_id);
    }

    fn configure_single_window(connection: &Connection, viewport: &Viewport, window_id: &WindowId) {
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &WindowGeometry::default(viewport));
        connection.enable_window_tracking(window_id);
    }
}
