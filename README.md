# todo-app-rs
A blazingly Fast Todo App built with Rust

1. Install Rust via Rustup
    1. Unix: `curl https://sh.rustup.rs -sSf | sh`
    2. Windows: `https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe`
2. `cargo build`
3. `cargo run`


Currently, the following commands are supported:

     help                              Displays this help message
     list                              Display the todo list
     add <todo item description>       Adds the item to the todo list
     done <todo item number>           Marks the item as done
     save                              Saves the entire list to `todo.md`
     quit                              Exit the program