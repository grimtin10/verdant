#![allow(unused)]

use verdant::{ElementState, Key, KeyEvent, NamedKey, Renderer, RendererResult, WindowEvent, canvas::RenderSurface, text::Font, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    let text_window = renderer.create_window("text input", 1024, 1024);
    let atlas_window = renderer.create_window("text atlas", 1024, 1024);

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

        if let Some(window) = renderer.get_window(text_window) {
            let size = window.get_mouse_x() / 8.;

            window.background(Color::BLACK);
            window.text_size(size);
            window.text(&font, 0., size, &text);
        }

        if let Some(window) = renderer.get_window(atlas_window) {
            window.background(Color::BLACK);
            window.image(font.atlas(), 0., 0., 1024., 1024.);
        }

        renderer.flush()?;
    }

    Ok(())
}
