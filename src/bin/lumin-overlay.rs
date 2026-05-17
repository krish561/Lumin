use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};

use gtk::gdk::prelude::SurfaceExt;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "dev.lumin.overlay";

fn get_socket_path(monitor_name: &str) -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| env::temp_dir().to_string_lossy().into_owned());

    PathBuf::from(runtime_dir).join(format!("lumin-overlay-{}.socket", monitor_name))
}

fn get_log_path(monitor_name: &str) -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| env::temp_dir().to_string_lossy().into_owned());

    PathBuf::from(runtime_dir).join(format!("lumin-overlay-{}.log", monitor_name))
}

fn log_debug(monitor_name: &str, message: impl AsRef<str>) {
    let path = get_log_path(monitor_name);
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", message.as_ref());
    }
}

fn brightness_to_opacity(brightness: u8) -> f64 {
    1.0 - (brightness.min(100) as f64 / 100.0)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let monitor_name = args
        .iter()
        .position(|a| a == "--monitor")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_default();

    if monitor_name.is_empty() {
        eprintln!("Error: --monitor required");
        std::process::exit(1);
    }

    let initial_brightness: u8 = args
        .iter()
        .position(|a| a == "--brightness")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let app = Application::builder().application_id(APP_ID).build();

    let monitor_name_arc = Arc::new(monitor_name);
    let _ = fs::remove_file(get_log_path(&monitor_name_arc));
    log_debug(
        &monitor_name_arc,
        format!(
            "start monitor={} brightness={} opacity={}",
            monitor_name_arc,
            initial_brightness,
            brightness_to_opacity(initial_brightness)
        ),
    );

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("lumin-overlay")
            .decorated(false)
            .build();
        window.add_css_class("lumin-overlay-window");

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_can_target(false);

        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_exclusive_zone(-1);

        window.set_opacity(brightness_to_opacity(initial_brightness));

        let display = gtk::gdk::Display::default().expect("no display");
        let css = gtk::CssProvider::new();
        css.load_from_data(
            ".lumin-overlay-window,
             .lumin-overlay-window.background,
             window.lumin-overlay-window,
             window.lumin-overlay-window > *,
             .lumin-overlay-surface {
                background: black !important;
                background-color: black !important;
                background-image: none !important;
                color: black !important;
                box-shadow: none !important;
                border: none !important;
                outline: none !important;
            }",
        );
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        let monitors = display.monitors();
        for i in 0..monitors.n_items() {
            let monitor = monitors
                .item(i)
                .and_downcast::<gtk::gdk::Monitor>()
                .unwrap();
            if monitor.connector().unwrap_or_default() == *monitor_name_arc {
                log_debug(
                    &monitor_name_arc,
                    format!("matched monitor connector={}", monitor_name_arc),
                );
                window.set_monitor(Some(&monitor));
                break;
            }
        }

        let monitor_for_realize = Arc::clone(&monitor_name_arc);
        window.connect_realize(move |win| {
            if let Some(surface) = win.native().and_then(|n| n.surface()) {
                let empty_region = gtk::cairo::Region::create();
                surface.set_input_region(Some(&empty_region));
                surface.set_opaque_region(Some(&empty_region));
                log_debug(
                    &monitor_for_realize,
                    "realized surface; input/opaque regions cleared",
                );
            }
        });

        let drawing_area = DrawingArea::new();
        drawing_area.add_css_class("lumin-overlay-surface");
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);

        let monitor_for_draw = Arc::clone(&monitor_name_arc);
        drawing_area.set_draw_func(move |_, cr, width, height| {
            log_debug(
                &monitor_for_draw,
                format!("draw width={width} height={height}"),
            );
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = cr.fill();
        });

        window.set_child(Some(&drawing_area));
        window.present();

        let socket_path = get_socket_path(&monitor_name_arc);
        let _ = fs::remove_file(&socket_path);
        let listener = UnixListener::bind(&socket_path).expect("failed to bind socket");

        let (sender, receiver) = async_channel::unbounded::<u8>();

        thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buf = [0u8; 1];
                if stream.read_exact(&mut buf).is_ok() {
                    let _ = sender.send_blocking(buf[0]);
                }
            }
        });

        let window_clone = window.clone();
        let monitor_for_ipc = Arc::clone(&monitor_name_arc);

        glib::MainContext::default().spawn_local(async move {
            while let Ok(brightness_percent) = receiver.recv().await {
                let opacity = brightness_to_opacity(brightness_percent);
                log_debug(
                    &monitor_for_ipc,
                    format!("ipc brightness={brightness_percent} opacity={opacity}"),
                );
                window_clone.set_opacity(opacity);
            }
        });
    });

    app.run_with_args::<String>(&[]);
}
