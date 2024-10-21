mod app;
use app::App;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    console_error_panic_hook::set_once();

    yew::Renderer::<App>::new().render();
}