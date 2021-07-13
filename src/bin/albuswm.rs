#[macro_use]
extern crate albuswm;

use albuswm::layout::*;
use albuswm::{config, gen_groups, Albus, Result};

fn main() -> Result<()> {
    albuswm::intiailize_logger()?;

    #[rustfmt::skip]
    let keys_bound_to_commands = config::parser::get_bound_commands();
    let group_defs = config::parser::get_bound_groups();
    let (innergaps, outergaps) = config::parser::get_gaps();

    let layouts = layouts![
        TileLayout::new("tile", innergaps, outergaps),
        CenterMaster::new("c_master", innergaps, outergaps),
    ];

    let (keys_bound_to_commands_with_group_bindings, groups) =
        gen_groups(keys_bound_to_commands, group_defs);

    Albus::new(keys_bound_to_commands_with_group_bindings, groups, &layouts)?.run();

    Ok(())
}
