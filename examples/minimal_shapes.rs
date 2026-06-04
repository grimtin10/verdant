use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, rgb, shapes::{Drawable, Rect}};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("minimal drawing", 800, 600);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(rgb(0.15, 0.15, 0.15));

            // imperative style
            window.fill(rgb(0.8, 0.3, 0.3));
            window.ellipse(400., 300., 100., 100.);

            // declarative style
            Rect::at(100., 100.)
                .size(150., 150.)
                .fill(rgb(0.3, 0.6, 0.8))
                .draw(window);
        }

        renderer.flush()?;
    }

    Ok(())
}
