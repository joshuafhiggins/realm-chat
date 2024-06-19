use realm_client::Realm;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::program(Realm::title, Realm::update, Realm::view)
        .load(Realm::load)
        .subscription(Realm::subscription)
        .font(include_bytes!("../assets/icons.ttf").as_slice())
        .window_size((1280.0, 720.0))
        .run()
}