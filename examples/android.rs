use std::time::Instant;

use verdant::{AndroidApp, prelude::*};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) -> RendererResult<()> {
    let mut renderer = Renderer::new(app)?;
    let window_id = renderer.create_window("verdant android", 1000, 1000);

    let start = Instant::now();
    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if event == WindowEvent::CloseRequested {
                renderer.close_window(id);
            }
        }

        if let Some(mut window) = renderer.get_window(window_id) {
            window.set_view(1000., 1000., ViewMode::Crop);

            window.background(Color::GRAY);

            window.fill(Color::RED);
            window.rect(375., 375., 250., 250.);

            let time = start.elapsed().as_secs_f32() * 2.5;
            let x = time.cos() * 200.;
            let y = (time * 2.).sin() * 200.;

            window.fill(Color::GREEN);
            window.ellipse(x + 500., y + 500., 75., 75.);
        }

        renderer.flush()?;
    }

    Ok(())
}
