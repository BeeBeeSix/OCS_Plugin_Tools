# Changelog

All notable changes to Tools are recorded here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/) —
`0.x.y` means an initial version, still being tested.

This is `CHANGELOG.md`  which the GitHub
release notes are taken from). `scripts/release.ps1` moves the entries under
**[Unreleased]** into the new version when a release is made.

## [Unreleased]

## [0.3.0] - 2026-11-23

### Fixed
- **S2P** did nothing for splines drawn through fit points (they have no control points) and gave only the control polygon for the others. It now follows the curve of both kinds, as the host draws it; closed and periodic splines are skipped.
- **API** Fixed for OCS-V2026.47

## [0.2.4] - 2026-11-19
### Added
- Ported from the old VBA macros: **3PB**, **3PP**, **3PV** (RL blocks / points on a 3D polyline), **3B** (make breaklines), **3P2P**, **2P3P**, **3P0**, **3DF**, **3DL**, **S2P** (polyline conversions), **RLD**, **RLG** (RL difference, grade), **CTM**, **CTI**, **CTS**, **CCBL** (contours), **C*D** (batch runner, this drawing only) and **ZXL** (zoom extents on every layout). An internal `TTAPPLY` command applies the changes of the pick-driven ones.
- `TOOLSHELP` also prints the version of the loaded library, to tell an old library from a new one.

### Fixed
- **VPM** / **VPSI** from MSPACE: when no viewport is found by the layout, every viewport in the drawing is offered to the click; the "none found" message now says how many viewports the drawing has.

### Changed
- **DSM** / **DSMM**: the scale factor is typed as text, with `p` for a decimal point (`2p5` = 2.5, named `1:2p5` / `1:2p5m`); decimal factors are no longer rounded in the scale name.
- **VPM** and **VPSI** work from MSPACE of a layout: select nothing, run the command and click inside the active viewport. VPSI shows the scale in the prompt and keeps asking until you press Enter.

### Added
- **DSM** and **DSMM** (drawing scale in metres / millimetres, from DSM.LSP): ask for the scale factor, add the annotation scale to the scale list, make it current and set `DIMSCALE`.

## [0.2.4] - 2026-11-19

### Added
- **TOOLSHELP** and a **Help** ribbon button: opens the built-in help page (`help/ocs_tools_help.html`) in the browser.
- Aliases from TT-START.LSP (see the README table): `Y`, `UC3`, `C2`, zoom shortcuts, layer-wide `F*`/`O*`/`T*`/`A*`, osnap `OE`/`OI`/`ON`, purge `PUA`/`PULI`/`PUB`, units `AH`/`AV` and more.

## [0.2.3] - 2026-11-19
- MACOS removed as having issues with release !

## [0.2.2] - 2026-11-19
- **VPSI** (viewport scale information, from VPSI.LSP).
- **XFZ**, **XTW**, **XFIX** (xref layer freeze / thaw / fix to own layer, from XFIX.LSP) and **VPM** (viewport outline to model space, from VP-OUTLINE.LSP).
- Shortcuts from the lisp: `01`-`09` view presets, `ZE` zoom extents, `UV` restore the UCS saved by DVT, `UW` UCS World + view twist 0.
- **DVT** (twisted view), ported from DVT.LSP: draws the pick line on `defpoints`, sets UCS World then a UCS along the line, turns the view to plan, resets snap angle and current layer, and saves the UCS as `CURRENT_VIEW`.

### Fixed
- `OE`, `OI`, `ON`, `AH` and `AV` set their variables through the host's `SETVAR` command; the plugin API can only set `CLAYER` and `SNAPANG` directly.

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
