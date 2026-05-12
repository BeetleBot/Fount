# Implementation Plan: Version History System

This document outlines the plan for implementing a robust Version History system in FountTUI, allowing users to track, compare, and revert changes in their screenplays.

## Objectives
- Allow users to manually save "named versions" (e.g., "First Draft", "Revisions for Director").
- Implement an automatic versioning system that captures states at key milestones.
- Provide a UI for browsing history, viewing statistics, and restoring previous states.
- (Phase 2) Add side-by-side diffing to visualize changes between versions.

## 1. Architecture & Data Model

### Data Structure
Introduce a `Version` struct that extends the concept of a Snapshot:
```rust
pub struct Version {
    pub label: String,          // User-provided or auto-generated
    pub path: PathBuf,          // Path to the .fountain file in the versions dir
    pub timestamp: SystemTime,
    pub word_count: usize,
    pub scene_count: usize,
}
```

### Storage
- Versions will be stored in `<data_dir>/versions/<script_name>/`.
- Unlike Snapshots (which are transient and pruned), Versions are persistent until manually deleted.

### VersionManager
- Create a `VersionManager` to handle listing, saving, and pruning (if configured).

## 2. Logic Implementation

### Commands
- `/version save [label]`: Captures the current state and saves it with the given label.
- `/version list`: Opens the Version History modal.
- `/version restore`: (Contextual) Restores the selected version.

### Integration
- Hook into the `save` workflow: Optional config `auto_version_on_manual_save`.
- Milestone triggers: Create a version every 1000 words or 10 scenes (configurable).

## 3. UI Design

### Version History Modal
- A centered table similar to the Snapshots/Sprints view.
- Columns: `Label`, `Date`, `Words`, `Scenes`.
- Selection highlights the version and shows a summary in the footer.

### Interactions
- `Enter`: Restore version (replaces current buffer, with an "Undo" backup).
- `O`: Open version in a new buffer.
- `R`: Rename selected version label.
- `D`: (Phase 2) Open Diff view.

## 4. Implementation Steps

### Step 1: Foundation
- Create `src/app/version.rs`.
- Implement `Version` and `VersionManager`.
- Update `App` struct to include `VersionManager` and `versions` list state.

### Step 2: Commands & Basic Saving
- Implement `/version save` logic.
- Add label prompting UI for manual saves.

### Step 3: Version Browser UI
- Implement `draw_version_history` in `src/app/ui/panes/mod.rs` (or a dedicated file).
- Add `AppMode::VersionHistory`.

### Step 4: Restoration & Buffers
- Implement `restore_version` logic.
- Ensure restoration is "safe" (captures a temporary snapshot of current state before overwriting).

### Step 5: Advanced Features (Future)
- Side-by-side diff view using the `similar` crate.
- Visualizing word count growth over time in the X-Ray panel.

## 5. Tasks List
- [ ] Research `similar` crate for diffing performance.
- [ ] Define precise directory structure for per-file history.
- [ ] Implement `Version` struct and serialization.
- [ ] Build the Version History UI table.
- [ ] Integrate with Command Bar for labeled saves.
- [ ] Add "Open in New Buffer" support for versions.
