# Changelog

All notable changes to Blatman are recorded here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/) —
`0.x.y` means an initial version, still being tested.

This is `CHANGELOG.md`  which the GitHub
release notes are taken from). `scripts/release.ps1` moves the entries under
**[Unreleased]** into the new version when a release is made.

## [Unreleased]

## [0.2.3] - 2026-11-19
- MACOS removed as having issues with release !

## [0.2.2] - 2026-11-19

### Added
- **VPSI** (viewport scale information, from VPSI.LSP).
- **XFZ**, **XTW**, **XFIX** (xref layer freeze / thaw / fix to own layer, from XFIX.LSP) and **VPM** (viewport outline to model space, from VP-OUTLINE.LSP).
- Shortcuts from the lisp: `01`-`09` view presets, `ZE` zoom extents, `UV` restore the UCS saved by DVT, `UW` UCS World + view twist 0.
- **DVT** (twisted view), ported from DVT.LSP: draws the pick line on `defpoints`, sets UCS World then a UCS along the line, turns the view to plan, resets snap angle and current layer, and saves the UCS as `CURRENT_VIEW`.

### Changed
- Plugin renamed from Steal to **Tools** (`OCS_Plugin_Tools`): Steal is now one tool inside it. Plugin id is now `com.beebeesix.ocs-plugin-tools`, so remove the old `com.beebeesix.ocs-plugin-steal` plugin folder before installing.
- The log file is now `%TEMP%\ocs-tools.log`.

## [0.1.1] - 2026-11-17

### Added
LAYOUTS

### Changed


## [0.1.0] - 2026-11-17

### Added

### Fixed

