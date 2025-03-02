mod character;
mod conn;
mod game;
mod schema;
mod view;

#[tokio::main]
async fn main() -> Result<(), iced::Error> {
    env_logger::init();
    view::main()
}

// OPENAI_API_KEY=xxxx cargo run
