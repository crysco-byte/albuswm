use std::collections::HashMap;
use xcb_util::{ewmh, icccm};


struct WindowId(xcb::Window);
impl WindowId {
    fn to_x(&self) -> xcb::Window {
        self.0
    }
}

enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    DropdownMenu,
    PopupMenu,
    Notification,
    Combo,
    Dnd,
    Normal
}

enum WindowState {
    Modal,
    Sticky,
    Hidden,
    Fullscreen,
    DemandsAttention
}

struct Connection {
    ewmh_connection: ewmh::Connection,
    root_window: WindowId,
    screen_index: i32,
    atoms: InternedAtoms,
    window_type: HashMap<xcb::Atom, WindowType>,
    window_state: HashMap<xcb::Atom, WindowState>,
}


macro_rules! atoms {
    ( $( $name:ident ),+ ) => {
        #[allow(non_snake_case)]
        struct InternedAtoms {
            $(
                pub $name: xcb::Atom
            ),*
        }

        impl InternedAtoms {
            pub fn new(conn: &xcb::Connection) -> Result<InternedAtoms, std::io::Error> {
                Ok(InternedAtoms {
                    $(
                        $name: Connection::get_intern_atom(conn, stringify!($name))
                    ),*
                })
            }
        }
    };
    // Allow trailing comma:
    ( $( $name:ident ),+ , ) => (atoms!($( $name ),+);)
}

atoms!(WM_DELETE_WINDOW, WM_PROTOCOLS,);




impl Connection {
    pub fn connect() -> Result<Connection, std::io::Error> {
        let (xcb_connection, screen_index) =
            xcb::Connection::connect(None).expect("Failed to connect to X server");
        let ewmh_connection = ewmh::Connection::connect(xcb_connection).map_err(|(e,_)| e).unwrap();
        let root_window = Self::get_root_window(&ewmh_connection, screen_index);
        let atoms = InternedAtoms::new(&ewmh_connection).expect("Failed to get intern atoms");
        let window_type = Self::get_types(&ewmh_connection);
        let window_state = Self::get_states(&ewmh_connection);
        Ok(Self {
            ewmh_connection,
            root_window,
            screen_index,
            atoms,
            window_type,
            window_state
        })
    }

    //pub fn install_as_wm(&self, key_handlers: &KeyHandlers) -> Result<()> {
    
    //pub fn update_ewmh_desktops(&self, groups: &Stack<Group>) {

    pub fn get_top_level_windows(&self) -> Vec<WindowId> {
        let windows = xcb::query_tree(&self.ewmh_connection, self.root_window.to_x())
            .get_reply().expect("Could not get wm reply.")
            .children()
            .iter()
            .map(|w| WindowId(*w))
            .collect();
        windows
    }
    
    pub fn get_root_window_id(&self) -> &WindowId {
        &self.root_window
    } 

    pub fn close_window(&self, window_id: &WindowId) {
        if self.can_use_wm_delete(window_id) {
            self.close_with_wm_delete(window_id);
        }else {
            xcb::destroy_window(&self.ewmh_connection, window_id.to_x());
        }
    }

    pub fn configure_window(&self, window_id: &WindowId, x:u32, y:u32, width:u32, height:u32) {
        let values = Self::get_config_window_values(x,y,width,height);
        xcb::configure_window(&self.ewmh_connection, window_id.to_x(), &values);
    }

    pub fn get_window_geometry(&self, window_id: &WindowId) -> (u32, u32) {
        let reply = xcb::get_geometry(&self.ewmh_connection, window_id.to_x())
            .get_reply()
            .unwrap();
        (u32::from(reply.width()), u32::from(reply.height()))
    }

    pub fn map_window(&self, window_id: &WindowId) {
        xcb::map_window(&self.ewmh_connection, window_id.to_x());
    }

    pub fn unmap_window(&self, window_id: &WindowId) {
        xcb::unmap_window(&self.ewmh_connection, window_id.to_x());
    }

    pub fn focus_nothing(&self) {
        ewmh::set_active_window(&self.ewmh_connection, self.screen_index, xcb::NONE);
    }

    pub fn enable_window_tracking(&self, window_id: &WindowId) {
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_ENTER_WINDOW | xcb::EVENT_MASK_STRUCTURE_NOTIFY,
        )];
        xcb::change_window_attributes(&self.ewmh_connection, window_id.to_x(), &values);
    }

    pub fn disable_window_tracking(&self, window_id: &WindowId) {
        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
        xcb::change_window_attributes(&self.ewmh_connection, window_id.to_x(), &values);
    }

    pub fn focus_window(&self, window_id: &WindowId) {
        xcb::set_input_focus(
            &self.ewmh_connection,
            xcb::INPUT_FOCUS_POINTER_ROOT as u8,
            window_id.to_x(),
            xcb::CURRENT_TIME
        );
        ewmh::set_active_window(&self.ewmh_connection, self.screen_index, window_id.to_x());
    }

    fn get_config_window_values(x:u32, y:u32, width:u32, height:u32) -> [(u16, u32); 4]{
        [
            (xcb::CONFIG_WINDOW_X as u16, x),
            (xcb::CONFIG_WINDOW_Y as u16, y),
            (xcb::CONFIG_WINDOW_WIDTH as u16, width),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, height),
        ]
    }

    
    fn can_use_wm_delete(&self, window_id: &WindowId) -> bool {
            self
            .get_wm_protocols(window_id)
            .map(|protocols| protocols.contains(
                    &self.atoms.WM_DELETE_WINDOW
            )).unwrap_or(false)
    }
    
    fn get_wm_protocols(&self, window_id: &WindowId) -> Result<Vec<xcb::Atom>, std::io::Error> {
        let reply = icccm::get_wm_protocols(&self.ewmh_connection, window_id.to_x(), 
            self.atoms.WM_PROTOCOLS).get_reply().expect("Could not get wm protocols.");
        Ok(reply.atoms().to_vec())
    }


    fn flush_connection(&self) {
        self.ewmh_connection.flush();
    }

    fn close_with_wm_delete(&self, window_id: &WindowId) {
            let data = xcb::ClientMessageData::from_data32([
                self.atoms.WM_DELETE_WINDOW,
                xcb::CURRENT_TIME,
                0,0,0
            ]);
            let event = 
                xcb::ClientMessageEvent::new(32,window_id.to_x(),
                    self.atoms.WM_PROTOCOLS, data);
            xcb::send_event(&self.ewmh_connection,
                false, window_id.to_x(),
                xcb::EVENT_MASK_NO_EVENT,
                &event);
    }
    
    fn get_intern_atom(xcb_connection: &xcb::Connection, atom_name: &str) -> xcb::Atom {
        xcb::intern_atom(xcb_connection, false, atom_name).get_reply().unwrap().atom()
    }

    fn get_root_window(ewmh_connection: &ewmh::Connection, screen_index: i32) -> WindowId {
        WindowId(ewmh_connection.get_setup()
            .roots()
            .nth(screen_index as usize)
            .ok_or_else(|| panic!("Invalid Screen"))
            .unwrap()
            .root())
    }

    fn get_types(ewmh_connection: &ewmh::Connection) -> HashMap<xcb::Atom, WindowType> {
        let mut result = HashMap::new();
        result.insert(ewmh_connection.WM_WINDOW_TYPE_DESKTOP(), WindowType::Desktop);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_DOCK(), WindowType::Dock);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_TOOLBAR(), WindowType::Toolbar);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_MENU(), WindowType::Menu);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_UTILITY(), WindowType::Utility);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_DIALOG(), WindowType::Dialog);
        result.insert(ewmh_connection.WM_WINDOW_TYPE_SPLASH(), WindowType::Splash);
        result
    }

    fn get_states(ewmh_connection: &ewmh::Connection) -> HashMap<xcb::Atom, WindowState> {
        let mut result = HashMap::new();
        result.insert(ewmh_connection.WM_STATE_MODAL(), WindowState::Modal);
        result.insert(ewmh_connection.WM_STATE_STICKY(), WindowState::Sticky);
        result.insert(ewmh_connection.WM_STATE_HIDDEN(), WindowState::Hidden);
        result.insert(ewmh_connection.WM_STATE_FULLSCREEN(), WindowState::Fullscreen);
        result.insert(
            ewmh_connection.WM_STATE_DEMANDS_ATTENTION(),
            WindowState::DemandsAttention,
        );
        result
    }

}
