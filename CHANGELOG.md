# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-XX-XX

### Changed
- Update Bevy to 0.18

## [0.2.1] - 2025-11-30

### Added
- `set_elapsed_seconds(seconds)` method to programmatically set the clock's elapsed time
  - Enables save/load systems to restore the exact in-game time when loading saved games
  - Useful for any system that needs to synchronize the clock to a specific time state

## [0.2.0] - 2025-01-24

### Added
- **Custom Calendar System** - Support for non-Gregorian calendars (fantasy worlds, sci-fi settings)
  - `CustomCalendar` struct with fully configurable time units
  - Configurable minutes per hour, hours per day
  - Custom month definitions with names and varying day counts
  - Custom weekday names
  - Epoch support with custom names and start years
  - Expression-Based Leap Year System using boolean expressions:
  - Custom calendar example (`custom_calendar.rs`) demonstrating fantasy calendar loaded from RON file
  - Format specifier `%E` for era names in custom calendars
  - Format specifier `%A` for weekday names in custom calendars
  - Format specifier `%B` for custom month names


## [0.1.0] - Initial Release

### Added
- Core in-game clock functionality
- Date & time tracking with configurable start date/time
- Flexible speed control (multiplier or real-time duration per day)
- Pause and resume functionality
- Flexible datetime formatting using chrono
- Event system for time-based intervals (hourly, daily, custom)
- `InGameClockPlugin` for easy Bevy integration
- Examples: basic, events, digital_clock

[0.3.0]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.3.0
[0.2.1]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.2.1
[0.2.0]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.2.0
[0.1.0]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.1.0