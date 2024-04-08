use std::sync::OnceLock;

use gtk::glib::{self};
use rustemon::client::RustemonClient;

#[derive(Debug, Default, Clone, Copy, glib::Enum)]
#[enum_type(name = "Categories")]
pub enum ResourceGroup {
    #[default]
    Pokemon,
    Moves,
}

pub fn rustemon_client() -> &'static RustemonClient {
    static CLIENT: OnceLock<RustemonClient> = OnceLock::new();
    CLIENT.get_or_init(|| rustemon::client::RustemonClient::default())
}
