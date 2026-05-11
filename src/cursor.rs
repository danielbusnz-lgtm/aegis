use gtk::gdk::{Display, Texture};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, CssProvider, Image, glib};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use std::time::Duration;

const APP_ID: &str = "com.tabby.cursor-mvp";
const WINDOW_SIZE: i32 = 40;
const CURSOR_SIZE: i32 = 12;
const SMOOTHING: f64 = 0.055;
const TICK_MS: u64 = 8;
const Y_OFFSET: i32 = -60;

pub fn cursor(x: i32, y: i32) -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data(
            "window { background: transparent; }
             label { color: #d97757; font-size: 24px; }",
        );
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("could not connect to a display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(WINDOW_SIZE)
            .default_height(WINDOW_SIZE)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, true);
        window.set_margin(Edge::Top, y);
        window.set_margin(Edge::Left, x);

        let cursor_bytes = include_bytes!("../assets/cursor.png");
        let texture = Texture::from_bytes(&glib::Bytes::from_static(cursor_bytes))
            .expect("failed to load cursor texture");
        let image = Image::from_paintable(Some(&texture));
        image.set_pixel_size(CURSOR_SIZE);
        window.set_child(Some(&image));

        window.connect_realize(|window| {
            if let Some(surface) = window.surface() {
                let empty_region = gtk::cairo::Region::create();
                surface.set_input_region(Some(&empty_region));
            }
        });

        window.present();
        println!("[gtk] cursor window presented");

        // Smoothing state — runs on the GTK main thread at ~60fps, safe to
        // touch widgets from here.
        let mut cursor_x = x as f64;
        let mut cursor_y = y as f64;
        let win = window.clone();
        glib::timeout_add_local(Duration::from_millis(TICK_MS), move || {
            if let Ok((mx, my)) = crate::mouse::mouse_movement() {
                cursor_x += (mx as f64 - cursor_x) * SMOOTHING;
                cursor_y += (my as f64 - cursor_y) * SMOOTHING;
                win.set_margin(Edge::Left, cursor_x as i32);
                win.set_margin(Edge::Top, cursor_y as i32 + Y_OFFSET);
            }
            glib::ControlFlow::Continue
        });
    });

    app.run()
}
