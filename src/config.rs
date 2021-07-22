use crate::cmd::Command;
use crate::ModKey;
use std::collections::HashMap;
use std::str::FromStr;
type LayoutName = String;
type WorkSpaceName = String;
pub type XKeyValue = u32;
pub type BoundCommand = (Vec<ModKey>, XKeyValue, Command);
pub type BoundWorkSpace = (ModKey, XKeyValue, WorkSpaceName, LayoutName);
type Innergaps = u32;
type Outergaps = u32;

mod config_file_handler;
mod key_parse;

pub struct Parser {
    deserialized_config: config_deserializer::Config,
}

impl Parser {
    pub fn new() -> Self {
        config_file_handler::null_check_config();
        Self {
            deserialized_config: Self::get_deserialized_config(),
        }
    }

    pub fn get_gaps(&self) -> (Innergaps, Outergaps) {
        info!("Getting gaps");
        (
            self.deserialized_config.gaps.inner,
            self.deserialized_config.gaps.outer,
        )
    }

    pub fn get_bound_commands(&self) -> Vec<BoundCommand> {
        info!("Getting bound commands");
        Self::get_parsed_bindings(self.deserialized_config.clone())
    }

    pub fn get_bound_workspaces(&self) -> Vec<BoundWorkSpace> {
        info!("Getting bound workspaces");
        let mut result: Vec<BoundWorkSpace> = Vec::new();
        let work_space_defs: Vec<HashMap<String, String>> =
            self.deserialized_config.work_spaces.clone();
        for work_space in work_space_defs {
            if let Ok(parsed) = Self::parse_work_space(work_space.clone()) {
                result.push(parsed);
            } else {
                error!("Could not parse workspace: {:?} continuing ...", work_space);
                continue;
            }
        }
        result
    }

    fn get_deserialized_config() -> config_deserializer::Config {
        let config: String = config_file_handler::read_config_file();
        config_deserializer::deserialize_config(config)
    }

    fn parse_work_space(work_space: HashMap<String, String>) -> Result<BoundWorkSpace, ()> {
        let mask: ModKey = key_parse::parse_mask_keys(work_space["masks"].clone())[0];
        let xk_key: u32 = key_parse::safe_xk_parse(&work_space["key"])?;
        Ok((
            mask,
            xk_key,
            work_space["name"].clone(),
            work_space["layout"].clone(),
        ))
    }

    fn get_parsed_bindings(deserialized_config: config_deserializer::Config) -> Vec<BoundCommand> {
        let mut key_bindings: Vec<BoundCommand> =
            Self::parse_keybindings_from_config(deserialized_config.key_bindings);
        let spawn_bindings: Vec<BoundCommand> =
            Self::parse_spawn_bindings_from_config(deserialized_config.spawn_bindings);
        key_bindings.extend(spawn_bindings);
        key_bindings
    }

    fn parse_keybindings_from_config(
        key_bindings: Vec<HashMap<String, String>>,
    ) -> Vec<BoundCommand> {
        info!("Parsing keybindings");
        let mut result: Vec<BoundCommand> = Vec::new();
        for key_binding in key_bindings {
            if let Ok(parsed_mask_and_key) = key_parse::parse_mask_and_key(
                key_binding["masks"].clone(),
                key_binding["key"].clone(),
            ) {
                let lazy_command: Command = lazy_commands::get_cmd_based_on_action(
                    &lazy_commands::ActionTypes::from_str(&key_binding["function"]).unwrap(),
                );
                result.push((parsed_mask_and_key.0, parsed_mask_and_key.1, lazy_command));
            } else {
                error!("Could not parse {:?} continuing ...", key_binding);
                continue;
            }
        }
        result
    }

    fn parse_spawn_bindings_from_config(
        spawn_bindings: Vec<HashMap<String, String>>,
    ) -> Vec<BoundCommand> {
        info!("Parsing spawn bindings");
        let mut result: Vec<BoundCommand> = Vec::new();
        for spawn_kb in spawn_bindings {
            if let Ok(parsed_mask_and_key) =
                key_parse::parse_mask_and_key(spawn_kb["masks"].clone(), spawn_kb["key"].clone())
            {
                let lazy_command: Command = Self::get_lazy_spawn_command(
                    spawn_kb["command"].clone(),
                    spawn_kb["args"].clone(),
                );
                result.push((parsed_mask_and_key.0, parsed_mask_and_key.1, lazy_command));
            } else {
                error!("Could not parse {:?} continuing", spawn_kb);
                continue;
            }
        }
        result
    }

    fn get_lazy_spawn_command(command: String, pipe_separated_args: String) -> Command {
        lazy_commands::lazy_spawn(command, Self::split_args(pipe_separated_args))
    }

    fn split_args(pipe_separated_args: String) -> Vec<String> {
        pipe_separated_args
            .split("|")
            .map(|i| {if i=="~" {"".to_string()}else{i.to_string()}})
            .collect()
    }
}

mod config_deserializer {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize, Debug, Clone)]
    pub struct Config {
        pub key_bindings: Vec<HashMap<String, String>>,
        pub spawn_bindings: Vec<HashMap<String, String>>,
        pub work_spaces: Vec<HashMap<String, String>>,
        pub gaps: Gaps,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct Gaps {
        pub inner: u32,
        pub outer: u32,
    }

    pub fn deserialize_config(config_file: String) -> Config {
        info!("Deserializing config");
        serde_yaml::from_str(&config_file).expect("Could not parse config file")
    }
}

mod lazy_commands {
    use crate::cmd::{self, Command};

    #[derive(EnumString)]
    pub enum ActionTypes {
        CloseFocused,
        FocusNext,
        FocusPrev,
        DecreaseMaster,
        IncreaseMaster,
        LayoutNext,
    }

    pub fn get_cmd_based_on_action(action: &ActionTypes) -> Command {
        match action {
            ActionTypes::CloseFocused => cmd::lazy::close_focused_window(),
            ActionTypes::FocusNext => cmd::lazy::focus_next(),
            ActionTypes::FocusPrev => cmd::lazy::focus_previous(),
            ActionTypes::IncreaseMaster => cmd::lazy::increase_master(),
            ActionTypes::DecreaseMaster => cmd::lazy::decrease_master(),
            ActionTypes::LayoutNext => cmd::lazy::layout_next(),
        }
    }

    pub fn lazy_spawn(command: String, args: Vec<String>) -> Command {
        cmd::lazy::spawn(command, args)
    }
}
