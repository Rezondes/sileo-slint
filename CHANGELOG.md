# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Toaster` component with six positions, a theme that follows the color scheme, and
  settings for duration, roundness, offset and gap.
- `Sileo` global to show, dismiss and clear toasts from Slint and from Rust, with
  shortcuts for success, error, warning, info and loading toasts.
- Toasts of the kinds success, loading, error, warning, info and action, with an
  optional description, button, icon, fill and roundness.
- Morphing: showing a toast with the id of a visible toast changes that toast in place.
- Autopilot that expands a toast after it appears and collapses it before it leaves.
- Expand on hover, pause of all timers while hovering, and swipe to dismiss.
- `sileo_slint::library_paths()` for the build script and `sileo_slint::init!` to back
  the toast list.
- Demo example with a frame dump for visual checks.
