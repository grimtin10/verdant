use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("canvas", 800, 600);

    let canvas = renderer.create_canvas(400, 400)?;

    canvas.draw(|canvas| {
        canvas.rect(0., 0., 200., 200.);
    });

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(Color::BLACK);
            window.composite(&canvas, 100., 100., 400., 400.);
            canvas.draw(|c| {
                c.composite(&canvas, 300., 0., 100., 400.);
            });
        }

        renderer.flush()?;
    }

    Ok(())
}
