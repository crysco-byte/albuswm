#[macro_use]
extern crate albuswm;

use albuswm::config_handler;
use albuswm::layout::*;
use albuswm::{cmd, Albus, ModKey, Result};

// macro_rules! spawn {
//     ($cmd:expr) => (::albuswm::cmd::lazy::spawn(::std::process::Command::new($cmd)));
//     ($cmd:expr, $($arg:expr),*) => {{
//         let mut command = ::std::process::Command::new($cmd);
//         $(
//             command.arg($arg);
//         )*
//         ::albuswm::cmd::lazy::spawn(command)
//     }}
// }

fn main() -> Result<()> {
    albuswm::intiailize_logger()?;

    let modkey = ModKey::Mod1;
    let shift = ModKey::Shift;
    //let ctrl = ModKey::Control;

    #[rustfmt::skip]
    let mut keys = config_handler::parser::get_keys_from_config_file();

    let padding = 20;
    let layouts = layouts![
        TileLayout::new("tile"),
        StackLayout::new("stack-padded", padding),
        StackLayout::new("stack", 0),
    ];

    let groups = groups! {
        keys,
        shift,
        [
            ([modkey], XK_a, "alpha", "stack"),
            ([modkey], XK_s, "beta",  "stack"),
            ([modkey], XK_d, "gamma", "stack"),
            ([modkey], XK_f, "delta", "stack"),
        ]
    };

    Albus::new(keys, groups, &layouts)?.run();

    Ok(())
}
