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

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack:&Stack<WindowId>){
        if stack.is_empty() {return};
        let focused_window_id = stack.focused().unwrap();
        let _: () = stack.iter().enumerate()
            .map(|(i, window_id)| {
                if window_id != focused_window_id {
                    connection.disable_window_tracking(window_id);
                    connection.configure_window(
                        window_id,
                        viewport.x + (if i < 1 {0} else {viewport.x / (i as u32)}),
                        viewport.y,
                        viewport.width / (i + 1) as u32,
                        viewport.height,
                    )
                }
            }).collect();
        connection.disable_window_tracking(focused_window_id);
        connection.configure_window(
            focused_window_id,
            viewport.x + 150,
            viewport.y + 50,
            viewport.width - 300,
            viewport.height - 100
        );
        connection.stack_window_above(focused_window_id);
        connection.enable_window_tracking(focused_window_id);
    }
}

// impl Layout for FloatingMasterLayout {
//     fn name(&self) -> &str {
//         &self.name
//     }

//     fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
//         if stack.is_empty() {
//             return;
//         }
//         let focused_window = stack.focused().unwrap();
//         let _: () = stack
//             .iter()
//             .map(|window_id| {
//                 if window_id != focused_window {
//                     Self::configure_normal(window_id, connection, viewport, stack.len() as u32);
//                 }
//             }).collect();
//         Self::configure_master(focused_window, connection, viewport);
//     }
// }

// impl FloatingMasterLayout {
//     fn configure_master(window_id: &WindowId, connection: &Connection, viewport: &Viewport) {
//         connection.disable_window_tracking(window_id);
//         connection.configure_window(
//             window_id,
//             viewport.x + 150,
//             viewport.y + 50,
//             viewport.width - 300,
//             viewport.height - 100,
//         );
//         connection.stack_window_above(window_id);
//         connection.enable_window_tracking(window_id);
//     }

//     fn configure_normal(window_id: &WindowId, connection: &Connection, viewport: &Viewport, stack_length: u32) {
//         connection.disable_window_tracking(window_id);
//         connection.configure_window(
//             window_id,
//             viewport.x + (viewport.x / (stack_length - 1)),
//             viewport.y,
//             viewport.width / stack_length,
//             viewport.height,
//         );
//         connection.enable_window_tracking(window_id);
//     }
// }
