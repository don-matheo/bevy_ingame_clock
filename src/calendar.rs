//! Calendar system implementations for the in-game clock.
//!
//! This module provides the core calendar trait and implementations for both
//! standard Gregorian calendars and custom fantasy calendars.

use chrono::{Datelike, Duration, NaiveDateTime, Timelike};
use evalexpr::*;
use serde::{Deserialize, Serialize};

/// Trait for implementing custom calendar systems
///
/// This trait provides default implementations for Gregorian calendar time units,
/// which can be overridden by custom calendar implementations with different values.
pub trait Calendar: Send + Sync {
    /// Format the current date
    fn format_date(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String;
    
    /// Format the current time
    fn format_time(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String;
    
    /// Format the current date and time
    fn format_datetime(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String;
    
    /// Get date components as (year, month, day)
    fn get_date(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime) -> (i32, u32, u32);
    
    /// Get time components as (hour, minute, second)
    fn get_time(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime) -> (u32, u32, u32);
    
    /// Get seconds per day for this calendar system
    ///
    /// Default: 86400 (24 hours × 60 minutes × 60 seconds - standard Gregorian day)
    /// Custom calendars should override this to return their calculated value based on
    /// hours_per_day and minutes_per_hour configuration.
    fn seconds_per_day(&self) -> u32 {
        86400
    }
    
    /// Get seconds per hour for this calendar system
    ///
    /// Default: 3600 (60 minutes × 60 seconds - standard Gregorian hour)
    /// Custom calendars should override this to return their calculated value based on
    /// minutes_per_hour configuration.
    fn seconds_per_hour(&self) -> u32 {
        3600
    }
    
    /// Get seconds per week for this calendar system
    ///
    /// Default: 604800 (7 days × 86400 seconds - standard Gregorian week)
    /// Custom calendars should override this to return their calculated value based on
    /// the number of weekday names and seconds_per_day().
    fn seconds_per_week(&self) -> u32 {
        self.seconds_per_day() * 7
    }
}

/// Default Gregorian calendar implementation using chrono
///
/// This is a lightweight wrapper that delegates all date/time opepochtions to chrono's
/// [`NaiveDateTime`]. It doesn't store month or weekday names - those are handled
/// by chrono's formatting (e.g., `%B` for month name, `%A` for weekday name).
///
/// Use this for standard real-world calendars. For fantasy/custom calendars with
/// different time units or custom month/weekday names, use [`CustomCalendar`].
#[derive(Debug, Clone)]
pub struct GregorianCalendar;

impl Calendar for GregorianCalendar {
    fn format_date(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let dt = start_datetime + Duration::milliseconds((elapsed_seconds * 1000.0) as i64);
        let fmt = format.unwrap_or("%Y-%m-%d");
        dt.format(fmt).to_string()
    }
    
    fn format_time(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let dt = start_datetime + Duration::milliseconds((elapsed_seconds * 1000.0) as i64);
        let fmt = format.unwrap_or("%H:%M:%S");
        dt.format(fmt).to_string()
    }
    
    fn format_datetime(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let dt = start_datetime + Duration::milliseconds((elapsed_seconds * 1000.0) as i64);
        let fmt = format.unwrap_or("%Y-%m-%d %H:%M:%S");
        dt.format(fmt).to_string()
    }
    
    fn get_date(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime) -> (i32, u32, u32) {
        let dt = start_datetime + Duration::milliseconds((elapsed_seconds * 1000.0) as i64);
        (dt.year(), dt.month(), dt.day())
    }
    
    fn get_time(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime) -> (u32, u32, u32) {
        let dt = start_datetime + Duration::milliseconds((elapsed_seconds * 1000.0) as i64);
        (dt.hour(), dt.minute(), dt.second())
    }
}

/// Month definition combining name and length
///
/// Each month can have a different number of base days and additional leap days
/// that are added during leap years. This allows for flexible calendar systems
/// where leap days can be distributed across different months.
///
/// # Leap Days
///
/// The `leap_days` field specifies how many extra days this month has during a leap year.
/// When a year is determined to be a leap year (based on the calendar's `leap_years` cycle),
/// each month's total length becomes `days + leap_days`.
///
/// # Examples
///
/// ```
/// # use bevy_ingame_clock::Month;
/// // A month with 30 base days and 1 leap day
/// let month = Month::new("Frostmoon", 30, 1);
/// // In a normal year: 30 days
/// // In a leap year: 31 days
///
/// // A month with no leap days
/// let month = Month::new("Suntide", 21, 0);
/// // Always 21 days regardless of leap year
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Month {
    pub name: String,
    /// The base number of days in this month
    pub days: u32,
    /// Additional days added to this month during leap years
    pub leap_days: u32,
}

impl Month {
    pub fn new(name: impl Into<String>, days: u32, leap_days: u32) -> Self {
        Self {
            name: name.into(),
            days,
            leap_days,
        }
    }
}

/// Epoch definition for calendar system
///
/// Represents a reference point in time for year counting, with an optional
/// descriptive name (e.g., "Common Epoch", "Age of Magic").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epoch {
    pub name: String,
    pub start_year: i64,
}

impl Epoch {
    pub fn new(name: impl Into<String>, start_year: i64) -> Self {
        Self {
            name: name.into(),
            start_year,
        }
    }
}

/// Default leap year rule: no leap years
fn default_leap_years() -> String {
    "false".to_string()
}

/// Returns default epoch (Common Epoch starting at year 1)
fn default_epoch() -> Epoch {
    Epoch::new("Common Epoch", 1)
}

/// Custom calendar with fully configurable time units and structure
///
/// This calendar system allows you to create fantasy or alternative calendar systems
/// with custom time units, month lengths, and leap year rules. Unlike [`GregorianCalendar`],
/// this stores explicit month and weekday names in the `months` and `weekdays` fields
/// which you can access directly.
///
/// # Accessing Month and Weekday Names
///
/// ```
/// # use bevy_ingame_clock::{CustomCalendar, Month, Epoch};
/// let calendar = CustomCalendar::builder()
///     .month(Month::new("Frostmoon", 30, 0))
///     .weekday("Moonday")
///     .weekday("Fireday")
///     .build();
///
/// // Access months
/// for month in &calendar.months {
///     println!("Month: {}", month.name);
/// }
///
/// // Access weekdays
/// for weekday in &calendar.weekdays {
///     println!("Weekday: {}", weekday);
/// }
/// ```
///
/// # Leap Year System
///
/// The leap year system is controlled by the `leap_years` expression field and the `leap_days`
/// field in each [`Month`]:
///
/// 1. **Leap Year Expression**: Use boolean expressions to define leap year rules.
///    Examples: `"# % 4 == 0"`, `"# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)"`
///
/// 2. **Leap Day Distribution**: Each month can specify extra days (`leap_days`) gained
///    during leap years, allowing flexible distribution across months.
///
/// 3. **Total Year Length**: Normal year = sum of `days`; Leap year = sum of `(days + leap_days)`.
///
/// See [`CustomCalendar::builder()`](CustomCalendar::builder) for usage examples.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCalendar {
    /// Number of minutes in one hour
    pub minutes_per_hour: u32,
    /// Number of hours in one day
    pub hours_per_day: u32,
    /// The months of the year with their day counts.
    /// You can access these directly to retrieve month names and properties.
    pub months: Vec<Month>,
    /// Names of the weekdays.
    /// You can access these directly to retrieve weekday names.
    pub weekdays: Vec<String>,
    /// Leap year expression: a boolean expression using `#` as year placeholder.
    /// Examples: `"false"`, `"# % 4 == 0"`, `"# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)"`
    #[serde(default = "default_leap_years")]
    pub leap_years: String,
    /// The epoch information for this calendar (reference point for year counting)
    pub epoch: Epoch,
}

/// Builder for creating a [`CustomCalendar`] with a fluent API
///
/// This builder provides a more ergonomic way to construct custom calendars,
/// similar to the builder pattern used by [`crate::InGameClock`].
///
/// See [`CustomCalendar::builder()`](CustomCalendar::builder) for usage examples.
#[derive(Debug, Clone, Default)]
pub struct CustomCalendarBuilder {
    minutes_per_hour: Option<u32>,
    hours_per_day: Option<u32>,
    months: Vec<Month>,
    weekdays: Vec<String>,
    leap_years: Option<String>,
    epoch: Option<Epoch>,
}

impl CustomCalendarBuilder {
    /// Create a new calendar builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the number of minutes per hour
    pub fn minutes_per_hour(mut self, minutes: u32) -> Self {
        self.minutes_per_hour = Some(minutes);
        self
    }
    
    /// Set the number of hours per day
    pub fn hours_per_day(mut self, hours: u32) -> Self {
        self.hours_per_day = Some(hours);
        self
    }
    
    /// Add a month to the calendar
    pub fn month(mut self, month: Month) -> Self {
        self.months.push(month);
        self
    }
    
    /// Add multiple months to the calendar
    pub fn months(mut self, months: Vec<Month>) -> Self {
        self.months = months;
        self
    }
    
    /// Add a weekday name
    pub fn weekday(mut self, name: impl Into<String>) -> Self {
        self.weekdays.push(name.into());
        self
    }
    
    /// Set all weekday names at once
    pub fn weekdays(mut self, names: Vec<String>) -> Self {
        self.weekdays = names;
        self
    }
    
    /// Set the leap year expression using `#` as year placeholder
    pub fn leap_years(mut self, expression: impl Into<String>) -> Self {
        self.leap_years = Some(expression.into());
        self
    }
    
    /// Set the epoch/epoch for the calendar
    pub fn epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = Some(epoch);
        self
    }
    
    /// Build the custom calendar
    ///
    /// # Defaults
    /// - `minutes_per_hour`: 60
    /// - `hours_per_day`: 24
    /// - `leap_years`: `"false"`
    /// - `epoch`: "Common Epoch" starting at year 1
    ///
    /// # Panics
    /// Panics if no months or weekday names were added
    pub fn build(self) -> CustomCalendar {
        let minutes_per_hour = self.minutes_per_hour.unwrap_or(60);
        let hours_per_day = self.hours_per_day.unwrap_or(24);
        let leap_years = self.leap_years.unwrap_or_else(default_leap_years);
        let epoch = self.epoch.unwrap_or_else(default_epoch);
        
        assert!(!self.months.is_empty(), "Must have at least one month");
        assert!(!self.weekdays.is_empty(), "Must have at least one weekday name");
        
        CustomCalendar {
            minutes_per_hour,
            hours_per_day,
            months: self.months,
            weekdays: self.weekdays,
            leap_years,
            epoch,
        }
    }
}

impl CustomCalendar {
    /// Start building a new custom calendar with builder pattern
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::{CustomCalendar, Month, Epoch};
    /// let calendar = CustomCalendar::builder()
    ///     .minutes_per_hour(20)
    ///     .hours_per_day(8)
    ///     .month(Month::new("Frostmoon", 20, 3))
    ///     .weekday("Moonday")
    ///     .weekday("Fireday")
    ///     .weekday("Waterday")
    ///     .weekday("Earthday")
    ///     .weekday("Starday")
    ///     .leap_years("# % 2 == 0")
    ///     .epoch(Epoch::new("Age of Magic", 1000))
    ///     .build();
    /// ```
    pub fn builder() -> CustomCalendarBuilder {
        CustomCalendarBuilder::default()
    }
    
    fn days_per_year(&self) -> u32 {
        self.months.iter().map(|m| m.days).sum()
    }
    
    /// Check if a given year is a leap year according to this calendar's leap year expression
    pub fn is_leap_year(&self, year: i32) -> bool {
        // Replace # placeholder with the actual year value
        let expression = self.leap_years.replace("#", &year.to_string());
        
        // Evaluate the expression
        eval_boolean(&expression)
            .unwrap_or(false)
    }
    
    fn seconds_per_minute(&self) -> u32 {
        60 // Keep seconds at 60 for consistency
    }
    
    /// Get the weekday name for the current elapsed time
    fn get_weekday(&self, elapsed_seconds: f64) -> String {
        let total_days = (elapsed_seconds / self.seconds_per_day() as f64).floor() as i64;
        let weekday_index = (total_days % self.weekdays.len() as i64) as usize;
        self.weekdays[weekday_index].clone()
    }
}

impl Calendar for CustomCalendar {
    fn seconds_per_day(&self) -> u32 {
        self.seconds_per_hour() * self.hours_per_day
    }
    
    fn seconds_per_hour(&self) -> u32 {
        self.seconds_per_minute() * self.minutes_per_hour
    }
    
    fn seconds_per_week(&self) -> u32 {
        self.seconds_per_day() * self.weekdays.len() as u32
    }
    
    fn get_date(&self, elapsed_seconds: f64, _start_datetime: NaiveDateTime) -> (i32, u32, u32) {
        let total_days = (elapsed_seconds / self.seconds_per_day() as f64).floor() as i64;
        let days_per_year = self.days_per_year() as i64;
        
        let years_since_epoch = total_days / days_per_year;
        let year = self.epoch.start_year + years_since_epoch;
        let day_of_year = (total_days % days_per_year) as u32;
        let is_leap_year = self.is_leap_year(year as i32);
        // Find which month and day within that month
        let mut days_remaining = day_of_year;
        let mut month = 1u32;
        
        for (idx, month_def) in self.months.iter().enumerate() {
            if is_leap_year {
                if days_remaining < month_def.days + month_def.leap_days {
                    month = (idx + 1) as u32;
                    break;
                }
                days_remaining -= month_def.days + month_def.leap_days;
            } else {
                if days_remaining < month_def.days {
                    month = (idx + 1) as u32;
                    break;
                }
                days_remaining -= month_def.days;
            }
        }
        
        let day = days_remaining + 1; // 1-indexed
        
        (year as i32, month, day)
    }
    
    fn get_time(&self, elapsed_seconds: f64, _start_datetime: NaiveDateTime) -> (u32, u32, u32) {
        let seconds_per_day = self.seconds_per_day() as f64;
        let seconds_today = elapsed_seconds % seconds_per_day;
        
        let seconds_per_hour = self.seconds_per_hour() as f64;
        let seconds_per_minute = self.seconds_per_minute() as f64;
        
        let hour = (seconds_today / seconds_per_hour).floor() as u32;
        let remaining = seconds_today % seconds_per_hour;
        let minute = (remaining / seconds_per_minute).floor() as u32;
        let second = (remaining % seconds_per_minute).floor() as u32;
        
        (hour, minute, second)
    }
    
    fn format_date(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let (year, month, day) = self.get_date(elapsed_seconds, start_datetime);
        let weekday = self.get_weekday(elapsed_seconds);
        
        if let Some(fmt) = format {
            // Simple custom format support
            fmt.replace("%Y", &year.to_string())
                .replace("%m", &format!("{:02}", month))
                .replace("%d", &format!("{:02}", day))
                .replace("%B", &self.months[(month - 1) as usize].name)
                .replace("%E", &self.epoch.name)
                .replace("%A", &weekday)
        } else {
            format!("{:04}-{:02}-{:02}", year, month, day)
        }
    }
    
    fn format_time(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let (hour, minute, second) = self.get_time(elapsed_seconds, start_datetime);
        
        if let Some(fmt) = format {
            fmt.replace("%H", &format!("{:02}", hour))
                .replace("%M", &format!("{:02}", minute))
                .replace("%S", &format!("{:02}", second))
        } else {
            format!("{:02}:{:02}:{:02}", hour, minute, second)
        }
    }
    
    fn format_datetime(&self, elapsed_seconds: f64, start_datetime: NaiveDateTime, format: Option<&str>) -> String {
        let date = self.format_date(elapsed_seconds, start_datetime, None);
        let time = self.format_time(elapsed_seconds, start_datetime, None);
        
        if let Some(fmt) = format {
            let (year, month, day) = self.get_date(elapsed_seconds, start_datetime);
            let (hour, minute, second) = self.get_time(elapsed_seconds, start_datetime);
            let weekday = self.get_weekday(elapsed_seconds);
            
            fmt.replace("%Y", &year.to_string())
                .replace("%m", &format!("{:02}", month))
                .replace("%d", &format!("{:02}", day))
                .replace("%B", &self.months[(month - 1) as usize].name)
                .replace("%E", &self.epoch.name)
                .replace("%A", &weekday)
                .replace("%H", &format!("{:02}", hour))
                .replace("%M", &format!("{:02}", minute))
                .replace("%S", &format!("{:02}", second))
        } else {
            format!("{} {}", date, time)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_calendar_intervals() {
        let custom_calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(20)
                        .month(Month::new("Month1", 20, 0))
            .weekdays(vec![
                "Day1".to_string(),
                "Day2".to_string(),
                "Day3".to_string(),
                "Day4".to_string(),
                "Day5".to_string(),
            ])
            .leap_years("false")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();

        assert_eq!(custom_calendar.seconds_per_hour(), 3600); // 60 * 60
        assert_eq!(custom_calendar.seconds_per_day(), 72000); // 20 * 60 * 60
        assert_eq!(custom_calendar.seconds_per_week(), 360000); // 72000 * 5
    }
    
    #[test]
    fn test_custom_calendar_leap_years() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 2))  // 30 days, +2 in leap years
            .month(Month::new("Month2", 30, 1))  // 30 days, +1 in leap years
            .month(Month::new("Month3", 30, 0))  // 30 days, no leap days
            .weekdays(vec![
                "Day1".to_string(), "Day2".to_string(), "Day3".to_string(),
                "Day4".to_string(), "Day5".to_string(), "Day6".to_string(),
                "Day7".to_string()
            ])
            .leap_years("# % 2 == 0")  // leap year every 2 years
            .epoch(Epoch::new("Test Epoch", 1000))
            .build();

        // Test leap year detection
        assert!(calendar.is_leap_year(1000));  // Start year (1000 % 2 == 0)
        assert!(!calendar.is_leap_year(1001)); // Not a leap year
        assert!(calendar.is_leap_year(1002));  // Leap year (1002 % 2 == 0)
        assert!(!calendar.is_leap_year(1003));
        assert!(calendar.is_leap_year(1004));
        
        // Test that negative years work correctly
        assert!(calendar.is_leap_year(0));
        assert!(calendar.is_leap_year(-2));
        assert!(!calendar.is_leap_year(-1));
    }
    
    #[test]
    fn test_custom_calendar_no_leap_years() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 5))  // leap_days is ignored
            .weekdays(vec![
                "Day1".to_string(), "Day2".to_string(), "Day3".to_string(),
                "Day4".to_string(), "Day5".to_string(), "Day6".to_string(),
                "Day7".to_string()
            ])
            .leap_years("false")  // no leap years
            .epoch(Epoch::new("No Leap Epoch", 0))
            .build();

        assert!(!calendar.is_leap_year(0));
        assert!(!calendar.is_leap_year(4));
        assert!(!calendar.is_leap_year(100));
        assert!(!calendar.is_leap_year(1000));
    }
    
    #[test]
    fn test_fantasy_calendar_date_calculation() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(20)
            .hours_per_day(8)
                        .month(Month::new("Frostmoon", 20, 3))
            .month(Month::new("Thawmoon", 21, 0))
            .month(Month::new("Bloomtide", 19, 2))
            .weekdays(vec!["Moonday".to_string(), "Fireday".to_string(), "Waterday".to_string(),
                 "Earthday".to_string(), "Starday".to_string()])
            .leap_years("# % 2 == 0")  // leap year every 2 years
            .epoch(Epoch::new("Age of Magic", 1000))
            .build();
        
        let start_datetime = chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        
        // At 0 seconds, should be start of year 1000
        let (year, month, day) = calendar.get_date(0.0, start_datetime);
        assert_eq!(year, 1000);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        
        // Test leap year (1000 % 2 == 0, so it's a leap year)
        assert!(calendar.is_leap_year(1000));
        
        // Normal year: 20 + 21 + 19 = 60 days
        // Leap year: 23 + 21 + 21 = 65 days
        let seconds_per_day = 8 * 20 * 60; // 9600 seconds per day
        
        // Test first day of second month in leap year (day 24 = 23 days of Frostmoon + 1)
        let elapsed = 23.0 * seconds_per_day as f64;
        let (year, month, day) = calendar.get_date(elapsed, start_datetime);
        assert_eq!(year, 1000);
        assert_eq!(month, 2);  // Thawmoon
        assert_eq!(day, 1);
    }
    
    #[test]
    fn test_expression_based_leap_year_gregorian() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Jan", 31, 0))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)")
            .epoch(Epoch::new("CE", 0))
            .build();
        
        // Test classic Gregorian leap year cases
        assert!(calendar.is_leap_year(2000));  // Divisible by 400
        assert!(!calendar.is_leap_year(1900)); // Divisible by 100 but not 400
        assert!(calendar.is_leap_year(2004));  // Divisible by 4 but not 100
        assert!(!calendar.is_leap_year(2001)); // Not divisible by 4
        assert!(calendar.is_leap_year(2024));  // Divisible by 4 but not 100
        assert!(!calendar.is_leap_year(2100)); // Divisible by 100 but not 400
        assert!(calendar.is_leap_year(2400));  // Divisible by 400
    }
    
    #[test]
    fn test_expression_based_leap_year_custom() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 1))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("# % 5 == 0")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();
        
        assert!(calendar.is_leap_year(0));
        assert!(!calendar.is_leap_year(1));
        assert!(!calendar.is_leap_year(4));
        assert!(calendar.is_leap_year(5));
        assert!(calendar.is_leap_year(10));
        assert!(!calendar.is_leap_year(13));
    }
    
    #[test]
    fn test_expression_based_leap_year_complex() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 1))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("(# % 3 == 0 && # % 9 != 0) || # % 27 == 0")
            .epoch(Epoch::new("Complex Epoch", 0))
            .build();
        
        assert!(calendar.is_leap_year(3));   // Divisible by 3, not by 9
        assert!(calendar.is_leap_year(6));   // Divisible by 3, not by 9
        assert!(!calendar.is_leap_year(9));  // Divisible by 9 but not by 27
        assert!(calendar.is_leap_year(12));  // Divisible by 3, not by 9
        assert!(!calendar.is_leap_year(18)); // Divisible by 9 but not by 27
        assert!(calendar.is_leap_year(27));  // Divisible by 27
        assert!(calendar.is_leap_year(54));  // Divisible by 27
    }
    
    #[test]
    fn test_expression_invalid_returns_false() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 1))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("invalid expression here")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();
        
        // Should return false for invalid expression
        assert!(!calendar.is_leap_year(2000));
        assert!(!calendar.is_leap_year(2004));
    }
    
    #[test]
    fn test_leap_year_simple_expression() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 1))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("# % 4 == 0")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();
        
        assert!(calendar.is_leap_year(0));
        assert!(calendar.is_leap_year(4));
        assert!(!calendar.is_leap_year(1));
    }
    
    #[test]
    fn test_leap_year_serde_expression() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .month(Month::new("Month1", 30, 1))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .leap_years("# % 3 == 0")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();
        
        assert!(calendar.is_leap_year(0));
        assert!(calendar.is_leap_year(3));
        assert!(!calendar.is_leap_year(1));
    }
    
    #[test]
    fn test_custom_calendar_builder() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(20)
            .hours_per_day(8)
                        .month(Month::new("Frostmoon", 20, 3))
            .month(Month::new("Thawmoon", 21, 0))
            .month(Month::new("Bloomtide", 19, 2))
            .weekday("Moonday")
            .weekday("Fireday")
            .weekday("Waterday")
            .weekday("Earthday")
            .weekday("Starday")
            .leap_years("# % 2 == 0")
            .epoch(Epoch::new("Age of Magic", 1000))
            .build();
        
        assert_eq!(calendar.minutes_per_hour, 20);
        assert_eq!(calendar.hours_per_day, 8);
        assert_eq!(calendar.months.len(), 3);
        assert_eq!(calendar.weekdays.len(), 5);
        assert_eq!(calendar.leap_years, "# % 2 == 0");
        assert_eq!(calendar.epoch.name, "Age of Magic");
        assert_eq!(calendar.epoch.start_year, 1000);
    }
    
    #[test]
    fn test_custom_calendar_builder_with_bulk_methods() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
                        .months(vec![
                Month::new("January", 31, 0),
                Month::new("February", 28, 1),
            ])
            .weekdays(vec![
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
                "Saturday".to_string(),
                "Sunday".to_string(),
            ])
            .leap_years("# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)")
            .epoch(Epoch::new("Common Epoch", 1))
            .build();
        
        assert_eq!(calendar.months.len(), 2);
        assert_eq!(calendar.weekdays.len(), 7);
    }
    
    #[test]
    fn test_custom_calendar_builder_default_leap_years() {
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(60)
            .hours_per_day(24)
            .month(Month::new("Month1", 30, 0))
            .weekdays(vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ])
            .epoch(Epoch::new("Test", 0))
            .build();
        
        assert_eq!(calendar.leap_years, "false");
        assert!(!calendar.is_leap_year(2000));
        assert!(!calendar.is_leap_year(1900));
        assert!(!calendar.is_leap_year(2004));
    }
    
    #[test]
    fn test_custom_calendar_builder_with_time_defaults() {
        let calendar = CustomCalendar::builder()
            .month(Month::new("Frostmoon", 30, 0))
            .month(Month::new("Thawmoon", 31, 0))
            .weekday("Moonday")
            .weekday("Fireday")
            .build();
        
        assert_eq!(calendar.minutes_per_hour, 60);
        assert_eq!(calendar.hours_per_day, 24);
        assert_eq!(calendar.months.len(), 2);
        assert_eq!(calendar.months[0].name, "Frostmoon");
        assert_eq!(calendar.weekdays.len(), 2);
        assert_eq!(calendar.weekdays[0], "Moonday");
        assert_eq!(calendar.leap_years, "false");
    }
    
    #[test]
    #[should_panic(expected = "Must have at least one month")]
    fn test_custom_calendar_builder_no_months() {
        CustomCalendar::builder()
            .weekday("Monday")
            .build();
    }
    
    #[test]
    #[should_panic(expected = "Must have at least one weekday name")]
    fn test_custom_calendar_builder_no_weekdays() {
        CustomCalendar::builder()
            .month(Month::new("Month1", 30, 0))
            .build();
    }
}