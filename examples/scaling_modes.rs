use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, shapes::ScalingMode, types::Color, view::ViewMode, window::WindowProperties};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = WindowProperties::new("outline/radius scaling", 500, 500).resizable(true).build(&mut renderer);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            window.set_view(500., 500., ViewMode::Letterbox);
            window.background(Color::GRAY);

            window.clear_style();

            window.outline(Color::BLACK, 3.);

            window.outline_scaling(ScalingMode::Geometric);
            window.rect(13.33, 10., 230., 230.);

            window.outline_scaling(ScalingMode::Constant);
            window.rect(256.66, 10., 230., 230.);

            window.corner_radius(25.);
            window.scaling_modes(ScalingMode::Geometric, ScalingMode::Geometric);
            window.rect(13.33, 256.66, 230., 230.);

            window.scaling_modes(ScalingMode::Constant, ScalingMode::Constant);
            window.rect(256.66, 256.66, 230., 230.);
        }

        renderer.flush()?;
    }

    Ok(())
}
