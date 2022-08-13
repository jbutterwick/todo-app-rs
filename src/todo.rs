use crate::item::*;
use crate::output::{ColoredString, Output, Outputtable};

use crate::response::*;

pub struct Todo {
	pub(crate) item_list: ItemList,
}

impl Todo {
	pub fn dispatch<T>(mut self, input: &str) -> Output<String> {
		match input {
			_ => {
				self.item_list.items.push(Item::from(input));
				Output {
					kind: &ResponseType::Exit,
					value: &String::from(self.item_list),
				}
			}
		}
	}
}
