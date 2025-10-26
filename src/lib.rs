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

mod calendar;

pub use calendar::{Calendar, GregorianCalendar, Month, Epoch, CustomCalendar, CustomCalendarBuilder};

use bevy::prelude::*;
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use std::sync::Arc;

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
    /// // One in-game day passes every 60 real seconds
    /// let clock = InGameClock::new().with_day_duration(60.0);
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
    /// clock.set_day_duration(60.0); // One in-game day per 60 real seconds
    /// ```
    pub fn set_day_duration(&mut self, real_seconds_per_day: f32) {
        let calendar_seconds_per_day = self.calendar.seconds_per_day() as f32;
        self.speed = calendar_seconds_per_day / real_seconds_per_day;
    }

    /// Gets the current day duration (how many real-time seconds it takes for one in-game day to pass).
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

        assert_eq!(ClockInterval::Second.as_seconds(&custom_calendar), 1);
        assert_eq!(ClockInterval::Minute.as_seconds(&custom_calendar), 60);
        assert_eq!(ClockInterval::Hour.as_seconds(&custom_calendar), 3600); // 60 * 60
        assert_eq!(ClockInterval::Day.as_seconds(&custom_calendar), 72000); // 20 * 60 * 60
        assert_eq!(ClockInterval::Week.as_seconds(&custom_calendar), 360000); // 72000 * 5
        assert_eq!(ClockInterval::Custom(90).as_seconds(&custom_calendar), 90);
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
    fn test_custom_calendar_builder_integration_with_clock() {
        // Test that builder-created calendar works with InGameClock
        let calendar = CustomCalendar::builder()
            .minutes_per_hour(20)
            .hours_per_day(8)
                        .month(Month::new("Month1", 30, 0))
            .weekday("Day1")
            .weekday("Day2")
            .weekday("Day3")
            .weekday("Day4")
            .weekday("Day5")
            .leap_years("false")
            .epoch(Epoch::new("Test Epoch", 0))
            .build();
        
        let clock = InGameClock::new()
            .with_calendar(calendar)
            .with_day_duration(60.0);
        
        // Verify the calendar is working
        let (year, month, day) = clock.current_date();
        assert_eq!(year, 0);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
    }
}