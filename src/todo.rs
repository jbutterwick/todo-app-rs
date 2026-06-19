use crate::item::{Item, Status};
use chrono::NaiveDate;
use std::fs;

pub struct Todo {
	pub item_vec: Vec<Item>,
	pub file_path: String,
}

impl Todo {
	pub fn new(file_path: String) -> Self {
		Self {
			item_vec: vec![],
			file_path,
		}
	}

	pub fn from_existing(existing_list: &str, file_path: String) -> Self {
		let item_vec = existing_list
			.lines()
			.filter(|line| !line.is_empty())
			.map(Item::parse_line)
			.collect();
		Self {
			item_vec,
			file_path,
		}
	}

	pub fn save_to_file(&self) -> Result<(), String> {
		let mut file_string = String::new();
		for item in &self.item_vec {
			file_string.push_str(&(item.get_file_string() + "\n"));
		}
		fs::write(&self.file_path, file_string)
			.map_err(|e| format!("failed to write {}: {e}", self.file_path))
	}

	pub fn export_to_md(&self) -> Result<(), String> {
		let mut string = String::new();
		for item in &self.item_vec {
			string.push_str(&(item.get_file_string() + "\n"));
		}
		fs::write("TODO.md", string).map_err(|e| format!("failed to write TODO.md: {e}"))
	}

	// ---- index-based mutations (the UI passes the selected real index) ----

	pub fn add(&mut self, description: String) {
		self.item_vec.push(Item::from(description.as_str()));
	}

	pub fn edit(&mut self, index: usize, description: String) {
		if let Some(item) = self.item_vec.get_mut(index) {
			item.description = description;
		}
	}

	pub fn toggle_done(&mut self, index: usize) {
		if let Some(item) = self.item_vec.get_mut(index) {
			item.state = if item.state == Status::Checked {
				Status::Open
			} else {
				Status::Checked
			};
		}
	}

	pub fn set_status(&mut self, index: usize, status: Status) {
		if let Some(item) = self.item_vec.get_mut(index) {
			item.state = status;
		}
	}

	pub fn adjust_priority(&mut self, index: usize, delta: i8) {
		if let Some(item) = self.item_vec.get_mut(index) {
			item.priority = (item.priority + delta).max(0);
		}
	}

	pub fn set_due_date(&mut self, index: usize, date: Option<NaiveDate>) {
		if let Some(item) = self.item_vec.get_mut(index) {
			item.due_date = date;
		}
	}

	pub fn remove(&mut self, index: usize) {
		if index < self.item_vec.len() {
			self.item_vec.remove(index);
		}
	}

	pub fn sort(&mut self) {
		self.item_vec.sort();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sample() -> Todo {
		let mut t = Todo::new("test.xit".into());
		t.add("buy milk".into());
		t
	}

	#[test]
	fn add_creates_open_zero_priority() {
		let t = sample();
		assert_eq!(t.item_vec.len(), 1);
		assert_eq!(t.item_vec[0].state, Status::Open);
		assert_eq!(t.item_vec[0].priority, 0);
		assert_eq!(t.item_vec[0].description, "buy milk");
	}

	#[test]
	fn toggle_done_flips() {
		let mut t = sample();
		t.toggle_done(0);
		assert_eq!(t.item_vec[0].state, Status::Checked);
		t.toggle_done(0);
		assert_eq!(t.item_vec[0].state, Status::Open);
	}

	#[test]
	fn set_status_and_edit() {
		let mut t = sample();
		t.set_status(0, Status::Ongoing);
		assert_eq!(t.item_vec[0].state, Status::Ongoing);
		t.edit(0, "buy oat milk".into());
		assert_eq!(t.item_vec[0].description, "buy oat milk");
	}

	#[test]
	fn priority_clamps_at_zero() {
		let mut t = sample();
		t.adjust_priority(0, 2);
		assert_eq!(t.item_vec[0].priority, 2);
		t.adjust_priority(0, -5);
		assert_eq!(t.item_vec[0].priority, 0);
	}

	#[test]
	fn due_date_set_and_clear() {
		let mut t = sample();
		let d = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
		t.set_due_date(0, Some(d));
		assert_eq!(t.item_vec[0].due_date, Some(d));
		t.set_due_date(0, None);
		assert_eq!(t.item_vec[0].due_date, None);
	}

	#[test]
	fn remove_shifts() {
		let mut t = sample();
		t.add("walk dog".into());
		t.remove(0);
		assert_eq!(t.item_vec.len(), 1);
		assert_eq!(t.item_vec[0].description, "walk dog");
		t.remove(99); // out of bounds is a no-op
		assert_eq!(t.item_vec.len(), 1);
	}

	#[test]
	fn sort_orders_by_status_then_priority() {
		let mut t = Todo::new("test.xit".into());
		t.add("a done".into());
		t.toggle_done(0);
		t.add("b ongoing".into());
		t.set_status(1, Status::Ongoing);
		t.sort();
		// Ongoing ranks before Checked
		assert_eq!(t.item_vec[0].state, Status::Ongoing);
		assert_eq!(t.item_vec[1].state, Status::Checked);
	}

	#[test]
	fn xit_round_trip_is_idempotent() {
		for line in [
			"[ ]  buy milk",
			"[@]  !! ship release -> 2026-07-01",
			"[x]  fix a -> b mapping", // "->" in the description, no real date
		] {
			let item = Item::parse_line(line);
			assert_eq!(item.get_file_string(), line, "round trip changed {line:?}");
		}
	}

	#[test]
	fn clean_description_survives_save_load() {
		let mut t = Todo::new("test.xit".into());
		t.add("buy milk".into());
		t.adjust_priority(0, 2);
		t.set_due_date(0, NaiveDate::from_ymd_opt(2026, 7, 1));
		let parsed = Item::parse_line(&t.item_vec[0].get_file_string());
		assert_eq!(parsed.description, "buy milk"); // no leading spaces / markers
		assert_eq!(parsed.priority, 2);
		assert_eq!(parsed.due_date, NaiveDate::from_ymd_opt(2026, 7, 1));
	}
}
