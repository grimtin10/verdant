use verdant::{Renderer, RendererResult, WindowEvent, text::{Font, Span, TextStyle}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("rich text", 500, 500);

    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(Color::BLACK);

            window.rich_text(0., 16., &[
                Span {
                    text: "red text ".into(),
                    style: TextStyle {
                        size: 16.,
                        font: font.clone(),
                        color: Color::RED,
                    },
                },
                Span {
                    text: "blue text ".into(),
                    style: TextStyle {
                        size: 16.,
                        font: font.clone(),
                        color: Color::BLUE,
                    }
                }
            ]);
        }

        renderer.flush()?;
    }

    Ok(())
}
