use std::sync::OnceLock;

pub mod pokeapi;
pub mod resource_object;

pub fn tokoi_runtime() -> &'static tokio::runtime::Runtime {
    use tokio::runtime::Runtime;
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}
