use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, shapes::{Drawable, Ellipse, Rect}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("drawing API example", 1000, 1000);

    // shape that persists between frames
    let mut mouse_ellipse = Ellipse::at(0., 0.)
        .size(100., 100.)
        .fill(Color::GREEN)
        .outline(Color::BLACK, 2.0);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(Color::GRAY);

            // imperative, state-machine API
            window.fill(Color::RED);
            window.rect(500., 250., 250., 250.);

            // more "declarative", builder pattern API
            Rect::at(250., 500.)
                .size(250., 250.)
                .fill(Color::BLUE)
                .draw(window);

            mouse_ellipse.position(window.get_mouse_x(), window.get_mouse_y());
            mouse_ellipse.draw(window);
        }

        renderer.flush()?;
    }

    Ok(())
}
