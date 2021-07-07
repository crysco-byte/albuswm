use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowGeometry, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct TileLayout {
    name: String,
    resized_width: i16,
}

impl Layout for TileLayout {
    fn name(&self) -> &str {
        &self.name
    }
    fn layout(
        &self,
        connection: &Connection,
        viewport: &Viewport,
        stack: &Stack<WindowId>,
        master: &Option<WindowId>,
    ) {
        if stack.is_empty() {
            return;
        }
        let master_id = if master.is_none() {
            stack.focused().unwrap()
        } else {
            master.as_ref().unwrap()
        };
        if stack.len() < 2 {
            Self::configure_single_window(connection, viewport, master_id);
        } else {
            self.tile(connection, viewport, stack, master_id);
        }
    }

    fn resize_left(&mut self, viewport: &Viewport, resize_amount: i16) {
        self.resized_width -=
            if self.resized_width > -((viewport.width / 2) as i16 - (viewport.width / 8) as i16) {
                resize_amount
            } else {
                return;
            };
    }

    fn resize_right(&mut self, viewport: &Viewport, resize_amount: i16) {
        self.resized_width +=
            if self.resized_width < ((viewport.width / 2) as i16 - (viewport.width / 8) as i16) {
                resize_amount
            } else {
                return;
            };
    }
}

impl TileLayout {
    pub fn new<S: Into<String>>(name: S) -> TileLayout {
        Self {
            name: name.into(),
            resized_width: 0,
        }
    }

    fn tile(
        &self,
        connection: &Connection,
        viewport: &Viewport,
        stack: &Stack<WindowId>,
        focused_id: &WindowId,
    ) {
        self.configure_focused_window(connection, viewport, focused_id);
        let mut accumulator = 0;
        for window_id in stack.iter() {
            if window_id != focused_id {
                self.configure_unfocused_window(
                    accumulator,
                    connection,
                    stack,
                    viewport,
                    window_id,
                );
                accumulator += 1;
            }
        }
    }

    fn configure_unfocused_window(
        &self,
        i: u32,
        connection: &Connection,
        stack: &Stack<WindowId>,
        viewport: &Viewport,
        window_id: &WindowId,
    ) {
        let unfocused_geometry = self.get_unfocused_geometry(i, stack, viewport);
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &unfocused_geometry);
        connection.enable_window_tracking(window_id);
    }

    fn configure_focused_window(
        &self,
        connection: &Connection,
        viewport: &Viewport,
        window_id: &WindowId,
    ) {
        let focused_geometry = self.get_focused_geometry(viewport);
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &focused_geometry);
        connection.enable_window_tracking(window_id);
    }

    fn get_unfocused_geometry(
        &self,
        i: u32,
        stack: &Stack<WindowId>,
        viewport: &Viewport,
    ) -> WindowGeometry {
        let x = ((viewport.width / 2) as i16 + self.resized_width) as u32;
        let width = ((viewport.width / 2) as i16 - (self.resized_width)) as u32;
        let height = viewport.height / (stack.len() - 1) as u32;
        WindowGeometry {
            x,
            y: i as u32 * height,
            width,
            height,
        }
    }

    fn get_focused_geometry(&self, viewport: &Viewport) -> WindowGeometry {
        let width = (((viewport.width / 2) as i16) + (self.resized_width)) as u32;
        WindowGeometry {
            x: viewport.x,
            y: viewport.y,
            width,
            height: viewport.height,
        }
    }

    fn configure_single_window(connection: &Connection, viewport: &Viewport, window_id: &WindowId) {
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &WindowGeometry::default(viewport));
        connection.enable_window_tracking(window_id);
    }
}
