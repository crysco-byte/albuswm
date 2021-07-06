use x11::keysym;

pub mod parser {
    use super::lazy_commands;
    use super::*;
    use crate::cmd::Command;
    use crate::ModKey;
    use std::collections::HashMap;
    type LayoutName = String;
    type GroupName = String;
    pub type XKeyValue = u32;
    type BoundCommand = (Vec<ModKey>, XKeyValue, Command);
    type BoundGroup = (ModKey, XKeyValue, GroupName, LayoutName);

    pub fn get_keys_from_config_file() -> Vec<BoundCommand> {
        null_check_config();
        let config = config_file_handler::read_config_file();
        let deserialized_config: config_deserializer::Config =
            config_deserializer::deserialize_config(config);
        get_parsed_keys(deserialized_config)
    }

    pub fn get_parsed_group_definitions() -> Vec<BoundGroup> {
        null_check_config();
        let config = config_file_handler::read_config_file();
        let deserialized_config: config_deserializer::GroupDefinition =
            config_deserializer::deserialize_config(config).group_definitions;
        let mut result = Vec::new();
        for data_group in deserialized_config.groups {
            if let Ok(parsed) = parse_group_def_types(data_group.clone()) {
                result.push(parsed);
            } else {
                error!("Could not group definitions: {:?}", data_group);
                continue;
            }
        }
        result
    }

    fn parse_group_def_types(data_group: HashMap<String, String>) -> Result<BoundGroup, ()> {
        let mask = key_parse::parse_mask_keys(vec![data_group["mask"].clone()])[0];
        let xk_key = safe_xk_parse(&data_group["key"])?;
        Ok((
            mask,
            xk_key,
            data_group["name"].clone(),
            data_group["layout"].clone(),
        ))
    }

    fn null_check_config() {
        if !config_file_handler::config_file_exists() {
            config_file_handler::create_default_config_file();
        }
    }

    fn get_parsed_keys(parsed_config: config_deserializer::Config) -> Vec<BoundCommand> {
        let mut key_bindings = parse_keybinding_str_keys_to_types(parsed_config.key_bindings);
        let spawn_bindings = parse_spawn_bindings_str_keys_to_types(parsed_config.spawn_bindings);
        key_bindings.extend(spawn_bindings);
        key_bindings
    }

    fn parse_keybinding_str_keys_to_types(
        key_bindings: config_deserializer::KeyBindingDefinition,
    ) -> Vec<BoundCommand> {
        let mut result: Vec<BoundCommand> = Vec::new();
        let kb_to_vec = keybindings_to_vec(key_bindings);
        for (i, data_group) in kb_to_vec.into_iter().enumerate() {
            if let Ok(parsed_mask_and_key) = key_parse::parse_mask_and_key(
                data_group["mask"].clone(),
                data_group["key"][0].clone(),
            ) {
                let lazy_command = lazy_commands::get_cmd_based_on_action(
                    &lazy_commands::lookup_actiontypes_by_index(i),
                );
                result.push((parsed_mask_and_key.0, parsed_mask_and_key.1, lazy_command));
            } else {
                error!("Could not parse: {:?}", data_group);
                continue;
            }
        }
        result
    }

    fn parse_spawn_bindings_str_keys_to_types(
        spawn_bindings: config_deserializer::SpawnBindingDefinition,
    ) -> Vec<BoundCommand> {
        let mut result: Vec<BoundCommand> = Vec::new();
        for data_group in spawn_bindings.spawns {
            if let Ok(parsed_mask_and_key) = key_parse::parse_mask_and_key(
                data_group["mask"].clone(),
                data_group["key"][0].clone(),
            ) {
                let lazy_command = lazy_commands::lazy_spawn(
                    data_group["command"].clone(),
                    data_group["args"].clone(),
                );
                result.push((parsed_mask_and_key.0, parsed_mask_and_key.1, lazy_command));
            } else {
                error!("Could not parse {:?}", data_group);
                continue;
            }
        }
        result
    }

    fn keybindings_to_vec(
        kb: config_deserializer::KeyBindingDefinition,
    ) -> Vec<HashMap<String, Vec<String>>> {
        vec![
            kb.close_focused,
            kb.focus_next,
            kb.focus_prev,
            kb.resize_left,
            kb.resize_right,
            kb.layout_next,
        ]
    }
}

mod key_parse {
    pub use super::*;
    use crate::ModKey;
    use std::str::FromStr;

    pub fn parse_mask_and_key(
        mask: Vec<String>,
        xk_key: String,
    ) -> Result<(Vec<ModKey>, parser::XKeyValue), ()> {
        Ok((parse_mask_keys(mask), safe_xk_parse(&xk_key)?))
    }

    pub fn parse_mask_keys(masks: Vec<String>) -> Vec<ModKey> {
        let mut result: Vec<ModKey> = Vec::new();
        for key in masks {
            result.push(ModKey::from_str(&key).expect("Could not parse mask keys"));
        }
        result
    }
}

mod config_deserializer {
    use serde::Deserialize;
    use std::collections::HashMap;
    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub key_bindings: KeyBindingDefinition,
        pub spawn_bindings: SpawnBindingDefinition,
        pub group_definitions: GroupDefinition,
    }

    #[derive(Deserialize, Debug)]
    pub struct GroupDefinition {
        pub groups: Vec<HashMap<String, String>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct KeyBindingDefinition {
        pub close_focused: HashMap<String, Vec<String>>,
        pub focus_next: HashMap<String, Vec<String>>,
        pub focus_prev: HashMap<String, Vec<String>>,
        pub resize_left: HashMap<String, Vec<String>>,
        pub resize_right: HashMap<String, Vec<String>>,
        pub layout_next: HashMap<String, Vec<String>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct SpawnBindingDefinition {
        pub spawns: Vec<HashMap<String, Vec<String>>>,
    }

    pub fn deserialize_config(config_file: String) -> Config {
        toml::from_str(&config_file).expect("Could not parse config file")
    }
}

mod lazy_commands {
    use crate::cmd::{self, Command};
    pub enum ActionTypes {
        CloseFocused,
        FocusNext,
        FocusPrev,
        ResizeLeft,
        ResizeRight,
        LayoutNext,
    }

    pub fn lookup_actiontypes_by_index(i: usize) -> ActionTypes {
        match i {
            0 => ActionTypes::CloseFocused,
            1 => ActionTypes::FocusNext,
            2 => ActionTypes::FocusPrev,
            3 => ActionTypes::ResizeLeft,
            4 => ActionTypes::ResizeRight,
            5 => ActionTypes::LayoutNext,
            _ => panic!("Index out of bounds"),
        }
    }

    pub fn get_cmd_based_on_action(action: &ActionTypes) -> Command {
        match action {
            ActionTypes::CloseFocused => cmd::lazy::close_focused_window(),
            ActionTypes::FocusNext => cmd::lazy::focus_next(),
            ActionTypes::FocusPrev => cmd::lazy::focus_previous(),
            ActionTypes::ResizeRight => cmd::lazy::resize_right(),
            ActionTypes::ResizeLeft => cmd::lazy::resize_left(),
            ActionTypes::LayoutNext => cmd::lazy::layout_next(),
        }
    }

    pub fn lazy_spawn(command: Vec<String>, args: Vec<String>) -> Command {
        cmd::lazy::spawn(command[0].clone(), args)
    }
}

mod config_file_handler {
    use std::fs;
    use std::io::{Read, Write};
    use xdg::BaseDirectories;

    pub fn create_default_config_file() {
        let xdg_dirs = BaseDirectories::with_prefix("albus").unwrap();
        let config_path = xdg_dirs
            .place_config_file("config.toml")
            .expect("Could not create config file");
        let mut config_file = fs::File::create(config_path).expect("Failed to write config file");
        config_file
            .write_all(DEFAULT_CONFIG.as_bytes())
            .expect("Could not write config");
    }

    pub fn read_config_file() -> String {
        let xdg_dirs = BaseDirectories::with_prefix("albus").unwrap();
        let config_file_path = xdg_dirs.find_config_file("config.toml").unwrap();
        let mut file = fs::File::open(config_file_path).expect("Could not open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    }

    pub fn config_file_exists() -> bool {
        let xdg_dirs = BaseDirectories::with_prefix("albus").unwrap();
        xdg_dirs.find_config_file("config.toml").is_some()
    }

    static DEFAULT_CONFIG: &str = r#"
[key_bindings]
close_focused = {mask=["Mod1"], key=["XK_w"]}
focus_next = {mask=["Mod1"], key=["XK_j"]}
focus_prev = {mask=["Mod1"], key=["XK_k"]}
resize_left = {mask=["Mod1"], key=["XK_h"]}
resize_right = {mask=["Mod1"], key=["XK_l"]}
layout_next = {mask=["Mod1"], key=["XK_Tab"]}

[spawn_bindings]
spawns = [
{command=["pkill"], args=["Xorg"], mask=["Mod1"], key=["XK_q"]},
{command=["qutebrowser"], args=[""], mask=["Mod1"], key=["XK_o"]},
{command=["alacritty"], args=[""], mask=["Mod1"], key=["XK_Return"]},
{command=["rofi"], args=["-combi-modi", "drun, run, ssh", "-theme", "slate", "-show", "combi", "-icon-theme", "Papirus", "-show-icons"], mask=["Mod1"], key=["XK_p"]}
]

[group_definitions]
groups = [
    {mask = "Mod1", key="XK_a", name="alpha", layout="tile"},
    {mask = "Mod1", key="XK_s", name="beta", layout="tile"},
    {mask = "Mod1", key="XK_d", name="gamma", layout="tile"},
    {mask = "Mod1", key="XK_f", name="delta", layout="tile"},
]
    "#;
}

pub fn safe_xk_parse(string: &str) -> Result<u32, ()> {
    match string {
        "XK_a" => Ok(keysym::XK_a),
        "XK_b" => Ok(keysym::XK_b),
        "XK_c" => Ok(keysym::XK_c),
        "XK_d" => Ok(keysym::XK_d),
        "XK_e" => Ok(keysym::XK_e),
        "XK_f" => Ok(keysym::XK_f),
        "XK_g" => Ok(keysym::XK_g),
        "XK_h" => Ok(keysym::XK_h),
        "XK_i" => Ok(keysym::XK_i),
        "XK_j" => Ok(keysym::XK_j),
        "XK_k" => Ok(keysym::XK_k),
        "XK_l" => Ok(keysym::XK_l),
        "XK_m" => Ok(keysym::XK_m),
        "XK_n" => Ok(keysym::XK_n),
        "XK_o" => Ok(keysym::XK_o),
        "XK_p" => Ok(keysym::XK_p),
        "XK_q" => Ok(keysym::XK_q),
        "XK_r" => Ok(keysym::XK_r),
        "XK_s" => Ok(keysym::XK_s),
        "XK_t" => Ok(keysym::XK_t),
        "XK_u" => Ok(keysym::XK_u),
        "XK_v" => Ok(keysym::XK_v),
        "XK_w" => Ok(keysym::XK_w),
        "XK_x" => Ok(keysym::XK_x),
        "XK_y" => Ok(keysym::XK_y),
        "XK_z" => Ok(keysym::XK_z),
        "XK_0" => Ok(keysym::XK_0),
        "XK_1" => Ok(keysym::XK_1),
        "XK_2" => Ok(keysym::XK_2),
        "XK_3" => Ok(keysym::XK_3),
        "XK_4" => Ok(keysym::XK_4),
        "XK_5" => Ok(keysym::XK_5),
        "XK_6" => Ok(keysym::XK_6),
        "XK_7" => Ok(keysym::XK_7),
        "XK_8" => Ok(keysym::XK_8),
        "XK_9" => Ok(keysym::XK_9),
        "XK_Return" => Ok(keysym::XK_Return),
        "XK_Tab" => Ok(keysym::XK_Tab),
        _ => Err(()),
    }
}
