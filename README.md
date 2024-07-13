# Zayka

The Key absorber for [Zay].

[Zay]: (https://github.com/anametologin/zay)

## About

It's simple one pixel window application for getting user keyboard input and send it to Kwin script: [Zay]

## How it works

- Kwin script [Zay] registers shortcut
- after the shortcut called, [Zay] makes this window app active
- [Zay] showing `hop lables` on all normal windows in current virtual desktop
- [Zay] asks every 200ms via DBus this app with hope to receive user input (I will try to use DbusSignal)
- [Zay] makes appropriate window active based on user input
- for security reasons allowed only one letter replies: 'a', 'aa', 'bbbbbb' etc... and '#escape'

## Compilation

1. Install Rust by following the [Rust Getting Started Guide](https://www.rust-lang.org/learn/get-started).
   Once this is done, you should have the `rustc` compiler and the `cargo` build system installed in your path.
2. Compile with comand
   ```
   cargo build -r
   ```
3. Path to binary: `./target/release/zayka`
