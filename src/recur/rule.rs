//! The recurrence rule type and its RRULE-string parser/formatter.
//!
//! Date-level subset of RFC 5545 RRULE — no time-of-day. Covers FREQ
//! daily/weekly/monthly/yearly, INTERVAL, BYDAY (with ordinals), BYMONTHDAY,
//! BYMONTH, BYYEARDAY, BYWEEKNO, BYSETPOS, WKST, and COUNT/UNTIL.

use chrono::{NaiveDate, Weekday};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Freq {
	Daily,
	Weekly,
	Monthly,
	Yearly,
}

/// A weekday with an optional ordinal: `1SU` = first Sunday, `-1FR` = last
/// Friday. `ord == None` means "every <weekday>" within the period.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NWeekday {
	pub ord: Option<i8>,
	pub weekday: Weekday,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Recurrence {
	pub freq: Freq,
	pub interval: u32, // >= 1
	pub by_month: Vec<u32>,
	pub by_month_day: Vec<i8>,
	pub by_year_day: Vec<i16>,
	pub by_week_no: Vec<i8>,
	pub by_day: Vec<NWeekday>,
	pub by_set_pos: Vec<i16>,
	pub wkst: Weekday,
	pub count: Option<u32>,
	pub until: Option<NaiveDate>,
}

impl Default for Recurrence {
	fn default() -> Self {
		Self {
			freq: Freq::Daily,
			interval: 1,
			by_month: vec![],
			by_month_day: vec![],
			by_year_day: vec![],
			by_week_no: vec![],
			by_day: vec![],
			by_set_pos: vec![],
			wkst: Weekday::Mon,
			count: None,
			until: None,
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum RruleError {
	Empty,
	MissingFreq,
	UnknownPart(String),
	BadFreq(String),
	BadInt { key: &'static str, val: String },
	BadWeekday(String),
	BadDate { key: &'static str, val: String },
	OutOfRange { key: &'static str, val: i64 },
	ZeroValue { key: &'static str },
}

fn weekday_from_code(s: &str) -> Option<Weekday> {
	Some(match s {
		"SU" => Weekday::Sun,
		"MO" => Weekday::Mon,
		"TU" => Weekday::Tue,
		"WE" => Weekday::Wed,
		"TH" => Weekday::Thu,
		"FR" => Weekday::Fri,
		"SA" => Weekday::Sat,
		_ => return None,
	})
}

fn weekday_code(w: Weekday) -> &'static str {
	match w {
		Weekday::Sun => "SU",
		Weekday::Mon => "MO",
		Weekday::Tue => "TU",
		Weekday::Wed => "WE",
		Weekday::Thu => "TH",
		Weekday::Fri => "FR",
		Weekday::Sat => "SA",
	}
}

fn parse_nweekday(s: &str) -> Result<NWeekday, RruleError> {
	if s.len() < 2 {
		return Err(RruleError::BadWeekday(s.to_string()));
	}
	let (ord_str, wd_str) = s.split_at(s.len() - 2);
	let weekday = weekday_from_code(wd_str).ok_or_else(|| RruleError::BadWeekday(s.to_string()))?;
	let ord = if ord_str.is_empty() {
		None
	} else {
		let n: i8 = ord_str
			.parse()
			.map_err(|_| RruleError::BadWeekday(s.to_string()))?;
		if n == 0 {
			return Err(RruleError::ZeroValue { key: "BYDAY" });
		}
		Some(n)
	};
	Ok(NWeekday { ord, weekday })
}

/// Parse a comma list of signed ints, rejecting 0 and out-of-`range` values.
fn parse_signed_list(
	val: &str,
	key: &'static str,
	min: i64,
	max: i64,
) -> Result<Vec<i64>, RruleError> {
	val.split(',')
		.map(|p| {
			let n: i64 = p
				.trim()
				.parse()
				.map_err(|_| RruleError::BadInt {
					key,
					val: p.to_string(),
				})?;
			if n == 0 {
				return Err(RruleError::ZeroValue { key });
			}
			if n < min || n > max {
				return Err(RruleError::OutOfRange { key, val: n });
			}
			Ok(n)
		})
		.collect()
}

fn parse_until(val: &str) -> Result<NaiveDate, RruleError> {
	let date_part = val.split('T').next().unwrap_or(val);
	NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
		.or_else(|_| NaiveDate::parse_from_str(date_part, "%Y%m%d"))
		.map_err(|_| RruleError::BadDate {
			key: "UNTIL",
			val: val.to_string(),
		})
}

impl Recurrence {
	pub fn from_rrule(s: &str) -> Result<Self, RruleError> {
		let s = s.trim();
		let s = s.strip_prefix("RRULE:").unwrap_or(s).trim();
		if s.is_empty() {
			return Err(RruleError::Empty);
		}
		let mut r = Recurrence::default();
		let mut freq_set = false;
		for part in s.split(';') {
			let part = part.trim();
			if part.is_empty() {
				continue;
			}
			let (key, val) = part
				.split_once('=')
				.ok_or_else(|| RruleError::UnknownPart(part.to_string()))?;
			let key = key.trim().to_ascii_uppercase();
			let val = val.trim();
			match key.as_str() {
				"FREQ" => {
					r.freq = match val.to_ascii_uppercase().as_str() {
						"DAILY" => Freq::Daily,
						"WEEKLY" => Freq::Weekly,
						"MONTHLY" => Freq::Monthly,
						"YEARLY" => Freq::Yearly,
						_ => return Err(RruleError::BadFreq(val.to_string())),
					};
					freq_set = true;
				}
				"INTERVAL" => {
					let n: u32 = val.parse().map_err(|_| RruleError::BadInt {
						key: "INTERVAL",
						val: val.to_string(),
					})?;
					if n == 0 {
						return Err(RruleError::ZeroValue { key: "INTERVAL" });
					}
					r.interval = n;
				}
				"COUNT" => {
					r.count = Some(val.parse().map_err(|_| RruleError::BadInt {
						key: "COUNT",
						val: val.to_string(),
					})?);
				}
				"UNTIL" => r.until = Some(parse_until(val)?),
				"WKST" => {
					r.wkst =
						weekday_from_code(val).ok_or_else(|| RruleError::BadWeekday(val.to_string()))?
				}
				"BYMONTH" => {
					r.by_month = parse_signed_list(val, "BYMONTH", 1, 12)?
						.into_iter()
						.map(|n| n as u32)
						.collect()
				}
				"BYMONTHDAY" => {
					r.by_month_day = parse_signed_list(val, "BYMONTHDAY", -31, 31)?
						.into_iter()
						.map(|n| n as i8)
						.collect()
				}
				"BYYEARDAY" => {
					r.by_year_day = parse_signed_list(val, "BYYEARDAY", -366, 366)?
						.into_iter()
						.map(|n| n as i16)
						.collect()
				}
				"BYWEEKNO" => {
					r.by_week_no = parse_signed_list(val, "BYWEEKNO", -53, 53)?
						.into_iter()
						.map(|n| n as i8)
						.collect()
				}
				"BYSETPOS" => {
					r.by_set_pos = parse_signed_list(val, "BYSETPOS", -366, 366)?
						.into_iter()
						.map(|n| n as i16)
						.collect()
				}
				"BYDAY" => {
					r.by_day = val
						.split(',')
						.map(|p| parse_nweekday(p.trim()))
						.collect::<Result<_, _>>()?
				}
				other => return Err(RruleError::UnknownPart(other.to_string())),
			}
		}
		if !freq_set {
			return Err(RruleError::MissingFreq);
		}
		Ok(r)
	}

	pub fn to_rrule(&self) -> String {
		let mut parts = vec![format!(
			"FREQ={}",
			match self.freq {
				Freq::Daily => "DAILY",
				Freq::Weekly => "WEEKLY",
				Freq::Monthly => "MONTHLY",
				Freq::Yearly => "YEARLY",
			}
		)];
		if self.interval > 1 {
			parts.push(format!("INTERVAL={}", self.interval));
		}
		if self.wkst != Weekday::Mon {
			parts.push(format!("WKST={}", weekday_code(self.wkst)));
		}
		let join = |v: &[i64]| {
			v.iter()
				.map(|n| n.to_string())
				.collect::<Vec<_>>()
				.join(",")
		};
		if !self.by_month.is_empty() {
			parts.push(format!(
				"BYMONTH={}",
				join(&self.by_month.iter().map(|&n| n as i64).collect::<Vec<_>>())
			));
		}
		if !self.by_week_no.is_empty() {
			parts.push(format!(
				"BYWEEKNO={}",
				join(&self.by_week_no.iter().map(|&n| n as i64).collect::<Vec<_>>())
			));
		}
		if !self.by_year_day.is_empty() {
			parts.push(format!(
				"BYYEARDAY={}",
				join(&self.by_year_day.iter().map(|&n| n as i64).collect::<Vec<_>>())
			));
		}
		if !self.by_month_day.is_empty() {
			parts.push(format!(
				"BYMONTHDAY={}",
				join(&self.by_month_day.iter().map(|&n| n as i64).collect::<Vec<_>>())
			));
		}
		if !self.by_day.is_empty() {
			let s = self
				.by_day
				.iter()
				.map(|nwd| match nwd.ord {
					Some(o) => format!("{o}{}", weekday_code(nwd.weekday)),
					None => weekday_code(nwd.weekday).to_string(),
				})
				.collect::<Vec<_>>()
				.join(",");
			parts.push(format!("BYDAY={s}"));
		}
		if !self.by_set_pos.is_empty() {
			parts.push(format!(
				"BYSETPOS={}",
				join(&self.by_set_pos.iter().map(|&n| n as i64).collect::<Vec<_>>())
			));
		}
		if let Some(u) = self.until {
			parts.push(format!("UNTIL={}", u.format("%Y%m%d")));
		} else if let Some(c) = self.count {
			parts.push(format!("COUNT={c}"));
		}
		parts.join(";")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn nwd(ord: Option<i8>, weekday: Weekday) -> NWeekday {
		NWeekday { ord, weekday }
	}

	#[test]
	fn parses_common_rules() {
		let r = Recurrence::from_rrule("FREQ=WEEKLY;INTERVAL=2;BYDAY=MO").unwrap();
		assert_eq!(r.freq, Freq::Weekly);
		assert_eq!(r.interval, 2);
		assert_eq!(r.by_day, vec![nwd(None, Weekday::Mon)]);

		let r = Recurrence::from_rrule("FREQ=MONTHLY;BYDAY=1SU").unwrap();
		assert_eq!(r.by_day, vec![nwd(Some(1), Weekday::Sun)]);

		let r = Recurrence::from_rrule("FREQ=MONTHLY;BYDAY=-1FR;BYSETPOS=-1").unwrap();
		assert_eq!(r.by_day, vec![nwd(Some(-1), Weekday::Fri)]);
		assert_eq!(r.by_set_pos, vec![-1]);

		let r = Recurrence::from_rrule("FREQ=YEARLY;BYMONTH=1,7;BYMONTHDAY=15").unwrap();
		assert_eq!(r.by_month, vec![1, 7]);
		assert_eq!(r.by_month_day, vec![15]);
	}

	#[test]
	fn accepts_rrule_prefix_and_until_formats() {
		let r = Recurrence::from_rrule("RRULE:FREQ=DAILY;UNTIL=20261231").unwrap();
		assert_eq!(r.until, NaiveDate::from_ymd_opt(2026, 12, 31));
		let r = Recurrence::from_rrule("FREQ=DAILY;UNTIL=2026-12-31T00:00:00Z").unwrap();
		assert_eq!(r.until, NaiveDate::from_ymd_opt(2026, 12, 31));
	}

	#[test]
	fn round_trips() {
		for s in [
			"FREQ=DAILY;INTERVAL=2",
			"FREQ=WEEKLY;INTERVAL=2;BYDAY=MO",
			"FREQ=MONTHLY;BYDAY=1SU",
			"FREQ=MONTHLY;BYDAY=-1FR;BYSETPOS=-1",
			"FREQ=MONTHLY;BYMONTHDAY=-2",
			"FREQ=YEARLY;BYMONTH=1,7;BYMONTHDAY=15",
			"FREQ=WEEKLY;WKST=SU;BYDAY=SU,SA",
			"FREQ=DAILY;COUNT=5",
		] {
			let r = Recurrence::from_rrule(s).unwrap();
			assert_eq!(r.to_rrule(), s, "round trip changed {s:?}");
			assert_eq!(Recurrence::from_rrule(&r.to_rrule()).unwrap(), r);
		}
	}

	#[test]
	fn rejects_bad_input() {
		assert_eq!(Recurrence::from_rrule(""), Err(RruleError::Empty));
		assert_eq!(
			Recurrence::from_rrule("INTERVAL=2"),
			Err(RruleError::MissingFreq)
		);
		assert_eq!(
			Recurrence::from_rrule("FREQ=HOURLY"),
			Err(RruleError::BadFreq("HOURLY".into()))
		);
		assert_eq!(
			Recurrence::from_rrule("FREQ=DAILY;INTERVAL=0"),
			Err(RruleError::ZeroValue { key: "INTERVAL" })
		);
		assert_eq!(
			Recurrence::from_rrule("FREQ=MONTHLY;BYMONTHDAY=0"),
			Err(RruleError::ZeroValue { key: "BYMONTHDAY" })
		);
		assert!(matches!(
			Recurrence::from_rrule("FREQ=WEEKLY;BYDAY=XX"),
			Err(RruleError::BadWeekday(_))
		));
		assert!(matches!(
			Recurrence::from_rrule("FREQ=DAILY;FOO=1"),
			Err(RruleError::UnknownPart(_))
		));
	}
}
