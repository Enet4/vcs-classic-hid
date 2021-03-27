## simon

A game of Simon Says!
Follow the sequence instructed by the controller!

### Running

In the project's root folder:

```sh
cargo run --release --bin vcs-classic-hid-simon
```

Or to run this game in simulated mode (no device required):

```sh
cargo run --release --bin vcs-classic-hid-simon --features simulator
```

### How to play

- Press the primary button or the menu button to start.
- Pay close attention to the sequence of directions presented.
- Move the stick in either of the 4 directions to repeat the sequence.
- There is no practical limit to the sequence. One failure, it's game over.
- Press the Fuji button to end the program.
