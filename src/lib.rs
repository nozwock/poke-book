use std::sync::OnceLock;

use fuzzy_matcher::skim::SkimMatcherV2;
use tokio::runtime::Runtime;

pub mod pokeapi;
pub mod resource_object;

pub fn tokoi_runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

pub fn skim_matcher() -> &'static SkimMatcherV2 {
    static RUNTIME: OnceLock<SkimMatcherV2> = OnceLock::new();
    RUNTIME.get_or_init(|| SkimMatcherV2::default())
}
