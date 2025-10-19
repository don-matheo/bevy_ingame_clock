//! A basic example demonstrating the bevy_ingame_clock plugin.
//!
//! This example shows:
//! - How to add the InGameClockPlugin to your app
//! - How to read and display the clock time and date
//! - How to pause/resume the clock with the spacebar
//! - How to adjust clock speed with +/- keys (doubles/halves)
//! - How to set speed by day duration with number keys

use bevy::prelude::*;
use bevy_ingame_clock::{InGameClock, InGameClockPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InGameClockPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (display_time, handle_input))
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a camera
    commands.spawn(Camera2d);

    // Example: Set a custom start date/time
    // To use a specific start date instead of current date/time, insert the resource in main():
    // app.insert_resource(InGameClock::with_start_datetime(2024, 6, 15, 8, 0, 0).with_speed(1.0))
    // Or use day duration: .with_day_duration(60.0) for 1 day per 60 real seconds

    // Spawn UI text to display the clock
    commands.spawn((
        Text::new("In-game Clock Example\n\nControls:\nSpace: Pause/Resume\n+/-: Double/Halve Speed\n1-6: Set Day Duration\nR: Reset\n\nDate & Time: 0000-00-00 00:00:00"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        ClockDisplay,
    ));
}

#[derive(Component)]
struct ClockDisplay;

fn display_time(clock: Res<InGameClock>, mut query: Query<&mut Text, With<ClockDisplay>>) {
    if let Ok(mut text) = query.single_mut() {
        let status = if clock.paused { "PAUSED" } else { "Running" };
        let day_duration = clock.day_duration();
        **text = format!(
            "In-game Clock Example\n\nControls:\nSpace: Pause/Resume\n+/-: Double/Halve Speed\n1-6: Set Day Duration\nR: Reset\n\nDate & Time: {}\nSpeed: {:.1}x\nDay Duration: {:.1}s\nStatus: {}",
            clock.format_datetime(None),
            clock.speed,
            day_duration,
            status
        );
    }
}

fn handle_input(mut clock: ResMut<InGameClock>, keyboard: Res<ButtonInput<KeyCode>>) {
    // Toggle pause with spacebar
    if keyboard.just_pressed(KeyCode::Space) {
        clock.toggle_pause();
        println!(
            "Clock {}",
            if clock.paused { "paused" } else { "resumed" }
        );
    }

    // Increase speed with + or = (double the speed)
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        clock.speed = (clock.speed * 2.0).min(16384.0);
        println!("Clock speed: {:.1}x (1 day per {:.1}s)", clock.speed, clock.day_duration());
    }

    // Decrease speed with - (halve the speed)
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        clock.speed = (clock.speed * 0.5).max(0.0625);
        println!("Clock speed: {:.1}x (1 day per {:.1}s)", clock.speed, clock.day_duration());
    }

    // Set day duration with number keys
    if keyboard.just_pressed(KeyCode::Digit1) {
        clock.set_day_duration(30.0); // 1 day per 30 seconds
        println!("Clock speed: {:.1}x (1 day per 30s)", clock.speed);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        clock.set_day_duration(60.0); // 1 day per 1 minute
        println!("Clock speed: {:.1}x (1 day per 60s)", clock.speed);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        clock.set_day_duration(300.0); // 1 day per 5 minutes
        println!("Clock speed: {:.1}x (1 day per 5min)", clock.speed);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        clock.set_day_duration(600.0); // 1 day per 10 minutes
        println!("Clock speed: {:.1}x (1 day per 10min)", clock.speed);
    }
    if keyboard.just_pressed(KeyCode::Digit5) {
        clock.set_day_duration(1200.0); // 1 day per 20 minutes
        println!("Clock speed: {:.1}x (1 day per 20min)", clock.speed);
    }
    if keyboard.just_pressed(KeyCode::Digit6) {
        clock.set_day_duration(86400.0); // 1 day per 24 hours (real-time)
        println!("Clock speed: {:.1}x (real-time)", clock.speed);
    }

    // Reset with R
    if keyboard.just_pressed(KeyCode::KeyR) {
        clock.elapsed_seconds = 0.0;
        clock.speed = 1.0;
        clock.paused = false;
        println!("Clock reset");
    }
}