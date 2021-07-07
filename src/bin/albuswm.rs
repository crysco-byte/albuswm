#[macro_use]
extern crate albuswm;

use albuswm::layout::*;
use albuswm::{config, gen_groups, Albus, Result};

fn main() -> Result<()> {
    albuswm::intiailize_logger()?;

    #[rustfmt::skip]
    let keys_bound_to_commands = config::parser::get_bound_commands();
    let group_defs = config::parser::get_bound_groups();

    let padding = 20;
    let layouts = layouts![
        TileLayout::new("tile", 5, 20),
        StackLayout::new("stack-padded", padding),
        StackLayout::new("stack", 0),
    ];

    let (keys_bound_to_commands_with_group_bindings, groups) =
        gen_groups(keys_bound_to_commands, group_defs);

    Albus::new(keys_bound_to_commands_with_group_bindings, groups, &layouts)?.run();

    Ok(())
}
