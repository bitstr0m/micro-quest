mod character;
mod conn;
mod game;
mod schema;
mod view;

// TODO
// - Start sending content in JSON as well (defining initial schema and getting LLM to understand
// it)
// - Quest "manifest" response and quest details screen/tab/sidebar
// - Add action options to schema, getting the LLM provide them and making them clickable text to
// autosend the option
// - Better prompt
// - Proper markdown parsing/rendering
// - Text colors
// - Text effects like shaking, mexican wave, pulsing, etc.
// - Parse description text for tags, like <location>Foo Coast</location>
// - You could ask for description-level or tag-level attributes that give emotion or theme to a
// word (such as "sinister"), and we can make formatting decisions based on those

// Implement each quest step as a separate fade-in screen
// So when the user provides input, the screen fades out, and the response text fades in (possibly
// a slight delay between paragraphs for style)
// I think it will hold the attention better
// It also provides a point and location for any generated images and is more obvious what text-to-speech is being read

// BLUE SKY
// You could try and tell a story better
// An example could the flash of lightning with a generated image of a villain in the rain at the
// end of an alley, put some animation to the still images like a graphic novel
// Could give the AI a list of sound effects / soundscapes it can request

#[tokio::main]
async fn main() -> Result<(), iced::Error> {
    env_logger::init();
    view::main()
}

// OPENAI_API_KEY=xxxx cargo run
