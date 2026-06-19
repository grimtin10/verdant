use verdant::prelude::*;
use verdant::window::WindowDraw;

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    let stretch = renderer.create_window_ext(WindowProperties {
        title: "views, stretched".into(),
        width: 500,
        height: 500,
        resizable: true,
        ..Default::default()
    });

    let letterbox = renderer.create_window_ext(WindowProperties {
        title: "views, letterboxed".into(),
        width: 500,
        height: 500,
        resizable: true,
        ..Default::default()
    });

    let crop = renderer.create_window_ext(WindowProperties {
        title: "views, crop".into(),
        width: 500,
        height: 500,
        resizable: true,
        ..Default::default()
    });

    // helper closure
    let content = |mut window: WindowDraw| {
        window.fill(rgb(0.1, 0.1, 0.1));
        window.rect(0., 0., 500., 500.);

        window.fill(Color::SALMON);
        window.ellipse(150., 150., 150., 150.);

        window.fill(Color::FOREST_GREEN);
        window.rect(250., 250., 250., 250.);
    };

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(mut window) = renderer.get_window(stretch) {
            window.background(Color::BLACK);

            // sets the logical view to be 500x500, and to stretch to fit
            window.set_view(500., 500., ViewMode::Stretch);

            content(window);
        }

        if let Some(mut window) = renderer.get_window(letterbox) {
            window.background(Color::BLACK);

            // sets the logical view to be 500x500, and to letterbox (in web this is contain) to fit
            window.set_view(500., 500., ViewMode::Letterbox);

            content(window);
        }

        if let Some(mut window) = renderer.get_window(crop) {
            window.background(Color::BLACK);

            // sets the logical view to be 500x500, and to crop (in web this is cover) with the window
            window.set_view(500., 500., ViewMode::Crop);

            content(window);
        }

        renderer.flush()?;
    }

    Ok(())
}
