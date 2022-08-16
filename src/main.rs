extern crate core;

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

	let todo = Todo {
		item_list: ItemList { items: vec![] },
	};

	match rep_loop(todo, None) {
		Ok(result) => {
			println!("{}", result);
		}
		Err(error) => panic!("error with message: {:?}", error),
	}
}

fn rep_loop(todo: Todo, result: Option<Result<String, Error>>) -> Result<String, Error> {
	let mut command = String::new();

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");

	match todo.dispatch(&command) {
		Output {
			kind: ResponseType::Continue,
			value,
		} => Ok(String::from(value)),
		Output {
			kind: ResponseType::Exit,
			value,
		} => Ok(String::from(value)),
		Output {
			kind: ResponseType::Error,
			value,
		} => Err(Error),
	}
}
