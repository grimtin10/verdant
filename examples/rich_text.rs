use verdant::{Renderer, RendererResult, WindowEvent, canvas::RenderSurface, rgba255, text::{Font, HorizontalAlign, Span, TextStyle, VerticalAlign}, types::Color};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = renderer.create_window("rich text", 750, 230);

    // with Font::load (or any function that takes a `ByteSource`)
    // you can either use `include_bytes` or just give the path
    // in this case we use `include_bytes!` to make sure you can run the example from anywhere
    let font = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;

    // define the spans in the rich text
    let spans = &[
        // you can either construct the struct directly
        Span {
            text: "red text\nnewline\nvery long line ".into(),
            font: font.clone(),
            style: TextStyle {
                size: 32.,
                color: Color::RED,
                ..Default::default()
            },
        },
        // or use `Span::new`
        Span::new(
            "blue text\nblue newline",
            &font,
            TextStyle {
                size: 48.,
                color: Color::BLUE,
                ..Default::default()
            }
        )
    ];

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window_id) {
            window.background(Color::BLACK);

            // set the text alignment (where the origin is)
            // defaults to the top left, this changes it to the top center
            window.text_align(HorizontalAlign::Center, VerticalAlign::Top);

            // set the line alignment (where each line sits within the bounds of the text)
            // defaults to the left, this changes it to the center
            window.line_align(HorizontalAlign::Center);

            window.outline(Color::GREEN, 4.);
            window.fill(rgba255(0., 255., 0., 50.));

            // get the width and height of the rich text and draw the bounds
            let (w, h) = window.rich_text_size(spans).into();
            window.rect(375. - w / 2., 10., w, h);

            // draw the rich text
            window.rich_text(375., 10., spans);
        }

        renderer.flush()?;
    }

    Ok(())
}
