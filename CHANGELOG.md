# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-05-30

### Added

- Fallible conversion support with error handling, accumulation, and mapping for fallible conversions.

## [0.7.1] - 2026-04-18

### Added

- Box-related hints.

## [0.7.0] - 2026-02-09

### Changed

- Updated workspace package to Rust 2024 edition.

## [0.6.2] - 2026-01-29

### Changed

- Improved support for generic parameters.
- Allowed `&self` in `with` methods.
- Refined examples and config helper documentation.

## [0.6.1] - 2025-01-20

### Changed

- Improved `with` usability when `is_try` is active.

## [0.6.0] - 2025-01-20

### Changed

- Added hints for derive, removing the need for a helper `with` module.

## [0.5.0] - 2024-09-02

### Added

- Support for renaming skipped fields.

## [0.4.4] - 2024-09-02

### Added

- Support for skipped and additional fields on default values.

## [0.4.3] - 2024-08-26

### Added

- Chrono mapper.

## [0.4.2] - 2024-07-29

### Added

- Workspace-level `std` feature flag.

## [0.4.1] - 2024-07-28

### Changed

- Gated the `with` helper configuration behind a feature flag.

## [0.4.0] - 2024-07-18

### Added

- Example showing `no_std` usage.

### Changed

- Refactored to support custom derives, improved documentation, and refined errors.

### Fixed

- Fixed spelling issues.

## [0.3.1] - 2024-07-13

### Removed

- The `try_with` helper and its variants.

## [0.3.0] - 2024-07-13

### Changed

- Refactored `with` helpers, allowing different `with` in `From` and `Into` implementations.

### Fixed

- General documentation fixes.

## [0.2.2] - 2024-07-03

### Added

- Support for helper `extra` functions.

## [0.2.1] - 2023-12-03

### Added

- Option to ignore all extra fields and variants.

## [0.2.0] - 2023-10-13

### Added

- Customization options for default values.

### Fixed

- Fixed license formatting.

## [0.1.0] - 2023-10-12

### Added

- Initial commit and base workspace implementation.
