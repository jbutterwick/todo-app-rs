# todo-app-rs
A blazingly Fast Todo App built with Rust

1. Install Rust via Rustup
    1. Unix: `curl https://sh.rustup.rs -sSf | sh`
    2. Windows: `https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe`
2. `cargo build`
3. `cargo run`


Currently, the following commands are supported:

      help     | h                         Displays this help message
      list     | l                         Display the todo list
      add      | a | + <item description>  Adds the item to the todo list
      remove   | r | - <item>              Removes the item from the todo list
      done     | x <item>                  Marks the item as done
      undo     | o <item>                  Marks the item as not done
      obsolete | ~ <item>                  Marks the item as obsolete
      ongoing  | @ <item>                  Marks the item as ongoing
      question | ? <item>                  Marks the item as question
      duedate  | d <date> <item>           Gives the item a due date
      priority | p <-|+> <priority> <item> Adds or subtracts priority from the item
      save     | s <name=todo.xit>         Saves the entire list to specified filename
      quit     | q                         Exit the program