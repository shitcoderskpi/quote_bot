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

| Call      | Description |
|-----------|-------------|
| `(px N)`  | Pixels      |
| `(pt N)`  | Points      |
| `(pct N)` | Percentage  |
| `(auto)`  | Auto sizing |

---

## Colors

| Call             | Example              | Description                                  |
|------------------|----------------------|----------------------------------------------|
| `(hex S)`        | `(hex "#FF0000")`    | Hex string `#RGB`, `#RRGGBB`, or `#RRGGBBAA` |
| `(rgb R G B)`    | `(rgb 255 0 0)`      | RGB, each 0–255                              |
| `(rgba R G B A)` | `(rgba 255 0 0 128)` | RGBA, each 0–255                             |

---

## Paints (fill / stroke values)

| Call                                    | Example                                                                                | Description                                   |
|-----------------------------------------|----------------------------------------------------------------------------------------|-----------------------------------------------|
| `(solid COLOR)`                         | `(solid (hex "#FFF"))`                                                                 | Solid fill.                                   |
| `(linear-gradient COLOR COLOR)`         | `(linear-gradient (hex "#fff") (hex "#000"))`                                          | Simple top to bottom gradient.                |
| `(linear-gradient ANGLE STOP STOP ...)` | `(linear-gradient (angle 0) (stop 0 (hex "#fff")) (stop 100 (hex "#000")))`            | Rich gradient with custom angle and stops     |
| `(radial-gradient STOP STOP ...)`       | `(radial-gradient (stop 0 (hex "#fff")) (stop 100 (hex "#000")))`                      | Radial gradient from center to edge.          |
| `(sweep-gradient ANGLE ANGLE STOP ...)` | `(sweep-gradient (angle 0) (angle 360) (stop 0 (hex "#fff")) (stop 100 (hex "#000")))` | Sweep gradient from start angle to end angle. |

### Rich gradient helpers

| Call               | Description                |
|--------------------|----------------------------|
| `(angle DEG)`      | Gradient angle in degrees. |
| `(stop PCT COLOR)` | Gradient color stop.       |

**Example:**
```scheme
(linear-gradient (angle 135)
  (stop 0   (hex "#FF6B6B"))
  (stop 50  (hex "#4ECDC4"))
  (stop 100 (hex "#45B7D1")))
```

---

## Shapes

| Call                         | Description                           |
|------------------------------|---------------------------------------|
| `(circle)`                   | Circle (inscribed in the node's box)  |
| `(rect)`                     | Rectangle with sharp corners          |
| `(rounded-rect DIM)`         | Rectangle with uniform corner radius. |
| `(rounded-rect TL TR BR BL)` | Per-corner radii.                     |
| `(svg-path STR)`             | Arbitrary SVG path data.              |

---

## Node Constructors

### `(node [STYLE] CHILDREN...)`

Creates a layout node. First arg can be a `style`; remaining args are child nodes, text, or shapes.

```scheme
(node (style (flex-row) (gap (px 10)))
  (text "Hello" (size 16))
  (text "World" (size 16)))
```

Children that evaluate to `void` or `'()` are silently skipped.

A `node` with no explicit `style` uses default flex layout.

---

### image

Creates an image node from a base64 encoded string (only!). Optionally specify a clip shape. Clip shape can also 
accept dimensions, although is not a full shape implementation see [below](#shape)

```scheme
(image (get-payload 'image "") (circle))
```

---

### text

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
| `(weight N)`      | Font weight                                       |
| `(italic)`        | Sets italic font style                            |
| `(line-height N)` | Sets relative line height                         |
| `(wrap BOOL)`     | Sets whether text should wrap (`#t` or `#f`)      |
| `(align SYM)`     | `'start`, `'center`, `'end`, `'justify`           |

---

### shape

Creates a shape node with optional fill and stroke.

```scheme
(shape (circle)
  (fill (solid (hex "#FF0000"))))

(shape (rounded-rect (px 8))
  (fill (linear-gradient (hex "#eee") (hex "#ddd"))))
```

#### Fill & stroke wrappers

| Call                   | Description                        |
|------------------------|------------------------------------|
| `(fill PAINT)`         | Fill paint for the shape           |
| `(stroke PAINT WIDTH)` | Stroke paint + width for the shape |

---

## Style

Creates a style from one or more style modifiers, applied left-to-right.

```scheme
(style
  (direction 'row)
  (align-items 'center)
  (padding (px 10))
  (gap (px 6)))
```

---

### Property Functions

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

## Full Example
See [dark](templates/dark.scm) or [light](templates/light.scm) templates.
