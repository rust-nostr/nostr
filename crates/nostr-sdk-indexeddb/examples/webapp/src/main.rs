mod app;

use app::App;

fn main() {
    // Init logger
    //wasm_logger::init(wasm_logger::Config::default());
    tracing_wasm::set_as_global_default();

    // Start WASM app
    yew::Renderer::<App>::new().render();
}
