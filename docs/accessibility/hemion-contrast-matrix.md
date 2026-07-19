# Hemion console contrast matrix

This matrix covers every foreground/background pairing permitted by the G-01
console shell. `tests/console_shell.rs` calculates the WCAG relative-luminance
ratio from these same token values and requires at least 4.5:1. REV03 D-02 is
the authority for correcting the original `--ink-3` token.

| Foreground | Background | Use | Minimum |
|---|---|---|---:|
| `--ink-1 #E7EAEE` | `--surface-0 #14171C` | primary text | 4.5:1 |
| `--ink-2 #A9B1BC` | `--surface-0 #14171C` | secondary text | 4.5:1 |
| `--ink-3 #8993A1` | `--surface-0 #14171C` | metadata | 4.5:1 |
| `--ink-1 #E7EAEE` | `--surface-1 #1C2027` | panel primary text | 4.5:1 |
| `--ink-2 #A9B1BC` | `--surface-1 #1C2027` | panel secondary text | 4.5:1 |
| `--ink-3 #8993A1` | `--surface-1 #1C2027` | panel metadata | 4.5:1 |
| `--interactive #7FA6E8` | `--surface-0 #14171C` | links/focus | 4.5:1 |
| `--interactive #7FA6E8` | `--surface-1 #1C2027` | panel links/focus | 4.5:1 |
| `#D6A85F` | `--surface-1 #1C2027` | evidence-gap border (non-text) | 3:1 |
| `#EF8F9C` | `--surface-1 #1C2027` | contradiction border (non-text) | 3:1 |

Native anchors retain keyboard activation. Every shell link has a two-pixel
`focus-visible` outline, inspector-style content follows document order, and
the existing reduced-motion media query suppresses transition animation.
The dispute graph uses native buttons, a document-order card view, and a native
table alternative. Gap, withheld, and contradiction states use text labels and
border treatments in addition to color.
