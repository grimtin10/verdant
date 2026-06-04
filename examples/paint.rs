use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("paint", 1280, 720);

    let canvas = renderer.create_canvas(1280, 720)?;

    canvas.draw(|canvas| {
        canvas.background(Color::BLACK);
        canvas.outline(Color::WHITE, 2.5);
    });

    let mut last_mouse_x = 0.;
    let mut last_mouse_y = 0.;
    let mut mouse_down = false;
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            match event {
                WindowEvent::PointerButton { state, .. } => mouse_down = state.is_pressed(),
                WindowEvent::CloseRequested => { renderer.close_window(id); },
                _ => {}
            }
        }

        if let Some(window) = renderer.get_window(window) {
            let (mouse_x, mouse_y) = window.get_mouse_pos().into();

            window.background(Color::BLACK);

            window.composite(&canvas, 0., 0., 1280., 720.);

            if mouse_down {
                canvas.draw(|canvas| {
                    canvas.line(last_mouse_x, last_mouse_y, mouse_x, mouse_y);
                });
            }

            last_mouse_x = mouse_x;
            last_mouse_y = mouse_y;
        }

        renderer.flush()?;
    }

    Ok(())
}
