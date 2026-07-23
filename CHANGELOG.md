## Version 0.2.0 - 2026-07-23

### 🚀 Features

- *(cuelist)* PgUp and PgDn navigate to mark
- *(cuelist)* Home and End navigate to top and bottom of cuelist
- *(cue preview)* Resize program and preview display with +/-
- *(hotkeys)* Undo go and blank with INS key
- *(hotkeys)* Toggle autogo learn with SHIFT+T
- *(cue preview)* Countdown bar colors
- *(autogo)* Hint mode
- *(timecode)* ClicKS network timecode integration

### 🐛 Bug Fixes

- *(cuelist)* Fix cuelist width too small
- *(cuelist)* APPEND CUE now autoidents <ident>.1
- *(driver)* Fix syntax error
- *(cmdline)* Remove debug print
## Version 0.1.0 - 2026-07-14

### 💼 Other

- I2c, multidisplay
- Unified brightness
- I2c
- Correct size and brightness
- Correct size
- Remove old fonts, add new sans font
- Remove old transitions, add 4 fade speeds
- Remove PatchPointer and increase max cues to 12
- Selected_sequence_mut
- Commandline upgrades: range selection, insert and delete cues and seqs
- Add mark and description fields to Cue
- Support 12 sequences in commandline
- Edit mode
- Editable text fields in cuetable
- Fix goto ident parsing
- Add TIME cmd to remove autogo settings
- T to toggle autogo
- Update stuff
- Split c file, accurate buffer length
- Some performance optimisations
- (untested) double wave buffer
- 32 step full range brightness animation
- Clock divider for animation clock
- Transition::duration()
- TextContent::is_blank()
- Render fade transition and feature-gate startup images
- Add cargo workspace
- Add logicplot tool for esds driver
- Update CHANGELOG.md and Release 0.1.0

### 📚 Documentation

- Update RELEASE.md

### ⚙️ Miscellaneous Tasks

- Add cargo dist support for tekst and tekst-receiver
- Add RELEASE.md
- Git cliff init
- Fix cargo profiles
- Update CHANGELOG
- Update CHANGELOG.md and release 0.1.0
- Allow dirty release yml
