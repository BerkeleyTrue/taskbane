use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Weekday};

/// WARNING: mostly slopcoded, needs further testing
/// Parse a Taskwarrior-compatible date string relative to `now`.
/// Returns `None` for unrecognised input.
pub fn parse_date(input: &str, now: NaiveDateTime) -> Option<NaiveDateTime> {
    let raw = input.trim();
    let s = raw.to_lowercase();
    // Epoch first (with min-value guard), then ISO, then named — matches libshared order.
    parse_epoch(&s)
        .or_else(|| parse_iso_datetime_ext(raw))
        .or_else(|| parse_iso_datetime_basic(raw))
        .or_else(|| parse_iso_date_ext(raw))
        .or_else(|| parse_iso_date_basic(raw))
        .or_else(|| parse_duration(&s, now))
        .or_else(|| parse_named(&s, now))
}

// ── epoch ─────────────────────────────────────────────────────────────────────

// Valid epoch values must be >= 1980-01-01T00:00:00Z.
// This prevents small numbers (e.g. "12", "20240315") from being parsed as epochs.
// Matches libshared Datetime::parse_epoch EPOCH_MIN_VALUE.
const EPOCH_MIN: i64 = 315_532_800;

fn parse_epoch(s: &str) -> Option<NaiveDateTime> {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        let ts: i64 = s.parse().ok()?;
        if ts >= EPOCH_MIN {
            return DateTime::from_timestamp(ts, 0).map(|dt| dt.naive_utc());
        }
    }
    None
}

// ── ISO formats ───────────────────────────────────────────────────────────────

fn parse_iso_datetime_ext(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
}

fn parse_iso_datetime_basic(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S").ok()
}

fn parse_iso_date_ext(s: &str) -> Option<NaiveDateTime> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
}

fn parse_iso_date_basic(s: &str) -> Option<NaiveDateTime> {
    // Guard: must be exactly 8 digits to avoid colliding with epoch
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        NaiveDate::parse_from_str(s, "%Y%m%d")
            .ok()
            .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
    } else {
        None
    }
}

// ── duration ──────────────────────────────────────────────────────────────────

fn parse_duration(s: &str, now: NaiveDateTime) -> Option<NaiveDateTime> {
    // Split into leading digits and unit suffix
    let split = s.find(|c: char| c.is_alphabetic())?;
    let (num_str, unit) = s.split_at(split);
    let n: i64 = num_str.parse().ok()?;

    let result = match unit {
        "h" | "hour" | "hours" => now + Duration::hours(n),
        "d" | "day" | "days" => now + Duration::days(n),
        "w" | "week" | "weeks" => now + Duration::weeks(n),
        // months/years: use calendar arithmetic via chrono
        "mo" | "month" | "months" => add_months(now, n as i32),
        "y" | "year" | "years" => add_months(now, n as i32 * 12),
        "min" | "mins" | "minute" | "minutes" => now + Duration::minutes(n),
        _ => return None,
    };
    Some(result)
}

fn add_months(dt: NaiveDateTime, months: i32) -> NaiveDateTime {
    let mut year = dt.year();
    let mut month = dt.month() as i32 + months;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let day = dt.day().min(days_in_month(year, month as u32));
    NaiveDate::from_ymd_opt(year, month as u32, day)
        .unwrap()
        .and_time(dt.time())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    (next.unwrap() - NaiveDate::from_ymd_opt(year, month, 1).unwrap()).num_days() as u32
}

// ── named keywords ────────────────────────────────────────────────────────────

const MIN_PREFIX: usize = 3;

/// Returns true if `s` is a valid prefix (length >= min) of `keyword`.
fn matches_prefix(s: &str, keyword: &str, min: usize) -> bool {
    s.len() >= min && keyword.starts_with(s)
}

fn parse_named(s: &str, now: NaiveDateTime) -> Option<NaiveDateTime> {
    let today = now.date();

    // now (exact only)
    if s == "now" {
        return Some(now);
    }

    // yesterday / today / tomorrow
    if matches_prefix(s, "yesterday", MIN_PREFIX) {
        return Some((today - Duration::days(1)).and_hms_opt(0, 0, 0).unwrap());
    }
    if matches_prefix(s, "today", MIN_PREFIX) {
        return Some(today.and_hms_opt(0, 0, 0).unwrap());
    }
    if matches_prefix(s, "tomorrow", MIN_PREFIX) {
        return Some((today + Duration::days(1)).and_hms_opt(0, 0, 0).unwrap());
    }

    // later / someday (later min 3, someday min 4)
    if matches_prefix(s, "later", MIN_PREFIX) {
        return Some(
            NaiveDate::from_ymd_opt(9999, 12, 30)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
    }
    if matches_prefix(s, "someday", 4) {
        return Some(
            NaiveDate::from_ymd_opt(9999, 12, 30)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        );
    }

    type BoundaryFn = fn(NaiveDateTime) -> NaiveDateTime;
    // boundary terms — try longer patterns first to avoid prefix collisions
    // (e.g. soww must be tried before sow)
    let boundary_terms: &[(&str, BoundaryFn)] = &[
        // day
        ("sopd", |now| day_start(now.date() - Duration::days(1))),
        ("sod", |now| day_start(now.date())),
        ("sond", |now| day_start(now.date() + Duration::days(1))),
        ("eopd", |now| day_end(now.date() - Duration::days(1))),
        ("eod", |now| day_end(now.date())),
        ("eond", |now| day_end(now.date() + Duration::days(1))),
        // work-week — translated directly from libshared initializeSo/Eopww using
        // C's tm_wday convention (Sun=0, Mon=1, ..., Sat=6).
        // Note: "soww" on a Sunday gives NEXT Monday (wday=0 → +1 day), matching libshared.
        ("sopww", |now| {
            let w = wday(now.date());
            day_start(now.date() + Duration::days(-6 - w))
        }),
        ("soww", |now| {
            let w = wday(now.date());
            day_start(now.date() + Duration::days(1 - w))
        }),
        ("sonww", |now| {
            let w = wday(now.date());
            day_start(now.date() + Duration::days(8 - w))
        }),
        ("eopww", |now| {
            let w = wday(now.date());
            day_end(now.date() + Duration::days(-w - 2))
        }),
        ("eoww", |now| {
            let w = wday(now.date());
            day_end(now.date() + Duration::days(5 - w))
        }),
        ("eonww", |now| {
            let w = wday(now.date());
            day_end(now.date() + Duration::days(12 - w))
        }),
        // week — using extra = (tm_wday + 6) % 7 (Monday-based offset), matching libshared.
        ("sopw", |now| {
            let e = week_extra(now.date());
            day_start(now.date() + Duration::days(-e - 7))
        }),
        ("sow", |now| {
            let e = week_extra(now.date());
            day_start(now.date() + Duration::days(-e))
        }),
        ("sonw", |now| {
            let e = week_extra(now.date());
            day_start(now.date() + Duration::days(7 - e))
        }),
        ("eopw", |now| {
            let e = week_extra(now.date());
            day_end(now.date() + Duration::days(-e - 1))
        }),
        ("eow", |now| {
            let e = week_extra(now.date());
            day_end(now.date() + Duration::days(6 - e))
        }),
        ("eonw", |now| {
            let e = week_extra(now.date());
            day_end(now.date() + Duration::days(13 - e))
        }),
        // month
        ("sopm", |now| day_start(month_start(now.date(), -1))),
        ("som", |now| day_start(month_start(now.date(), 0))),
        ("sonm", |now| day_start(month_start(now.date(), 1))),
        ("eopm", |now| day_end(month_end(now.date(), -1))),
        ("eom", |now| day_end(month_end(now.date(), 0))),
        ("eonm", |now| day_end(month_end(now.date(), 1))),
        // quarter
        ("sopq", |now| day_start(quarter_start(now.date(), -1))),
        ("soq", |now| day_start(quarter_start(now.date(), 0))),
        ("sonq", |now| day_start(quarter_start(now.date(), 1))),
        ("eopq", |now| day_end(quarter_end(now.date(), -1))),
        ("eoq", |now| day_end(quarter_end(now.date(), 0))),
        ("eonq", |now| day_end(quarter_end(now.date(), 1))),
        // year
        ("sopy", |now| {
            day_start(NaiveDate::from_ymd_opt(now.year() - 1, 1, 1).unwrap())
        }),
        ("soy", |now| {
            day_start(NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap())
        }),
        ("sony", |now| {
            day_start(NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap())
        }),
        ("eopy", |now| {
            day_end(NaiveDate::from_ymd_opt(now.year() - 1, 12, 31).unwrap())
        }),
        ("eoy", |now| {
            day_end(NaiveDate::from_ymd_opt(now.year(), 12, 31).unwrap())
        }),
        ("eony", |now| {
            day_end(NaiveDate::from_ymd_opt(now.year() + 1, 12, 31).unwrap())
        }),
    ];

    for (keyword, f) in boundary_terms {
        if s == *keyword {
            return Some(f(now));
        }
    }

    // day names (min 3 chars, next future occurrence)
    let days = [
        ("sunday", Weekday::Sun),
        ("monday", Weekday::Mon),
        ("tuesday", Weekday::Tue),
        ("wednesday", Weekday::Wed),
        ("thursday", Weekday::Thu),
        ("friday", Weekday::Fri),
        ("saturday", Weekday::Sat),
    ];
    for (name, wd) in days {
        if matches_prefix(s, name, MIN_PREFIX) {
            return Some(day_start(next_weekday(today, wd)));
        }
    }

    // month names (min 3 chars, 1st of next occurrence)
    let months = [
        ("january", 1u32),
        ("february", 2),
        ("march", 3),
        ("april", 4),
        ("may", 5),
        ("june", 6),
        ("july", 7),
        ("august", 8),
        ("september", 9),
        ("october", 10),
        ("november", 11),
        ("december", 12),
    ];
    for (name, mo) in months {
        if matches_prefix(s, name, MIN_PREFIX) {
            return Some(day_start(next_month_occurrence(today, mo)));
        }
    }

    // ordinals: 1st, 2nd, 3rd, 4th..31st
    if let Some(day) = parse_ordinal(s) {
        return Some(day_start(next_ordinal(today, day)));
    }

    None
}

// ── date arithmetic helpers ───────────────────────────────────────────────────

fn day_start(d: NaiveDate) -> NaiveDateTime {
    d.and_hms_opt(0, 0, 0).unwrap()
}

fn day_end(d: NaiveDate) -> NaiveDateTime {
    d.and_hms_opt(23, 59, 59).unwrap()
}

/// Next occurrence of `wd` strictly after `from` (timeRelative=true).
fn next_weekday(from: NaiveDate, wd: Weekday) -> NaiveDate {
    let from_num = from.weekday().num_days_from_monday();
    let target_num = wd.num_days_from_monday();
    let days_ahead = (target_num + 7 - from_num) % 7;
    let days_ahead = if days_ahead == 0 { 7 } else { days_ahead };
    from + Duration::days(days_ahead as i64)
}

/// C's tm_wday equivalent: Sun=0, Mon=1, ..., Sat=6.
fn wday(d: NaiveDate) -> i64 {
    d.weekday().num_days_from_sunday() as i64
}

/// Monday-based extra offset used by libshared week boundary functions:
/// (tm_wday + 6) % 7 → Mon=0, Tue=1, ..., Sun=6.
fn week_extra(d: NaiveDate) -> i64 {
    (wday(d) + 6) % 7
}

fn month_start(d: NaiveDate, offset: i32) -> NaiveDate {
    let mut year = d.year();
    let mut month = d.month() as i32 + offset;
    while month > 12 {
        month -= 12;
        year += 1;
    }
    while month < 1 {
        month += 12;
        year -= 1;
    }
    NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap()
}

fn month_end(d: NaiveDate, offset: i32) -> NaiveDate {
    // Start of the month *after* the target, minus one day
    let start_of_next = month_start(d, offset + 1);
    start_of_next - Duration::days(1)
}

fn quarter_of(month: u32) -> u32 {
    (month - 1) / 3 // 0-based quarter index
}

fn quarter_start(d: NaiveDate, offset: i32) -> NaiveDate {
    let current_q = quarter_of(d.month()) as i32;
    let target_q = current_q + offset;
    let year_offset = target_q.div_euclid(4);
    let q = target_q.rem_euclid(4) as u32;
    let year = d.year() + year_offset;
    let month = q * 3 + 1;
    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
}

fn quarter_end(d: NaiveDate, offset: i32) -> NaiveDate {
    quarter_start(d, offset + 1) - Duration::days(1)
}

fn next_month_occurrence(today: NaiveDate, month: u32) -> NaiveDate {
    let candidate = NaiveDate::from_ymd_opt(today.year(), month, 1).unwrap();
    if candidate > today {
        candidate
    } else {
        NaiveDate::from_ymd_opt(today.year() + 1, month, 1).unwrap()
    }
}

fn next_ordinal(today: NaiveDate, day: u32) -> NaiveDate {
    let candidate = NaiveDate::from_ymd_opt(today.year(), today.month(), day);
    match candidate {
        Some(d) if d > today => d,
        _ => {
            // Advance to next month
            let next = month_start(today, 1);
            // Clamp day to that month's length
            let clamped = day.min(days_in_month(next.year(), next.month()));
            NaiveDate::from_ymd_opt(next.year(), next.month(), clamped).unwrap()
        }
    }
}

fn parse_ordinal(s: &str) -> Option<u32> {
    let (num_str, suffix) = if let Some(stripped) = s.strip_suffix("st") {
        (stripped, "st")
    } else if let Some(stripped) = s.strip_suffix("nd") {
        (stripped, "nd")
    } else if let Some(stripped) = s.strip_suffix("rd") {
        (stripped, "rd")
    } else if let Some(stripped) = s.strip_suffix("th") {
        (stripped, "th")
    } else {
        return None;
    };
    let n: u32 = num_str.parse().ok()?;
    if !(1..=31).contains(&n) {
        return None;
    }
    // Validate suffix matches number (1st, 2nd, 3rd, rest are th)
    let expected = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    if suffix == expected {
        Some(n)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    /// Fixed reference point matching the doc: 2024-03-05T12:34:56 (Tuesday, week 10)
    fn now() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2024, 3, 5)
            .unwrap()
            .and_hms_opt(12, 34, 56)
            .unwrap()
    }

    fn dt(y: i32, mo: u32, d: u32, h: u32, min: u32, s: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, mo, d)
            .unwrap()
            .and_hms_opt(h, min, s)
            .unwrap()
    }

    fn date(y: i32, mo: u32, d: u32) -> NaiveDateTime {
        dt(y, mo, d, 0, 0, 0)
    }

    fn end(y: i32, mo: u32, d: u32) -> NaiveDateTime {
        dt(y, mo, d, 23, 59, 59)
    }

    fn p(input: &str) -> Option<NaiveDateTime> {
        parse_date(input, now())
    }

    // ── epoch ────────────────────────────────────────────────────────────────

    #[test]
    fn epoch_valid() {
        // 2024-03-05T00:00:00 UTC = 1709596800
        assert_eq!(p("1709596800"), Some(date(2024, 3, 5)));
    }

    // ── ISO formats ──────────────────────────────────────────────────────────

    #[test]
    fn iso_datetime_extended() {
        assert_eq!(p("2024-03-15T09:00:00"), Some(dt(2024, 3, 15, 9, 0, 0)));
    }

    #[test]
    fn iso_datetime_basic() {
        assert_eq!(p("20240315T090000"), Some(dt(2024, 3, 15, 9, 0, 0)));
    }

    #[test]
    fn iso_date_extended() {
        assert_eq!(p("2024-03-15"), Some(date(2024, 3, 15)));
    }

    #[test]
    fn iso_date_basic() {
        assert_eq!(p("20240315"), Some(date(2024, 3, 15)));
    }

    // ── duration ─────────────────────────────────────────────────────────────

    #[test]
    fn duration_days() {
        assert_eq!(p("7d"), Some(dt(2024, 3, 12, 12, 34, 56)));
    }

    #[test]
    fn duration_weeks() {
        assert_eq!(p("2weeks"), Some(dt(2024, 3, 19, 12, 34, 56)));
    }

    #[test]
    fn duration_hours() {
        assert_eq!(p("1h"), Some(dt(2024, 3, 5, 13, 34, 56)));
    }

    #[test]
    fn duration_months() {
        assert_eq!(p("3mo"), Some(dt(2024, 6, 5, 12, 34, 56)));
    }

    #[test]
    fn duration_years() {
        assert_eq!(p("1y"), Some(dt(2025, 3, 5, 12, 34, 56)));
    }

    // ── relative days ────────────────────────────────────────────────────────

    #[test]
    fn keyword_now() {
        assert_eq!(p("now"), Some(now()));
    }

    #[test]
    fn keyword_yesterday() {
        assert_eq!(p("yesterday"), Some(date(2024, 3, 4)));
    }

    #[test]
    fn keyword_yesterday_abbrev() {
        assert_eq!(p("yes"), Some(date(2024, 3, 4)));
    }

    #[test]
    fn keyword_today() {
        assert_eq!(p("today"), Some(date(2024, 3, 5)));
    }

    #[test]
    fn keyword_today_abbrev() {
        assert_eq!(p("tod"), Some(date(2024, 3, 5)));
    }

    #[test]
    fn keyword_tomorrow() {
        assert_eq!(p("tomorrow"), Some(date(2024, 3, 6)));
    }

    #[test]
    fn keyword_tomorrow_abbrev() {
        assert_eq!(p("tom"), Some(date(2024, 3, 6)));
    }

    // ── day names (now = Tuesday; timeRelative=true → next occurrence) ───────

    #[test]
    fn day_wednesday() {
        assert_eq!(p("wednesday"), Some(date(2024, 3, 6)));
    }

    #[test]
    fn day_thursday() {
        assert_eq!(p("thursday"), Some(date(2024, 3, 7)));
    }

    #[test]
    fn day_friday() {
        assert_eq!(p("friday"), Some(date(2024, 3, 8)));
    }

    #[test]
    fn day_saturday() {
        assert_eq!(p("saturday"), Some(date(2024, 3, 9)));
    }

    #[test]
    fn day_sunday() {
        assert_eq!(p("sunday"), Some(date(2024, 3, 10)));
    }

    #[test]
    fn day_monday() {
        assert_eq!(p("monday"), Some(date(2024, 3, 11)));
    }

    #[test]
    fn day_tuesday_next_week() {
        // Today is Tuesday → next Tuesday
        assert_eq!(p("tuesday"), Some(date(2024, 3, 12)));
    }

    #[test]
    fn day_abbrev_mon() {
        assert_eq!(p("mon"), Some(date(2024, 3, 11)));
    }

    #[test]
    fn day_abbrev_fri() {
        assert_eq!(p("fri"), Some(date(2024, 3, 8)));
    }

    #[test]
    fn day_abbrev_sun() {
        assert_eq!(p("sun"), Some(date(2024, 3, 10)));
    }

    // ── month names (now = 2024-03-05; 1st already past → next year) ─────────

    #[test]
    fn month_april() {
        assert_eq!(p("april"), Some(date(2024, 4, 1)));
    }

    #[test]
    fn month_december() {
        assert_eq!(p("december"), Some(date(2024, 12, 1)));
    }

    #[test]
    fn month_january_wraps() {
        assert_eq!(p("january"), Some(date(2025, 1, 1)));
    }

    #[test]
    fn month_march_wraps() {
        // March 1 already passed in 2024
        assert_eq!(p("march"), Some(date(2025, 3, 1)));
    }

    #[test]
    fn month_abbrev_jan() {
        assert_eq!(p("jan"), Some(date(2025, 1, 1)));
    }

    #[test]
    fn month_abbrev_apr() {
        assert_eq!(p("apr"), Some(date(2024, 4, 1)));
    }

    #[test]
    fn month_abbrev_dec() {
        assert_eq!(p("dec"), Some(date(2024, 12, 1)));
    }

    // ── ordinals (now = 2024-03-05; timeRelative=true) ───────────────────────

    #[test]
    fn ordinal_10th_this_month() {
        assert_eq!(p("10th"), Some(date(2024, 3, 10)));
    }

    #[test]
    fn ordinal_5th_wraps() {
        // Today is the 5th → wraps to April
        assert_eq!(p("5th"), Some(date(2024, 4, 5)));
    }

    #[test]
    fn ordinal_1st_wraps() {
        assert_eq!(p("1st"), Some(date(2024, 4, 1)));
    }

    #[test]
    fn ordinal_2nd_wraps() {
        assert_eq!(p("2nd"), Some(date(2024, 4, 2)));
    }

    #[test]
    fn ordinal_3rd_wraps() {
        assert_eq!(p("3rd"), Some(date(2024, 4, 3)));
    }

    #[test]
    fn ordinal_31st_this_month() {
        assert_eq!(p("31st"), Some(date(2024, 3, 31)));
    }

    // ── day boundaries (now = 2024-03-05) ────────────────────────────────────

    #[test]
    fn boundary_sopd() {
        assert_eq!(p("sopd"), Some(date(2024, 3, 4)));
    }

    #[test]
    fn boundary_sod() {
        assert_eq!(p("sod"), Some(date(2024, 3, 5)));
    }

    #[test]
    fn boundary_sond() {
        assert_eq!(p("sond"), Some(date(2024, 3, 6)));
    }

    #[test]
    fn boundary_eopd() {
        assert_eq!(p("eopd"), Some(end(2024, 3, 4)));
    }

    #[test]
    fn boundary_eod() {
        assert_eq!(p("eod"), Some(end(2024, 3, 5)));
    }

    #[test]
    fn boundary_eond() {
        assert_eq!(p("eond"), Some(end(2024, 3, 6)));
    }

    // ── week boundaries (weekstart = Monday; this week Mon 03-04..Sun 03-10) ─

    #[test]
    fn boundary_sopw() {
        assert_eq!(p("sopw"), Some(date(2024, 2, 26)));
    }

    #[test]
    fn boundary_sow() {
        assert_eq!(p("sow"), Some(date(2024, 3, 4)));
    }

    #[test]
    fn boundary_sonw() {
        assert_eq!(p("sonw"), Some(date(2024, 3, 11)));
    }

    #[test]
    fn boundary_eopw() {
        assert_eq!(p("eopw"), Some(end(2024, 3, 3)));
    }

    #[test]
    fn boundary_eow() {
        assert_eq!(p("eow"), Some(end(2024, 3, 10)));
    }

    #[test]
    fn boundary_eonw() {
        assert_eq!(p("eonw"), Some(end(2024, 3, 17)));
    }

    // ── work-week boundaries (Mon–Fri; this week Mon 03-04..Fri 03-08) ───────

    #[test]
    fn boundary_sopww() {
        assert_eq!(p("sopww"), Some(date(2024, 2, 26)));
    }

    #[test]
    fn boundary_soww() {
        assert_eq!(p("soww"), Some(date(2024, 3, 4)));
    }

    #[test]
    fn boundary_sonww() {
        assert_eq!(p("sonww"), Some(date(2024, 3, 11)));
    }

    #[test]
    fn boundary_eopww() {
        assert_eq!(p("eopww"), Some(end(2024, 3, 1)));
    }

    #[test]
    fn boundary_eoww() {
        assert_eq!(p("eoww"), Some(end(2024, 3, 8)));
    }

    #[test]
    fn boundary_eonww() {
        assert_eq!(p("eonww"), Some(end(2024, 3, 15)));
    }

    // ── month boundaries (now = 2024-03-05; Feb has 29 days in 2024) ─────────

    #[test]
    fn boundary_sopm() {
        assert_eq!(p("sopm"), Some(date(2024, 2, 1)));
    }

    #[test]
    fn boundary_som() {
        assert_eq!(p("som"), Some(date(2024, 3, 1)));
    }

    #[test]
    fn boundary_sonm() {
        assert_eq!(p("sonm"), Some(date(2024, 4, 1)));
    }

    #[test]
    fn boundary_eopm() {
        assert_eq!(p("eopm"), Some(end(2024, 2, 29)));
    } // leap year

    #[test]
    fn boundary_eom() {
        assert_eq!(p("eom"), Some(end(2024, 3, 31)));
    }

    #[test]
    fn boundary_eonm() {
        assert_eq!(p("eonm"), Some(end(2024, 4, 30)));
    }

    // ── quarter boundaries (now = 2024-03-05; Q1 = Jan–Mar) ─────────────────

    #[test]
    fn boundary_sopq() {
        assert_eq!(p("sopq"), Some(date(2023, 10, 1)));
    }

    #[test]
    fn boundary_soq() {
        assert_eq!(p("soq"), Some(date(2024, 1, 1)));
    }

    #[test]
    fn boundary_sonq() {
        assert_eq!(p("sonq"), Some(date(2024, 4, 1)));
    }

    #[test]
    fn boundary_eopq() {
        assert_eq!(p("eopq"), Some(end(2023, 12, 31)));
    }

    #[test]
    fn boundary_eoq() {
        assert_eq!(p("eoq"), Some(end(2024, 3, 31)));
    }

    #[test]
    fn boundary_eonq() {
        assert_eq!(p("eonq"), Some(end(2024, 6, 30)));
    }

    // ── year boundaries ───────────────────────────────────────────────────────

    #[test]
    fn boundary_sopy() {
        assert_eq!(p("sopy"), Some(date(2023, 1, 1)));
    }

    #[test]
    fn boundary_soy() {
        assert_eq!(p("soy"), Some(date(2024, 1, 1)));
    }

    #[test]
    fn boundary_sony() {
        assert_eq!(p("sony"), Some(date(2025, 1, 1)));
    }

    #[test]
    fn boundary_eopy() {
        assert_eq!(p("eopy"), Some(end(2023, 12, 31)));
    }

    #[test]
    fn boundary_eoy() {
        assert_eq!(p("eoy"), Some(end(2024, 12, 31)));
    }

    #[test]
    fn boundary_eony() {
        assert_eq!(p("eony"), Some(end(2025, 12, 31)));
    }

    // ── later / someday ───────────────────────────────────────────────────────

    #[test]
    fn later_full() {
        assert_eq!(p("later"), Some(date(9999, 12, 30)));
    }

    #[test]
    fn later_abbrev() {
        assert_eq!(p("lat"), Some(date(9999, 12, 30)));
    }

    #[test]
    fn someday_full() {
        assert_eq!(p("someday"), Some(date(9999, 12, 30)));
    }

    #[test]
    fn someday_abbrev() {
        // min 4 chars for someday; "some" qualifies
        assert_eq!(p("some"), Some(date(9999, 12, 30)));
    }

    #[test]
    fn someday_too_short() {
        // "som" (3 chars) resolves to start-of-month, not someday
        assert_eq!(p("som"), Some(date(2024, 3, 1)));
    }

    // ── invalid ───────────────────────────────────────────────────────────────

    #[test]
    fn invalid_empty() {
        assert_eq!(p(""), None);
    }

    #[test]
    fn invalid_random() {
        assert_eq!(p("xyz"), None);
    }

    #[test]
    fn invalid_prefix_too_short() {
        assert_eq!(p("to"), None); // below min 3 for today/tomorrow
    }

    #[test]
    fn invalid_later_too_short() {
        assert_eq!(p("la"), None);
    }

    // ── case insensitive ──────────────────────────────────────────────────────

    #[test]
    fn case_today_upper() {
        assert_eq!(p("TODAY"), Some(date(2024, 3, 5)));
    }

    #[test]
    fn case_monday_mixed() {
        assert_eq!(p("Mon"), Some(date(2024, 3, 11)));
    }

    #[test]
    fn case_friday_upper() {
        assert_eq!(p("FRIDAY"), Some(date(2024, 3, 8)));
    }

    #[test]
    fn case_march_upper() {
        assert_eq!(p("MARCH"), Some(date(2025, 3, 1)));
    }
}
