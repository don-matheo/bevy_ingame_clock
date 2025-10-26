# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-24

### Added
- **Custom Calendar System** - Support for non-Gregorian calendars (fantasy worlds, sci-fi settings)
  - `CustomCalendar` struct with fully configurable time units
  - Configurable minutes per hour, hours per day
  - Custom month definitions with names and varying day counts
  - Custom weekday names (number of weekdays determines days per week)
  - Epoch support with custom names and start years
  - Expression-Based Leap Year System using boolean expressions:
    - Define leap year rules using simple string expressions with `#` as the year placeholder
    - Expression syntax supports: arithmetic (`%`, `+`, `-`, `*`, `/`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical operators (`&&`, `||`, `!`)
    - Examples: `"# % 4 == 0"` (every 4 years), `"# % 2 == 0"` (every 2 years), `"# % 4 == 0 && (# % 100 != 0 || # % 400 == 0)"` (Gregorian rule)
    - Powered by the `evalexpr` crate for expression evaluation
  - `leap_days` field per month to distribute leap days across months
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

[0.2.0]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.2.0
[0.1.0]: https://github.com/don-matheo/bevy_ingame_clock/releases/tag/v0.1.0