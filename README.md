# nono
Nonogram hint dispenser

At this time it only implements very basic techniques.
I made up the entire terminology around the hints by myself and there's no documentation of it.


## Getting started

You need Rust and Cargo to compile nono. Install them like this:
```sh
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```
See the [Rust and Cargo installation guide] for details.

Build and locate the executable:

```sh
cargo build && target/debug/nono --help
```


## Usage

`nono` reads puzzles from stdin.

```sh
nono < examples.txt
```

If you don't want to run all puzzles in a file, you can use `sed` do pick out a single line.
For example the 4th line:

```sh
sed -n 4p examples.txt | nono
```

## Themes

`nono` supports a few variations of its output format, a.k.a. themes.

### `unicode`

This is the default theme.
It gives you:
 * Human friendly renderings of puzzle states.
 * A trace of all passes that were computed.
 * A not-quite-as-human-friendly trace of all inferences that were made in each pass.

### `ascii`

This theme is identical to the `unicode` theme except puzzle states are rendered using only ASCII characters.
This is useful for terminals lacking unicode support.

### `brief`

This theme gives you:
 * Puzzle states in the one-line format.
 * No trace of what passes were computed.
 * No trace of what inferences were made.


## One-line format

See the included `examples.txt` for examples and run them through `nono` for interpretation.

[Rust and Cargo installation guide]: https://doc.rust-lang.org/cargo/getting-started/installation.html

