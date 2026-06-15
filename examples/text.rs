use verdant::{ElementState, Key, KeyEvent, NamedKey, Renderer, RendererResult, WindowEvent, canvas::RenderSurface, text::Font, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    // create a window both to show the text and show the atlas
    let text_window = renderer.create_window("text input", 1024, 1024);
    let atlas_window = renderer.create_window("text atlas", 1024, 1024);

    // with Font::load (or any function that takes a `ByteSource`)
    // you can either use `include_bytes` or just give the path
    // in this case we use `include_bytes!` to make sure you can run the example from anywhere
    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    let mut text = String::new();
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            match event {
                WindowEvent::CloseRequested => { renderer.close_window(id); },
                WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. } , .. } => {
                    match logical_key {
                        Key::Named(NamedKey::Backspace) => { text.pop(); },
                        Key::Named(NamedKey::Enter) => text += "\n",
                        Key::Character(c) => text += c.as_str(),
                        _ => {},
                    }
                }
                _ => {},
            }
        }

        // draw the text on the text window
        if let Some(window) = renderer.get_window(text_window) {
            window.background(Color::BLACK);
            window.font_size(32.);
            window.text(&font, 0., 0., &text);
        }

        // draw the atlas on the atlas window
        if let Some(window) = renderer.get_window(atlas_window) {
            window.background(Color::BLACK);

            // font.atlas() gives you a read-only reference to the atlas
            // if you need to mutate it, you can use font.atlas_mut()
            window.image(font.atlas(), 0., 0., 1024., 1024.);
        }

        renderer.flush()?;
    }

    Ok(())
}
