use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowId};
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
            connection.disable_window_tracking(master_window_id);
            connection.map_window(master_window_id);
            connection.configure_window(
                master_window_id,
                viewport.x,
                viewport.y,
                viewport.width,
                viewport.height
            );
            connection.enable_window_tracking(master_window_id);
        }else {
            Self::configure_master_window(master_window_id, connection, viewport);
            for window_id in stack.iter() {
                if window_id != master_window_id {
                    Self::configure_normal_window(
                        normal_window_acc,
                        window_id,
                        connection,
                        viewport
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
        connection.disable_window_tracking(window_id);
        connection.map_window(window_id);
        connection.configure_window(
            window_id,
            viewport.x + (if inter_num > 1 {viewport.width / inter_num} else {0}),
            viewport.y,
            viewport.width / inter_num,
            viewport.height,
        );
        connection.enable_window_tracking(window_id);
    }

    fn configure_master_window(
        master_window_id: &WindowId,
        connection: &Connection,
        viewport: &Viewport,
    ) {
        connection.disable_window_tracking(master_window_id);
        connection.map_window(master_window_id);
        connection.configure_window(
            master_window_id,
            viewport.x + 150,
            viewport.y + 50,
            viewport.width - 300,
            viewport.height - 100,
        );
        connection.stack_window_above(master_window_id);
        connection.enable_window_tracking(master_window_id);
    }
}
