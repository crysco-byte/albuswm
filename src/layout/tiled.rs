use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{ConfigureWindowDimensions, Connection, WindowId};
use crate::Viewport;

#[derive(Clone)]
pub struct TiledLayout {
    name: String,
    padding: u32,
}

impl TiledLayout {
    pub fn new<S: Into<String>>(name: S, padding: u32) -> TiledLayout {
        TiledLayout {
            name: name.into(),
            padding,
        }
    }
}

impl Layout for TiledLayout {
    fn name(&self) -> &str {
        &self.name
    }

    fn layout(&self, connection: &Connection, viewport: &Viewport, stack: &Stack<WindowId>) {
        if stack.is_empty() {
            return;
        }

        let tile_height = ((viewport.height - self.padding) / stack.len() as u32) - self.padding;

        for (i, window_id) in stack.iter().enumerate() {
            let dimensions = ConfigureWindowDimensions {
                x: viewport.x + self.padding,
                y: viewport.y + self.padding + (i as u32 * (tile_height + self.padding)),
                width: viewport.width - (self.padding * 22),
                height: tile_height,
            };
            connection.disable_window_tracking(window_id);
            connection.map_window(window_id);
            connection.configure_window(window_id, &dimensions);
            connection.enable_window_tracking(window_id);
        }
    }
}
