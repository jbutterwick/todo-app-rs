//! Occurrence generation for a [`Recurrence`].
//!
//! `occurrences(rule, anchor, window)` returns the ascending in-window dates.
//! The caller always passes a finite window (a visible month, or `[anchor,
//! today]` for backfill), which keeps generation bounded.

use super::rule::{Freq, Recurrence};
use chrono::{Datelike, Days, NaiveDate, Weekday};
use std::ops::RangeInclusive;

const MAX_PERIODS: u64 = 400_000; // safety bound against never-matching rules

pub fn occurrences(
	rule: &Recurrence,
	anchor: NaiveDate,
	window: RangeInclusive<NaiveDate>,
) -> Vec<NaiveDate> {
	let win_lo = *window.start();
	let win_hi = *window.end();
	let mut out: Vec<NaiveDate> = Vec::new();
	let mut count_seen: u32 = 0;

	'outer: for k in 0..MAX_PERIODS {
		// Compute this period's start and its candidate dates.
		let (pstart, mut cands) = match rule.freq {
			Freq::Daily => {
				let Some(d) = add_days(anchor, k as i64 * rule.interval as i64) else {
					break;
				};
				let c = if daily_passes(rule, d) { vec![d] } else { vec![] };
				(d, c)
			}
			Freq::Weekly => {
				let ws = week_start(anchor, rule.wkst);
				let Some(pstart) = add_days(ws, k as i64 * rule.interval as i64 * 7) else {
					break;
				};
				(pstart, expand_week(rule, pstart, anchor))
			}
			Freq::Monthly => {
				let Some(pstart) = month_start(anchor, k * rule.interval as u64) else {
					break;
				};
				let c = expand_month(rule, pstart.year(), pstart.month(), anchor);
				(pstart, c)
			}
			Freq::Yearly => {
				let yr = anchor.year() as i64 + k as i64 * rule.interval as i64;
				if !(-262143..=262142).contains(&yr) {
					break;
				}
				let Some(pstart) = NaiveDate::from_ymd_opt(yr as i32, 1, 1) else {
					break;
				};
				(pstart, expand_year(rule, yr as i32, anchor))
			}
		};

		// Termination: once a period starts past the window (or UNTIL) we're done —
		// but with COUNT we must keep walking to count correctly.
		if rule.count.is_none() {
			if let Some(u) = rule.until {
				if pstart > u {
					break;
				}
			}
			if pstart > win_hi {
				break;
			}
		} else if let Some(u) = rule.until {
			if pstart > u {
				break;
			}
		}

		cands.retain(|d| *d >= anchor);
		cands.sort_unstable();
		cands.dedup();
		apply_setpos(&mut cands, &rule.by_set_pos);

		for d in cands {
			if let Some(u) = rule.until {
				if d > u {
					break 'outer;
				}
			}
			if let Some(c) = rule.count {
				if count_seen >= c {
					break 'outer;
				}
				count_seen += 1;
			}
			if d >= win_lo && d <= win_hi {
				out.push(d);
			}
		}
		if let Some(c) = rule.count {
			if count_seen >= c {
				break;
			}
		}
	}

	out.sort_unstable();
	out.dedup();
	out
}

// ---- per-period expansion ----

fn daily_passes(rule: &Recurrence, d: NaiveDate) -> bool {
	if !rule.by_month.is_empty() && !rule.by_month.contains(&d.month()) {
		return false;
	}
	if !rule.by_month_day.is_empty() && !monthday_matches(d, &rule.by_month_day) {
		return false;
	}
	if !rule.by_day.is_empty() && !rule.by_day.iter().any(|n| n.weekday == d.weekday()) {
		return false;
	}
	if !rule.by_year_day.is_empty() && !yearday_matches(d, &rule.by_year_day) {
		return false;
	}
	true
}

fn expand_week(rule: &Recurrence, week_start: NaiveDate, anchor: NaiveDate) -> Vec<NaiveDate> {
	let mut c = vec![];
	for i in 0..7 {
		let Some(d) = add_days(week_start, i) else { continue };
		let weekday_ok = if rule.by_day.is_empty() {
			d.weekday() == anchor.weekday()
		} else {
			rule.by_day.iter().any(|n| n.weekday == d.weekday())
		};
		if !weekday_ok {
			continue;
		}
		if !rule.by_month.is_empty() && !rule.by_month.contains(&d.month()) {
			continue;
		}
		if !rule.by_month_day.is_empty() && !monthday_matches(d, &rule.by_month_day) {
			continue;
		}
		c.push(d);
	}
	c
}

fn expand_month(rule: &Recurrence, year: i32, month: u32, anchor: NaiveDate) -> Vec<NaiveDate> {
	if !rule.by_month.is_empty() && !rule.by_month.contains(&month) {
		return vec![];
	}
	let md_set: Option<Vec<NaiveDate>> = (!rule.by_month_day.is_empty()).then(|| {
		rule.by_month_day
			.iter()
			.filter_map(|&md| resolve_monthday(year, month, md))
			.collect()
	});
	let day_set: Option<Vec<NaiveDate>> = (!rule.by_day.is_empty()).then(|| {
		let mut v = vec![];
		for nwd in &rule.by_day {
			match nwd.ord {
				Some(o) => v.extend(nth_weekday_of_month(year, month, nwd.weekday, o)),
				None => v.extend(weekdays_of_month(year, month, nwd.weekday)),
			}
		}
		v
	});
	match (md_set, day_set) {
		(Some(md), Some(_)) => {
			// RFC: BYMONTHDAY ∩ BYDAY — keep month-days whose weekday is in BYDAY.
			let weekdays: Vec<Weekday> = rule.by_day.iter().map(|n| n.weekday).collect();
			md.into_iter()
				.filter(|d| weekdays.contains(&d.weekday()))
				.collect()
		}
		(Some(md), None) => md,
		(None, Some(day)) => day,
		(None, None) => resolve_monthday(year, month, anchor.day() as i8)
			.into_iter()
			.collect(),
	}
}

fn expand_year(rule: &Recurrence, year: i32, anchor: NaiveDate) -> Vec<NaiveDate> {
	let months: Vec<u32> = if rule.by_month.is_empty() {
		(1..=12).collect()
	} else {
		rule.by_month.clone()
	};
	let mut cands = vec![];
	let mut produced = false;

	for &yd in &rule.by_year_day {
		if let Some(d) = resolve_yearday(year, yd) {
			if rule.by_month.is_empty() || rule.by_month.contains(&d.month()) {
				cands.push(d);
			}
		}
		produced = true;
	}

	if !rule.by_week_no.is_empty() {
		let weekdays: Vec<Weekday> = rule.by_day.iter().map(|n| n.weekday).collect();
		for &wn in &rule.by_week_no {
			for d in week_no_days(year, wn) {
				if (weekdays.is_empty() || weekdays.contains(&d.weekday()))
					&& (rule.by_month.is_empty() || rule.by_month.contains(&d.month()))
				{
					cands.push(d);
				}
			}
		}
		produced = true;
	}

	if !rule.by_month_day.is_empty() {
		for &m in &months {
			for &md in &rule.by_month_day {
				if let Some(d) = resolve_monthday(year, m, md) {
					if rule.by_day.is_empty() || rule.by_day.iter().any(|n| n.weekday == d.weekday())
					{
						cands.push(d);
					}
				}
			}
		}
		produced = true;
	} else if !rule.by_day.is_empty() && rule.by_week_no.is_empty() {
		for nwd in &rule.by_day {
			match nwd.ord {
				Some(o) if rule.by_month.is_empty() => {
					cands.extend(nth_weekday_of_year(year, nwd.weekday, o))
				}
				Some(o) => {
					for &m in &months {
						cands.extend(nth_weekday_of_month(year, m, nwd.weekday, o));
					}
				}
				None => {
					for &m in &months {
						cands.extend(weekdays_of_month(year, m, nwd.weekday));
					}
				}
			}
		}
		produced = true;
	}

	if !produced {
		// plain YEARLY (optionally month-filtered): anchor's day-of-month.
		let ms = if rule.by_month.is_empty() {
			vec![anchor.month()]
		} else {
			months
		};
		for m in ms {
			if let Some(d) = NaiveDate::from_ymd_opt(year, m, anchor.day()) {
				cands.push(d);
			}
		}
	}
	cands
}

fn apply_setpos(cands: &mut Vec<NaiveDate>, setpos: &[i16]) {
	if setpos.is_empty() {
		return;
	}
	let n = cands.len() as i64;
	let mut picked = vec![];
	for &p in setpos {
		let idx = if p > 0 { p as i64 - 1 } else { n + p as i64 };
		if idx >= 0 && idx < n {
			picked.push(cands[idx as usize]);
		}
	}
	picked.sort_unstable();
	picked.dedup();
	*cands = picked;
}

// ---- date helpers ----

fn add_days(date: NaiveDate, n: i64) -> Option<NaiveDate> {
	if n >= 0 {
		date.checked_add_days(Days::new(n as u64))
	} else {
		date.checked_sub_days(Days::new((-n) as u64))
	}
}

fn week_start(date: NaiveDate, wkst: Weekday) -> NaiveDate {
	let offset = (date.weekday().num_days_from_monday() as i64
		- wkst.num_days_from_monday() as i64)
		.rem_euclid(7);
	add_days(date, -offset).unwrap_or(date)
}

fn month_start(anchor: NaiveDate, months_forward: u64) -> Option<NaiveDate> {
	let base = anchor.year() as i64 * 12 + anchor.month0() as i64;
	let total = base.checked_add(months_forward as i64)?;
	let year = total.div_euclid(12);
	let month = total.rem_euclid(12) as u32 + 1;
	if !(-262143..=262142).contains(&year) {
		return None;
	}
	NaiveDate::from_ymd_opt(year as i32, month, 1)
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
	let first_next = if month == 12 {
		NaiveDate::from_ymd_opt(year + 1, 1, 1)
	} else {
		NaiveDate::from_ymd_opt(year, month + 1, 1)
	};
	first_next
		.and_then(|d| d.pred_opt())
		.map(|d| d.day())
		.unwrap_or(28)
}

fn resolve_monthday(year: i32, month: u32, md: i8) -> Option<NaiveDate> {
	let last = last_day_of_month(year, month) as i32;
	let day = if md > 0 {
		md as i32
	} else {
		last + md as i32 + 1
	};
	if day < 1 || day > last {
		return None;
	}
	NaiveDate::from_ymd_opt(year, month, day as u32)
}

fn weekdays_of_month(year: i32, month: u32, weekday: Weekday) -> Vec<NaiveDate> {
	let mut v = vec![];
	let Some(mut d) = NaiveDate::from_ymd_opt(year, month, 1) else {
		return v;
	};
	while d.month() == month {
		if d.weekday() == weekday {
			v.push(d);
		}
		match d.succ_opt() {
			Some(n) => d = n,
			None => break,
		}
	}
	v
}

fn nth_weekday_of_month(year: i32, month: u32, weekday: Weekday, ord: i8) -> Option<NaiveDate> {
	let days = weekdays_of_month(year, month, weekday);
	nth(&days, ord)
}

fn weekdays_of_year(year: i32, weekday: Weekday) -> Vec<NaiveDate> {
	let mut v = vec![];
	let Some(mut d) = NaiveDate::from_ymd_opt(year, 1, 1) else {
		return v;
	};
	while d.year() == year {
		if d.weekday() == weekday {
			v.push(d);
		}
		match d.succ_opt() {
			Some(n) => d = n,
			None => break,
		}
	}
	v
}

fn nth_weekday_of_year(year: i32, weekday: Weekday, ord: i8) -> Option<NaiveDate> {
	let days = weekdays_of_year(year, weekday);
	nth(&days, ord)
}

/// 1-based positive / -1-based negative index into a sorted date list.
fn nth(days: &[NaiveDate], ord: i8) -> Option<NaiveDate> {
	let i = if ord > 0 {
		ord as i64 - 1
	} else {
		days.len() as i64 + ord as i64
	};
	if i >= 0 && (i as usize) < days.len() {
		Some(days[i as usize])
	} else {
		None
	}
}

fn resolve_yearday(year: i32, yd: i16) -> Option<NaiveDate> {
	let len = if NaiveDate::from_ymd_opt(year, 2, 29).is_some() {
		366
	} else {
		365
	};
	let ord = if yd > 0 {
		yd as i32
	} else {
		len + yd as i32 + 1
	};
	if ord < 1 || ord > len {
		return None;
	}
	NaiveDate::from_yo_opt(year, ord as u32)
}

fn monthday_matches(d: NaiveDate, list: &[i8]) -> bool {
	list.iter()
		.filter_map(|&md| resolve_monthday(d.year(), d.month(), md))
		.any(|x| x == d)
}

fn yearday_matches(d: NaiveDate, list: &[i16]) -> bool {
	list.iter()
		.filter_map(|&yd| resolve_yearday(d.year(), yd))
		.any(|x| x == d)
}

/// Best-effort ISO week-number days (exact for the common WKST=MO via chrono iso_week).
fn week_no_days(year: i32, wn: i8) -> Vec<NaiveDate> {
	use std::collections::BTreeMap;
	let mut weeks: BTreeMap<u32, Vec<NaiveDate>> = BTreeMap::new();
	let Some(mut d) = NaiveDate::from_ymd_opt(year, 1, 1) else {
		return vec![];
	};
	let end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(d);
	while d <= end {
		let iso = d.iso_week();
		if iso.year() == year {
			weeks.entry(iso.week()).or_default().push(d);
		}
		match d.succ_opt() {
			Some(n) => d = n,
			None => break,
		}
	}
	let maxw = *weeks.keys().max().unwrap_or(&0);
	let target = if wn > 0 {
		wn as u32
	} else {
		(maxw as i32 + wn as i32 + 1).max(0) as u32
	};
	weeks.get(&target).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::recur::rule::Recurrence;

	fn d(y: i32, m: u32, day: u32) -> NaiveDate {
		NaiveDate::from_ymd_opt(y, m, day).unwrap()
	}
	fn occ(rrule: &str, anchor: NaiveDate, lo: NaiveDate, hi: NaiveDate) -> Vec<NaiveDate> {
		occurrences(&Recurrence::from_rrule(rrule).unwrap(), anchor, lo..=hi)
	}

	#[test]
	fn every_other_day() {
		assert_eq!(
			occ("FREQ=DAILY;INTERVAL=2", d(2026, 6, 1), d(2026, 6, 1), d(2026, 6, 7)),
			vec![d(2026, 6, 1), d(2026, 6, 3), d(2026, 6, 5), d(2026, 6, 7)]
		);
	}

	#[test]
	fn every_two_weeks_on_monday() {
		assert_eq!(
			occ(
				"FREQ=WEEKLY;INTERVAL=2;BYDAY=MO",
				d(2026, 6, 1), // Monday
				d(2026, 6, 1),
				d(2026, 7, 15)
			),
			vec![d(2026, 6, 1), d(2026, 6, 15), d(2026, 6, 29), d(2026, 7, 13)]
		);
	}

	#[test]
	fn first_sunday_of_each_month() {
		let got = occ("FREQ=MONTHLY;BYDAY=1SU", d(2026, 1, 1), d(2026, 1, 1), d(2026, 12, 31));
		assert_eq!(got.len(), 12);
		assert_eq!(got[0], d(2026, 1, 4));
		assert_eq!(got[5], d(2026, 6, 7));
	}

	#[test]
	fn last_weekday_of_month_via_setpos() {
		assert_eq!(
			occ(
				"FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-1",
				d(2026, 6, 1),
				d(2026, 6, 1),
				d(2026, 6, 30)
			),
			vec![d(2026, 6, 30)] // Jun 30 2026 is a Tuesday
		);
	}

	#[test]
	fn second_to_last_day_of_month() {
		assert_eq!(
			occ("FREQ=MONTHLY;BYMONTHDAY=-2", d(2026, 2, 1), d(2026, 2, 1), d(2026, 2, 28)),
			vec![d(2026, 2, 27)]
		);
	}

	#[test]
	fn yearly_jan_and_jul_fifteenth() {
		assert_eq!(
			occ(
				"FREQ=YEARLY;BYMONTH=1,7;BYMONTHDAY=15",
				d(2026, 1, 1),
				d(2026, 1, 1),
				d(2026, 12, 31)
			),
			vec![d(2026, 1, 15), d(2026, 7, 15)]
		);
	}

	#[test]
	fn fifth_monday_missing_month_is_empty() {
		// June 2026 has only 4 Mondays (1,8,15,22,29 -> actually 5). Use Feb 2026 (4 Mondays).
		let got = occ("FREQ=MONTHLY;BYDAY=5MO", d(2026, 2, 1), d(2026, 2, 1), d(2026, 2, 28));
		assert!(got.is_empty(), "Feb 2026 has no 5th Monday, got {got:?}");
	}

	#[test]
	fn monthday_31_skips_short_months() {
		let got = occ("FREQ=MONTHLY;BYMONTHDAY=31", d(2026, 1, 1), d(2026, 1, 1), d(2026, 4, 30));
		assert_eq!(got, vec![d(2026, 1, 31), d(2026, 3, 31)]); // no Feb/Apr
	}

	#[test]
	fn yearly_feb_29_leap_only() {
		let got = occ(
			"FREQ=YEARLY;BYMONTH=2;BYMONTHDAY=29",
			d(2024, 1, 1),
			d(2024, 1, 1),
			d(2030, 12, 31),
		);
		assert_eq!(got, vec![d(2024, 2, 29), d(2028, 2, 29)]);
	}

	#[test]
	fn yearday_negative_one_is_dec_31() {
		assert_eq!(
			occ("FREQ=YEARLY;BYYEARDAY=-1", d(2026, 1, 1), d(2026, 1, 1), d(2026, 12, 31)),
			vec![d(2026, 12, 31)]
		);
	}

	#[test]
	fn count_caps_from_anchor_regardless_of_window() {
		// COUNT=3 from anchor; window starts after the 2nd occurrence.
		let got = occ("FREQ=DAILY;COUNT=3", d(2026, 6, 1), d(2026, 6, 2), d(2026, 6, 30));
		assert_eq!(got, vec![d(2026, 6, 2), d(2026, 6, 3)]); // 6/1 counted but out of window
	}

	#[test]
	fn until_is_inclusive() {
		let got = occ(
			"FREQ=DAILY;UNTIL=20260603",
			d(2026, 6, 1),
			d(2026, 6, 1),
			d(2026, 6, 30),
		);
		assert_eq!(got, vec![d(2026, 6, 1), d(2026, 6, 2), d(2026, 6, 3)]);
	}

	#[test]
	fn weekly_interval_phase_is_anchor_relative() {
		// Anchor Wed Jun 3 2026; every 2 weeks on Wed -> Jun 3, 17, Jul 1.
		assert_eq!(
			occ(
				"FREQ=WEEKLY;INTERVAL=2;BYDAY=WE",
				d(2026, 6, 3),
				d(2026, 6, 1),
				d(2026, 7, 5)
			),
			vec![d(2026, 6, 3), d(2026, 6, 17), d(2026, 7, 1)]
		);
	}

	#[test]
	fn plain_yearly_is_anchor_month_day() {
		assert_eq!(
			occ("FREQ=YEARLY", d(2026, 3, 5), d(2026, 1, 1), d(2028, 12, 31)),
			vec![d(2026, 3, 5), d(2027, 3, 5), d(2028, 3, 5)]
		);
	}
}
