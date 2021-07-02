use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{WindowGeometry, Connection, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct FloatingMasterLayout {
    name: String,
}

impl FloatingMasterLayout {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl Layout for FloatingMasterLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.is_empty() {
            return;
        };
        let master_window_id = stack.get_first_element().unwrap();
        let mut normal_window_acc = 1;
        if stack.len() < 2 {
            Self::configure_single_window(master_window_id, connection, viewport);
        } else {
            Self::configure_master_window(master_window_id, connection, viewport);
            for window_id in stack.iter() {
                if window_id != master_window_id {
                    Self::configure_normal_window(
                        normal_window_acc,
                        window_id,
                        connection,
                        viewport,
                    );
                    normal_window_acc += 1;
                }
            }
        }
    }
}
impl FloatingMasterLayout {
    fn configure_normal_window(
        inter_num: u32,
        window_id: &WindowId,
        connection: &Connection,
        viewport: &Viewport,
    ) {
        let geometry = WindowGeometry {
            x: viewport.x
                + (if inter_num > 1 {
                    viewport.width / inter_num
                } else {
                    0
                }),
            y: viewport.y,
            width: viewport.width / inter_num,
            height: viewport.height,
        };
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &geometry);
        connection.enable_window_tracking(window_id);
    }

    fn configure_single_window (
        window_id: &WindowId,
        connection: &Connection,
        viewport: &Viewport,
    ) {
        let default_window_geometry = WindowGeometry::default(viewport);
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(window_id, &default_window_geometry);
        connection.enable_window_tracking(window_id);
    }

    fn configure_master_window(
        master_window_id: &WindowId,
        connection: &Connection,
        viewport: &Viewport,
    ) {
        let geometry = WindowGeometry {
            x: viewport.x + 150,
            y: viewport.y + 50,
            width: viewport.width - 300,
            height: viewport.height - 100,
        };
        connection.disable_window_tracking(master_window_id);
        connection.map_window(master_window_id);
        connection.configure_window(master_window_id, &geometry);
        connection.stack_window_above(master_window_id);
        connection.enable_window_tracking(master_window_id);
    }
}
