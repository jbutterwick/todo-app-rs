# todo-app-rs
A blazingly fast keyboard-driven todo TUI built with Rust (powered by [ratatui](https://ratatui.rs)).

## Install & run

1. Install Rust via Rustup
    1. Unix: `curl https://sh.rustup.rs -sSf | sh`
    2. Windows: `https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe`
2. `cargo run` — opens `todo.xit` in the current directory
3. `cargo run -- path/to/list.xit` — open a specific file

The list loads on start, **auto-saves on every change**, and exports a plain-text
`TODO.md` on quit.

## Keys

Navigate the list and act on the **selected** item with single keys. Press `h`
(or `F1`) any time for this list.

      j / ↓   k / ↑   move selection        a / +   add (popup)
      g / G           top / bottom          e       edit selected (popup)
      x / Space        toggle done          D       set due date (popup)
      o                undo (mark open)      > / <   priority + / -
      @                ongoing               r / -   remove (confirm y/n)
      ~                obsolete              s       sort (status, priority, name)
      i                question              f       cycle status filter
      h / F1           help                  q / Esc quit (saves + exports TODO.md)

In a popup: type the text, **Enter** to confirm, **Esc** to cancel. An empty
due-date entry clears the date.

## File format (`.xit`)

One item per line: a status marker, an optional priority, the description, and an
optional `-> YYYY-MM-DD` due date.

      [ ]  open task
      [@]  !! ongoing task with priority -> 2026-07-01
      [x]  done task

Status markers: `[ ]` open, `[@]` ongoing, `[x]` done, `[~]` obsolete, `[?]` question.
Priority is a run of `!` (more `!` = higher); `>` / `<` adjust it.
