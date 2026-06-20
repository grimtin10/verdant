# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

(!) - breaking change
(#xyz) - fixed in the given PR

## [0.7.0] - 2026-06-19

### Added
- `Canvas::auto_flush` so you can now toggle whether or not you want canvases to be implicitly flushed when composited
- `prelude`, so now you can just `use verdant::prelude::*` to get access to the entire public API
- `event::WindowEvent` as an abstraction over `winit::WindowEvent` (along with other associated structs)
- Exposed `WinitEvent` (`winit::WindowEvent` re-exported under a different name) as an escape hatch for events that `WindowEvent` doesn't handle
- `Renderer::poll_raw` for if you need the raw `WinitEvent`s
- `glam` compatability feature to allow for full compatibilty with `glam`'s vector structs
- Breaking changes and PRs are now tracked in the changelog
- `WindowDraw` so that drawing to a window no longer acquires a lock on the inner canvas for every command

### Changed
- `CanvasState` is now `CanvasDraw` to more accurately reflect what it means for the user (!)
- `RenderSurface` has been moved from `canvas` to `render_surface` (!)
- Genericize all functions that take `VecN` to allow easier compatibility with `glam`
- `Renderer::poll` no longer returns `Vec<winit::WindowEvent>`, now returning `Vec<window::WindowEvent>` (!)
- Change crate description slightly
- `Renderer::get_window` now returns an `Option<WindowDraw>` instead of `Option<&mut Window>` (!)

### Fixed
- Added missing documentation on `Canvas`
- Fix deadlock with rendering canvases with cyclic references (`A` composites `B`, `B` composites `A`)
- `Renderer` is now guaranteed `!Send` on all platforms, as it is not thread-safe (`winit` likely already made it `!Send`, but I wanted to be sure)
- The boids example was entirely busted, because a previous "fix" actually broke it, and I thought it would be a great idea to not test it

Hopefully there won't be many breaking changes after this one.

## [0.6.2] - 2026-06-17

### Fixed
- Fix a bug where a bad implementation of `PartialEq` on `Image` was causing anything textured to not render properly (or sometimes, at all)
- Fix a bug where some images would not properly update if they were only referenced once

## [0.6.1] - 2026-06-16

### Added
- `Window::get_canvas`

### Changed
- Update README

### Fixed
- Fix typo in `Image::new()` where instead of erroring when the provided buffer was too small it errored when it was too large
- Fix incorrect version number in README
- Drawing text/images no longer break batches twice
- Fix typo in `TextLayout::from_spans` where right aligned spans were entirely wrong
- Fix typo in boids example that caused random number generation to be wrong

## [0.6.0] - 2026-06-15

### Added
- Spans can now be constructed with `Span::new`
- `VecN::length_squared`, `VecN::max`, and `VecN::longest`
- `TextLayout` and all associated functions/structs
- `RenderSurface::text_layout` and `RenderSurface::rich_text_layout`
- `text_size`, `text_width`, and `text_height` to `RenderSurface` (moved here due to the changes to the text system)
- `rich_text_size`, `rich_text_width`, and `rich_text_height` to `RenderSurface` (moved here due to the changes to the text system)

### Changed
- Update README
- Refactored text system to be cleaner and more performant
- Renamed `RenderSurface::text_size` to `RenderSurface::font_size` (!)
- Text vertical alignment now defaults to the top instead of the bottom
- Changed wording of `VecN` doc comments

### Removed
- `text::rich_text_*` and `text::text_*` (moved to `RenderSurface`) (!)

## [0.5.2] - 2026-06-13

### Added
- `Font::get_glyphs` (technically a part of an unfinished internal refactor, but the function is part of the public API)

### Changed
- Update README with macOS testing status
- `Font::get_or_load_glyph` no longer holds a lock on its internal position for the whole function
- Errors within the `ApplicationHandler` are now logged instead of ignored in lieu of proper error handling
- `examples/text.rs` no longer changes the text size based on mouse position

### Fixed
- The scratch texture used for recursive canvases is now invalidated on canvas resize
- Fixed unused warnings on non-linux systems
- "Fixed" mysterious bug where batches could have a higher vertex start index than the end index
- Fixed infinite loop on macOS when trying to display blank frame (#2)

## [0.5.1] - 2026-06-13

### Fixed
- Canvases were not properly reacting to calls to `resize`, causing crashes when the window was resized before the GPU context could be initialized

## [0.5.0] - 2026-06-12

### Added
- `RenderSurface::corner_radius`, `RenderSurface::corner_style` (and equiv. on `Style`)
- Scale-independent sizing, `RenderSurface::outline_scaling` and `RenderSurface::corner_scaling`, etc. (and equiv. on `Style`)
- Exposed `AdvancedWindowProperties` (`winit::window::WindowAttributes` re-exported under a different name)
- Added more properties to `WindowProperties`
- `Text::modify()`, `RichText::push_span()`, and `RichText::modify()` to allow for easier mutation of the underlying values of `Text` and `RichText`
- `Transform::get_safe_scale` and `Transform::transform_vector`
- Documented all `VecN` functions
- Font fallback
- More granular image/wgpu backend features to Cargo.toml
- Started tracking changelog

### Changed
- `Renderer::create_window_ext` now takes either `WindowProperties` or `AdvancedWindowProperties`
- All functions that returned `&mut Self` now return `Self` so you don't need to dereference/clone (!)
- When using `ViewMode::Stretch`, outlines and corner radii don't stretch unevenly anymore
- Cloning `Text` and `RichText` is now cheap and reference-counted
- Moved `WindowId` and `WindowProperties` from `types` to `window` (!)
- README update
- Lighten up dependencies, remove `thiserror` and change default enabled features for other deps
- Boids example now relies on `fastrand` instead of `rand`

### Removed
- `RenderSurface::round_rect`, in favor of having corner radius be part of the stateful style (!)
- The `Span` builder functions

### Fixed
- `Font::atlas()` and `Font::atlas_mut()` can no longer panic
- `Canvas::read()` and `Canvas::write()` can no longer panic
- Wrong/inconsistent documentation
- Canvases no longer create a new bind group per composite per frame
- Recursive canvases no longer create a new texture every frame
- Setting the texture to `A` then `None` then `A` again no longer creates a new batch
- `RichText` now has a `position` function, which was previously missing
