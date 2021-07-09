use std::fmt;

use crate::stack::Stack;
use crate::x::{Connection, WindowId, WindowGeometry};
use crate::Viewport;

mod stack;
mod tile;

pub use self::stack::StackLayout;
pub use self::tile::TileLayout;

pub trait LayoutClone {
    fn clone_box(&self) -> Box<dyn Layout>;
}

impl<T> LayoutClone for T
where
    T: 'static + Layout + Clone,
{
    fn clone_box(&self) -> Box<dyn Layout> {
        Box::new(self.clone())
    }
}

pub trait Layout: LayoutClone {
    fn name(&self) -> &str;
    fn layout(
        &self,
        connection: &Connection,
        viewport: &Viewport,
        stack: &Stack<WindowId>,
        master: &Option<WindowId>,
    );
    fn resize_right(&mut self, viewport: &Viewport, resize_amount: i16);
    fn resize_left(&mut self, viewport: &Viewport, resize_amount: i16);
}

impl Clone for Box<dyn Layout> {
    fn clone(&self) -> Box<dyn Layout> {
        self.clone_box()
    }
}

impl fmt::Debug for dyn Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Layout {{ \"{}\" }}", self.name())
    }
}


fn configure_single_window(connection: &Connection, viewport: &Viewport, window_id: &WindowId) {
    connection.disable_window_tracking(window_id);
    connection.map_window(window_id);
    connection.configure_window(window_id, &WindowGeometry::default(viewport));
    connection.enable_window_tracking(window_id);
}
