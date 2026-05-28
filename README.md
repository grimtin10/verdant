# verdant

[![Crates.io](https://img.shields.io/crates/v/verdant)](https://crates.io/crates/verdant)
[![Docs.rs](https://docs.rs/verdant/badge.svg)](https://docs.rs/verdant)

A rendering and windowing library built on `winit` and `wgpu`, aiming to be lightweight and performant.

> **Note:** Verdant is a work in progress. The API is unstable, and 3D support is planned for the future.

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
verdant = "0.2.2"
```

### Cargo Features
By default, Verdant includes `image` and `text` features. If you don't need asset loading and/or text and want a lighter dependency tree, you can disable them:

```toml
[dependencies]
verdant = { version = "0.2.2", default-features = false }
```

---

## Examples

### 1. Minimal Window
The absolute bare minimum code to open a window and keep it alive.

```rust
use verdant::{Renderer, RendererResult, WindowEvent};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    renderer.create_window("minimal verdant example", 500, 500);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }
    }

    Ok(())
}
```

### 2. Drawing
Verdant supports both imperative (stateful) and declarative (builder-pattern) drawing. You can mix them freely.

```rust
use verdant::{Renderer, RendererResult, WindowEvent, rgb, shapes::{Drawable, Rect}};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("minimal shapes", 800, 600);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            window.background(rgb(0.15, 0.15, 0.15));

            // imperative
            window.fill(rgb(0.8, 0.3, 0.3));
            window.ellipse(400., 300., 100., 100.);

            // declarative
            Rect::at(100., 100.)
                .size(150., 150.)
                .fill(rgb(0.3, 0.6, 0.8))
                .draw(window);
        }

        renderer.flush()?;
    }

    Ok(())
}
```
