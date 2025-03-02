# uQuest

uQuest utilises OpenAI to run a short, linear D&D-esque quest for a single character.
It was mainly just a learning experience and isn't going to be actively developed.
You need an OpenAI API key to use uQuest.

The startup time is a bit slow as it waits for OpenAI to respond.

To run:
```
OPENAI_API_KEY=xxxx cargo run
```

You can also use the `RUST_LOG` environment variable to output more information, e.g., `RUST_LOG=micro_quest=debug`.
