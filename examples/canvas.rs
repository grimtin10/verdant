use verdant::prelude::*;

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("canvas", 800, 600);

    let canvas = renderer.create_canvas(400, 400)?;

    // draw a white rectangle onto the canvas
    canvas.draw(|c| {
        c.rect(0., 0., 200., 200.);
    });

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(mut window) = renderer.get_window(window_id) {
            window.background(Color::BLACK);

            // composite the canvas onto the window
            window.composite(&canvas, 100., 100., 400., 400.);

            // canvases can composite to themselves for recursive effects
            canvas.draw(|c| {
                c.composite(&canvas, 300., 0., 100., 400.);
            });
        }

        renderer.flush()?;
    }

    Ok(())
}
