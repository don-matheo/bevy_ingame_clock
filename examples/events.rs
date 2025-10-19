//! Example demonstrating clock interval events.
//!
//! This example shows:
//! - How to register/unregister clock intervals dynamically
//! - How to handle different interval events
//! - Using built-in intervals (Second, Minute, Hour, Day, Week)
//! - Using custom intervals
//! - Toggle events on/off with number keys

use bevy::prelude::*;
use bevy_ingame_clock::{ClockCommands, ClockInterval, ClockIntervalEvent, InGameClock, InGameClockPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InGameClockPlugin)
        .init_resource::<ActiveIntervals>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_interval_events, display_info, handle_input))
        .run();
}

#[derive(Resource)]
struct ActiveIntervals {
    second: bool,
    minute: bool,
    hour: bool,
    day: bool,
    week: bool,
    custom: bool,
}

impl Default for ActiveIntervals {
    fn default() -> Self {
        Self {
            second: true,
            minute: true,
            hour: true,
            day: true,
            week: false,
            custom: true,
        }
    }
}

fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn(Camera2d);

    // Set a fast clock speed to demonstrate events quickly
    commands.insert_resource(
        InGameClock::with_start_datetime(2024, 1, 1, 0, 0, 0)
            .with_day_duration(600.0) // 1 day per 600 real seconds
    );

    // Register initial intervals based on defaults
    commands.register_clock_interval(ClockInterval::Second);
    commands.register_clock_interval(ClockInterval::Minute);
    commands.register_clock_interval(ClockInterval::Hour);
    commands.register_clock_interval(ClockInterval::Day);
    commands.register_clock_interval(ClockInterval::Custom(30));

    // Spawn UI text
    commands.spawn((
        Text::new("Clock Events Example\n\nControls:\nSpace: Pause/Resume\n+/-: Speed Up/Down\nR: Reset\n1-6: Toggle Events\n\nWatching for events..."),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        EventDisplay,
    ));
}

#[derive(Component)]
struct EventDisplay;

#[derive(Resource, Default)]
struct EventLog {
    recent_events: Vec<String>,
}

fn handle_interval_events(
    mut events: MessageReader<ClockIntervalEvent>,
    mut log: Local<EventLog>,
    active: Res<ActiveIntervals>,
) {
    for event in events.read() {
        // Only log events that are currently enabled
        let should_log = match event.interval {
            ClockInterval::Second => active.second,
            ClockInterval::Minute => active.minute,
            ClockInterval::Hour => active.hour,
            ClockInterval::Day => active.day,
            ClockInterval::Week => active.week,
            ClockInterval::Custom(_) => active.custom,
        };

        if !should_log {
            continue;
        }

        let message = match event.interval {
            ClockInterval::Second => format!("⏱️  Second passed (count: {})", event.count),
            ClockInterval::Minute => format!("⏰ Minute passed (count: {})", event.count),
            ClockInterval::Hour => format!("🕐 Hour passed (count: {})", event.count),
            ClockInterval::Day => format!("📅 Day passed (count: {})", event.count),
            ClockInterval::Week => format!("📆 Week passed (count: {})", event.count),
            ClockInterval::Custom(seconds) => format!("⚡ Custom interval ({} seconds) passed (count: {})", seconds, event.count),
        };
        
        println!("{}", message);
        
        // Keep last 10 events
        log.recent_events.push(message);
        if log.recent_events.len() > 10 {
            log.recent_events.remove(0);
        }
    }
}

fn display_info(
    clock: Res<InGameClock>,
    mut query: Query<&mut Text, With<EventDisplay>>,
    log: Local<EventLog>,
    active: Res<ActiveIntervals>,
) {
    if let Ok(mut text) = query.single_mut() {
        let status = if clock.paused { "PAUSED" } else { "Running" };
        let events_text = if log.recent_events.is_empty() {
            "Waiting for events...".to_string()
        } else {
            log.recent_events.join("\n")
        };
        
        let interval_status = format!(
            "1: Second [{}]\n2: Minute [{}]\n3: Hour [{}]\n4: Day [{}]\n5: Week [{}]\n6: Custom(30s) [{}]",
            if active.second { "ON" } else { "OFF" },
            if active.minute { "ON" } else { "OFF" },
            if active.hour { "ON" } else { "OFF" },
            if active.day { "ON" } else { "OFF" },
            if active.week { "ON" } else { "OFF" },
            if active.custom { "ON" } else { "OFF" },
        );
        
        **text = format!(
            "Clock Events Example\n\nControls:\nSpace: Pause/Resume\n+/-: Speed Up/Down\nR: Reset\n\nToggle Events:\n{}\n\nDate & Time: {}\nSpeed: {:.1}x (1 day per {:.1}s)\nStatus: {}\n\nRecent Events:\n{}",
            interval_status,
            clock.format_datetime(None),
            clock.speed,
            clock.day_duration(),
            status,
            events_text
        );
    }
}

fn handle_input(
    mut commands: Commands,
    mut clock: ResMut<InGameClock>,
    mut active: ResMut<ActiveIntervals>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Toggle pause with spacebar
    if keyboard.just_pressed(KeyCode::Space) {
        clock.toggle_pause();
    }

    // Increase speed with +
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        clock.speed = (clock.speed * 2.0).min(16384.0);
    }

    // Decrease speed with -
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        clock.speed = (clock.speed * 0.5).max(0.0625);
    }

    // Toggle Second events with 1
    if keyboard.just_pressed(KeyCode::Digit1) {
        active.second = !active.second;
        if active.second {
            commands.register_clock_interval(ClockInterval::Second);
        }
        println!("Second events: {}", if active.second { "enabled" } else { "disabled" });
    }

    // Toggle Minute events with 2
    if keyboard.just_pressed(KeyCode::Digit2) {
        active.minute = !active.minute;
        if active.minute {
            commands.register_clock_interval(ClockInterval::Minute);
        }
        println!("Minute events: {}", if active.minute { "enabled" } else { "disabled" });
    }

    // Toggle Hour events with 3
    if keyboard.just_pressed(KeyCode::Digit3) {
        active.hour = !active.hour;
        if active.hour {
            commands.register_clock_interval(ClockInterval::Hour);
        }
        println!("Hour events: {}", if active.hour { "enabled" } else { "disabled" });
    }

    // Toggle Day events with 4
    if keyboard.just_pressed(KeyCode::Digit4) {
        active.day = !active.day;
        if active.day {
            commands.register_clock_interval(ClockInterval::Day);
        }
        println!("Day events: {}", if active.day { "enabled" } else { "disabled" });
    }

    // Toggle Week events with 5
    if keyboard.just_pressed(KeyCode::Digit5) {
        active.week = !active.week;
        if active.week {
            commands.register_clock_interval(ClockInterval::Week);
        }
        println!("Week events: {}", if active.week { "enabled" } else { "disabled" });
    }

    // Toggle Custom events with 6
    if keyboard.just_pressed(KeyCode::Digit6) {
        active.custom = !active.custom;
        if active.custom {
            commands.register_clock_interval(ClockInterval::Custom(30));
        }
        println!("Custom(30s) events: {}", if active.custom { "enabled" } else { "disabled" });
    }

    // Reset with R
    if keyboard.just_pressed(KeyCode::KeyR) {
        clock.elapsed_seconds = 0.0;
        clock.set_day_duration(600.0);
        clock.paused = false;
        println!("Clock reset");
    }
}