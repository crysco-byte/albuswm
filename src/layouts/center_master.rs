use crate::Viewport;
use crate::window_stack::Stack;

pub struct CenterMaster {
    gapsoh:u32,
    gapsov:u32
}

// fix later
impl CenterMaster {
    pub fn layout (&self, &Connection, viewport: &Viewport, stack: &Stac) {
        if stack.is_empty() {return}
        let focused_id = stack.focused().unwrap();

        stack.iter().map(|window_id| {
            if focused_id == window_id {continue};
            connection.disable_window_tracking(window_id);
            connection.unmap_window(window_id);
            connection.enable_window_tracking(window_id);
        })
        connection.disable_window_tracking(focuse_id);
        connection.map_window(focused_id);
        connection.configure_window (
            focused_id,
            viewport.x + self.gapsoh,
            viewport.y + self.gapsov,
            viewport.width - (self.gapsoh *2),
            viewport.height - (self.gapsov *2)
        );
        connection.enable_window_tracking(focused_id);
    }
}
