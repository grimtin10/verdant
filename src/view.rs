// this is put in a separate file to prevent being able to just directly set the variables
// "why not just set the variables?"
// because i wanna make it impossible to forget to update the state

use crate::{canvas::CanvasContext, transform::Transform2d, vec::Vec2};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// Doesn't apply any scaling, leaving everything as-is.
    #[default]
    Unscaled,

    /// Scales the view to fill the window entirely, ignoring aspect ratio.
    Stretch,

    /// Scales the view uniformly to fit inside the window, preserving aspect ratio.
    /// Empty bars appear on the sides or top/bottom as needed (Letterboxing).
    Letterbox,

    /// Scales the view uniformly to fill the window completely, preserving aspect ratio.
    /// If the aspect ratios differ, the parts of the view that extend beyond the
    /// window boundaries will be cut off (Cropping).
    Crop,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct View {
    window_size: Vec2,

    origin: Vec2,
    size: Option<Vec2>,
    mode: ViewMode,

    // TODO: this variable name is a little wrong...
    letterbox: (f32,  f32, f32, f32),
    transform: Transform2d,
}

impl View {
    // TODO: ...and this function name is dumb
    fn get_view(&self) -> (f32, f32, f32, f32) {
        if self.mode == ViewMode::Unscaled { return (1., 1., 0., 0.) };

        let Some(view_size) = self.size else { return (1., 1., 0., 0.) };

        let (screen_w, screen_h) = (self.window_size.x, self.window_size.y);
        let (view_w, view_h) = view_size.into();

        if self.mode == ViewMode::Stretch {
            return (screen_w / view_w, screen_h / view_h, 0., 0.)
        };

        let scale_w = screen_w / view_w;
        let scale_h = screen_h / view_h;

        let scale = if self.mode == ViewMode::Letterbox {
            scale_w.min(scale_h)
        } else {
            scale_w.max(scale_h)
        };

        let x = (screen_w - (view_w * scale)) / 2.;
        let y = (screen_h - (view_h * scale)) / 2.;

        (scale, scale, x, y)
    }

    fn update(&mut self, context: &mut CanvasContext) {
        let origin = self.origin;
        self.letterbox = self.get_view();
        let (scale_x, scale_y, x, y) = self.letterbox;

        self.transform = *Transform2d::translation(origin.x, origin.y)
            .scale(scale_x, scale_y)
            .translate(x, y);

        context.update_transform(self.transform * context.local_transform);
    }

    pub(crate) fn set(&mut self, view: View, context: &mut CanvasContext) {
        self.origin = view.origin;
        self.size = view.size;
        self.mode = view.mode;
        self.update(context);
    }

    pub(crate) fn set_origin(&mut self, origin: Vec2, context: &mut CanvasContext) {
        self.origin = origin;
        self.update(context);
    }

    pub(crate) fn set_size(&mut self, size: Option<Vec2>, context: &mut CanvasContext) {
        self.size = size;
        self.update(context);
    }

    pub(crate) fn set_mode(&mut self, mode: ViewMode, context: &mut CanvasContext) {
        self.mode = mode;
        self.update(context);
    }

    pub(crate) fn set_window_size(&mut self, window_size: Vec2, context: &mut CanvasContext) {
        self.window_size = window_size;
        self.update(context);
    }

    pub(crate) fn origin(&self) -> Vec2 {
        self.origin
    }

    pub(crate) fn window_size(&self) -> Vec2 {
        self.window_size
    }

    pub(crate) fn letterbox(&self) -> (f32, f32, f32, f32) {
        self.letterbox
    }

    pub(crate) fn transform(&self) -> Transform2d {
        self.transform
    }
}
