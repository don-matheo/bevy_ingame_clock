//! Example demonstrating custom calendar systems.
//!
//! This example shows how to use the CustomCalendar to create a game world
//! with a custom calendar system different from the Gregorian calendar.
//! The calendar configuration is loaded from a RON file.
//!
//! Controls:
//! - Space: Pause/Resume
//! - +/-: Speed Up/Down
//! - R: Reset clock

use bevy::prelude::*;
use bevy_ingame_clock::{ClockCommands, ClockInterval, ClockIntervalEvent, CustomCalendar, InGameClock, InGameClockPlugin};
use std::fs;

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

    // Load the fantasy calendar from RON file
    let calendar_config = fs::read_to_string("examples/fantasy_calendar.ron")
        .expect("Failed to read fantasy_calendar.ron");
    
    let fantasy_calendar: CustomCalendar = ron::from_str(&calendar_config)
        .expect("Failed to parse fantasy_calendar.ron");
    
    // Print calendar info to console
    println!("\n=== Loaded Fantasy Calendar from RON ===");
    println!("Configuration file: examples/fantasy_calendar.ron");

    println!("\nCalendar structure:");
    println!("  - {} minutes per hour", fantasy_calendar.minutes_per_hour);
    println!("  - {} hours per day", fantasy_calendar.hours_per_day);
    println!("  - {} days per week", fantasy_calendar.days_per_week);
    println!("  - Week starts with: {}", fantasy_calendar.weekday_names[0]);
    
    let total_days: u32 = fantasy_calendar.months.iter().map(|m| m.days).sum();
    println!("  - {} months with varying lengths", fantasy_calendar.months.len());
    for month in &fantasy_calendar.months {
        println!("    - {}: {} days", month.name, month.days);
    }
    println!("  - {} days per year", total_days);
    println!("\nEra/Epoch:");
    println!("  - Name: {}", fantasy_calendar.era.name);
    println!("  - Starting year: {}", fantasy_calendar.era.start_year);
    println!("\nTime progression: 1 in-game day = 60 real seconds");
    println!("\nControls:");
    println!("  Space - Pause/Resume");
    println!("  +/-   - Speed Up/Down");
    println!("  R     - Reset clock");
    println!();

    // Create a clock with the fantasy calendar
    // One in-game day passes every 60 real seconds
    let clock = InGameClock::new()
        .with_calendar(fantasy_calendar)
        .with_day_duration(60.0);

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
                println!("🌅 A week has passed! (Week count: {})", event.count);
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
            
            // Display using custom format with weekday, month names and era
            let custom_format = clock.format_datetime(Some("%A, %E Year %Y, %B %d - %H:%M:%S"));
            
            // Get raw components
            let (year, month, day) = clock.current_date();
            let (hour, minute, second) = clock.current_time();
            
            let status = if clock.paused { "PAUSED" } else { "RUNNING" };
            
            text.0 = format!(
                "=== Fantasy Calendar Clock ===\n\
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
                Controls:\n\
                  Space - Pause/Resume\n\
                  +/-   - Speed Up/Down\n\
                  R     - Reset clock",
                status,
                datetime,
                custom_format,
                date,
                time,
                year, month, day, hour, minute, second,
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