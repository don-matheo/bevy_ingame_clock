//! # Bevy In-Game Clock
//!
//! A plugin for the Bevy game engine that provides an in-game clock and calendar system.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_ingame_clock::InGameClockPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(InGameClockPlugin)
//!         .run();
//! }
//! ```

use bevy::prelude::*;
use chrono::{Datelike, Duration, NaiveDateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use evalexpr::*;

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
    /// days_per_week configuration and seconds_per_day().
    fn seconds_per_week(&self) -> u32 {
        self.seconds_per_day() * 7
    }
}

/// Default Gregorian calendar implementation using chrono
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

/// Era/Epoch definition for calendar system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Era {
    pub name: String,
    pub start_year: i64,
}

impl Era {
    pub fn new(name: impl Into<String>, start_year: i64) -> Self {
        Self {
            name: name.into(),
            start_year,
        }
    }
}
/// Default leap year rule: no leap years (always returns false)
fn default_leap_years() -> String {
    "false".to_string()
}


/// Custom calendar with fully configurable time units and structure
///
/// This calendar system allows you to create fantasy or alternative calendar systems
/// with custom time units, month lengths, and leap year rules.
///
/// # Leap Year System
///
/// The leap year system is controlled by the `leap_years` field and the `leap_days` field
/// in each [`Month`]. Here's how it works:
///
/// 1. **Leap Year Cycle**: The `leap_years` field determines how often leap years occur.
///    If `leap_years` is 4, then years 0, 4, 8, 12, etc. are leap years.
///    If `leap_years` is 0, there are no leap years.
///
/// 2. **Leap Day Distribution**: Each month can specify how many extra days (`leap_days`)
///    it gains during a leap year. This allows you to distribute leap days across multiple
///    months or concentrate them in specific months.
///
/// 3. **Total Year Length**: In a normal year, the year length is the sum of all month `days`.
///    In a leap year, it's the sum of all month `(days + leap_days)`.
///
/// # Examples
///
/// ```
/// # use bevy_ingame_clock::{CustomCalendar, Month, Era};
/// // Fantasy calendar with leap years every 2 years
/// let calendar = CustomCalendar::new(
///     20,  // 20 minutes per hour
///     8,   // 8 hours per day
///     5,   // 5 days per week
///     vec![
///         Month::new("Frostmoon", 20, 3),  // 20 days normally, 23 in leap years
///         Month::new("Thawmoon", 21, 0),   // Always 21 days
///         Month::new("Bloomtide", 19, 2),  // 19 days normally, 21 in leap years
///     ],
///     vec!["Moonday".to_string(), "Fireday".to_string(), "Waterday".to_string(),
///          "Earthday".to_string(), "Starday".to_string()],
///     "# % 2 == 0".to_string(),  // Leap year every 2 years (years 0, 2, 4, 6, ...)
///     Era::new("Age of Magic", 1000),
/// );
///
/// // Check if specific years are leap years
/// assert!(calendar.is_leap_year(1000));  // Start year is leap year
/// assert!(!calendar.is_leap_year(1001));
/// assert!(calendar.is_leap_year(1002));
///
/// // Normal year: 20 + 21 + 19 = 60 days
/// // Leap year: 23 + 21 + 21 = 65 days (5 extra leap days distributed across months)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCalendar {
    /// Number of minutes in one hour
    pub minutes_per_hour: u32,
    /// Number of hours in one day
    pub hours_per_day: u32,
    /// Number of days in one week
    pub days_per_week: u32,
    /// The months of the year with their day counts
    pub months: Vec<Month>,
    /// Names of the weekdays (must match `days_per_week` in length)
    pub weekday_names: Vec<String>,
    /// Leap year expression: a boolean expression that evaluates whether a year is a leap year
    /// Use `#` as placeholder for the year value. Supports arithmetic, comparison, and logical operators.
    ///
    /// # Examples
    /// - No leap years: `"false"`
    /// - Every 2 years: `"# % 2 == 0"`
    /// - Every 4 years: `"# % 4 == 0"`
    /// - Gregorian: `"# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)"`
    #[serde(default = "default_leap_years")]
    pub leap_years: String,
    /// The era/epoch information for this calendar
    pub era: Era,
}

impl CustomCalendar {
    /// Create a new custom calendar with fully configurable units
    /// The first weekday in the weekday_names list is considered day 0 of the week
    pub fn new(
        minutes_per_hour: u32,
        hours_per_day: u32,
        days_per_week: u32,
        months: Vec<Month>,
        weekday_names: Vec<String>,
        leap_years: String,
        era: Era,
    ) -> Self {
        assert_eq!(weekday_names.len(), days_per_week as usize, "Weekday names must match days_per_week");
        assert!(!months.is_empty(), "Must have at least one month");
        
        Self {
            minutes_per_hour,
            hours_per_day,
            days_per_week,
            months,
            weekday_names,
            leap_years,
            era,
        }
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
        let weekday_index = (total_days % self.days_per_week as i64) as usize;
        self.weekday_names[weekday_index].clone()
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
        self.seconds_per_day() * self.days_per_week
    }
    
    fn get_date(&self, elapsed_seconds: f64, _start_datetime: NaiveDateTime) -> (i32, u32, u32) {
        let total_days = (elapsed_seconds / self.seconds_per_day() as f64).floor() as i64;
        let days_per_year = self.days_per_year() as i64;
        
        let years_since_epoch = total_days / days_per_year;
        let year = self.era.start_year + years_since_epoch;
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
                .replace("%E", &self.era.name)
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
                .replace("%E", &self.era.name)
                .replace("%A", &weekday)
                .replace("%H", &format!("{:02}", hour))
                .replace("%M", &format!("{:02}", minute))
                .replace("%S", &format!("{:02}", second))
        } else {
            format!("{} {}", date, time)
        }
    }
}

/// Event fired when a specific time interval has passed
#[derive(Message, Debug, Clone)]
pub struct ClockIntervalEvent {
    /// The interval that triggered this event
    pub interval: ClockInterval,
    /// The number of times this interval has passed since the clock started
    pub count: u64,
}

/// Defines different time intervals for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockInterval {
    /// Every second
    Second,
    /// Every minute
    Minute,
    /// Every hour
    Hour,
    /// Every day
    Day,
    /// Every week
    Week,
    /// Custom interval in seconds
    Custom(u32),
}

impl ClockInterval {
    /// Get the duration of this interval in seconds, based on the calendar
    pub fn as_seconds(&self, calendar: &dyn Calendar) -> u32 {
        match self {
            ClockInterval::Second => 1,
            ClockInterval::Minute => 60,
            ClockInterval::Hour => calendar.seconds_per_hour(),
            ClockInterval::Day => calendar.seconds_per_day(),
            ClockInterval::Week => calendar.seconds_per_week(),
            ClockInterval::Custom(seconds) => *seconds,
        }
    }
}

/// The main plugin for the in-game clock system.
///
/// Add this plugin to your Bevy app to enable in-game clock functionality.
pub struct InGameClockPlugin;

impl Plugin for InGameClockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InGameClock>()
            .init_resource::<ClockIntervalTrackers>()
            .add_message::<ClockIntervalEvent>()
            .add_systems(Update, update_clock)
            .add_systems(Update, check_intervals);
    }
}

/// Resource that tracks when intervals should fire events
#[derive(Resource, Default)]
struct ClockIntervalTrackers {
    trackers: Vec<IntervalTracker>,
}

struct IntervalTracker {
    interval: ClockInterval,
    last_trigger_seconds: f64,
    count: u64,
}

/// Resource that represents the in-game clock.
///
/// This tracks the in-game time which can run at a different speed than real time.
#[derive(Resource, Clone)]
pub struct InGameClock {
    /// The elapsed in-game time in seconds since the start_datetime
    pub elapsed_seconds: f64,
    /// The speed multiplier for the clock (1.0 = real-time, 2.0 = double speed, etc.)
    pub speed: f32,
    /// Whether the clock is currently running
    pub paused: bool,
    /// The start date/time for the in-game clock
    pub start_datetime: NaiveDateTime,
    /// The calendar system used for date/time calculations and formatting
    calendar: Arc<dyn Calendar>,
}

impl std::fmt::Debug for InGameClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InGameClock")
            .field("elapsed_seconds", &self.elapsed_seconds)
            .field("speed", &self.speed)
            .field("paused", &self.paused)
            .field("start_datetime", &self.start_datetime)
            .field("calendar", &"<Calendar>")
            .finish()
    }
}

impl Default for InGameClock {
    fn default() -> Self {
        // Use current UTC date/time as default
        let now = Utc::now().naive_utc();
        
        Self {
            elapsed_seconds: 0.0,
            speed: 1.0,
            paused: false,
            start_datetime: now,
            calendar: Arc::new(GregorianCalendar),
        }
    }
}

impl InGameClock {
    /// Creates a new in-game clock with default settings (current date/time)
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an interval to trigger events
    ///
    /// # Examples
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_ingame_clock::{InGameClock, ClockInterval};
    /// fn setup(mut commands: Commands) {
    ///     let mut clock = InGameClock::new();
    ///     // Register to receive events every in-game hour
    ///     commands.insert_resource(clock);
    /// }
    /// ```
    pub fn register_interval(world: &mut World, interval: ClockInterval) {
        let mut trackers = world.resource_mut::<ClockIntervalTrackers>();
        
        // Don't register duplicates
        if !trackers.trackers.iter().any(|t| t.interval == interval) {
            trackers.trackers.push(IntervalTracker {
                interval,
                last_trigger_seconds: 0.0,
                count: 0,
            });
        }
    }

    /// Creates a new in-game clock with a specific start date and time
    pub fn with_start_datetime(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Self {
        let start_datetime = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap(),
        );
        
        Self {
            elapsed_seconds: 0.0,
            speed: 1.0,
            paused: false,
            start_datetime,
            calendar: Arc::new(GregorianCalendar),
        }
    }

    /// Creates a new in-game clock with a custom calendar system
    pub fn with_calendar(mut self, calendar: impl Calendar + 'static) -> Self {
        self.calendar = Arc::new(calendar);
        self
    }

    /// Sets the clock speed multiplier
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Sets the clock speed based on how many real-time seconds it takes for one in-game day to pass.
    /// Takes into account the calendar's seconds_per_day value.
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::InGameClock;
    /// // One in-game day passes every 60 real seconds (1 minute)
    /// let clock = InGameClock::new().with_day_duration(60.0);
    ///
    /// // One in-game day passes every 1200 real seconds (20 minutes)
    /// let clock = InGameClock::new().with_day_duration(1200.0);
    /// ```
    pub fn with_day_duration(mut self, real_seconds_per_day: f32) -> Self {
        // Get the calendar's seconds per day
        let calendar_seconds_per_day = self.calendar.seconds_per_day() as f32;
        // If real_seconds_per_day = 60, then speed = calendar_seconds_per_day / 60
        // This means the game runs (calendar_seconds_per_day / 60)x faster than real time
        self.speed = calendar_seconds_per_day / real_seconds_per_day;
        self
    }

    /// Sets the start datetime from a NaiveDateTime
    pub fn with_start(mut self, datetime: NaiveDateTime) -> Self {
        self.start_datetime = datetime;
        self
    }

    /// Pauses the clock
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the clock
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggles the pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Sets the clock speed multiplier
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Sets the clock speed based on how many real-time seconds it takes for one in-game day to pass.
    /// Takes into account the calendar's seconds_per_day value.
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::InGameClock;
    /// let mut clock = InGameClock::new();
    ///
    /// // One in-game day passes every 60 real seconds (1 minute)
    /// clock.set_day_duration(60.0);
    ///
    /// // One in-game day passes every 1200 real seconds (20 minutes)
    /// clock.set_day_duration(1200.0);
    /// ```
    pub fn set_day_duration(&mut self, real_seconds_per_day: f32) {
        let calendar_seconds_per_day = self.calendar.seconds_per_day() as f32;
        self.speed = calendar_seconds_per_day / real_seconds_per_day;
    }

    /// Gets the current day duration (how many real-time seconds it takes for one in-game day to pass)
    /// Takes into account the calendar's seconds_per_day value.
    pub fn day_duration(&self) -> f32 {
        let calendar_seconds_per_day = self.calendar.seconds_per_day() as f32;
        calendar_seconds_per_day / self.speed
    }

    /// Gets the current NaiveDateTime based on elapsed time
    pub fn current_datetime(&self) -> NaiveDateTime {
        let duration = Duration::milliseconds((self.elapsed_seconds * 1000.0) as i64);
        self.start_datetime + duration
    }

    /// Gets the current time as hours, minutes, and seconds
    pub fn as_hms(&self) -> (u32, u32, u32) {
        let dt = self.current_datetime();
        (dt.hour(), dt.minute(), dt.second())
    }

    /// Gets the current date as (year, month, day)
    pub fn current_date(&self) -> (i32, u32, u32) {
        self.calendar.get_date(self.elapsed_seconds, self.start_datetime)
    }

    /// Gets the current time as (hour, minute, second)
    pub fn current_time(&self) -> (u32, u32, u32) {
        self.calendar.get_time(self.elapsed_seconds, self.start_datetime)
    }

    /// Formats the current date with an optional custom format string.
    ///
    /// If no format is provided, defaults to "YYYY-MM-DD" (%Y-%m-%d).
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::InGameClock;
    /// let clock = InGameClock::with_start_datetime(2024, 6, 15, 8, 30, 0);
    /// assert_eq!(clock.format_date(None), "2024-06-15");
    /// assert_eq!(clock.format_date(Some("%d/%m/%Y")), "15/06/2024");
    /// assert_eq!(clock.format_date(Some("%B %d, %Y")), "June 15, 2024");
    /// ```
    pub fn format_date(&self, format: Option<&str>) -> String {
        self.calendar.format_date(self.elapsed_seconds, self.start_datetime, format)
    }

    /// Formats the current time with an optional custom format string.
    ///
    /// If no format is provided, defaults to "HH:MM:SS" (%H:%M:%S).
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::InGameClock;
    /// let clock = InGameClock::with_start_datetime(2024, 6, 15, 14, 30, 45);
    /// assert_eq!(clock.format_time(None), "14:30:45");
    /// assert_eq!(clock.format_time(Some("%I:%M %p")), "02:30 PM");
    /// assert_eq!(clock.format_time(Some("%H:%M")), "14:30");
    /// ```
    pub fn format_time(&self, format: Option<&str>) -> String {
        self.calendar.format_time(self.elapsed_seconds, self.start_datetime, format)
    }

    /// Formats the current date and time with an optional custom format string.
    ///
    /// If no format is provided, defaults to "YYYY-MM-DD HH:MM:SS" (%Y-%m-%d %H:%M:%S).
    ///
    /// # Examples
    /// ```
    /// # use bevy_ingame_clock::InGameClock;
    /// let clock = InGameClock::with_start_datetime(2024, 6, 15, 14, 30, 45);
    /// assert_eq!(clock.format_datetime(None), "2024-06-15 14:30:45");
    /// assert_eq!(clock.format_datetime(Some("%d/%m/%Y %H:%M")), "15/06/2024 14:30");
    /// assert_eq!(clock.format_datetime(Some("%B %d, %Y at %I:%M %p")), "June 15, 2024 at 02:30 PM");
    /// ```
    pub fn format_datetime(&self, format: Option<&str>) -> String {
        self.calendar.format_datetime(self.elapsed_seconds, self.start_datetime, format)
    }

    /// Get the calendar used by this clock
    pub fn calendar(&self) -> &Arc<dyn Calendar> {
        &self.calendar
    }
}

/// System that updates the in-game clock based on real time
fn update_clock(mut clock: ResMut<InGameClock>, time: Res<Time>) {
    if !clock.paused {
        clock.elapsed_seconds += time.delta_secs_f64() * clock.speed as f64;
    }
}

/// System that checks registered intervals and fires events
fn check_intervals(
    clock: Res<InGameClock>,
    mut trackers: ResMut<ClockIntervalTrackers>,
    mut events: MessageWriter<ClockIntervalEvent>,
) {
    if clock.paused {
        return;
    }

    for tracker in &mut trackers.trackers {
        let interval_seconds = tracker.interval.as_seconds(clock.calendar().as_ref()) as f64;
        
        // Check how many times this interval has passed
        let current_intervals = (clock.elapsed_seconds / interval_seconds).floor() as u64;
        let previous_intervals = (tracker.last_trigger_seconds / interval_seconds).floor() as u64;
        
        // Fire events for each interval that passed
        for _ in previous_intervals..current_intervals {
            tracker.count += 1;
            events.write(ClockIntervalEvent {
                interval: tracker.interval,
                count: tracker.count,
            });
        }
        
        tracker.last_trigger_seconds = clock.elapsed_seconds;
    }
}

/// Commands extension trait for registering clock intervals
pub trait ClockCommands {
    /// Register an interval to trigger clock events
    ///
    /// # Examples
    /// ```no_run
    /// # use bevy::prelude::*;
    /// # use bevy_ingame_clock::{ClockCommands, ClockInterval};
    /// fn setup(mut commands: Commands) {
    ///     // Register to receive events every in-game hour
    ///     commands.register_clock_interval(ClockInterval::Hour);
    ///
    ///     // Register a custom interval (every 90 seconds)
    ///     commands.register_clock_interval(ClockInterval::Custom(90));
    /// }
    /// ```
    fn register_clock_interval(&mut self, interval: ClockInterval);
}

impl ClockCommands for Commands<'_, '_> {
    fn register_clock_interval(&mut self, interval: ClockInterval) {
        self.queue(move |world: &mut World| {
            InGameClock::register_interval(world, interval);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_default() {
        let clock = InGameClock::default();
        assert_eq!(clock.elapsed_seconds, 0.0);
        assert_eq!(clock.speed, 1.0);
        assert!(!clock.paused);
    }

    #[test]
    fn test_clock_with_speed() {
        let clock = InGameClock::new().with_speed(2.0);
        assert_eq!(clock.speed, 2.0);
    }

    #[test]
    fn test_clock_pause() {
        let mut clock = InGameClock::new();
        assert!(!clock.paused);
        clock.pause();
        assert!(clock.paused);
        clock.resume();
        assert!(!clock.paused);
    }

    #[test]
    fn test_clock_toggle_pause() {
        let mut clock = InGameClock::new();
        assert!(!clock.paused);
        clock.toggle_pause();
        assert!(clock.paused);
        clock.toggle_pause();
        assert!(!clock.paused);
    }

    #[test]
    fn test_as_hms() {
        let mut clock = InGameClock::with_start_datetime(2024, 1, 1, 0, 0, 0);
        
        // Test 0 seconds
        assert_eq!(clock.as_hms(), (0, 0, 0));
        
        // Test 1 hour, 2 minutes, 3 seconds
        clock.elapsed_seconds = 3723.0;
        assert_eq!(clock.as_hms(), (1, 2, 3));
        
        // Test day wrap (should show hour 0-23)
        clock.elapsed_seconds = 86400.0 + 3600.0; // 1 day + 1 hour
        assert_eq!(clock.as_hms(), (1, 0, 0));
    }


    #[test]
    fn test_with_start_datetime() {
        let clock = InGameClock::with_start_datetime(2024, 1, 15, 10, 30, 45);
        assert_eq!(clock.elapsed_seconds, 0.0);
        let (year, month, day) = clock.current_date();
        assert_eq!((year, month, day), (2024, 1, 15));
        let (hour, minute, second) = clock.current_time();
        assert_eq!((hour, minute, second), (10, 30, 45));
    }

    #[test]
    fn test_current_datetime() {
        let mut clock = InGameClock::with_start_datetime(2024, 1, 15, 10, 30, 45);
        
        // Test at start
        let (year, month, day) = clock.current_date();
        let (hour, minute, second) = clock.current_time();
        assert_eq!((year, month, day), (2024, 1, 15));
        assert_eq!((hour, minute, second), (10, 30, 45));
        
        // Test after 1 hour
        clock.elapsed_seconds = 3600.0;
        let (year, month, day) = clock.current_date();
        let (hour, minute, second) = clock.current_time();
        assert_eq!((year, month, day), (2024, 1, 15));
        assert_eq!((hour, minute, second), (11, 30, 45));
        
        // Test after crossing midnight
        clock.elapsed_seconds = 14.0 * 3600.0; // 14 hours
        let (year, month, day) = clock.current_date();
        let (hour, minute, second) = clock.current_time();
        assert_eq!((year, month, day), (2024, 1, 16));
        assert_eq!((hour, minute, second), (0, 30, 45));
    }

    #[test]
    fn test_format_date() {
        let clock = InGameClock::with_start_datetime(2024, 3, 5, 0, 0, 0);
        assert_eq!(clock.format_date(None), "2024-03-05");
        assert_eq!(clock.format_date(Some("%d/%m/%Y")), "05/03/2024");
        assert_eq!(clock.format_date(Some("%B %d, %Y")), "March 05, 2024");
    }

    #[test]
    fn test_format_datetime() {
        let clock = InGameClock::with_start_datetime(2024, 12, 31, 23, 59, 59);
        assert_eq!(clock.format_datetime(None), "2024-12-31 23:59:59");
        assert_eq!(clock.format_datetime(Some("%d/%m/%Y %H:%M")), "31/12/2024 23:59");
    }

    #[test]
    fn test_format_time() {
        let clock = InGameClock::with_start_datetime(2024, 6, 15, 14, 30, 45);
        assert_eq!(clock.format_time(None), "14:30:45");
        assert_eq!(clock.format_time(Some("%I:%M %p")), "02:30 PM");
        assert_eq!(clock.format_time(Some("%H:%M")), "14:30");
    }

    #[test]
    fn test_month_overflow() {
        let mut clock = InGameClock::with_start_datetime(2024, 1, 31, 0, 0, 0);
        
        // Add 24 hours (should go to Feb 1)
        clock.elapsed_seconds = 24.0 * 3600.0;
        let (year, month, day) = clock.current_date();
        assert_eq!((year, month, day), (2024, 2, 1));
    }

    #[test]
    fn test_year_overflow() {
        let mut clock = InGameClock::with_start_datetime(2024, 12, 31, 23, 0, 0);
        
        // Add 1 hour (should go to next year)
        clock.elapsed_seconds = 3600.0;
        let (year, month, day) = clock.current_date();
        let (hour, _, _) = clock.current_time();
        assert_eq!((year, month, day), (2025, 1, 1));
        assert_eq!(hour, 0);
    }

    #[test]
    fn test_with_day_duration() {
        // One in-game day passes every 60 real seconds
        let clock = InGameClock::new().with_day_duration(60.0);
        // Speed should be 86400 / 60 = 1440
        assert_eq!(clock.speed, 1440.0);

        // One in-game day passes every 1200 real seconds (20 minutes)
        let clock = InGameClock::new().with_day_duration(1200.0);
        // Speed should be 86400 / 1200 = 72
        assert_eq!(clock.speed, 72.0);
    }

    #[test]
    fn test_set_day_duration() {
        let mut clock = InGameClock::new();
        
        // One in-game day passes every 120 real seconds (2 minutes)
        clock.set_day_duration(120.0);
        assert_eq!(clock.speed, 720.0);
        assert_eq!(clock.day_duration(), 120.0);

        // One in-game day passes every 86400 real seconds (1 real day)
        clock.set_day_duration(86400.0);
        assert_eq!(clock.speed, 1.0);
        assert_eq!(clock.day_duration(), 86400.0);
    }

    #[test]
    fn test_day_duration_getter() {
        let clock = InGameClock::new().with_speed(1440.0);
        // At 1440x speed, one day passes in 86400 / 1440 = 60 seconds
        assert_eq!(clock.day_duration(), 60.0);

        let clock = InGameClock::new().with_speed(1.0);
        // At 1x speed, one day passes in 86400 seconds
        assert_eq!(clock.day_duration(), 86400.0);
    }

    #[test]
    fn test_clock_interval_as_seconds() {
        let gregorian = GregorianCalendar;
        assert_eq!(ClockInterval::Second.as_seconds(&gregorian), 1);
        assert_eq!(ClockInterval::Minute.as_seconds(&gregorian), 60);
        assert_eq!(ClockInterval::Hour.as_seconds(&gregorian), 3600);
        assert_eq!(ClockInterval::Day.as_seconds(&gregorian), 86400);
        assert_eq!(ClockInterval::Week.as_seconds(&gregorian), 604800);
        assert_eq!(ClockInterval::Custom(90).as_seconds(&gregorian), 90);
    }
    
    #[test]
    fn test_custom_calendar_intervals() {
        let custom_calendar = CustomCalendar::new(
            60, // minutes_per_hour
            20, // hours_per_day
            5,  // days_per_week
            vec![Month::new("Month1", 20, 0)],
            vec![
                "Day1".to_string(),
                "Day2".to_string(),
                "Day3".to_string(),
                "Day4".to_string(),
                "Day5".to_string(),
            ],
            "false".to_string(),
            Era::new("Test Era", 0),
        );

        assert_eq!(ClockInterval::Second.as_seconds(&custom_calendar), 1);
        assert_eq!(ClockInterval::Minute.as_seconds(&custom_calendar), 60);
        assert_eq!(ClockInterval::Hour.as_seconds(&custom_calendar), 3600); // 60 * 60
        assert_eq!(ClockInterval::Day.as_seconds(&custom_calendar), 72000); // 20 * 60 * 60
        assert_eq!(ClockInterval::Week.as_seconds(&custom_calendar), 360000); // 72000 * 5
        assert_eq!(ClockInterval::Custom(90).as_seconds(&custom_calendar), 90);
    }
    
    #[test]
    fn test_custom_calendar_leap_years() {
        // Create a calendar with leap years every 2 years
        let calendar = CustomCalendar::new(
            60, // minutes_per_hour
            24, // hours_per_day
            7,  // days_per_week
            vec![
                Month::new("Month1", 30, 2),  // 30 days, +2 in leap years
                Month::new("Month2", 30, 1),  // 30 days, +1 in leap years
                Month::new("Month3", 30, 0),  // 30 days, no leap days
            ],
            vec![
                "Day1".to_string(), "Day2".to_string(), "Day3".to_string(),
                "Day4".to_string(), "Day5".to_string(), "Day6".to_string(),
                "Day7".to_string()
            ],
            "# % 2 == 0".to_string(),  // leap year every 2 years
            Era::new("Test Era", 1000),
        );

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
        // Calendar with leap_years = 0 should have no leap years
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 5)],  // leap_days is ignored
            vec![
                "Day1".to_string(), "Day2".to_string(), "Day3".to_string(),
                "Day4".to_string(), "Day5".to_string(), "Day6".to_string(),
                "Day7".to_string()
            ],
            "false".to_string(),  // no leap years
            Era::new("No Leap Era", 0),
        );

        assert!(!calendar.is_leap_year(0));
        assert!(!calendar.is_leap_year(4));
        assert!(!calendar.is_leap_year(100));
        assert!(!calendar.is_leap_year(1000));
    }
    
    #[test]
    fn test_fantasy_calendar_date_calculation() {
        // Test the fantasy calendar from the example
        let calendar = CustomCalendar::new(
            20, // minutes_per_hour
            8,  // hours_per_day
            5,  // days_per_week
            vec![
                Month::new("Frostmoon", 20, 3),
                Month::new("Thawmoon", 21, 0),
                Month::new("Bloomtide", 19, 2),
            ],
            vec!["Moonday".to_string(), "Fireday".to_string(), "Waterday".to_string(),
                 "Earthday".to_string(), "Starday".to_string()],
            "# % 2 == 0".to_string(),  // leap year every 2 years
            Era::new("Age of Magic", 1000),
        );
        
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
        // Test Gregorian calendar leap year rule
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Jan", 31, 0)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)".to_string(),
            Era::new("CE", 0),
        );
        
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
        // Test custom expression: leap year every 5 years
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 1)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "# % 5 == 0".to_string(),
            Era::new("Test Era", 0),
        );
        
        assert!(calendar.is_leap_year(0));
        assert!(!calendar.is_leap_year(1));
        assert!(!calendar.is_leap_year(4));
        assert!(calendar.is_leap_year(5));
        assert!(calendar.is_leap_year(10));
        assert!(!calendar.is_leap_year(13));
    }
    
    #[test]
    fn test_expression_based_leap_year_complex() {
        // Test complex expression: leap year if (divisible by 3 AND not divisible by 9) OR divisible by 27
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 1)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "(# % 3 == 0 && # % 9 != 0) || # % 27 == 0".to_string(),
            Era::new("Complex Era", 0),
        );
        
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
        // Test that invalid expressions return false instead of panicking
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 1)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "invalid expression here".to_string(),
            Era::new("Test Era", 0),
        );
        
        // Should return false for invalid expression
        assert!(!calendar.is_leap_year(2000));
        assert!(!calendar.is_leap_year(2004));
    }
    
    #[test]
    fn test_leap_year_simple_expression() {
        // Test simple leap year expression
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 1)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "# % 4 == 0".to_string(),
            Era::new("Test Era", 0),
        );
        
        assert!(calendar.is_leap_year(0));
        assert!(calendar.is_leap_year(4));
        assert!(!calendar.is_leap_year(1));
    }
    
    #[test]
    fn test_leap_year_serde_expression() {
        // Test expression serialization from RON
        let calendar = CustomCalendar::new(
            60, 24, 7,
            vec![Month::new("Month1", 30, 1)],
            vec![
                "Mon".to_string(), "Tue".to_string(), "Wed".to_string(),
                "Thu".to_string(), "Fri".to_string(), "Sat".to_string(), "Sun".to_string()
            ],
            "# % 3 == 0".to_string(),
            Era::new("Test Era", 0),
        );
        
        assert!(calendar.is_leap_year(0));
        assert!(calendar.is_leap_year(3));
        assert!(!calendar.is_leap_year(1));
    }
}