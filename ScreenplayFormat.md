# Screenplay Page Export Formatting Specifications

This document outlines the exact page layout and element-by-element specifications used for generating final, export-quality PDFs in the Fount screenplay editor. These values are derived directly from the application's stylesheet configurations and printing engine.

---

## 1. Page Geometry & Typography

### Base Metrics
- **Font**: Courier (Courier Prime) at exactly **12pt** (monospaced, where 1 character width = 7.2 points / 10 pitch, and 1 line height = 12 points).
- **Line Spacing**: Exactly 12pt (6 lines per inch).

### Paper Sizes
- **US Letter**: 8.5 in × 11.0 in (612 pt × 792 pt).
- **A4**: 8.27 in × 11.69 in (595.3 pt × 841.9 pt).

### Base Page Margins (Relative to Paper Edge)
- **Top Margin**: 40 points (approx. 0.56 in / 3.3 lines).
- **Bottom Margin**: 50 points (approx. 0.69 in / 4.2 lines).
  - *A4 Offset*: A4 pages add an extra 10 points to the bottom margin (60 pt total) to accommodate the longer page length.
- **Left Margin (Binding Margin)**: Calculated by adding container padding + page margin:
  - **US Letter**: 60 pt (padding-left) + 30 pt (margin-left) + 22 pt (margin-left-us) = **112 points (1.55 inches)**.
  - **A4**: 60 pt (padding-left) + 30 pt (margin-left) + 20 pt (margin-left-a4) = **110 points (1.53 inches)**.
- **Right Margin**: 25 points (approx. 0.35 in) from the right boundary of the container.

---

## 2. Element Layout Sizing & Indentations

All horizontal widths and left margins below are defined in character units (`ch`), where `1ch = 7.2 pt`. The horizontal indents are measured from the starting boundary of the left margin.

| Element Type | Target Width (US Letter) | Left Margin Indent (US Letter) | Target Width (A4) | Left Margin Indent (A4) | Spacing Before (Lines) | Casing & Styling |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Action** | 63 ch (453.6 pt) | 7 ch (50.4 pt) | 60 ch (432.0 pt) | 7 ch (50.4 pt) | 1 line | Sentence case |
| **Scene Heading** | 57 ch (410.4 pt) | 7 ch (50.4 pt) | 55 ch (396.0 pt) | 7 ch (50.4 pt) | 2 lines | UPPERCASE, Bold (Optional) |
| **Character Cue** | 40 ch (288.0 pt) | 26 ch (187.2 pt) | 38 ch (273.6 pt) | 26 ch (187.2 pt) | 1 line | UPPERCASE |
| **Parenthetical** | 28 ch (201.6 pt) | 22 ch (158.4 pt)* | 28 ch (201.6 pt) | 22 ch (158.4 pt)* | 0 lines | Mixed case, wrapped in `()` |
| **Dialogue** | 35 ch (252.0 pt) | 17 ch (122.4 pt) | 35 ch (252.0 pt) | 17 ch (122.4 pt) | 0 lines | Sentence case |
| **Transition** | Full Width | Right Aligned | Full Width | Right Aligned | 1 line | UPPERCASE |
| **Centered Text** | 62 ch (446.4 pt) | Centered | 60 ch (432.0 pt) | Centered | 1 line | Sentence case |
| **Lyrics** | Full Width | Centered | Full Width | Centered | 1 line | Italic |
| **Shot** | Full Width | 7 ch (50.4 pt) | Full Width | 7 ch (50.4 pt) | 1 line | UPPERCASE, Bold (Optional) |

> [!NOTE]
> \* **Parenthetical Left Indentation Offset**: The PDF exporter applies an additional hard-coded indent of **7.25 points** (approx. 1 character) to parentheticals on top of their base indent, resulting in a total left indent of `22ch + 7.25pt` (approx. 165.6 pt / 2.3 inches).

---

## 3. Dual Dialogue Column Formatting

When two characters speak simultaneously (marked by a `^` trailing character), the exporter splits the text area into two equal horizontal columns:

- **Left Column Width**: 50% of the page width.
- **Right Column Width**: 50% of the page width.

### Left & Right Column Element Specifications:
- **Character Name**:
  - Width: 21 ch (US Letter) / 20 ch (A4)
  - Left Margin: 7 ch
- **Parenthetical**:
  - Width: 26 ch (US Letter) / 25 ch (A4)
  - Left Margin: 4 ch
- **Dialogue**:
  - Width: 28 ch (US Letter) / 27 ch (A4)
  - Left Margin: 0 ch (aligned with the column's left edge)
  - Right Margin: 1 ch (adds padding between columns)

---

## 4. Title Page Formatting

The title page acts as a standalone sheet at the start of the screenplay document. It does not display headers, footers, or page numbers.

### Page Margins & Bounds
- **Top Margin**: 40 pt (approx. 0.56 in)
- **Bottom Margin**: 65 pt (approx. 0.90 in)
- **Left Margin**: 40 pt (approx. 0.56 in)
- **Right Margin**: 40 pt (approx. 0.56 in)

### Top Block (Title & Authorship Info)
This block is drawn inside a single text container spanning the full width of the margins (`Page Width - 80 pt`) with a height of exactly **400 pt**, positioned at `y = 40 pt` (from the page top edge):
- **Title (`title`)**:
  - Position: Offset from the top of the container by 20 lines (**240 pt** / 3.33 in).
  - Styling: Centered, UPPERCASE.
- **Credit (`credit`)**:
  - Position: 1 blank line spacing (**12 pt**) below the title block.
  - Styling: Centered.
- **Author/Authors (`authors`)**:
  - Position: 1 blank line spacing (**12 pt**) below the credit block.
  - Styling: Centered.
- **Source (`source`)**:
  - Position: 1 blank line spacing (**12 pt**) below the author block.
  - Styling: Centered.

### Bottom Block (Contact & Date Info)
This block starts directly below the Top Block at `y = 440 pt`. The available height is `Page Height - 440 pt - 65 pt (bottom margin) - 48 pt (approx. 4 line heights for buffer)`.
The horizontal layout is split into two bottom-aligned columns:
- **Left Column (65% width)**:
  - Sizing: Width is `(Page Width - 80 pt) * 0.65 - 10 pt`.
  - Contents: **Contact Info (`contact`)** followed by any other custom title page metadata.
  - Styling: Left-aligned, **bottom-aligned** vertically.
- **Right Column (35% width)**:
  - Sizing: Width is `(Page Width - 80 pt) * 0.35 - 10 pt`.
  - Contents: **Draft Date (`draft date`)**.
  - Styling: Right-aligned, **bottom-aligned** vertically.

---

## 5. Screenplay Pagination & Spacing Rules

Screenplays must follow rigid page-splitting guidelines to prevent structural errors during printing:

### Page Numbering
- **First Page**: The page number is hidden on Page 1(Title page is not included in this).
- **Subsequent Pages**: Page numbers start on Page 2 (printed as `2.`, `3.`, etc.).
- **Position**: Printed at the top-right corner of the page, 0.5 inches (36 pt) below the top edge, aligned with the right edge of the text boundary.

### Scene Headings (Orphans Protection)
- A scene heading **cannot** stand alone at the bottom of a page. 
- If there is not enough vertical space for the heading plus at least **two lines** of subsequent Action or Dialogue, the entire scene heading is pushed to the top of the next page.

### Dialogue Splits
- If a block of dialogue must split across a page boundary:
  - A minimum of **two lines** of dialogue must remain on the first page.
  - A minimum of **two lines** of dialogue must carry over to the second page.
  - A centered dialogue break indicator `(MORE)` is inserted at the bottom of the first page.
  - The character's name followed by `(CONT'D)` is printed at the top of the second page.
  - Single-line dialogue blocks or blocks containing a parenthetical with only one line of dialogue are never split; the entire speech is pushed to the next page.
