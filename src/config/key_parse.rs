use crate::ModKey;
use std::str::FromStr;
use x11::keysym;

pub fn parse_mask_and_key(
    mask: String,
    xk_key: String,
) -> Result<(Vec<ModKey>, super::XKeyValue), ()> {
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
