# CSS Reference

## Layout

| Property | Values |
|----------|--------|
| `display` | `flex`, `block`, `inline`, `none` |
| `flex-direction` | `row`, `column`, `row-reverse`, `column-reverse` |
| `justify-content` | `flex-start`, `center`, `flex-end`, `space-between`, `space-around`, `space-evenly` |
| `align-items` | `stretch`, `flex-start`, `center`, `flex-end` |
| `flex-wrap` | `nowrap`, `wrap` |
| `flex-grow` | number |
| `flex-shrink` | number |
| `gap` | length |

## Box Model

| Property | Values |
|----------|--------|
| `width`, `height` | length, percentage, `auto` |
| `min-width`, `min-height` | length, percentage, `auto` |
| `max-width`, `max-height` | length, percentage, `auto` |
| `margin` | shorthand (1-4 values) |
| `margin-top/right/bottom/left` | length |
| `padding` | shorthand (1-4 values) |
| `padding-top/right/bottom/left` | length |
| `border` | `width style color` |
| `border-width` | length |
| `border-color` | color |
| `border-radius` | length (1 or 4 values) |

## Position

| Property | Values |
|----------|--------|
| `position` | `relative`, `absolute`, `fixed` |
| `top/right/bottom/left` | length, percentage, `auto` |
| `z-index` | integer |

## Visual

| Property | Values |
|----------|--------|
| `background-color` | color |
| `color` | color (inherited) |
| `opacity` | 0.0 - 1.0 |

## Text

| Property | Values |
|----------|--------|
| `font-size` | length |
| `font-weight` | `normal`, `bold`, `lighter`, `bolder`, number |
| `font-family` | family name (inherited) |
| `text-align` | `left`, `center`, `right` |
| `line-height` | number or length |

## Overflow

| Property | Values |
|----------|--------|
| `overflow` | `visible`, `hidden`, `scroll` |

## Pseudo-classes

| Pseudo-class | Behavior |
|--------------|----------|
| `:hover` | Style swapped on mouse enter/leave, animated if `transition` is set |

## Transitions

```css
.btn {
    transition: all 0.2s;
}
.btn:hover {
    background-color: #c73650;
}
```

Properties that interpolate: `background-color`, `color`, `border-color`, `opacity`, `font-size`, `border-radius`, `padding`, `margin`.

## Animations

```css
@keyframes fadeIn {
    from { opacity: 0 }
    to { opacity: 1 }
}

@keyframes pulse {
    0% { opacity: 1 }
    50% { opacity: 0.5 }
    100% { opacity: 1 }
}

.element {
    animation: fadeIn 1s;
    animation: pulse 2s infinite;
    animation: slide 1s 0.2s infinite alternate;
}
```

Supported `animation` sub-properties: `animation-name`, `animation-duration`, `animation-delay`, `animation-iteration-count` (number or `infinite`), `animation-direction` (`normal`, `reverse`, `alternate`).

## Selectors

| Selector | Example | Specificity |
|----------|---------|-------------|
| Tag | `div`, `h1` | 1 |
| Class | `.card`, `.btn` | 10 |
| ID | `#main` | 100 |
| Universal | `*` | 0 |
| Compound (same element) | `div.btn`, `.card.active` | sum |
| Descendant | `.card h1`, `#main .text` | sum |
| Hover | `.btn:hover`, `div:hover` | +10 |

## Units

| Unit | Example |
|------|---------|
| `px` | `16px` |
| `%` | `50%` |
| `em` | `1.5em` (relative to element font-size) |
| `rem` | `2rem` (relative to 16px root) |
| `vh` | `100vh` |
| `vw` | `50vw` |

## Colors

Named: `red`, `blue`, `white`, `transparent`, `coral`, `crimson`, `dodgerblue`, etc. (40+ names)

Hex: `#RGB`, `#RRGGBB`, `#RRGGBBAA`

Functions: `rgb(255, 128, 0)`, `rgba(255, 0, 0, 0.5)`
