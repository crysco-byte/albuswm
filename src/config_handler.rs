use x11::keysym;

// TODO: Parse mask keys definitions (mod_key = "Mod1")
// TODO: Support multiple arguments when spawning a command
// TODO: Support configurable resize ammount
// TODO: Support configurable layout groups

pub mod parser {
    use super::config_file_handler;
    use super::lazy_commands;
    use crate::cmd::Command;
    use crate::ModKey;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        key_def: KeyDef,
        key_bindings: KeyBindings,
        spawn_bindings: SpawnBindings,
    }

    #[derive(Deserialize, Debug)]
    pub struct KeyDef {
        mod_key: String,
        shift: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct KeyBindings {
        close_focused: HashMap<String, Vec<String>>,
        focus_next: HashMap<String, Vec<String>>,
        focus_prev: HashMap<String, Vec<String>>,
        resize_left: HashMap<String, Vec<String>>,
        resize_right: HashMap<String, Vec<String>>,
        layout_next: HashMap<String, Vec<String>>,
    }

    #[derive(Deserialize, Debug)]
    pub struct SpawnBindings {
        spawns: Vec<HashMap<String, Vec<String>>>,
    }

    pub fn get_keys_from_config_file() -> Vec<(Vec<ModKey>, u32, Command)> {
        if !config_file_handler::config_file_exists() {
            config_file_handler::create_default_config_file();
        }
        let config = config_file_handler::read_config_file();
        let deserialized_config: Config = deserialize_config(config);
        get_parsed_keys(deserialized_config)
    }

    fn deserialize_config(config_file: String) -> Config {
        toml::from_str(&config_file).expect("Could not parse config file")
    }

    fn get_parsed_keys(parsed_config: Config) -> Vec<(Vec<ModKey>, u32, Command)> {
        let mut key_bindings = parse_keybinding_str_keys_to_types(parsed_config.key_bindings);
        let spawn_bindings = parse_spawn_bindings_str_keys_to_types(parsed_config.spawn_bindings);
        key_bindings.extend(spawn_bindings);
        key_bindings
    }

    fn parse_keybinding_str_keys_to_types(
        key_bindings: KeyBindings,
    ) -> Vec<(Vec<ModKey>, u32, Command)> {
        let mut result: Vec<(Vec<ModKey>, u32, Command)> = Vec::new();
        let kb_to_vec = keybindings_to_vec(key_bindings);
        for (i, binding) in kb_to_vec.into_iter().enumerate() {
            let masks = parse_mask_keys(binding["mask"].clone());
            let xk_key = super::safe_xk_parse(&binding["key"][0].clone()).expect("XK_key not in safe parse range");
            let lazy_command = lazy_commands::get_cmd_based_on_action(
                &lazy_commands::lookup_actiontypes_by_index(i),
            )
            .unwrap();
            result.push((masks, xk_key, lazy_command));
        }
        result
    }

    fn parse_spawn_bindings_str_keys_to_types(
        spawn_bindings: SpawnBindings,
    ) -> Vec<(Vec<ModKey>, u32, Command)> {
        let mut result: Vec<(Vec<ModKey>, u32, Command)> = Vec::new();
        for data_group in spawn_bindings.spawns {
            let masks = parse_mask_keys(data_group["mask"].clone());
            let xk_key = super::safe_xk_parse(&data_group["key"][0].clone()).expect("XK_key not in safe parse range");
            let lazy_command = lazy_commands::lazy_spawn(data_group["command"].clone())
                .expect("Could not get spawn command");
            result.push((masks, xk_key, lazy_command));
        }
        result
    }

    fn keybindings_to_vec(kb: KeyBindings) -> Vec<HashMap<String, Vec<String>>> {
        vec![
            kb.close_focused,
            kb.focus_next,
            kb.focus_prev,
            kb.resize_left,
            kb.resize_right,
            kb.layout_next,
        ]
    }

    fn parse_mask_keys(masks: Vec<String>) -> Vec<ModKey> {
        let mut result: Vec<ModKey> = Vec::new();
        for key in masks {
            result.push(ModKey::from_str(&key).expect("Could not parse mask keys"));
        }
        result
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

    pub fn get_cmd_based_on_action(action: &ActionTypes) -> std::result::Result<Command, ()> {
        match action {
            ActionTypes::CloseFocused => Ok(cmd::lazy::close_focused_window()),
            ActionTypes::FocusNext => Ok(cmd::lazy::focus_next()),
            ActionTypes::FocusPrev => Ok(cmd::lazy::focus_previous()),
            ActionTypes::ResizeRight => Ok(cmd::lazy::resize_right()),
            ActionTypes::ResizeLeft => Ok(cmd::lazy::resize_left()),
            ActionTypes::LayoutNext => Ok(cmd::lazy::layout_next()),
        }
    }

    pub fn lazy_spawn(cmd_vec: Vec<String>) -> Result<Command, ()> {
        if cmd_vec.is_empty() {
            ()
        }
        if cmd_vec.len() < 2 {
            let command = std::process::Command::new(cmd_vec[0].clone());
            Ok(cmd::lazy::spawn(command))
        } else {
            Err(())
        }
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
[key_def]
mod_key = "Mod1"
shift = "Shift"

[key_bindings]
close_focused = {mask=["mod"], key=["XK_w"]}
focus_next = {mask=["mod"], key=["XK_j"]}
focus_prev = {mask=["mod"], key=["XK_k"]}
resize_left = {mask=["mod"], key=["XK_h"]}
resize_right = {mask=["mod"], key=["XK_l"]}
layout_next = {mask=["mod"], key=["XK_Tab"]}

[spawn_bindings]
spawns = [
{command=["qutebrowser"], args=[""], mask=["mod"], key=["XK_o"]},
{command=["alacritty"], args=[""], mask=["mod"], key=["XK_Return"]},
]
    "#;
}


fn safe_xk_parse(string: &str) -> Result<u32, ()> {
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
        _ => Err(())
    }
}


