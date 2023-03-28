use std::{
    sync::{atomic::AtomicBool, Once},
    thread,
};

use miniquad::{conf::Conf, Context, EventHandler, KeyCode, KeyMods};

static SPACE_PRESSED: AtomicBool = AtomicBool::new(false);

struct AppState;

impl EventHandler for AppState {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        // Clear the screen to a nice deep blue color
        ctx.clear(Some((0., 1., 1., 1.)), None, None);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Space {
            SPACE_PRESSED.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if keycode == KeyCode::Space {
            SPACE_PRESSED.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

fn run() {
    miniquad::start(
        Conf {
            window_title: "Spacebar Window".to_owned(),
            window_width: 800,
            window_height: 600,
            ..Default::default()
        },
        |_| Box::new(AppState),
    );
    std::process::exit(0);
}

pub fn start() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        thread::spawn(|| {
            run();
        });
    });
}

// Other threads can access the current state of the spacebar using this method.
pub fn is_spacebar_pressed() -> bool {
    SPACE_PRESSED.load(std::sync::atomic::Ordering::Relaxed)
}
