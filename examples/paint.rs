use verdant::prelude::*;

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("paint", 1280, 720);

    let canvas = renderer.create_canvas(1280, 720)?;

    // initialize the canvas with a black background and set the line color and width
    canvas.draw(|canvas| {
        canvas.background(Color::BLACK);
        canvas.outline(Color::WHITE, 2.5);
    });

    // keep track of the last mouse position to draw lines between then and now
    let mut last_mouse_x = 0.;
    let mut last_mouse_y = 0.;
    let mut mouse_down = false;
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            match event {
                WindowEvent::PointerButton { pressed, .. } => mouse_down = pressed,
                WindowEvent::CloseRequested => { renderer.close_window(id); },
                _ => {}
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            let (mouse_x, mouse_y) = window.get_mouse_pos().into();

            // draw the canvas onto the window
            window.composite(&canvas, 0., 0., 1280., 720.);

            // draw the lines when the mouse is pressed
            if mouse_down {
                canvas.draw(|canvas| {
                    canvas.line(last_mouse_x, last_mouse_y, mouse_x, mouse_y);
                });
            }

            // update the last mouse position
            last_mouse_x = mouse_x;
            last_mouse_y = mouse_y;
        }

        renderer.flush()?;
    }

    Ok(())
}
