use std::sync::OnceLock;

use gtk::glib::{self, translate::FromGlib};
use rustemon::client::RustemonClient;

#[derive(Debug, Default, Clone, Copy, glib::Enum)]
#[enum_type(name = "Categories")]
pub enum ResourceGroup {
    #[default]
    Pokemon,
    Moves,
    Abilities,
}

impl From<u32> for ResourceGroup {
    fn from(value: u32) -> Self {
        unsafe { Self::from_glib(value as i32) }
    }
}

pub fn rustemon_client() -> &'static RustemonClient {
    static CLIENT: OnceLock<RustemonClient> = OnceLock::new();
    CLIENT.get_or_init(|| rustemon::client::RustemonClient::default())
}
