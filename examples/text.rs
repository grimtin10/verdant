#![allow(unused)]

use verdant::{ElementState, Key, KeyEvent, NamedKey, Renderer, RendererResult, WindowEvent, font::Font, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    let window = renderer.create_window("text input", 1024, 1024);

    let mut font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    let mut text = String::new();
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            match event {
                WindowEvent::CloseRequested => { renderer.close_window(id); },
                WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, .. } , .. } => {
                    match logical_key {
                        Key::Named(NamedKey::Backspace) => text = text.chars().next_back().map(|c| &text[..text.len() - c.len_utf8()]).unwrap_or("").to_string(),
                        Key::Character(c) => text += c.as_str(),
                        _ => {},
                    }
                }
                _ => {},
            }
        }

        if let Some(window) = renderer.get_window(window) {
            let size = window.get_mouse_x() / 8.;

            window.background(Color::BLACK);
            window.text_size(size);
            window.text(&font, 0., size, &text);
            window.image(&font.atlas(), 0., size * 2., 1024., 1024.);
        }

        renderer.flush()?;
    }

    Ok(())
}
