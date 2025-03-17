<!-- next-header -->

## [Unreleased] - ReleaseDate

### Fixed

- Ensure compatability with rust 1.82

## [0.6.0] - 2025-03-16

### Added

- man files and shell completions (they can be generated using `cargo xtask`)

### Fixed

- [timings with spaces now work without enclosing them in quotation marks](https://github.com/Wasabi375/procrastinate/issues/11)
- [The day after "monthly" is now optional](https://github.com/Wasabi375/procrastinate/issues/12)
- [Fixed next notification date in list command](https://github.com/Wasabi375/procrastinate/issues/13)

## [0.5.0] - 2024-10-05

### Breaking

- Procrastinate now differentiates between Time and Day based notification delays.
    A delay of "1d" will now be the same as tomorrow and will notify at the first
    time notifications are checked on the next day. It will no longer notify in 
    exactly 24 hours. 
    To notify in 24 hours use "24h" instead.

    This change comes with a update to the ron file format.
    `Delay(( secs: <secs>, nanos: 0))` will need to be changed to `Delay(Seconds(<secs>))`
    or `Delay(Days(<days>))`.
    For ease of conversion there are 86400 seconds in a day and 2592000 seconds in
    a month (30 days).

### Added

- added "--ron" format option for "procrastinate list"

## [0.4.1] - 2024-09-29

### Changed

- Changed `procrastinate list` output text from "last notified" to "last notification"

## [0.4.0] - 2024-09-17

### Added

- Added changelog
- [`procrastinate list` command to print all scheduled notifications](https://github.com/Wasabi375/procrastinate/issues/2)
- [sleep repeating notifications - repeat them once at a different interval](https://github.com/Wasabi375/procrastinate/issues/6)
- [allow for sticky notification that only vanish when clicked](https://github.com/Wasabi375/procrastinate/issues/3)


## [0.3.2] - 2024-09-13

### Features
- schedule one-time and repeating notifications
- support for local(directory based) and user notifications
- background daemon to display scheduled notifications

<!-- next-url -->
[Unreleased]: https://github.com/wasabi375/procrastinate/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/wasabi375/procrastinate/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/wasabi375/procrastinate/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/wasabi375/procrastinate/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/wasabi375/procrastinate/compare/0.3.2...v0.4.0
[0.3.2]: https://github.com/wasabi375/procrastinate/compare/cd38477e3a142789371bf512c0fe2fb524e97c80...0.3.2
