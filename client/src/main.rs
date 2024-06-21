use realm_client::RealmApp;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    iced::program(RealmApp::title, RealmApp::update, RealmApp::view)
        .load(RealmApp::load)
        .subscription(RealmApp::subscription)
        .font(include_bytes!("../assets/icons.ttf").as_slice())
        .window_size((1280.0, 720.0))
        .run()
}