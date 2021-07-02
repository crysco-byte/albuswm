#[macro_use]
extern crate albuswm;

use albuswm::layout::*;
use albuswm::{cmd, Albus, ModKey, Result};

macro_rules! spawn {
    ($cmd:expr) => (::albuswm::cmd::lazy::spawn(::std::process::Command::new($cmd)));
    ($cmd:expr, $($arg:expr),*) => {{
        let mut command = ::std::process::Command::new($cmd);
        $(
            command.arg($arg);
        )*
        ::albuswm::cmd::lazy::spawn(command)
    }}
}

fn main() -> Result<()> {
    albuswm::intiailize_logger()?;

    let modkey = ModKey::Mod1;
    let shift = ModKey::Shift;
    //let ctrl = ModKey::Control;

    #[rustfmt::skip]
    let mut keys = keys![
        ([modkey], XK_w, cmd::lazy::close_focused_window()),
        ([modkey], XK_j, cmd::lazy::focus_next()),
        ([modkey], XK_k, cmd::lazy::focus_previous()),
        ([modkey, shift], XK_j, cmd::lazy::shuffle_next()),
        ([modkey, shift], XK_k, cmd::lazy::shuffle_previous()),
        ([modkey], XK_Tab, cmd::lazy::layout_next()),

        ([modkey], XK_Return, spawn!("alacritty")),
        ([modkey], XK_o, spawn!("qutebrowser")),
        ([modkey], XK_q, spawn!("pkill", "Xorg")),
        ([modkey], XK_p, spawn!( "rofi", "-combi-modi", "drun,run,ssh","-theme", "slate", "-show", "combi", "-icon-theme", "Papirus", "-show-icons" )),
    ];

    let padding = 20;
    let layouts = layouts![
        FloatingMasterLayout::new("floating_master"),
        StackLayout::new("stack-padded", padding),
        StackLayout::new("stack", 0),
        TiledLayout::new("tiled", padding),
    ];

    let groups = groups! {
        keys,
        shift,
        [
            ([modkey], XK_a, "alpha", "stack"),
            ([modkey], XK_s, "beta", "stack"),
            ([modkey], XK_d, "gamma", "tiled"),
            ([modkey], XK_f, "delta", "tiled"),
        ]
    };

    Albus::new(keys, groups, &layouts)?.run();

    Ok(())
}
