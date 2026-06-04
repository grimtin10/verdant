use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("scoped state", 500, 500);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(Color::BLACK);

            window.fill(Color::RED);

            // all style changes are reset back to how they were before this is run
            window.with_style(|window| {
                window.fill(Color::GREEN);
                window.rect(150., 150., 150., 150.);
            });

            // this ellipse draws with Color::RED as the fill
            window.ellipse(300., 300., 75., 75.);
        }

        renderer.flush()?;
    }

    Ok(())
}
