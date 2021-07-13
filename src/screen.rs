use {
    super::{Connection, Dock, Viewport, WindowId},
    std::{cell::RefCell, cmp},
};

#[derive(Default)]
pub struct Screen {
    vec: RefCell<Vec<Dock>>,
}

impl Screen {
    pub fn add_dock(&mut self, conn: &Connection, window_id: WindowId) {
        let strut_partial = conn.get_strut_partial(&window_id);
        self.vec.borrow_mut().push(Dock {
            window_id,
            strut_partial,
        });
    }

    pub fn remove_dock(&mut self, window_id: &WindowId) {
        self.vec.borrow_mut().retain(|d| &d.window_id != window_id);
    }

    /// Figure out the usable area of the screen based on the STRUT_PARTIAL of
    /// all docks.
    pub fn viewport(&self, screen_width: u32, screen_height: u32) -> Viewport {
        let (left, right, top, bottom): (u32, u32, u32, u32) = self
            .vec
            .borrow()
            .iter()
            .filter_map(|o| o.strut_partial.as_ref())
            .fold((0, 0, 0, 0), |(left, right, top, bottom), s| {
                // We don't bother looking at the start/end members of the
                // StrutPartial - treating it more like a Strut.
                (
                    cmp::max(left, s.left()),
                    cmp::max(right, s.right()),
                    cmp::max(top, s.top()),
                    cmp::max(bottom, s.bottom()),
                )
            });
        let viewport = Viewport {
            x: left,
            y: top,
            width: screen_width - left - right,
            height: screen_height - top - bottom,
        };
        debug!("Calculated Viewport as {:?}", viewport);
        viewport
    }
}
