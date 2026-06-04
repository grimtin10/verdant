use crate::{transform::Transform2d, vec::Vec2};

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

    fn update(&mut self) {
        let origin = self.origin;
        self.letterbox = self.get_view();
        let (scale_x, scale_y, x, y) = self.letterbox;

        self.transform = *Transform2d::translation(origin.x, origin.y)
            .scale(scale_x, scale_y)
            .translate(x, y);
    }

    pub(crate) fn set(&mut self, view: View) {
        self.origin = view.origin;
        self.size = view.size;
        self.mode = view.mode;
        self.update();
    }

    pub(crate) fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
        self.update();
    }

    pub(crate) fn set_size(&mut self, size: Option<Vec2>) {
        self.size = size;
        self.update();
    }

    pub(crate) fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
        self.update();
    }

    pub(crate) fn set_window_size(&mut self, window_size: Vec2) {
        self.window_size = window_size;
        self.update();
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
