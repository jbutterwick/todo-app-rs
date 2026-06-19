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
      x / Space        toggle done          D       set due date (picker)
      o                undo (mark open)      > / <   priority + / -
      @                ongoing               r / -   remove (confirm y/n)
      ~                obsolete              s       sort (status, priority, name)
      i                question              f       cycle status filter
      c                calendar view         t       theme picker
      h / F1           help                  q / Esc quit (saves + exports TODO.md)

In a popup: type the text, **Enter** to confirm, **Esc** to cancel.

The **due-date picker** (`D`) offers both a calendar and a text field. It opens
in text mode — type `YYYY-MM-DD` (empty clears the date). Press **Tab** to switch
to the calendar and pick a day with the arrow keys (`←`/`→` day, `↑`/`↓` week,
`[`/`]` month); **Enter** sets whichever is active.

## Calendar (`c`)

A month grid (left) with every due date highlighted, beside a list (right) of the
items due on the cursor's day. Move the day cursor with the arrow keys (`←`/`→` a
day, `↑`/`↓` a week), `[` / `]` to change month.

The calendar isn't just a view — every editing action works here too. `j` / `k`
select an item in the due list, and the same keys as the main list act on it
(`x` done, `o`/`@`/`~`/`i` status, `>`/`<` priority, `e` edit, `D` due date,
`r` remove, `s` sort). `a` / `+` adds a todo **due on the cursor's date**.

`c` or `Esc` returns to the main list.

## Themes (`t`)

Press `t` to open the theme picker. Scroll with `j` / `k` (or arrows) and the UI
**previews each theme live** as you move; **Enter** keeps the highlighted theme,
**Esc** reverts to the one you started on.

Themes are defined in `themes.toml` — bootstrapped with the Catppuccin family
(Latte / Frappé / Macchiato / Mocha), Monokai, Dracula, Nord, Gruvbox Dark,
Tokyo Night, Solarized Dark, and Rosé Pine. Add your own by appending a
`[[theme]]` block; colours are hex (`#rrggbb`) or ratatui names. The app reads
`./themes.toml` if present (edit and restart to apply), otherwise a copy baked in
at build time. The selected theme is not persisted across runs.

## File format (`.xit`)

One item per line: a status marker, an optional priority, the description, and an
optional `-> YYYY-MM-DD` due date.

      [ ]  open task
      [@]  !! ongoing task with priority -> 2026-07-01
      [x]  done task

Status markers: `[ ]` open, `[@]` ongoing, `[x]` done, `[~]` obsolete, `[?]` question.
Priority is a run of `!` (more `!` = higher); `>` / `<` adjust it.
