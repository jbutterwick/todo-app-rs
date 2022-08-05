mod item;
mod output;
mod todo;

use std::io;

fn main() {
	println!("Enter a command. Enter help to list available commands: ");

	let mut command = String::new(); // Defined an empty String

	// str vs String
	// String::new()

	io::stdin()
		.read_line(&mut command)
		.expect("Failed to read command");

	println!("You entered: {command}");
}
