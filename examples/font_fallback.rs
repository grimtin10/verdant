use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, text::{Font, HorizontalAlign, VerticalAlign}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("font fallback", 500, 500);

    // with Font::load (or any function that takes a `ByteSource`)
    // you can either use `include_bytes` or just give the path
    // in this case we use `include_bytes!` to make sure you can run the example from anywhere
    let inter = Font::load(include_bytes!("assets/Inter-VariableFont_opsz,wght.ttf"))?;
    let noto_emoji = Font::load(include_bytes!("assets/NotoEmoji-VariableFont_wght.ttf"))?;

    inter.add_fallback(noto_emoji);

    let text = "render normal text\n😀🕴️🌹\nand emoji";
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            window.background(Color::BLACK);

            window.font_size(32.);
            window.text_align(HorizontalAlign::Center, VerticalAlign::Top);
            window.line_align(HorizontalAlign::Center);
            window.text(&inter, 250., 0., text);
        }

        renderer.flush()?;
    }

    Ok(())
}
