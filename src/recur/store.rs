//! Recurrence sidecar: `todo.xit` -> `todo.recur.toml`. Mirrors `theme.rs`'s
//! resilient load (missing/malformed file -> empty store, app still launches).

use super::rule::Recurrence;
use super::series::{Override, RecurringSeries, Snapshot};
use crate::item::Status;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub struct SeriesStore {
	pub series: Vec<RecurringSeries>,
	pub path: String,
	next_id: u64,
}

pub fn sidecar_path(xit_path: &str) -> String {
	std::path::Path::new(xit_path)
		.with_extension("recur.toml")
		.to_string_lossy()
		.into_owned()
}

impl SeriesStore {
	pub fn load(xit_path: &str) -> Self {
		let path = sidecar_path(xit_path);
		let text = std::fs::read_to_string(&path).unwrap_or_default();
		let file: StoreFile = toml::from_str(&text).unwrap_or_default();
		let series: Vec<RecurringSeries> = file.series.into_iter().filter_map(dto_to_series).collect();
		let next_id = series.iter().map(|s| s.id).max().map_or(0, |m| m + 1);
		Self {
			series,
			path,
			next_id,
		}
	}

	pub fn save(&self) -> Result<(), String> {
		let file = StoreFile {
			series: self.series.iter().map(series_to_dto).collect(),
		};
		let text = toml::to_string_pretty(&file).map_err(|e| e.to_string())?;
		std::fs::write(&self.path, text).map_err(|e| format!("failed to write {}: {e}", self.path))
	}

	fn next_id(&mut self) -> u64 {
		let id = self.next_id;
		self.next_id += 1;
		id
	}

	/// Create a new series, returning its id.
	pub fn add(&mut self, title: String, priority: i8, rule: Recurrence, anchor: NaiveDate) -> u64 {
		let id = self.next_id();
		self.series
			.push(RecurringSeries::new(id, title, priority, rule, anchor));
		id
	}

	pub fn get(&self, id: u64) -> Option<&RecurringSeries> {
		self.series.iter().find(|s| s.id == id)
	}

	pub fn get_mut(&mut self, id: u64) -> Option<&mut RecurringSeries> {
		self.series.iter_mut().find(|s| s.id == id)
	}

	pub fn remove(&mut self, id: u64) {
		self.series.retain(|s| s.id != id);
	}

	/// This-and-future split at `d`; returns the new (future) series id.
	pub fn split(&mut self, id: u64, d: NaiveDate) -> Option<u64> {
		let new_id = self.next_id();
		let idx = self.series.iter().position(|s| s.id == id)?;
		let future = self.series[idx].split_at(d, new_id);
		self.series.push(future);
		Some(new_id)
	}
}

// ---- serde DTOs (own the TOML shape; persist the RRULE string, dates as Y-M-D) ----

#[derive(Default, Serialize, Deserialize)]
struct StoreFile {
	#[serde(default)]
	series: Vec<SeriesDto>,
}

#[derive(Serialize, Deserialize)]
struct SeriesDto {
	id: u64,
	title: String,
	#[serde(default)]
	priority: i8,
	rrule: String,
	anchor: String,
	start: String,
	#[serde(default)]
	end: Option<String>,
	#[serde(default)]
	exdates: Vec<String>,
	#[serde(default)]
	overrides: Vec<OverrideDto>,
}

#[derive(Serialize, Deserialize)]
struct OverrideDto {
	date: String,
	state: String, // "[x]" etc. (Status::as_str trimmed)
	#[serde(default)]
	title: Option<String>, // present only for frozen Done snapshots
	#[serde(default)]
	priority: Option<i8>,
}

fn fmt_date(d: NaiveDate) -> String {
	d.format("%Y-%m-%d").to_string()
}
fn parse_date(s: &str) -> Option<NaiveDate> {
	NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}
fn state_code(s: &Status) -> String {
	s.as_str().trim().to_string()
}

fn series_to_dto(s: &RecurringSeries) -> SeriesDto {
	SeriesDto {
		id: s.id,
		title: s.title.clone(),
		priority: s.priority,
		rrule: s.rule.to_rrule(),
		anchor: fmt_date(s.anchor),
		start: fmt_date(s.start),
		end: s.end.map(fmt_date),
		exdates: s.exdates.iter().map(|d| fmt_date(*d)).collect(),
		overrides: s
			.overrides
			.iter()
			.map(|(date, ov)| match ov {
				Override::State(st) => OverrideDto {
					date: fmt_date(*date),
					state: state_code(st),
					title: None,
					priority: None,
				},
				Override::Done(snap) => OverrideDto {
					date: fmt_date(*date),
					state: state_code(&snap.state),
					title: Some(snap.title.clone()),
					priority: Some(snap.priority),
				},
			})
			.collect(),
	}
}

fn dto_to_series(dto: SeriesDto) -> Option<RecurringSeries> {
	let rule = Recurrence::from_rrule(&dto.rrule).ok()?;
	let anchor = parse_date(&dto.anchor)?;
	let start = parse_date(&dto.start)?;
	let mut s = RecurringSeries::new(dto.id, dto.title, dto.priority, rule, anchor);
	s.start = start;
	s.end = dto.end.as_deref().and_then(parse_date);
	s.exdates = dto.exdates.iter().filter_map(|x| parse_date(x)).collect();
	for o in dto.overrides {
		let (Some(date), Some(state)) = (parse_date(&o.date), Status::from_str(&o.state)) else {
			continue;
		};
		let ov = match o.title {
			Some(title) => Override::Done(Snapshot {
				title,
				priority: o.priority.unwrap_or(0),
				state,
			}),
			None => Override::State(state),
		};
		s.overrides.insert(date, ov);
	}
	Some(s)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sidecar_path_cases() {
		assert_eq!(sidecar_path("todo.xit"), "todo.recur.toml");
		assert_eq!(sidecar_path("/a/b/foo.xit"), "/a/b/foo.recur.toml");
		assert_eq!(sidecar_path("notes"), "notes.recur.toml");
	}

	#[test]
	fn save_then_load_round_trips() {
		let path = "/tmp/__recur_test.xit";
		let _ = std::fs::remove_file(sidecar_path(path));
		let mut store = SeriesStore::load(path);
		let anchor = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
		let id = store.add(
			"Water".into(),
			2,
			Recurrence::from_rrule("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO").unwrap(),
			anchor,
		);
		let s = store.get_mut(id).unwrap();
		s.skip(NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
		s.set_override(
			NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
			Override::Done(Snapshot {
				title: "Water".into(),
				priority: 2,
				state: Status::Checked,
			}),
		);
		s.set_override(
			NaiveDate::from_ymd_opt(2026, 6, 29).unwrap(),
			Override::State(Status::Ongoing),
		);
		store.save().unwrap();

		let loaded = SeriesStore::load(path);
		assert_eq!(loaded.series.len(), 1);
		let ls = loaded.get(id).unwrap();
		assert_eq!(ls.title, "Water");
		assert_eq!(ls.rule.to_rrule(), "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO");
		assert_eq!(ls.priority, 2);
		assert!(ls.exdates.contains(&NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()));
		assert_eq!(ls.overrides.len(), 2);
		let _ = std::fs::remove_file(sidecar_path(path));
	}

	#[test]
	fn missing_file_is_empty_store() {
		let store = SeriesStore::load("/tmp/__definitely_absent_recur.xit");
		assert!(store.series.is_empty());
	}
}
