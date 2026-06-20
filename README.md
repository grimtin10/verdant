# verdant

[![Crates.io](https://img.shields.io/crates/v/verdant)](https://crates.io/crates/verdant)
[![Docs.rs](https://docs.rs/verdant/badge.svg)](https://docs.rs/verdant)
[![Grind Compliant](https://img.shields.io/badge/Grind-Compliant-forest)](https://github.com/The-Grindhouse/grindlines)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?&logo=discord&logoColor=white)](https://discord.gg/MD8QeNV2pC)

> **Verdant is a work-in-progress. Parts of the API may change at any time.**

<img width="800" alt="boids" src="https://github.com/user-attachments/assets/50e31e90-9e4b-4cf3-8702-e27c5dd8bfd2" style="max-width: 100%; height: auto" />

*See [boids.rs](examples/boids.rs)*

<img width="800" alt="clock" src="https://github.com/user-attachments/assets/1841ac1d-d3e1-4efc-b5d9-b8186a67dc53" style="max-width: 100%; height: auto" />

*See [clock.rs](examples/clock.rs)*

<img width="800" alt="verdant paint" src="https://github.com/user-attachments/assets/2a791f8a-c498-4676-bb05-6e68d398debd" style="max-width: 100%; height: auto" />

*See [paint.rs](examples/paint.rs)*

---

## What is Verdant?

Verdant is a rendering and windowing library for Rust, made to be accessible.

Built on top of `wgpu` and `winit`, Verdant focuses on a clean, expressive, and easy-to-use API that makes common rendering tasks easy and mistakes difficult.

Finally, a foundational rendering library that is easy to use.

```rust
use verdant::prelude::*;

fn main() -> RendererResult<()> {
    // initialize the renderer and create a window
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("verdant hello world", 800, 600);

    // main drawing loop
    // `is_running` returns true while there is at least one window open
    while renderer.is_running() {
        // poll for events...
        for (id, event) in renderer.poll() {
            // ...and handle window closing
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        // start drawing by getting a window
        if let Some(window) = renderer.get_window(window_id) {
            // clearing the background to a neutral color
            window.background(rgb(0.15, 0.15, 0.15));

            // and drawing a circle
            window.fill(rgb(0.8, 0.3, 0.3));
            window.ellipse(400., 300., 100., 100.);
        }

        // and finishing off the frame by flushing all draw commands to the GPU
        renderer.flush()?;
    }

    Ok(())
}
```

Verdant aims to be performant and powerful, while still remaining lightweight and nice to use.

---

## What Verdant is *not*
Verdant is a rendering/windowing library. It is not a game engine or a UI framework, it does not handle audio, it does not handle physics.
It is a foundation you can build those things on top of.

Verdant is also not *just* a creative coding library or *just* meant for simple tasks (however it can be used for both of those things!)
An accessible API does not mean a compromised one; Verdant is designed to be capable.

---

## Why Verdant?

### SDF-based rendering

Almost all Verdant primitives are rendered using SDFs (signed-distance-fields).
This allows for your graphics to have perfect anti-aliasing and rounding, at any scale, while remaining efficient.

### Multiple windows are first-class

Most graphics libraries start with a single window and treat additional windows as an advanced feature.
Verdant easily and cleanly supports multiple windows, built into the API since day one.

*See [multi_window.rs](examples/multi_window.rs)*

### Scoped state

Most graphics libraries have functions like `push_state` and `pop_state`.
This is powerful, but it can lead to mistakes where state is leaked out of the context where you need it.
Verdant prevents this by providing scoped state through closures.

*See [scoped_state.rs](examples/scoped_state.rs)*

### Pick your API

Verdant doesn't force you to stick to a state machine API or a more "retained" builder pattern API, instead allowing you to choose between either one.

```rs
// imperative, state-machine API
window.fill(Color::RED);
window.rect(500., 250., 250., 250.);

// more "declarative", builder pattern API
Rect::at(250., 500.)
    .size(250., 250.)
    .fill(Color::BLUE)
    .draw(window);
```

You can use the imperative, stateful API for quick prototyping or things that change every frame, or you can pick the declarative, builder API for things that hold state between frames.

*See [drawing.rs](examples/drawing.rs)*

### Rich text

Verdant provides both a basic text API and a rich text API, allowing you to mix fonts, colors, sizes, and styles, all in a single function call.

<details>
<summary>Screenshot</summary>
<img width="800" src="https://github.com/user-attachments/assets/859c9eff-11d6-4048-8791-f4cc7e25a23a" style="max-width: 100%; height: auto" />
</details>

*See [rich_text.rs](examples/rich_text.rs)*

### Canvases are first-class

Verdant allows you to render to off-screen surfaces just as easily as rendering to a window.
It even lets you render a canvas to itself, allowing for recursive effects!

*See [canvas.rs](examples/canvas.rs) and [paint.rs](examples/paint.rs)*

### Views and coordinate systems

Verdant allows you to define a logical rendering space, independent of the physical window size.
This allows you to create scalable applications without having to manually handle aspect-ratio math.

<details>
<summary>Screenshot</summary>
<img width="800" alt="image" src="https://github.com/user-attachments/assets/a28250a2-8b80-4e34-84d1-e130cae74957" style="max-width: 100%; height: auto" />
</details>

*See [views.rs](examples/views.rs)*

### Configurable scaling
Verdant lets you choose how outlines and corner radii scale. They can behave like physical geometry, remain at a constant pixel size, or respond independently to transforms and view scaling.
This makes it easy to build everything from scalable interfaces to interactive visualizations without sacrificing visual consistency.

<details>
<summary>Screenshot</summary>
<img width="800" alt="scaling" src="https://github.com/user-attachments/assets/135db8d2-92c5-4a2b-a369-b5ecbb68c2ad" style="max-width: 100%; height: auto" />
*(The window has been resized to be much larger than the original size, on the left the outlines and corner radii scale with the window, and on the right they stay at a constant pixel size)*
</details>

*See [scaling_modes.rs](examples/scaling_modes.rs)*

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
verdant = "0.7"
```
Currently, Verdant only supports desktop, though web/mobile support is planned soon.

<details>
<summary>
<strong>Cargo Features</strong>
</summary>

By default, Verdant includes granular features to control what gets built. If you don't need asset loading and/or text and want a lighter dependency tree, you can disable them:

```toml
[dependencies]
verdant = { version = "0.7", default-features = false, features = ["vulkan", "dx12", "metal"] }
```

If you want legacy OpenGL support:

```toml
[dependencies]
verdant = { version = "0.7", default-features = false, features = ["vulkan", "dx12", "metal", "gles"] }
```

It also has granular features for different image formats:

```toml
[dependencies]
verdant = { version = "0.7", default-features = false, features = ["vulkan", "dx12", "metal", "image-png", "image-jpeg", "image-webp"] }
```

By default, "text", "image-png", "image-jpeg", "dx12", "metal", and "vulkan" are enabled.
</details>

<details>
<summary>
<strong>Running the Examples</strong>
</summary>
To try out the examples locally, clone the repository and run them using Cargo:

```sh
git clone https://github.com/grimtin10/verdant.git
cd verdant
cargo run --example boids --release
cargo run --example clock --release
```
</details>

## MSRV

The MSRV of Verdant is 1.89.0.

## Feedback

For any feedback on the API or for feature requests, please go to the [issues page](https://github.com/grimtin10/verdant/issues).

## License

Verdant is licensed under MPL-2.0.

Applications built with Verdant may be distributed under any license.
Only modifications to Verdant itself are required to remain MPL-licensed.
