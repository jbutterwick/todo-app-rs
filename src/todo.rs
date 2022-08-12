use crate::item::*;
use crate::output::ColoredString;
use crate::response::Response;

pub struct Todo {
	items: Vec<Item>,
}

trait Dispatch {
	fn dispatch(input: str) -> Response;
}

impl Dispatch for Todo {
	pub fn dispatch(input: &str) -> Response {
		todo!()
	}
}
