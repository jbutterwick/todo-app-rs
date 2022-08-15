# todo-app-rs
A blazingly Fast Todo App built with Rust

1. Install Rust via Rustup
    1. Unix: `curl https://sh.rustup.rs -sSf | sh`
    2. Windows: `https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe`
2. `cargo build`
3. `cargo run`

3 lifetime ellision rules:
1. each parameter that is a reference will get its own lifetime parameter
2. if there is exactly one input lifetime parameter, that lifetime is assigned to all output lifetime parameters
3. if there are multiple input lifetime parameters, but one of them is &self or &mut self the lifetime of self is assigned to all output lifetime parameters

in any other case, you will need to explicitly define lifetimes.

functions cannot return a reference to data created inside the function, because when the function is finished, any variables created within it die.