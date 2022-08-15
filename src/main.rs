mod item;
mod output;
mod response;
mod todo;

use crate::item::ItemList;
use crate::output::Output;
use crate::response::ResponseType;
use crate::todo::Todo;
use std::fmt::Error;
use std::io;

fn main() {
	println!("Todo List");
	println!("Enter a command. Enter `help` to list available commands: ");

	match rep_loop() {
		Ok(_) => (),
		Err(_) => (),
	}
}

fn rep_loop() -> Result<String, Error> {
	let mut command = String::new();

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");

	let todo = Todo {
		item_list: ItemList { items: vec![] },
	};

	// this is a turbo fish!!

	match todo.dispatch(&command) {
		Output {
			kind: ResponseType::Continue,
			value,
		} => Ok(String::from(value)),
		Output {
			kind: ResponseType::Exit,
			value,
		} => Ok(String::from(value)),
	}
}
