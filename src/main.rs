mod application;
#[rustfmt::skip]
pub mod config;
pub mod models;
pub mod pokeapi;
pub mod widgets;

use fuzzy_matcher::skim::SkimMatcherV2;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

use gettextrs::{gettext, LocaleCategory};
use gtk::{gio, glib};

use self::application::ExampleApplication;
use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    glib::set_application_name(&gettext("PokeBook"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = ExampleApplication::default();
    app.run()
}

pub fn tokoi_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

pub fn skim_matcher() -> &'static SkimMatcherV2 {
    static RUNTIME: OnceLock<SkimMatcherV2> = OnceLock::new();
    RUNTIME.get_or_init(|| SkimMatcherV2::default())
}
