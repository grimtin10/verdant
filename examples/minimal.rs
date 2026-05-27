use verdant::{Renderer, RendererResult, WindowEvent};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    renderer.create_window("minimal verdant example", 500, 500);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }
    }

    Ok(())
}
