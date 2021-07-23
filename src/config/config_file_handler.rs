use std::fs;
use std::io::{Read, Write};
use xdg::BaseDirectories;

pub fn create_default_config_file() {
    info!("Creating default config file");
    let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
    let config_path: std::path::PathBuf = xdg_dirs
        .place_config_file("config.yaml")
        .expect("Could not create config file");
    let mut config_file = fs::File::create(config_path).expect("Failed to write config file");
    config_file
        .write_all(DEFAULT_CONFIG.as_bytes())
        .expect("Could not write config");
}

pub fn null_check_config() {
    if !config_file_exists() {
        create_default_config_file();
    }
}

pub fn read_config_file() -> String {
    info!("Reading config file");
    let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
    let config_file_path: std::path::PathBuf = xdg_dirs.find_config_file("config.yaml").unwrap();
    let mut file: fs::File = fs::File::open(config_file_path).expect("Could not open config file");
    let mut contents: String = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

pub fn config_file_exists() -> bool {
    info!("Checking if the config file exists");
    let xdg_dirs: BaseDirectories = BaseDirectories::with_prefix("volan").unwrap();
    xdg_dirs.find_config_file("config.yaml").is_some()
}

static DEFAULT_CONFIG: &str = "
# Masks and command arguments can be separated by pipe symbols (|)
# Example:
# {command: mkdir, args: -p|dir1|dir2|dir3, key:XK_n, masks: Mod1|Shift}

key_bindings:
  - {function: CloseFocused,      masks: Mod1,          key: XK_w  }
  - {function: FocusNext,         masks: Mod1,          key: XK_j  }
  - {function: FocusPrev,         masks: Mod1,          key: XK_k  }
  - {function: DecreaseMaster,    masks: Mod1,          key: XK_h  }
  - {function: IncreaseMaster,    masks: Mod1,          key: XK_l  }
  - {function: LayoutNext,        masks: Mod1,          key: XK_Tab}
  - {function: IncreaseInnerGaps, masks: Mod1,          key: XK_1  }
  - {function: DecreaseInnerGaps, masks: Mod1,          key: XK_2  }
  - {function: IncreaseOuterGaps, masks: Mod1|Shift,    key: XK_1  }
  - {function: DecreaseOuterGaps, masks: Mod1|Shift,    key: XK_2  }

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
