//! Example demonstrating a visual digital clock with date display.
//!
//! This example shows:
//! - Digital time display with segment-style visualization
//! - Date calendar display
//! - Vintage digital clock styling
//! - Interactive speed controls

use bevy::prelude::*;
use bevy_ingame_clock::{InGameClock, InGameClockPlugin};
use chrono::Datelike;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InGameClockPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (update_time_display, update_date_display, handle_input))
        .run();
}

#[derive(Component)]
struct TimeDisplay;

#[derive(Component)]
struct DateDisplay;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(InGameClock::default());

    // Main display background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.08, 0.12, 0.18),
            custom_size: Some(Vec2::new(600.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, -1.0),
    ));

    // Display border
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.25, 0.35),
            custom_size: Some(Vec2::new(620.0, 220.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, -2.0),
    ));

    // Time display
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        width: Val::Px(600.0),
        height: Val::Px(60.0),
        left: Val::Percent(50.0),
        top: Val::Percent(40.0),
        margin: UiRect {
            left: Val::Px(-300.0),
            ..default()
        },
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 100.0,
                ..default()
            },
            TextColor(Color::srgb(0.3, 0.8, 0.9)),
            TimeDisplay,
        ));
    });

    // Date display background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.18, 0.25),
            custom_size: Some(Vec2::new(400.0, 80.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -150.0, 0.0),
    ));

    // Date display border
    commands.spawn((
        Sprite {
            color: Color::srgb(0.15, 0.25, 0.35),
            custom_size: Some(Vec2::new(420.0, 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -150.0, -1.0),
    ));

    // Date display
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        width: Val::Px(400.0),
        height: Val::Px(80.0),
        left: Val::Percent(50.0),
        top: Val::Percent(65.0),
        margin: UiRect {
            left: Val::Px(-200.0),
            ..default()
        },
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(0.3, 0.8, 0.9)),
            DateDisplay,
        ));
    });

    // Controls text
    commands.spawn((
        Text::new("Digital Clock\n\nControls:\nSpace: Pause/Resume\n+/-: Speed Up/Down\nR: Reset"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        },
    ));

    // Speed indicator
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
    ));
}

fn update_time_display(
    clock: Res<InGameClock>,
    mut query: Query<&mut Text, With<TimeDisplay>>,
) {
    if let Ok(mut text) = query.single_mut() {
        let (hour, minute, second) = clock.as_hms();
        **text = format!("{:02}:{:02}:{:02}", hour, minute, second);
    }
}

fn update_date_display(
    clock: Res<InGameClock>,
    mut query: Query<&mut Text, With<DateDisplay>>,
) {
    if let Ok(mut text) = query.single_mut() {
        let (year, month, day) = clock.current_date();
        let weekday = clock.current_datetime().weekday();
        
        **text = format!(
            "{}, {:02} {} {}",
            weekday,
            day,
            match month {
                1 => "January", 2 => "February", 3 => "March", 4 => "April",
                5 => "May", 6 => "June", 7 => "July", 8 => "August",
                9 => "September", 10 => "October", 11 => "November", 12 => "December",
                _ => "Unknown"
            },
            year
        );
    }
}

fn handle_input(
    mut clock: ResMut<InGameClock>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Toggle pause
    if keyboard.just_pressed(KeyCode::Space) {
        clock.toggle_pause();
    }

    // Increase speed
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        clock.speed = (clock.speed * 2.0).min(16384.0);
        println!("Speed: {:.1}x (1 day per {:.1}s)", clock.speed, clock.day_duration());
    }

    // Decrease speed
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        clock.speed = (clock.speed * 0.5).max(0.0625);
        println!("Speed: {:.1}x (1 day per {:.1}s)", clock.speed, clock.day_duration());
    }

    // Reset to realtime
    if keyboard.just_pressed(KeyCode::KeyR) {
        *clock = InGameClock::default();
        println!("Clock reset to realtime");
    }
}