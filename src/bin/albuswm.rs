#[macro_use]
extern crate albuswm;

use albuswm::layout::*;
use albuswm::{config_handler, gen_groups, Albus, Result};

fn main() -> Result<()> {
    albuswm::intiailize_logger()?;

    #[rustfmt::skip]
    let keys_bound_to_commands = config_handler::parser::get_keys_from_config_file();
    let group_defs = config_handler::parser::get_parsed_group_definitions();

    let padding = 20;
    let layouts = layouts![
        TileLayout::new("tile"),
        StackLayout::new("stack-padded", padding),
        StackLayout::new("stack", 0),
    ];

    let (keys_bound_to_commands_with_group_bindings, groups) = gen_groups(keys_bound_to_commands, group_defs);

    Albus::new(keys_bound_to_commands_with_group_bindings, groups, &layouts)?.run();

    Ok(())
}
