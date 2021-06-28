use std::os::raw::c_uint;


type ModMask = c_uint;
type Key = c_uint;

enum ModKey {
    Shift,
    Control
}

impl Modkey {
    fn mask(self) -> ModMask {
        match self {
            ModKey::Shift => xcb::MOD_MASK_SHIFT,
            ModKey::Control => xcb::MOD_MASK_CONTROL,
        }
    }
}
