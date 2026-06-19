//! A recurring series: a rule plus per-occurrence state, with this-and-future
//! split editing. Occurrences are computed on the fly (no materialization).

use super::engine;
use super::rule::Recurrence;
use crate::item::Status;
use chrono::NaiveDate;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::RangeInclusive;

/// A frozen snapshot captured when an occurrence is completed/marked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
	pub title: String,
	pub priority: i8,
	pub state: Status,
}

/// Per-occurrence state on a specific date.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Override {
	/// A non-frozen status (e.g. Ongoing) — still follows the series title.
	State(Status),
	/// A completed occurrence: title/priority/state frozen at completion time.
	Done(Snapshot),
}

#[derive(Clone, Debug)]
pub struct RecurringSeries {
	pub id: u64,
	pub title: String,
	pub priority: i8,
	pub rule: Recurrence,
	pub anchor: NaiveDate, // phase origin — never changes (preserves cadence across splits)
	pub start: NaiveDate,  // first active date (== anchor for the original; == D after a split)
	pub end: Option<NaiveDate>, // extra hard cap beyond rule.until (used by splits)
	pub exdates: BTreeSet<NaiveDate>,
	pub overrides: BTreeMap<NaiveDate, Override>,
}

impl RecurringSeries {
	pub fn new(id: u64, title: String, priority: i8, rule: Recurrence, anchor: NaiveDate) -> Self {
		Self {
			id,
			title,
			priority,
			rule,
			anchor,
			start: anchor,
			end: None,
			exdates: BTreeSet::new(),
			overrides: BTreeMap::new(),
		}
	}

	/// Occurrence dates within `window`, floored at `start`, capped at `end`,
	/// minus skipped dates. Phase is computed from `anchor`.
	pub fn occurrences_in(&self, window: RangeInclusive<NaiveDate>) -> Vec<NaiveDate> {
		let lo = (*window.start()).max(self.start);
		let hi = self.end.map_or(*window.end(), |e| (*window.end()).min(e));
		if lo > hi {
			return vec![];
		}
		let mut occ = engine::occurrences(&self.rule, self.anchor, lo..=hi);
		occ.retain(|d| !self.exdates.contains(d));
		occ
	}

	pub fn occurs_on(&self, date: NaiveDate) -> bool {
		self.occurrences_in(date..=date).contains(&date)
	}

	/// Effective (title, priority, state) shown for an occurrence on `date`.
	pub fn view(&self, date: NaiveDate) -> (String, i8, Status) {
		match self.overrides.get(&date) {
			Some(Override::Done(s)) => (s.title.clone(), s.priority, s.state.clone()),
			Some(Override::State(st)) => (self.title.clone(), self.priority, st.clone()),
			None => (self.title.clone(), self.priority, Status::Open),
		}
	}

	pub fn set_override(&mut self, date: NaiveDate, ov: Override) {
		self.overrides.insert(date, ov);
	}

	pub fn clear_override(&mut self, date: NaiveDate) {
		self.overrides.remove(&date);
	}

	pub fn skip(&mut self, date: NaiveDate) {
		self.exdates.insert(date);
	}

	/// Split at `d`: cap this series at `d-1` and return a new series active from
	/// `d` that carries the SAME cadence phase (original `anchor`). Overrides and
	/// exdates on/after `d` move to the new series; the caller applies the edit to it.
	pub fn split_at(&mut self, d: NaiveDate, new_id: u64) -> RecurringSeries {
		self.resolve_count_to_until(); // make the split purely date-based

		let mut future = self.clone();
		future.id = new_id;
		future.start = d;
		future.overrides = self.overrides.split_off(&d); // keys >= d move to future
		future.exdates = self.exdates.split_off(&d);

		let cap = d.pred_opt().unwrap_or(d);
		self.end = Some(self.end.map_or(cap, |e| e.min(cap)));
		self.rule.until = Some(self.rule.until.map_or(cap, |u| u.min(cap)));
		future
	}

	/// Delete this occurrence and all later ones (cap the series at `d-1`).
	pub fn truncate_before(&mut self, d: NaiveDate) {
		self.resolve_count_to_until();
		let cap = d.pred_opt().unwrap_or(d);
		self.end = Some(self.end.map_or(cap, |e| e.min(cap)));
		self.rule.until = Some(self.rule.until.map_or(cap, |u| u.min(cap)));
		let _ = self.overrides.split_off(&d);
		let _ = self.exdates.split_off(&d);
	}

	/// Convert a COUNT rule to an equivalent UNTIL so splits are date-based.
	fn resolve_count_to_until(&mut self) {
		if self.rule.count.is_none() {
			return;
		}
		let all = engine::occurrences(&self.rule, self.anchor, self.anchor..=NaiveDate::MAX);
		if let Some(&last) = all.last() {
			self.rule.until = Some(self.rule.until.map_or(last, |u| u.min(last)));
		}
		self.rule.count = None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn d(y: i32, m: u32, day: u32) -> NaiveDate {
		NaiveDate::from_ymd_opt(y, m, day).unwrap()
	}
	fn series(rrule: &str, anchor: NaiveDate) -> RecurringSeries {
		RecurringSeries::new(1, "Water".into(), 0, Recurrence::from_rrule(rrule).unwrap(), anchor)
	}

	#[test]
	fn occurrences_floor_at_start_and_skip_exdates() {
		let mut s = series("FREQ=WEEKLY;BYDAY=MO", d(2026, 6, 1));
		s.start = d(2026, 6, 15); // pretend split
		s.skip(d(2026, 6, 22));
		let got = s.occurrences_in(d(2026, 6, 1)..=d(2026, 6, 30));
		assert_eq!(got, vec![d(2026, 6, 15), d(2026, 6, 29)]); // 6/1,6/8 floored; 6/22 skipped
	}

	#[test]
	fn view_reflects_overrides_and_freezes_done() {
		let mut s = series("FREQ=DAILY", d(2026, 6, 1));
		s.set_override(
			d(2026, 6, 2),
			Override::Done(Snapshot {
				title: "Old title".into(),
				priority: 1,
				state: Status::Checked,
			}),
		);
		s.set_override(d(2026, 6, 3), Override::State(Status::Ongoing));
		s.title = "New title".into();

		// frozen snapshot keeps the old title; State override follows current title
		assert_eq!(s.view(d(2026, 6, 2)).0, "Old title");
		assert_eq!(s.view(d(2026, 6, 2)).2, Status::Checked);
		assert_eq!(s.view(d(2026, 6, 3)), ("New title".into(), 0, Status::Ongoing));
		assert_eq!(s.view(d(2026, 6, 4)), ("New title".into(), 0, Status::Open));
	}

	#[test]
	fn split_preserves_phase_and_partitions_overrides() {
		// every other week on Monday, anchored 2026-06-01
		let mut s = series("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO", d(2026, 6, 1));
		s.set_override(d(2026, 6, 1), Override::State(Status::Ongoing)); // before split
		s.set_override(d(2026, 6, 29), Override::State(Status::Ongoing)); // on/after split

		let mut future = s.split_at(d(2026, 6, 29), 2);
		future.title = "Renamed".into();

		// old half stops before the split; phase intact (6/1, 6/15)
		assert_eq!(
			s.occurrences_in(d(2026, 6, 1)..=d(2026, 12, 31)),
			vec![d(2026, 6, 1), d(2026, 6, 15)]
		);
		// future half keeps the SAME alternating phase (6/29, 7/13, ...), not re-anchored
		let f = future.occurrences_in(d(2026, 6, 1)..=d(2026, 7, 31));
		assert_eq!(f, vec![d(2026, 6, 29), d(2026, 7, 13), d(2026, 7, 27)]);

		// overrides partitioned at the split date
		assert!(s.overrides.contains_key(&d(2026, 6, 1)));
		assert!(!s.overrides.contains_key(&d(2026, 6, 29)));
		assert!(future.overrides.contains_key(&d(2026, 6, 29)));
		assert_eq!(future.title, "Renamed");
	}

	#[test]
	fn truncate_before_caps_series() {
		let mut s = series("FREQ=DAILY", d(2026, 6, 1));
		s.truncate_before(d(2026, 6, 4));
		assert_eq!(
			s.occurrences_in(d(2026, 6, 1)..=d(2026, 6, 30)),
			vec![d(2026, 6, 1), d(2026, 6, 2), d(2026, 6, 3)]
		);
	}

	#[test]
	fn count_resolves_to_until_on_split() {
		let mut s = series("FREQ=DAILY;COUNT=10", d(2026, 6, 1));
		let future = s.split_at(d(2026, 6, 5), 2);
		// old half: 6/1..6/4 ; future half keeps remaining within original count window
		assert_eq!(s.occurrences_in(d(2026, 6, 1)..=d(2026, 6, 30)).len(), 4);
		assert_eq!(
			future.occurrences_in(d(2026, 6, 1)..=d(2026, 6, 30)),
			vec![d(2026, 6, 5), d(2026, 6, 6), d(2026, 6, 7), d(2026, 6, 8), d(2026, 6, 9), d(2026, 6, 10)]
		);
	}
}
