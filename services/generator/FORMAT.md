# COFFIN

**C**himeric **O**ffset **F**rankenstein **F**ormat for **I**mage **N**otation

The generator uses COFFIN internally as the intermediate representation between the template renderer and the scene compositor. Each payload is a sequence of typed, length-prefixed segments:
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
A text layer rendered on top of the SVG background using Parley layout.
Format: `x;y;wrap_width;alignment;font_family;font_size;font_weight;color;bg_color;text_content`

| Field          | Type     | Description                                                                                                                                                                                                          |
|----------------|----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `x`            | `i32`    | X-coordinate translation                                                                                                                                                                                             |
| `y`            | `i32`    | Y-coordinate translation                                                                                                                                                                                             |
| `wrap_width`   | `i32`    | Maximum width before wrapping the line (0 for no limit)                                                                                                                                                              |
| `alignment`    | `i32`    | Alignment of the text block. The last digit sets horizontal alignment (`0`=Left, `1`=Center, `2`=Right). The tens digit sets vertical alignment (`0`=Top, `1`=Baseline). E.g. `0` (Top Left), `12` (Baseline Right). |
| `font_family`  | `String` | Font family name (e.g. `sans-serif`, `Inter`, `Times New Roman`)                                                                                                                                                     |
| `font_size`    | `f32`    | Font size in pixels                                                                                                                                                                                                  |
| `font_weight`  | `u16`    | Standard font weight (e.g. 400 for regular, 700 for bold)                                                                                                                                                            |
| `color`        | `String` | Hex color code, optionally with alpha: `#RRGGBB` or `#RRGGBBAA`                                                                                                                                                      |
| `bg_color`     | `String` | (Optional) Hex color for text background rounded rectangle. Leave empty for no background.                                                                                                                           |
| `text_content` | `String` | The plain text to be rendered                                                                                                                                                                                        |

### 2: Chat ID
Used for matching the request to the origin chat/user when enqueueing the final generated image.

### 3: Rich Text
A text layer rendered on top of the SVG background that supports inline formatting and entities.
Format: `x;y;wrap_width;alignment;font_family;font_size;font_weight;color;monospace_color;link_color;entities;text_content`

| Field             | Type     | Description                                                                                                                                                                                                          |
|-------------------|----------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `x`               | `i32`    | X-coordinate translation                                                                                                                                                                                             |
| `y`               | `i32`    | Y-coordinate translation                                                                                                                                                                                             |
| `wrap_width`      | `i32`    | Maximum width before wrapping the line (0 for no limit)                                                                                                                                                              |
| `alignment`       | `i32`    | Alignment of the text block. The last digit sets horizontal alignment (`0`=Left, `1`=Center, `2`=Right). The tens digit sets vertical alignment (`0`=Top, `1`=Baseline). E.g. `0` (Top Left), `12` (Baseline Right). |
| `font_family`     | `String` | Font family name (e.g. `sans-serif`, `Inter`, `Times New Roman`)                                                                                                                                                     |
| `font_size`       | `f32`    | Font size in pixels                                                                                                                                                                                                  |
| `font_weight`     | `u16`    | Standard font weight (e.g. 400 for regular, 700 for bold)                                                                                                                                                            |
| `color`           | `String` | Hex color code for standard text                                                                                                                                                                                     |
| `monospace_color` | `String` | Hex color code for monospace entities (`code`, `pre`)                                                                                                                                                                |
| `link_color`      | `String` | Hex color code for link entities (`text_link`, `url`)                                                                                                                                                                |
| `entities`        | `String` | JSON array string of text entities                                                                                                                                                                                   |
| `text_content`    | `String` | The plain text to be rendered                                                                                                                                                                                        |

## Example
`0;22;<svg>...</svg>,1;45;10;10;200;1;sans-serif;16;400;#FF0000;Hello World,2;10;1234567890`
