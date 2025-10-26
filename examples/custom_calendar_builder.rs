//! Example demonstrating custom calendar creation using the builder pattern.
//!
//! This example shows how to programmatically create a custom calendar system
//! using the CustomCalendarBuilder, without needing configuration files.
//! This approach is ideal when calendar definitions are fixed in code.
//!
//! This sci-fi calendar features:
//! - 100 minutes per hour
//! - 10 hours per day (shorter days on a faster-rotating planet)
//! - 6 days per week
//! - 13 months per year with varying lengths
//! - Complex leap year rule (Gregorian-style)
//!
//! Controls:
//! - Space: Pause/Resume
//! - +/-: Speed Up/Down
//! - R: Reset clock

use bevy::prelude::*;
use bevy_ingame_clock::{
    ClockCommands, ClockInterval, ClockIntervalEvent, CustomCalendarBuilder, 
    Epoch, InGameClock, InGameClockPlugin, Month,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InGameClockPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (display_time, handle_input, handle_clock_events))
        .run();
}

#[derive(Component)]
struct ClockText;

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);

    // Create a custom sci-fi calendar using the builder pattern
    let scifi_calendar = CustomCalendarBuilder::new()
        .minutes_per_hour(100)
        .hours_per_day(10)
        .months(vec![
            Month::new("Primaris", 28, 0),
            Month::new("Secundus", 28, 0),
            Month::new("Tertius", 28, 1),   // Gains 1 day in leap years
            Month::new("Quartus", 28, 0),
            Month::new("Quintus", 28, 0),
            Month::new("Sextus", 28, 0),
            Month::new("Septimus", 28, 0),
            Month::new("Octavus", 28, 0),
            Month::new("Nonus", 28, 0),
            Month::new("Decimus", 28, 0),
            Month::new("Undecimus", 28, 0),
            Month::new("Duodecimus", 28, 0),
            Month::new("Ultimus", 29, 0),   // Slightly longer last month
        ])
        .weekdays(vec![
            "Solday".to_string(),
            "Lunaday".to_string(),
            "Marsday".to_string(),
            "Mercday".to_string(),
            "Jupday".to_string(),
            "Saturnday".to_string(),
        ])
        // Complex leap year rule similar to Gregorian calendar
        .leap_years("# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)")
        .epoch(Epoch::new("Galactic Standard Era", 2500))
        .build();

    // Print calendar info to console
    println!("\n=== Sci-Fi Calendar (Built with CustomCalendarBuilder) ===");
    println!("Calendar definition: Programmatically created in code");

    println!("\nCalendar structure:");
    println!("  - {} minutes per hour", scifi_calendar.minutes_per_hour);
    println!("  - {} hours per day", scifi_calendar.hours_per_day);
    println!("  - {} days per week", scifi_calendar.weekdays.len());
    println!("  - Week starts with: {}", scifi_calendar.weekdays[0]);
    println!("  - Leap year rule: {:?}", scifi_calendar.leap_years);
    
    let total_days: u32 = scifi_calendar.months.iter().map(|m| m.days).sum();
    let total_leap_days: u32 = scifi_calendar.months.iter().map(|m| m.leap_days).sum();
    println!("  - {} months with varying lengths", scifi_calendar.months.len());
    for month in &scifi_calendar.months {
        if month.leap_days > 0 {
            println!("    - {}: {} days (+{} leap days)", month.name, month.days, month.leap_days);
        } else {
            println!("    - {}: {} days", month.name, month.days);
        }
    }
    println!("  - {} days per normal year", total_days);
    println!("  - {} days per leap year", total_days + total_leap_days);
    println!("\nEpoch:");
    println!("  - Name: {}", scifi_calendar.epoch.name);
    println!("  - Starting year: {}", scifi_calendar.epoch.start_year);
    println!("\nTime progression: 1 in-game day = 30 real seconds");
    println!("\nControls:");
    println!("  Space - Pause/Resume");
    println!("  +/-   - Speed Up/Down");
    println!("  R     - Reset clock");
    println!();

    // Create a clock with the sci-fi calendar
    // One in-game day passes every 30 real seconds (faster than the fantasy calendar example)
    let clock = InGameClock::new()
        .with_calendar(scifi_calendar)
        .with_day_duration(30.0);

    commands.insert_resource(clock);

    // UI Text
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
        ClockText,
    ));

    // Register interval to receive events when a day passes
    commands.register_clock_interval(ClockInterval::Day);
    // Register interval to receive events when a week passes
    commands.register_clock_interval(ClockInterval::Week);
}

fn handle_clock_events(mut events: MessageReader<ClockIntervalEvent>) {
    for event in events.read() {
        match event.interval {
            ClockInterval::Day => {
                println!("🌅 A day has passed! (Day count: {})", event.count);
            },
            ClockInterval::Week => {
                println!("🗓️ A week has passed! (Week count: {})", event.count);
            },
            _ => {}
        }
    }
}

fn display_time(
    clock: Res<InGameClock>,
    mut query: Query<&mut Text, With<ClockText>>,
) {
    if clock.is_changed() || query.iter().next().is_some() {
        for mut text in &mut query {
            // Display using default format
            let datetime = clock.format_datetime(None);
            let date = clock.format_date(None);
            let time = clock.format_time(None);
            
            // Display using custom format with weekday, month names and epoch
            let custom_format = clock.format_datetime(Some("%A, %E Year %Y, %B %d - %H:%M:%S"));
            
            // Get raw components
            let (year, month, day) = clock.current_date();
            let (hour, minute, second) = clock.current_time();
            
            let status = if clock.paused { "PAUSED" } else { "RUNNING" };
            
            text.0 = format!(
                "=== Sci-Fi Calendar Clock (Builder Pattern) ===\n\
                \n\
                Status:            {}\n\
                \n\
                Default format:    {}\n\
                Custom format:     {}\n\
                Date only:         {}\n\
                Time only:         {}\n\
                Components:        Year {}, Month {}, Day {} | {}:{:02}:{:02}\n\
                \n\
                Speed:             {:.1}x\n\
                Day duration:      {:.1}s\n\
                \n\
                Calendar Info:\n\
                  100 minutes/hour, 10 hours/day, 6 days/week\n\
                  13 months/year (365 days normal, 366 in leap years)\n\
                  Leap year rule: Gregorian-style\n\
                \n\
                Controls:\n\
                  Space - Pause/Resume\n\
                  +/-   - Speed Up/Down\n\
                  R     - Reset clock",
                status,
                datetime,
                custom_format,
                date,
                time,
                year,
                month,
                day,
                hour,
                minute,
                second,
                clock.speed,
                clock.day_duration()
            );
        }
    }
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut clock: ResMut<InGameClock>,
) {
    // Pause/Resume
    if keyboard.just_pressed(KeyCode::Space) {
        clock.toggle_pause();
    }

    // Speed controls
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        let new_speed = clock.speed * 2.0;
        clock.set_speed(new_speed);
    }
    
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        let new_speed = (clock.speed / 2.0).max(0.1);
        clock.set_speed(new_speed);
    }

    // Reset
    if keyboard.just_pressed(KeyCode::KeyR) {
        clock.elapsed_seconds = 0.0;
    }
}