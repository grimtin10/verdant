use verdant::{ElementState, Key, KeyEvent, Renderer, RendererResult, WindowEvent, canvas::RenderSurface, text::{Font, HorizontalAlign, VerticalAlign}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let main_window = renderer.create_window("multi window", 1000, 500);

    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            match event {
                WindowEvent::CloseRequested => { renderer.close_window(id); continue },
                // only respond to key presses and character keys
                WindowEvent::KeyboardInput { event: KeyEvent {
                    logical_key: Key::Character(_),
                    state: ElementState::Pressed,
                    ..
                }, .. } => {
                    renderer.create_window("extra window", 250, 250);
                },
                _ => {}
            }
        }

        if let Some(window) = renderer.get_window(main_window) {
            window.background(Color::BLACK);

            window.font_size(32.);
            window.text_align(HorizontalAlign::Center, VerticalAlign::Center);
            window.text(&font, 500., 250., "Press any key to open a new window");
        }

        renderer.flush()?;
    }

    Ok(())
}
