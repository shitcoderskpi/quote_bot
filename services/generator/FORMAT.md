# COFFIN v2.1 "Nail"

**C**himeric **O**ffset **F**rankenstein **F**ormat for **I**mage **N**otation

The generator uses COFFIN syntax to define the structural blocks inside `.tem` template files. Each block is a sequence of typed, length-prefixed segments:
```text
type;length;data[,]
```

- `type`: An integer representing the message type
- `length`: The byte length of the payload
- `data`: The exact payload string
- `,`: An optional comma separating this segment from the next

## Message Types

### 0: SVG
The raw text data is parsed as an SVG document and rendered behind any text layers.

### 1: Text
A text layer rendered on top of the SVG background using Parley layout. Supports inline formatting and entities.
Format: `x;y;wrap_width;alignment;font_family;font_size;font_weight;color;bg_color;monospace_color;link_color;entities_key;text_content`

| Field             | Type     | Description                                                                                                                                                                                                          |
|-------------------|----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `x`               | `i32`    | X-coordinate translation                                                                                                                                                                                             |
| `y`               | `i32`    | Y-coordinate translation                                                                                                                                                                                             |
| `wrap_width`      | `i32`    | Maximum width before wrapping the line (0 for no limit)                                                                                                                                                              |
| `alignment`       | `i32`    | Alignment of the text block. The last digit sets horizontal alignment (`0`=Left, `1`=Center, `2`=Right). The tens digit sets vertical alignment (`0`=Top, `1`=Baseline). E.g. `0` (Top Left), `12` (Baseline Right). |
| `font_family`     | `String` | Font family name (e.g. `sans-serif`, `Inter`, `Times New Roman`)                                                                                                                                                     |
| `font_size`       | `f32`    | Font size in pixels                                                                                                                                                                                                  |
| `font_weight`     | `u16`    | Standard font weight (e.g. 400 for regular, 700 for bold)                                                                                                                                                            |
| `color`           | `String` | Hex color code for standard text, optionally with alpha: `#RRGGBB` or `#RRGGBBAA`                                                                                                                                    |
| `bg_color`        | `String` | (Optional) Hex color for text background rounded rectangle. Leave empty for no background.                                                                                                                           |
| `monospace_color` | `String` | (Optional) Hex color code for monospace entities (`code`, `pre`)                                                                                                                                                     |
| `link_color`      | `String` | (Optional) Hex color code for link entities (`text_link`, `url`)                                                                                                                                                     |
| `entities_key`    | `String` | (Optional) Key matching a JSON array of parsed text entities for this specific field                                                                                                                                 |
| `text_content`    | `String` | The plain text to be rendered                                                                                                                                                                                        |

### 2: Chat ID
Used for matching the request to the origin chat/user when enqueueing the final generated image.

## Example
`0;22;<svg>...</svg>,1;54;10;10;200;1;sans-serif;16;400;#FF0000;;;;;Hello World,2;10;1234567890`
