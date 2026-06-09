# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `RenderSurface::corner_radius` (and equiv. on `Style`)
- Scale-independent sizing, `RenderSurface::outline_scaling` and `RenderSurface::corner_scaling` (and equiv. on `Style`)
- Exposed `AdvancedWindowProperties` (`winit::window::WindowAttributes` re-exported under a different name)
- Added more properties to `WindowProperties`
- `Text::modify()`, `RichText::push_span()`, and `RichText::modify()` to allow for easier mutation of the underlying values of `Text` and `RichText`
- `Transform::get_safe_scale` and `Transform::transform_vector`
- Documented all `VecN` functions

### Changed
- `Renderer::create_window_ext` now takes either `WindowProperties` or `AdvancedWindowProperties`
- All functions that returned `&mut Self` now return `Self` so you don't need to dereference/clone
- When using `ViewMode::Stretch`, outlines don't stretch unevenly anymore
- Cloning `Text` and `RichText` is now cheap and reference-counted
- Moved `WindowId` and `WindowProperties` from `types` to `window`

### Removed
- `RenderSurface::round_rect`, in favor of having corner radius be part of the stateful style

### Fixed
- `Font::atlas()` and `Font::atlas_mut()` can no longer panic
- `Canvas::read()` and `Canvas::write()` can no longer panic
- Wrong/inconsistent documentation
