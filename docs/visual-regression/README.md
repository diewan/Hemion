# Hemion browser visual-regression baselines

The dated directories in this folder are immutable, reviewed browser captures.
They complement the executable accessibility and responsive-layout assertions in
`tests/console_shell.rs`; they do not replace those tests.

## 2026-07-23 baseline

The baseline was rendered by Hemion's real Dioxus WASM bundle with Dioxus CLI
0.7.9 and Firefox 152.0.6 on Linux. The server was started with:

```text
dx serve --platform web --addr 127.0.0.1 --port 8183
```

Firefox captured these routes after WASM hydration:

| File | Route | Viewport | Contract | SHA-256 |
| --- | --- | --- | --- | --- |
| `portfolio-desktop-1440x1000.png` | `/portfolio` | 1440x1000 | Desktop portfolio shell | `bb71d7b17353ccdebf6874f6b6d001e7e6d6d0edf606ae09b93c545328a83a4a` |
| `portfolio-web-390x844.png` | `/portfolio` | 390x844 | Narrow responsive shell and bottom navigation | `6cf52777e638f031df3004933501b213c463b38d7828c3623a78c42c8328efa4` |
| `anchoring-unavailable-desktop-1440x1000.png` | `/anchoring` | 1440x1000 | Explicit optional/unavailable anchoring semantics | `864df229d951933cf2802feb43b4fd202de9d505f7670c249275c8a3901ac3e8` |
| `unknown-object-error-web-390x844.png` | `/object/unknown/bad` | 390x844 | Explicit unknown-object error at narrow width | `9d7a3c862c848cf8f6a9b21ac026238b0682c9233d6e4f7c241149e52e49d8cd` |

The captures were visually checked for readable light-theme contrast, clipping,
overflow, responsive navigation, and explicit error/limitation language. A
baseline change requires a new dated directory, updated hashes, and human review;
do not overwrite an accepted dated capture.

The browser run exposed two defects that compile-only checks could not find:

- ambient native `CC`/`CFLAGS` caused unresolved `env` imports in the generated
  WASM module; `.cargo/config.toml` now pins Clang and the system archiver for
  WASM C dependencies on both host and container toolchains;
- the light token palette was applied over hard-coded dark shell backgrounds;
  the body and application shell now consume the active theme tokens.
