#![deny(warnings)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate strum_macros;

pub mod cmd;
pub mod config;
mod groups;
mod keys;
pub mod layout;
pub mod screen;
mod stack;
mod x;
use cmd::Command;

pub use crate::{groups::GroupBuilder, keys::ModKey, screen::Screen, stack::Stack};
use {
    crate::x::{Connection, StrutPartial, WindowId},
    failure::{Error, ResultExt},
};

use {
    crate::{
        groups::Group,
        keys::{KeyCombo, KeyHandlers},
        layout::Layout,
        x::{Event, WindowType},
    },
    std::rc::Rc,
};

pub type Result<T> = std::result::Result<T, Error>;

pub mod keysym {
    pub use x11::keysym::*;
}

/// Initializes a logger using the default configuration.
pub fn intiailize_logger() -> Result<()> {
    log_panics::init();

    let xdg_dirs = xdg::BaseDirectories::with_prefix("albus")?;
    let log_path = xdg_dirs
        .place_data_file("albus.log")
        .context("Could not create log file")?;

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                time::now().rfc3339(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::log_file(&log_path)?)
        .apply()?;

    Ok(())
}

pub fn gen_groups(
    keys: Vec<(Vec<ModKey>, u32, Command)>,
    groupdef: Vec<(ModKey, u32, String, String)>,
) -> (Vec<(Vec<ModKey>, u32, Command)>, Vec<GroupBuilder>) {
    let mut additional_keys = Vec::new();
    let mut groups = Vec::new();
    for item in groupdef {
        let (mask, key, group_name, layout_name) = (item.0, item.1, item.2.clone(), item.3);
        additional_keys.push(gen_move_window_to_group_keys!(mask, key, group_name));
        additional_keys.push(gen_switch_group_keys!(mask, key, group_name));
        groups.push(GroupBuilder::new(item.2, layout_name))
    }
    additional_keys.extend(keys);
    (additional_keys, groups)
}
#[macro_export]
macro_rules! gen_move_window_to_group_keys {
    {
        $mask:ident,
        $xk_key:ident,
        $group_name:ident
    } => {
        (vec![$mask, ModKey::Shift], $xk_key, $crate::cmd::lazy::move_window_to_group($group_name.clone()))
    }
}
#[macro_export]
macro_rules! gen_switch_group_keys {
    {
        $mask:ident,
        $xk_key:ident,
        $group_name:ident
    } => {
        (vec![$mask], $xk_key, $crate::cmd::lazy::switch_group($group_name))
    }
}

#[macro_export]
macro_rules! layouts {
    [$( $layout:expr ),+ $(,)*] => (
        vec![
            $(
                Box::new($layout) as Box<$crate::layout::Layout>
            ),+
        ]
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

struct Dock {
    window_id: WindowId,
    strut_partial: Option<StrutPartial>,
}

pub struct Albus {
    connection: Rc<Connection>,
    keys: KeyHandlers,
    groups: Stack<Group>,
    screen: Screen,
}

impl Albus {
    pub fn new<K>(keys: K, groups: Vec<GroupBuilder>, layouts: &[Box<dyn Layout>]) -> Result<Self>
    where
        K: Into<KeyHandlers>,
    {
        let keys = keys.into();
        let connection = Rc::new(Connection::connect()?);
        connection.install_as_wm(&keys)?;

        let groups = Stack::from(
            groups
                .into_iter()
                .map(|group: GroupBuilder| group.build(connection.clone(), layouts.to_owned()))
                .collect::<Vec<Group>>(),
        );

        let mut wm = Albus {
            keys,
            groups,
            connection: connection.clone(),
            screen: Screen::default(),
        };

        // Learn about existing top-level windows.
        let existing_windows = connection.top_level_windows()?;
        for window in existing_windows {
            wm.manage_window(window);
        }
        let viewport = wm.viewport();
        wm.group_mut().activate(viewport);
        wm.connection.update_ewmh_desktops(&wm.groups);

        Ok(wm)
    }

    fn viewport(&self) -> Viewport {
        let (width, height) = self
            .connection
            .get_window_geometry(self.connection.root_window_id());
        self.screen.viewport(width, height)
    }
    pub fn group(&self) -> &Group {
        self.groups.focused().expect("Invariant: No active group!")
    }

    pub fn group_mut(&mut self) -> &mut Group {
        self.groups
            .focused_mut()
            .expect("Invariant: No active group!")
    }

    pub fn switch_group<'a>(&'a mut self, name: String) {
        // If we're already on this group, do nothing.
        if self.group().name() == name {
            return;
        }

        self.group_mut().deactivate();
        self.groups.focus(|group| group.name() == name);
        let viewport = self.viewport();
        self.group_mut().activate(viewport);
        self.connection.update_ewmh_desktops(&self.groups);
    }

    /// Move the focused window from the active group to another named group.
    ///
    /// If the other named group does not exist, then the window is
    /// (unfortunately) lost.
    pub fn move_focused_to_group<'a>(&'a mut self, name: String) {
        // If the group is currently active, then do nothing. This avoids flicker as we
        // unmap/remap.
        if name == self.group().name() {
            return;
        }
        if let Some(removed) = self.group_mut().remove_focused() {
            let new_group = self.groups.iter_mut().find(|group| group.name() == name);
            match new_group {
                Some(new_group) => {
                    new_group.add_window(removed);
                }
                None => {
                    // It would be nice to put the window back in its group (or avoid taking it out
                    // of its group until we've checked the new group exists), but it's difficult
                    // to do this while keeping the borrow checker happy.
                    error!("Moved window to non-existent group: {}", name);
                }
            }
        }
    }

    /// Returns whether the window is a member of any group.
    fn is_window_managed(&self, window_id: &WindowId) -> bool {
        self.groups.iter().any(|g| g.contains(window_id))
    }

    pub fn manage_window(&mut self, window_id: WindowId) {
        debug!("Managing window: {}", window_id);

        // If we are already managing the window, then do nothing. We do not
        // want the window to end up in two groups at once. We shouldn't
        // be called in such cases, so treat it as an error.
        if self.is_window_managed(&window_id) {
            error!(
                "Asked to manage window that's already managed: {}",
                window_id
            );
            return;
        }

        let window_types = self.connection.get_window_types(&window_id);
        let dock = window_types.contains(&WindowType::Dock);

        self.connection
            .enable_window_key_events(&window_id, &self.keys);

        if dock {
            self.connection.map_window(&window_id);
            self.screen.add_dock(&self.connection, window_id);
            let viewport = self.viewport();
            self.group_mut().update_viewport(viewport);
        } else {
            self.connection.enable_window_tracking(&window_id);
            self.group_mut().add_window(window_id);
        }
    }

    pub fn unmanage_window(&mut self, window_id: &WindowId) {
        debug!("Unmanaging window: {}", window_id);

        // Remove the window from whichever Group it is in. Special case for
        // docks which aren't in any group.
        self.groups
            .iter_mut()
            .find(|group| group.contains(window_id))
            .map(|group| group.remove_window(window_id));
        self.screen.remove_dock(window_id);

        // The viewport may have changed.
        let viewport = self.viewport();
        self.group_mut().update_viewport(viewport);
    }

    pub fn run(mut self) {
        info!("Started WM, entering event loop.");
        let event_loop_connection = self.connection.clone();
        let event_loop = event_loop_connection.get_event_loop();
        for event in event_loop {
            match event {
                Event::MapRequest(window_id) => self.on_map_request(window_id),
                Event::UnmapNotify(window_id) => self.on_unmap_notify(&window_id),
                Event::DestroyNotify(window_id) => self.on_destroy_notify(&window_id),
                Event::KeyPress(key) => self.on_key_press(key),
                Event::EnterNotify(window_id) => self.on_enter_notify(&window_id),
            }
        }
        info!("Event loop exiting");
    }

    fn on_map_request(&mut self, window_id: WindowId) {
        if !self.is_window_managed(&window_id) {
            // If the window isn't in any group, then add it to the current group.
            // (This will have the side-effect of mapping the window, as new windows are focused
            // and focused windows are mapped).
            self.manage_window(window_id);
        } else if self.group().contains(&window_id) {
            // Otherwise, if the window is in the active group, focus it. The application probably
            // wants us to make it prominent. Log as there may be misbehaving applications that
            // constantly re-map windows and cause focus issues.
            info!(
                "Window {} asked to be mapped but is already mapped: focusing.",
                window_id
            );
            self.group_mut().focus(&window_id);
        }
    }

    fn on_unmap_notify(&mut self, window_id: &WindowId) {
        // We only receive an unmap notify event when the window is actually
        // unmapped by its application. When our layouts unmap windows, they
        // (should) do it by disabling event tracking first.
        self.unmanage_window(window_id);
    }

    fn on_destroy_notify(&mut self, window_id: &WindowId) {
        self.unmanage_window(window_id);
    }

    fn on_key_press(&mut self, key: KeyCombo) {
        if let Some(handler) = self.keys.get(&key) {
            if let Err(error) = (handler)(self) {
                error!("Error running command for key command {:?}: {}", key, error);
            }
        }
    }

    fn on_enter_notify(&mut self, window_id: &WindowId) {
        self.group_mut().focus(window_id);
    }
}
