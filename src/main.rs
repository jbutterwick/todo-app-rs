mod item;
mod output;
mod response;
mod todo;

use crate::todo::Todo;
use std::io;

fn main() {
	println!("Todo List");
	println!("Enter a command. Enter `help` to list available commands: ");

	let mut command = String::new(); // Defined an empty String on the Heap

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");

	let todo = Todo { items: vec![] };

	let response = todo.dispatch(&command);
}
