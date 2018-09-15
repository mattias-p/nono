# nono
Nonogram hint dispenser

At this time it only implements very basic techniques and isn't very specific about its hints.


## Getting started

You need Rust and Cargo to compile nono. Install them like this:
```sh
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```
See the [Rust and Cargo installation guide] for details.

Build and locate the executable:

```sh
cargo build
ls target/debug/nono
```


## Usage

`nono` reads puzzles from stdin.

```sh
./nono < examples.txt
```

If you don't want to run all puzzles in a file, you can use `sed` do pick out a single line.
For example the 4th line:

```sh
sed -n 4p examples.txt | ./nono
```


## Format

Puzzles are stored one per line.
See the included `examples.txt` for examples and run them through `nono` for interpretation.

[Rust and Cargo installation guide]: https://doc.rust-lang.org/cargo/getting-started/installation.html
