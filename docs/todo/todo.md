# Fount: Engineering Roadmap & Todo

This document outlines the planned architectural improvements and technical debts to address in Fount's core engine.

## 🏛️ Architectural Improvements

### [ ] Refactor the Layout Engine (`src/layout.rs`)
> [!IMPORTANT]
> The `build_layout` function is currently a monolithic loop exceeding 400 lines. This complexity makes it difficult to debug and prevents easy extension.

**Goal**: Decompose `build_layout` into specialized sub-processors:
- **PaginationProcessor**: Manage page breaks and numbering.
- **DialogueProcessor**: Handle "Continued" extensions and dual-dialogue logic.
- **SceneProcessor**: Handle scene numbering and mirroring.
- **MarkupProcessor**: Mapping Fountain markers to visual formatting.

### [ ] Implement Incremental Parsing & Layout
Currently, Fount re-processes the entire document on most changes. This will cause lag in scripts over 100 pages.
- **Task**: Implement a "Dirty Region" tracking system to only re-parse and re-layout changed scenes or pages.

---

## ⚡ Performance & Code Quality

### [ ] Optimize Parser Throughput (`src/parser.rs`)
- **Issue**: Every line is converted to a `Vec<char>` for Unicode handling, which is memory-heavy for large files.
- **Refactor**: Switch to direct `chars()` iteration or byte-offset tracking to reduce redundant allocations.

### [ ] Strengthen Error Handling
- **Issue**: Widespread use of `.unwrap()` or unchecked `Option` values in command execution and file I/O.
- **Task**: Introduce a custom `FountError` enum and migrate `src/app/methods/` to use the `Result` pattern systematically.

---

## 🛠️ Feature Backlog

- [ ] **Search & Replace**: Add a global command to find and replace terms across the buffer.
- [ ] **Live Navigator Preview**: Scroll the editor background dynamically as the user moves through the Scene Navigator (`Ctrl+H`).
- [ ] **Sticky Headings**: Display the current scene name in the footer or pin the scene heading to the top of the viewport during scrolling.

---

## ✅ Completed
- [x] **Match Count**: Show `[X/Y]` in the status bar during search navigation.
- [x] **Navigation Shortcuts**: `Alt+Up` / `Alt+Down` for jumping between search matches.
- [x] **Buffer Tabs**: Minimal adaptive tab bar for multi-buffer workflows.
- [x] **Save Prompt**: Updated `/w` to prompt for filenames on unnamed buffers.
- [x] **Dirty Indicator**: Visual `*` in status bar when a buffer has unsaved changes.