use verdant::prelude::*;

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    let first = renderer.create_window("window 1", 800, 800);
    let second = renderer.create_window("window 2", 800, 800);
    let third = renderer.create_window_ext(WindowProperties {
        title: "window 3".into(),
        width: 800,
        height: 800,
        resizable: true,
        ..Default::default()
    });

    let mut mouse_ellipse = Ellipse::new(
        Vec2::new(400., 400.),
        Vec2::new(50., 50.),
        Style {
            fill_color: Color::RED,
            outline_color: Color::BLACK,
            outline_width: 5.,
            ..Default::default()
        },
        Transform2d::identity(),
    );

    // with Image::load (or any function that takes a `ByteSource`)
    // you can either use `include_bytes` or just give the path
    let image = Image::load(include_bytes!("assets/1.png"))?;
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if let WindowEvent::CloseRequested = event {
                renderer.close_window(id);
            }
        }

        // imperative API
        if let Some(window) = renderer.get_window(first) {
            window.background(rgb(0.7, 0.3, 0.3));
            window.no_outline();

            window.fill(Color::WHITE);
            window.image(&image, 0., 0., window.get_mouse_x(), window.get_mouse_y());

            window.fill(rgb(0.3, 0.7, 0.7));
            window.rect(200., 200., 400., 400.);

            window.fill(rgb(0.7, 0.3, 0.7));
            window.ellipse(400., 400., 180., 180.);

            if window.is_focused() {
                mouse_ellipse.fill(rgb(1.0, 0.4, 0.4));
                mouse_ellipse.position(window.get_mouse_x(), window.get_mouse_y());
            }
            mouse_ellipse.draw(window);

            window.fill(rgba(0., 0., 0., 0.5));
            window.rect(0., 400., 800., 400.);

            window.outline_color(Color::BLACK);
            window.outline_width(10.0);
            window.line(400., 400., window.get_mouse_x(), window.get_mouse_y());
        }

        // declarative API
        if let Some(window) = renderer.get_window(second) {
            window.background(rgb(0.3, 0.7, 0.3));

            Rect::at(200., 200.)
                .size(400., 400.)
                .fill(rgb(0.7, 0.3, 0.7))
                .outline(Color::BLACK, 5.0)
                .corner_radius(25.)
                .draw(window);

            Ellipse::at(400., 400.)
                .size(18., 180.)
                .fill(rgb(0.7, 0.7, 0.3))
                .outline(Color::BLACK, 5.0)
                .transform(Transform2d::rotation_deg(45.))
                .draw(window);

            if window.is_focused() {
                mouse_ellipse.fill(rgb(0.4, 1.0, 0.4));
                mouse_ellipse.position(window.get_mouse_x(), window.get_mouse_y());
            }
            mouse_ellipse.draw(window);

            Rect::at(0., 400.)
                .size(800., 400.)
                .fill(rgba(0., 0., 0., 0.5))
                .outline(Color::BLACK, 5.0)
                .draw(window);
        }

        // custom view and origin
        if let Some(window) = renderer.get_window(third) {
            window.set_view(800., 800., ViewMode::Letterbox);
            window.set_origin(400., 400.);

            window.set_title(format!("window 3: {}x {}y", window.get_raw_mouse_x(), window.get_raw_mouse_y()));

            window.background(rgb(0.3, 0.3, 0.7));
            window.outline_color(rgb(0.0, 0.0, 0.0));
            window.outline_width(2.0);

            window.fill(rgb(0.7, 0.7, 0.3));

            window.with_style(|window| {
                window.corner_radius(100.);
                window.rect(-200., -200., 400., 400.);
            });

            window.with_transform(Transform2d::rotation_deg(window.get_mouse_x() / 800. * 360.), |window| {
                window.outline_color(rgb(1.0, 0.0, 0.0));
                window.fill(rgb(0.3, 0.7, 0.7));
                window.ellipse(0., 0., 180., 18.);
            });

            // the state (style, view) is restored after this
            window.with_style(|window| {
                window.set_origin(0., 0.);
                if window.is_focused() {
                    mouse_ellipse.fill(rgb(0.4, 0.4, 1.0));
                    mouse_ellipse.position(window.get_mouse_x(), window.get_mouse_y());
                }
                mouse_ellipse.draw(window);
            });

            window.outline_color(rgb(0.0, 1.0, 1.0));
            window.fill(rgba(0., 0., 0., 0.5));
            window.rect(-400., 0., 800., 400.);
        }

        renderer.flush()?;
    }

    Ok(())
}
