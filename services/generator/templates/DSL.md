# Template DSL Reference

Steel Scheme (`.scm`) template files build a **Node tree** using registered Rust functions. The template's last expression must evaluate to a `node`.

---

## Payload Access

```scheme
(define payload ...)  ;; auto-injected as assoc list from the JSON input

;; Helper — returns value for key, or default if missing
(define (get-payload key default-val)
  (let ((found (assoc key payload)))
    (if (and found (not (null? (cdr found))))
        (cdr found)
        default-val)))
```

---

## Units / Dimensions

| Call      | Description                           |
|-----------|---------------------------------------|
| `(px N)`  | Pixels (absolute)                     |
| `(pt N)`  | Points → pixels (×1.333 at 96 dpi)    |
| `(pct N)` | Percentage (0–100, mapped to 0.0–1.0) |
| `(auto)`  | Auto sizing                           |

---

## Colors

| Call             | Example              | Description                                  |
|------------------|----------------------|----------------------------------------------|
| `(hex S)`        | `(hex "#FF0000")`    | Hex string `#RGB`, `#RRGGBB`, or `#RRGGBBAA` |
| `(rgb R G B)`    | `(rgb 255 0 0)`      | RGB, each 0–255                              |
| `(rgba R G B A)` | `(rgba 255 0 0 128)` | RGBA, each 0–255                             |

---

## Paints (fill / stroke values)

| Call                                    | Description                                                                                                                          |
|-----------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `(solid COLOR)`                         | Solid fill. `(solid (hex "#FFF"))`                                                                                                   |
| `(linear-gradient COLOR COLOR)`         | Simple top→bottom gradient. `(linear-gradient (hex "#fff") (hex "#000"))`                                                            |
| `(linear-gradient ANGLE STOP STOP ...)` | Rich gradient with custom angle and stops                                                                                            |
| `(radial-gradient STOP STOP ...)`       | Radial gradient from center to edge. `(radial-gradient (stop 0 (hex "#fff")) (stop 100 (hex "#000")))`                               |
| `(sweep-gradient ANGLE ANGLE STOP ...)` | Sweep gradient from start angle to end angle. `(sweep-gradient (angle 0) (angle 360) (stop 0 (hex "#fff")) (stop 100 (hex "#000")))` |

### Rich gradient helpers

| Call               | Description                                                 |
|--------------------|-------------------------------------------------------------|
| `(angle DEG)`      | Gradient angle in degrees. `(angle 45)`                     |
| `(stop PCT COLOR)` | Gradient color stop. `(stop 0 (hex "#f00"))` — pct is 0–100 |

**Example:**
```scheme
(linear-gradient (angle 135)
  (stop 0   (hex "#FF6B6B"))
  (stop 50  (hex "#4ECDC4"))
  (stop 100 (hex "#45B7D1")))
```

---

## Shapes

| Call                           | Description                                                            |
|--------------------------------|------------------------------------------------------------------------|
| `(circle)`                     | Circle (inscribed in the node's box)                                   |
| `(rect)`                       | Rectangle with sharp corners                                           |
| `(rounded-rect DIM)`           | Rectangle with uniform corner radius. `(rounded-rect (px 16))`         |
| `(rounded-rect TL TR BR BL)`   | Per-corner radii. `(rounded-rect (px 16) (px 16) (px 16) (px 0))`      |
| `(svg-path STR)`               | Arbitrary SVG path data. `(svg-path "M 10 0 Q 10 18 0 18 L 10 18 Z")`  |

---

## Node Constructors

### `(node [STYLE] CHILDREN...)`

Creates a layout node. First arg can be a `style`; remaining args are child nodes, text, or shapes.

```scheme
(node (style (flex-row) (gap (px 10)))
  (text "Hello" (size 16))
  (text "World" (size 16)))
```

Children that evaluate to `void` or `'()` are silently skipped (useful for conditional children via `if`/`when`).

A `node` with no explicit `style` uses default flex layout.

---

### `(image BASE64_STR [CLIP_SHAPE])`

Creates an image node from a base64 encoded string (JPEG, PNG, WebP). Optionally specify a clip shape.

```scheme
(image (get-payload 'image "") (circle))
```

---

### `(text STRING [MODS...])`

Creates a text node. First arg is the string content; remaining args are text modifiers.

```scheme
(text "Hello world"
  (size 15)
  (color (hex "#000"))
  (family "Inter")
  (weight 700)
  (align 'center))
```

#### Text modifiers

| Call              | Description                                       |
|-------------------|---------------------------------------------------|
| `(size (unit))`   | Font size                                         |
| `(color COLOR)`   | Text color                                        |
| `(family STR)`    | Font family name. `"sans-serif"`, `"Inter"`, etc. |
| `(weight N)`      | Font weight: 400 = regular, 700 = bold            |
| `(italic)`        | Sets italic font style                            |
| `(line-height N)` | Sets relative line height (e.g. `1.2`)            |
| `(align SYM)`     | `'start`, `'center`, `'end`, `'justify`           |

---

### `(shape KIND [FILL] [STROKE])`

Creates a shape node with optional fill and stroke.

```scheme
(shape (circle)
  (fill (solid (hex "#FF0000"))))

(shape (rounded-rect (px 8))
  (fill (linear-gradient (hex "#eee") (hex "#ddd"))))
```

#### Fill / Stroke wrappers

| Call                   | Description                        |
|------------------------|------------------------------------|
| `(fill PAINT)`         | Fill paint for the shape           |
| `(stroke PAINT WIDTH)` | Stroke paint + width for the shape |

---

## Style

### `(style MODS...)`

Creates a style from one or more style modifiers, applied left-to-right.

```scheme
(style
  (direction 'row)
  (align-items 'center)
  (padding (px 10))
  (gap (px 6)))
```

---

### Style Modifiers — Property Functions

These accept a value argument and return a style modifier. Use inside `(style ...)`.

#### Layout Direction & Alignment

| Call                    | Values                                                                     | Description          |
|-------------------------|----------------------------------------------------------------------------|----------------------|
| `(direction SYM)`       | `'row` `'column` `'row-reverse` `'column-reverse`                          | Flex direction       |
| `(align-items SYM)`     | `'start` `'end` `'center` `'stretch` `'baseline`                           | Cross-axis alignment |
| `(justify-content SYM)` | `'start` `'end` `'center` `'space-between` `'space-around` `'space-evenly` | Main-axis alignment  |
| `(display SYM)`         | `'flex` `'none`                                                            | Display mode         |
| `(position SYM)`        | `'absolute` `'relative`                                                    | Position type        |

#### Sizing

| Call               | Arg                             | Description |
|--------------------|---------------------------------|-------------|
| `(width DIM)`      | `(px N)` / `(pct N)` / `(auto)` | Width       |
| `(height DIM)`     | `(px N)` / `(pct N)` / `(auto)` | Height      |
| `(max-width DIM)`  | `(px N)` / `(pct N)` / `(auto)` | Max width   |
| `(max-height DIM)` | `(px N)` / `(pct N)` / `(auto)` | Max height  |
| `(min-width DIM)`  | `(px N)` / `(pct N)` / `(auto)` | Min width   |
| `(min-height DIM)` | `(px N)` / `(pct N)` / `(auto)` | Min height  |

#### Spacing


| Call                     | Args        | Description                   |
|--------------------------|-------------|-------------------------------|
| `(padding DIM)`          | 1 dim       | Uniform padding (all 4 sides) |
| `(padding-xy DIM DIM)`   | horiz, vert | Horizontal + vertical padding |
| `(padding T R B L)`      | 4 dims      | Top, Right, Bottom, Left      |
| `(margin DIM)`           | 1 dim       | Uniform margin                |
| `(margin-right DIM)`     | 1 dim       | Right margin only             |
| `(margin-bottom DIM)`    | 1 dim       | Bottom margin only            |
| `(gap DIM)`              | 1 dim       | Uniform gap between children  |

#### Position Insets

| Call              | Args   | Description              |
|-------------------|--------|--------------------------|
| `(inset T R B L)` | 4 dims | All inset values at once |
| `(top DIM)`       | 1 dim  | Top inset                |
| `(right DIM)`     | 1 dim  | Right inset              |
| `(bottom DIM)`    | 1 dim  | Bottom inset             |
| `(left DIM)`      | 1 dim  | Left inset               |

#### Visual

| Call          | Arg     | Description  |
|---------------|---------|--------------|
| `(opacity N)` | 0.0–1.0 | Node opacity |
| `(rotate N)`  | degrees | Rotation     |

---

### Convenience Aliases

Zero-argument shorthands for common style modifiers. Use interchangeably with the property functions.

| Alias               | Equivalent                         |
|---------------------|------------------------------------|
| `(flex-row)`        | `(direction 'row)`                 |
| `(flex-column)`     | `(direction 'column)`              |
| `(align-start)`     | `(align-items 'start)`             |
| `(align-center)`    | `(align-items 'center)`            |
| `(align-end)`       | `(align-items 'end)`               |
| `(justify-center)`  | `(justify-content 'center)`        |
| `(justify-between)` | `(justify-content 'space-between)` |
| `(justify-evenly)`  | `(justify-content 'space-evenly)`  |
| `(hidden)`          | `(display 'none)`                  |
| `(absolute)`        | `(position 'absolute)`             |
| `(relative)`        | `(position 'relative)`             |

---

## Error Handling

All functions validate their arguments. Invalid inputs produce a **Scheme error** with a descriptive message and the generation **fails**. Errors are logged via `tracing::error!`.

Examples:
```
direction: unknown value 'rwo', expected one of: row, column, row-reverse, column-reverse
hex: invalid hex color "#GGG"
node: unexpected argument type: IntV(42)
```

---

## Full Example

```scheme
;; quote-bubble.scm

(define username (get-payload 'username "Unknown"))
(define content  (get-payload 'content ""))
(define role     (get-payload 'user_role "user"))

(define name-color
  (cond
    [(equal? role "creator")       (hex "#A682D1")]
    [(equal? role "administrator") (hex "#58AB63")]
    [else                          (hex "#4CA635")]))

(node (style (flex-row) (align-items 'end) (padding (px 4)))

  ;; Avatar
  (node (style (width (px 36)) (height (px 36))
               (align-items 'center) (justify-content 'center))
    (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (circle) (fill (linear-gradient (hex "#6DD5FA") (hex "#2980B9")))))
    (text "AB" (size 14) (color (hex "#fff")) (family "sans-serif") (weight 700)))

  ;; Bubble
  (node (style (flex-column) (padding (px 12)) (max-width (px 500))
               (margin-left (px 10)))
    (node (style (absolute) (inset (px 0) (px 0) (px 0) (px 0)))
      (shape (rounded-rect (px 16)) (fill (solid (hex "#EFFDDE")))))
    (text username (size 15) (color name-color) (family "sans-serif") (weight 700))
    (text content  (size 15) (color (hex "#000"))  (family "sans-serif"))))
```
