use crate::Albus;
use crate::Result;
use std::rc::Rc;

pub type Command = Rc<dyn Fn(&mut Albus) -> Result<()>>;

/// Lazy-functions which return a `Command` to do the requested action.
pub mod lazy {

    use std::rc::Rc;
    use std::sync::Mutex;

    use failure::ResultExt;

    use super::Command;

    /// Closes the currently focused window.
    pub fn close_focused_window() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().close_focused();
            Ok(())
        })
    }

    /// Moves the focus to the next window in the current group's stack.
    pub fn focus_next() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().focus_next();
            Ok(())
        })
    }

    pub fn resize_right() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().resize_right();
            Ok(())
        })
    }

    pub fn resize_left() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().resize_left();
            Ok(())
        })
    }

    /// Moves the focus to the previous window in the current group's stack.
    pub fn focus_previous() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().focus_previous();
            Ok(())
        })
    }

    /// Cycles to the next layout of the current group.
    pub fn layout_next() -> Command {
        Rc::new(|ref mut wm| {
            wm.group_mut().layout_next();
            Ok(())
        })
    }

    /// Spawns the specified command.
    /// The returned `Command` will spawn the `Command` each time it is called.
    pub fn spawn(cmd: String, args: Vec<String>) -> Command {
        let mut command = std::process::Command::new(cmd.clone());
        if args.len() > 0 && args[0] != "" {
            command.args(args);
        }
        let mutex = Mutex::new(command);
        Rc::new(move |_| {
            let mut command = mutex.lock().unwrap();
            command
                .spawn()
                .with_context(|_| format!("Could not spawn command: {:?}", *command))?;
            Ok(())
        })
    }

    /// Switches to the group specified by name.
    pub fn switch_group(name: String) -> Command {
        Rc::new(move |wm| {
            wm.switch_group(name.clone());
            Ok(())
        })
    }

    /// Moves the focused window on the active group to another group.
    pub fn move_window_to_group(name: String) -> Command {
        Rc::new(move |wm| {
            wm.move_focused_to_group(name.clone());
            Ok(())
        })
    }
}
