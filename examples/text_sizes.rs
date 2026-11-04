use verdant::prelude::*;

const SAMPLE: &str = "The quick brown fox jumps over the lazy dog 0123456789 illicit {}[]()";

fn main() -> RendererResult<()> {
    let mut renderer = Renderer::new()?;
    let window = renderer.create_window("text sizes", 1000, 500);

    let mono  = Font::load(include_bytes!("assets/JetBrainsMonoNerdFont_Regular.ttf"))?;
    let inter = Font::load(include_bytes!("assets/Inter-VariableFont_opsz,wght.ttf"))?;

    while renderer.is_running() {
        for (id, event) in renderer.poll() {
            if let WindowEvent::CloseRequested = event { renderer.close_window(id); }
        }

        if let Some(mut window) = renderer.get_window(window) {
            window.background(Color::WHITE);
            window.fill(Color::BLACK);

            let mut y = 10.;
            for (font, size) in [(&mono, 12.), (&mono, 14.3), (&mono, 20.), (&inter, 12.), (&inter, 14.3), (&inter, 20.)] {
                window.font_size(size);
                window.text(font, 10.25, y, SAMPLE);
                y += size * 1.8;
            }

            window.fill(Color::BLACK);
            window.rect(0., 250., 1000., 250.);
            window.fill(Color::WHITE);

            let mut y = 260.;
            for (font, size) in [(&mono, 12.), (&mono, 14.3), (&mono, 20.), (&inter, 12.), (&inter, 14.3), (&inter, 20.)] {
                window.font_size(size);
                window.text(font, 10.25, y, SAMPLE);
                y += size * 1.8;
            }
        }

        renderer.flush()?;
    }

    Ok(())
}
