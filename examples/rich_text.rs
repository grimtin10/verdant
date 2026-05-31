use verdant::{Renderer, RendererResult, WindowEvent, text::{Font, HorizontalAlign, Span, TextStyle, VerticalAlign}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("rich text", 750, 750);

    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.background(Color::BLACK);

            window.text_align(HorizontalAlign::Center, VerticalAlign::Top);

            window.rich_text(375., 0., &[
                Span {
                    text: "red text\nnewline\nvery long line ".into(),
                    font: font.clone(),
                    style: TextStyle {
                        size: 32.,
                        color: Color::RED,
                        line_align: HorizontalAlign::Center,
                        ..Default::default()
                    },
                },
                Span {
                    text: "blue text".into(),
                    font: font.clone(),
                    style: TextStyle {
                        size: 48.,
                        color: Color::BLUE,
                        ..Default::default()
                    }
                }
            ]);
        }

        renderer.flush()?;
    }

    Ok(())
}
