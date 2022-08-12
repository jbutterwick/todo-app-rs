use crate::item::*;

use crate::response::{Response, ResponseType};

pub struct Todo {
	pub(crate) items: Vec<Item>,
}

impl Todo {
	pub fn dispatch(mut self, input: &str) -> Response {
		match input {
			_ => {
				self.items.push(Item::from(input));
				Response {
					kind: ResponseType::Exit,
				}
			}
		}
	}
}
