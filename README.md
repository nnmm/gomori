# Gomori tournament

This repo contains tools for letting bots play the game of [Gomori](https://github.com/phwitti/gomori-rules).

## Running a game between two bots

```
cargo build --release
target/release/judge bots/random_bot.json bots/greedy_bot.json -n 1000
```

See the `--help` text of the judge for more options.

## Making a bot

The protocol that the bots use to play consists of JSON requests and responses via standard input/output, with the judge being the client and the bots being the servers.
For instance, the judge will send the bot a JSON message on its stdin asking it to choose up to five cards to play, and the bots will reply via stdout with a JSON message of its own.

If you use the [Rust](gomori), Python or [C#](https://github.com/phwitti/gomori-bot-csharp-template) helper packages, the protocol and game logic is already implemented for you. See their READMEs for more information.

### JSON protocol

To see what the messages look like, you can run the judge with `--log-level trace`.
Messages are newline-delimited, i.e. the JSON must be in a compact format with no newlines, and followed by a newline.

For an example for how the data could look in code (in this case, in Rust), see [`protocol_types.rs`](gomori/src/protocol_types.rs).

### Debugging illegal moves

The `--stop-on-first-illegal-move` option of the judge is useful for debugging.