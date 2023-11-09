# Bevy Benchy

Runs Bevy stress tests and outputs the results in a nice Markdown table.

## Usage

- edit `benchy.toml.example` and save as `benchy.toml`.
- `cargo run -- --dir ../bevy`

## Example output

||bevymark 120 1000|bevymark 60 500 mesh2d|
|-|-|-|
|Enhance Bevymark|45.84|54.44|
|Bevy 0.12|51.69 🟢+ 12.8%|55.16 🟢+ 1.3%|
