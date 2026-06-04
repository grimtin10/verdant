use verdant::{Renderer, RendererResult, WindowEvent};

fn main() -> RendererResult<()> {
    // initialize the renderer
    let mut renderer = Renderer::new()?;

    // create a new window with the given title, width, and height
    renderer.create_window("minimal verdant example", 500, 500);

    // the main renderer loop
    // `renderer.is_running()` returns true if there are any windows open
    while renderer.is_running() {
        // poll for events
        for (id, event) in renderer.poll() {
            // and handle the window closing
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        // normally you'd call `renderer.flush()` here
        // but since we're not drawing anything, we don't need to
    }

    Ok(())
}
