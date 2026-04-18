# Architect Mode: Fixes & Improvements Needed

The following issues were identified during initial testing of the Story Architect (Index Cards) mode.

## 🛠️ Outstanding Issues

### 1. New Scene Insertion is Broken
- **Symptom**: Pressing `n` does not correctly insert a new scene card or the buffer sync fails.
- **Potential Cause**: `add_card` in `src/app/methods/index_cards.rs` inserts lines into `self.lines` but might not be correctly triggering a full re-parse or handling empty buffer states.
- **Task**: Debug the `add_card` logic and ensure `selected_card_idx` updates to the newly created card immediately.

### 2. Wonky Card Shifting (Vertical Swaps)
- **Symptom**: Swapping cards horizontally (Shift+Left/Right) works, but swapping vertically (Shift+Up/Down) does nothing or behaves unexpectedly.
- **Cause**: The current `swap_cards` implementation only supports **adjacent** scene blocks. Swapping vertically across a 3-column grid is a non-adjacent swap.
- **Task**: Update `swap_cards` to handle non-adjacent block movement (Cut/Paste logic).

### 3. Missing Marker Colors
- **Symptom**: Cards are monochromatic. The Scene Navigator shows color-coded scenes based on metadata, but Index Cards do not.
- **Task**: Extract the marker color for each scene heading in `extract_scene_cards` and apply it to the card header or border in the UI.

---
