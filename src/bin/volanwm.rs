#[macro_use]
extern crate volanwm;

use volanwm::layout::*;
use volanwm::{config, gen_workspaces, Result, Volan};

fn main() -> Result<()> {
    volanwm::intiailize_logger()?;
    let parser = config::Parser::new();

    let keys_bound_to_commands: Vec<config::BoundCommand> = parser.get_bound_commands();
    let group_defs: Vec<config::BoundWorkSpace> = parser.get_bound_workspaces();
    let (innergaps, outergaps): (u32, u32) = parser.get_gaps();

    let layouts: Vec<Box<dyn volanwm::layout::Layout>> = layouts![
        TileLayout::new("tile", innergaps, outergaps),
        CenterMaster::new("c_master", innergaps, outergaps),
    ];

    let (keys_bound_to_commands_with_group_bindings, workspaces): (
        Vec<config::BoundCommand>,
        Vec<volanwm::WorkSpaceBuilder>,
    ) = gen_workspaces(keys_bound_to_commands, group_defs);

    Volan::new(
        keys_bound_to_commands_with_group_bindings,
        workspaces,
        &layouts,
    )?
    .run();

    Ok(())
}
