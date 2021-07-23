use std::rc::Rc;

use super::Viewport;
use crate::layout::Layout;
use crate::stack::Stack;
use crate::x::{Connection, WindowId};

#[derive(Clone)]
pub struct WorkSpaceBuilder {
    name: String,
    default_layout: String,
}

impl WorkSpaceBuilder {
    pub fn new<S1, S2>(name: S1, default_layout: S2) -> WorkSpaceBuilder
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        WorkSpaceBuilder {
            name: name.into(),
            default_layout: default_layout.into(),
        }
    }

    pub fn build(self, connection: Rc<Connection>, layouts: Vec<Box<dyn Layout>>) -> WorkSpace {
        let mut layouts_stack: Stack<Box<dyn super::layout::Layout>> = Stack::from(layouts);
        layouts_stack.focus(|layout| layout.name() == self.default_layout);

        WorkSpace {
            connection,
            name: self.name.clone(),
            active: false,
            stack: Stack::new(),
            layouts: layouts_stack,
            viewport: Viewport::default(),
            master: None,
        }
    }
}

pub struct WorkSpace {
    name: String,
    connection: Rc<Connection>,
    active: bool,
    stack: Stack<WindowId>,
    layouts: Stack<Box<dyn Layout>>,
    viewport: Viewport,
    master: Option<WindowId>,
}

impl WorkSpace {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn activate(&mut self, viewport: Viewport) {
        info!("Activating workspace: {}", self.name());
        self.active = true;
        self.viewport = viewport;
        self.perform_layout();
    }

    pub fn decrease_innergaps(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.decrease_innergaps();
        }
        self.perform_layout();
    }

    pub fn increase_innergaps(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.increase_innergaps();
        }
        self.perform_layout();
    }

    pub fn decrease_outergaps(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.decrease_outergaps();
        }
        self.perform_layout();
    }

    pub fn increase_outergaps(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.increase_outergaps();
        }
        self.perform_layout();
    }

    pub fn increase_master(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.increase_master(&self.viewport, 160);
        }
        self.perform_layout();
    }

    pub fn decrease_master(&mut self) {
        if let Some(layout) = self.layouts.focused_mut() {
            layout.decrease_master(&self.viewport, 160);
        }
        self.perform_layout();
    }

    pub fn update_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.perform_layout();
    }

    pub fn deactivate(&mut self) {
        info!("Deactivating workspace: {}", self.name());
        for window_id in self.stack.iter() {
            self.connection.disable_window_tracking(window_id);
            self.connection.unmap_window(window_id);
            self.connection.enable_window_tracking(window_id);
        }
        self.active = false;
    }

    fn change_master(&mut self) {
        if !self.stack.is_empty() {
            self.master = Some(*self.stack.focused().unwrap());
        }
    }

    fn perform_layout(&mut self) {
        if !self.active {
            return;
        }

        if let Some(layout) = self.layouts.focused() {
            layout.layout(&self.connection, &self.viewport, &self.stack, &self.master)
        }

        // Tell X to focus the focused window for this workspace, or to unset
        // it's focus if we have no windows.
        match self.stack.focused() {
            Some(window_id) => self.connection.focus_window(window_id),
            None => self.connection.focus_nothing(),
        }
    }

    pub fn add_window(&mut self, window_id: WindowId) {
        info!("Adding window to workspace {}: {}", self.name(), window_id);
        self.stack.push(window_id);
        self.master = Some(window_id);
        self.perform_layout();
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> WindowId {
        info!(
            "Removing window from workspace {}: {}",
            self.name(),
            window_id
        );
        let removed: WindowId = self.stack.remove(|w| w == window_id);
        self.change_master();
        self.perform_layout();
        removed
    }

    pub fn remove_focused(&mut self) -> Option<WindowId> {
        info!(
            "Removing focused window from workspace {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        let removed: Option<WindowId> = self.stack.remove_focused();
        self.change_master();
        self.perform_layout();
        removed.map(|window| {
            self.connection.disable_window_tracking(&window);
            self.connection.unmap_window(&window);
            self.connection.enable_window_tracking(&window);
            window
        })
    }

    pub fn contains(&self, window_id: &WindowId) -> bool {
        self.stack.iter().any(|w| w == window_id)
    }

    pub fn focus(&mut self, window_id: &WindowId) {
        info!(
            "Focusing window in workspace {}: {}",
            self.name(),
            window_id
        );
        self.stack.focus(|id| id == window_id);
        self.perform_layout();
    }

    pub fn close_focused(&mut self) {
        if let Some(window_id) = self.stack.focused() {
            self.connection.close_window(window_id);
            self.change_master();
        }
    }

    pub fn focus_next(&mut self) {
        self.stack.focus_next();
        info!(
            "Focusing next window in workspace {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.change_master();
        self.perform_layout();
    }

    pub fn focus_previous(&mut self) {
        self.stack.focus_previous();
        self.change_master();
        info!(
            "Focusing previous window in workspace {}: {:?}",
            self.name(),
            self.stack.focused()
        );
        self.perform_layout();
    }

    pub fn layout_next(&mut self) {
        self.layouts.focus_next();
        info!(
            "Switching to next layout in workspace {}: {:?}",
            self.name(),
            self.layouts.focused()
        );
        self.perform_layout();
    }

    pub fn layout_previous(&mut self) {
        self.layouts.focus_next();
        info!(
            "Switching to previous layout in workspace {}: {:?}",
            self.name(),
            self.layouts.focused()
        );
        self.layouts.focus_previous();
        self.perform_layout();
    }
}
