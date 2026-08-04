use verdant::prelude::*;

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window_id = WindowProperties::new("verdant transforms", 1350, 500)
        .resizable(true)
        .build(&mut renderer);

    let font = Font::load(include_bytes!("assets/Inter-VariableFont_opsz,wght.ttf"))?;

    let square = Rect::default().size(200., 200.).fill(Color::RED);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(mut window) = renderer.get_window(window_id) {
            window.set_view(1350., 500., ViewMode::Letterbox);

            window.background(Color::BLACK);
            window.fill(Color::WHITE);
            window.text_align(HorizontalAlign::Center, VerticalAlign::Top);
            window.font_size(32.);

            window.text(&font, 150., 20., "No Transform");
            square.draw_at(&mut window, 50., 80.);

            window.text(&font, 500., 20., "45° Rotation");
            window.with_transform(
                Transform2d::rotation_deg(45.)
                    .translate(500., 80.),

                |window| {
                    square.draw_at(window, 0., 0.);
                }
            );

            window.text(&font, 750., 20., "45° Skew");
            window.with_transform(
                Transform2d::skewed_deg(45., 0.)
                    .translate(625., 80.),

                |window| {
                    square.draw_at(window, 0., 0.);
                }
            );

            window.text(&font, 1200., 20., "2x Y-Scale");
            window.with_transform(
                Transform2d::scaling(1., 2.)
                    .translate(1100., 80.),

                |window| {
                    square.draw_at(window, 0., 0.);
                }
            );
        }

        renderer.flush()?;
    }

    Ok(())
}
