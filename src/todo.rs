use crate::item::*;

struct Todo {
	items: Vec<Item>,
}

trait Dispatch {}

impl Dispatch for Todo {}
