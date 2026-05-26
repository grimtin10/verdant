#![allow(unused)]

use time::OffsetDateTime;
use verdant::{Renderer, RendererResult, WindowEvent, rgb, rgba, types::Color, view::ViewMode};

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;

    let window = renderer.create_window("clock", 1000, 1000, true);

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if let WindowEvent::CloseRequested = event {
                renderer.close_window(id);
            }
        }

        if let Some(window) = renderer.get_window(window) {
            window.set_view(1000., 1000., ViewMode::Letterbox);
            window.set_origin(500., 500.);

            window.background(rgba(0., 0., 0., 0.25));

            window.no_fill();
            window.outline(Color::WHITE, 5.);

            window.ellipse(0., 0., 400., 400.);

            window.outline(Color::WHITE, 3.);
            for tick in 0..60 {
                let (sin, cos) = ((tick * 6) as f32).to_radians().sin_cos();
                let length = if tick % 5 == 0 { 20. } else { 12. };
                let outer = 390.;
                let inner = outer - length;
                window.line(sin * inner, -cos * inner, sin * outer, -cos * outer);
            }

            let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
            let seconds = now.second() as f32;
            let minutes = now.minute() as f32 + seconds / 60.;
            let hours   = (now.hour() % 12) as f32 + minutes / 60.;

            let (sin, cos) = (seconds * 6.).to_radians().sin_cos();
            window.outline(Color::RED, 1.5);
            window.line(sin * -30., -cos * -30., sin * 330., -cos * 330.);

            let (sin, cos) = (minutes * 6.).to_radians().sin_cos();
            window.outline(Color::WHITE, 2.5);
            window.line(0., 0., sin * 290., -cos * 290.);

            let (sin, cos) = (hours * 30.).to_radians().sin_cos();
            window.outline(Color::WHITE, 3.);
            window.line(0., 0., sin * 220., -cos * 220.);

            window.fill(rgb(0.75, 0.75, 0.75));
            window.no_outline();
            window.ellipse(0., 0., 5., 5.);
        }

        renderer.flush()?;
    }

    Ok(())
}
