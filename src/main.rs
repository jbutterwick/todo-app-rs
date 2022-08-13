mod item;
mod output;
mod response;
mod todo;

use crate::item::ItemList;
use crate::todo::Todo;
use std::io;

fn main() {
	println!("Todo List");
	println!("Enter a command. Enter `help` to list available commands: ");

	let mut command = String::new();

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");

	let todo = Todo {
		item_list: ItemList { items: vec![] },
	};

	println!("{}", todo.dispatch::<String>(&command))
}
