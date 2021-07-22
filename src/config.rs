use x11::keysym;

pub mod parser {
    use super::*;
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

    pub fn get_gaps() -> (Innergaps, Outergaps) {
        null_check_config();
        let deserialized_config: config_deserializer::Config = get_deserialized_config();
        (
            deserialized_config.gaps.inner,
            deserialized_config.gaps.outer,
        )
    }

    pub fn get_bound_commands() -> Vec<BoundCommand> {
        null_check_config();
        let deserialized_config: config_deserializer::Config = get_deserialized_config();
        get_parsed_bindings(deserialized_config)
    }

    pub fn get_bound_workspaces() -> Vec<BoundWorkSpace> {
        null_check_config();
        let mut result: Vec<BoundWorkSpace> = Vec::new();
        let work_space_defs: Vec<HashMap<String, String>> = get_deserialized_config().work_spaces;
        for work_space in work_space_defs {
            if let Ok(parsed) = parse_work_space(work_space.clone()) {
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
        let xk_key: u32 = safe_xk_parse(&work_space["key"])?;
        Ok((
            mask,
            xk_key,
            work_space["name"].clone(),
            work_space["layout"].clone(),
        ))
    }

    fn null_check_config() {
        if !config_file_handler::config_file_exists() {
            config_file_handler::create_default_config_file();
        }
    }

    fn get_parsed_bindings(deserialized_config: config_deserializer::Config) -> Vec<BoundCommand> {
        let mut key_bindings: Vec<BoundCommand> =
            parse_keybindings_from_config(deserialized_config.key_bindings);
        let spawn_bindings: Vec<BoundCommand> =
            parse_spawn_bindings_from_config(deserialized_config.spawn_bindings);
        key_bindings.extend(spawn_bindings);
        key_bindings
    }

    fn parse_keybindings_from_config(
        key_bindings: Vec<HashMap<String, String>>,
    ) -> Vec<BoundCommand> {
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
        let mut result: Vec<BoundCommand> = Vec::new();
        for spawn_kb in spawn_bindings {
            if let Ok(parsed_mask_and_key) =
                key_parse::parse_mask_and_key(spawn_kb["masks"].clone(), spawn_kb["key"].clone())
            {
                let lazy_command: Command =
                    get_lazy_spawn_command(spawn_kb["command"].clone(), spawn_kb["args"].clone());
                result.push((parsed_mask_and_key.0, parsed_mask_and_key.1, lazy_command));
            } else {
                error!("Could not parse {:?} continuing", spawn_kb);
                continue;
            }
        }
        result
    }

    fn get_lazy_spawn_command(command: String, pipe_separated_args: String) -> Command {
        lazy_commands::lazy_spawn(command, split_args(pipe_separated_args))
    }

    fn split_args(pipe_separated_args: String) -> Vec<String> {
        pipe_separated_args
            .split("|")
            .map(|i| i.to_string())
            .collect()
    }
}

mod key_parse {
    pub use super::*;
    use crate::ModKey;
    use std::str::FromStr;

    pub fn parse_mask_and_key(
        mask: String,
        xk_key: String,
    ) -> Result<(Vec<ModKey>, parser::XKeyValue), ()> {
        Ok((parse_mask_keys(mask), safe_xk_parse(&xk_key)?))
    }

    pub fn parse_mask_keys(mask: String) -> Vec<ModKey> {
        let mut result: Vec<ModKey> = Vec::new();
        for key in split_mask_keys(mask) {
            result.push(str_to_mask_key(key));
        }
        result
    }

    fn split_mask_keys(pipe_separated_masks: String) -> Vec<String> {
        pipe_separated_masks
            .split("|")
            .map(|i| i.to_string())
            .collect()
    }

    fn str_to_mask_key(str_mask: String) -> ModKey {
        if let Ok(parsed_key) = ModKey::from_str(&str_mask) {
            parsed_key
        } else {
            error!(
                "Mask key {}, is not a valid key defaulting to Mod1",
                str_mask
            );
            ModKey::Mod1
        }
    }
}

mod config_deserializer {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub key_bindings: Vec<HashMap<String, String>>,
        pub spawn_bindings: Vec<HashMap<String, String>>,
        pub work_spaces: Vec<HashMap<String, String>>,
        pub gaps: Gaps,
    }

    #[derive(Deserialize, Debug)]
    pub struct Gaps {
        pub inner: u32,
        pub outer: u32,
    }

    pub fn deserialize_config(config_file: String) -> Config {
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

mod config_file_handler {
    use std::fs;
    use std::io::{Read, Write};
    use xdg::BaseDirectories;

    pub fn create_default_config_file() {
        let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
        let config_path: std::path::PathBuf = xdg_dirs
            .place_config_file("config.yaml")
            .expect("Could not create config file");
        let mut config_file = fs::File::create(config_path).expect("Failed to write config file");
        config_file
            .write_all(DEFAULT_CONFIG.as_bytes())
            .expect("Could not write config");
    }

    pub fn read_config_file() -> String {
        let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
        let config_file_path: std::path::PathBuf =
            xdg_dirs.find_config_file("config.yaml").unwrap();
        let mut file: fs::File =
            fs::File::open(config_file_path).expect("Could not open config file");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    }

    pub fn config_file_exists() -> bool {
        let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
        xdg_dirs.find_config_file("config.yaml").is_some()
    }

    static DEFAULT_CONFIG: &str = "
# Masks and command arguments can be separated by pipe symbols (|)
# Example:
# {command: mkdir, args: -p|dir1|dir2|dir3, key:XK_n, masks: Mod1|Shift}

key_bindings:
  - {function: CloseFocused,      masks: Mod1,    key: XK_w  }
  - {function: FocusNext,         masks: Mod1,    key: XK_j  }
  - {function: FocusPrev,         masks: Mod1,    key: XK_k  }
  - {function: DecreaseMaster,    masks: Mod1,    key: XK_h  }
  - {function: IncreaseMaster,    masks: Mod1,    key: XK_l  }
  - {function: LayoutNext,        masks: Mod1,    key: XK_Tab}

spawn_bindings:
  - {command: pkill,          args: Xorg|volanwm,     key: XK_q,      masks: Mod1}
  - {command: qutebrowser,    args:,                  key: XK_o,      masks: Mod1}
  - {command: alacritty,      args:,                  key: XK_Return, masks: Mod1}
  - {command: emacs,          args:,                  key: XK_space,  masks: Mod1}
  - {command: dmenu_run,      args:,                  key: XK_p,      masks: Mod1}

work_spaces:
    - {name: alpha,     layout: c_master,   key: XK_a,      masks: Mod1}
    - {name: beta,      layout: c_master,   key: XK_s,      masks: Mod1}
    - {name: gamma,     layout: tile,       key: XK_d,      masks: Mod1}
    - {name: delta,     layout: tile,       key: XK_f,      masks: Mod1}

gaps:
  inner: 5
  outer: 20
    ";
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
        "XK_space" => Ok(keysym::XK_space),
        _ => Err(()),
    }
}
