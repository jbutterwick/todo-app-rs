# todo-app-rs
A blazingly Fast Todo App built with Rust

1. Install Rust via Rustup
    1. Unix: `curl https://sh.rustup.rs -sSf | sh`
    2. Windows: `https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe`
2. `cargo build`
3. `cargo run`


Currently, the following commands are supported:

      help    | h                                 Displays this help message
      list    | l                                 Display the todo list
      add     | a  <todo item description>        Adds the item to the todo list
      remove  | rm <item index or description>    Removes the item from the todo list
      done    | d  <item index or description>    Marks the item as done
      flip    | f  <item index or description>    Flips the items done state
      save    | s                                 Saves the entire list to `TODO.md`
      quit    | q                                 Exit the program