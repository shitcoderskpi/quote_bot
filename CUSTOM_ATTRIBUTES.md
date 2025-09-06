# Custom `<text>` Attributes

### `wrap-width`
- **Type:** `int`
- **Description:** Sets the maximum width of the text. If the text exceeds this width, it will wrap into multiple lines.

### `wrap-mode`
- **Type:** `enum`
- **Options:**
  - `none` — No wrapping.
  - `word` — Wrap by word (default).
  - `word-char` — Wrap by word, but if a word is too long, wrap by character.
- **Description** Sets wrap mode.
