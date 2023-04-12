#![allow(unused)]

pub use b::Button;

#[cfg(not(feature = "emulate"))]
mod b {
    use std::{
        sync::{atomic::AtomicBool, Arc},
        thread::{self, JoinHandle},
    };

    use evdev::{enumerate, Device};

    fn find_keyboard_devices() -> Vec<Device> {
        enumerate()
            .map(|(_, dev)| dev)
            .filter(|dev| dev.supported_events().contains(evdev::EventType::KEY))
            .collect()
    }

    pub struct Button {
        threads: Vec<(JoinHandle<()>, Arc<AtomicBool>)>,
    }

    fn watch_key(mut dev: Device, pressed: Arc<AtomicBool>) {
        loop {
            for event in dev.fetch_events().unwrap() {
                match event.kind() {
                    evdev::InputEventKind::Key(k) => {
                        if k == evdev::Key::KEY_SPACE {
                            pressed.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // may need to update this later to support hotplugging
    impl Button {
        pub fn create() -> Self {
            let devices = find_keyboard_devices();
            eprintln!("Found {} keyboard devices", devices.len());
            let threads = devices
                .into_iter()
                .map(|device| {
                    let pressed = Arc::new(AtomicBool::new(false));
                    let pressed_c = pressed.clone();
                    let th = thread::spawn(move || {
                        watch_key(device, pressed_c);
                    });
                    (th, pressed)
                })
                .collect();
            Self { threads }
        }

        pub fn pressed(&self) -> Option<bool> {
            Some(
                self.threads
                    .iter()
                    .any(|(_, pressed)| pressed.load(std::sync::atomic::Ordering::Relaxed)),
            )
        }
    }
}

#[cfg(feature = "emulate")]
mod b {
    use std::{
        sync::{
            atomic::{self, AtomicBool},
            Arc,
        },
        thread,
    };

    use miniquad::{conf::Conf, Context, EventHandler, KeyCode, KeyMods};

    /// Button emulator. The button is displayed as a window. Holding down space while the
    /// window is in focus is equivalent to pressing the button.
    pub struct Button {
        pressed: Arc<AtomicBool>,

        /// the worker that is running the window
        thread: thread::JoinHandle<()>,
    }

    impl Button {
        pub fn create() -> Self {
            let pressed = Arc::new(AtomicBool::new(false));
            let pressed_clone = pressed.clone();

            let thread = thread::spawn(move || {
                AppState {
                    pressed: pressed_clone,
                }
                .run();
            });

            Button { pressed, thread }
        }

        pub fn pressed(&self) -> Option<bool> {
            if self.thread.is_finished() {
                None
            } else {
                Some(self.pressed.load(atomic::Ordering::Relaxed))
            }
        }
    }

    struct AppState {
        pressed: Arc<AtomicBool>,
    }

    impl AppState {
        fn run(self) {
            miniquad::start(
                Conf {
                    window_title: "Spacebar Window".to_owned(),
                    window_width: 800,
                    window_height: 600,
                    ..Default::default()
                },
                |_| Box::new(self),
            );
        }
    }

    impl EventHandler for AppState {
        fn update(&mut self, _ctx: &mut Context) {}

        fn draw(&mut self, _ctx: &mut Context) {}

        fn key_down_event(
            &mut self,
            _ctx: &mut Context,
            keycode: KeyCode,
            _keymods: KeyMods,
            _repeat: bool,
        ) {
            if keycode == KeyCode::Space {
                self.pressed.store(true, atomic::Ordering::Relaxed);
            }
        }

        fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
            if keycode == KeyCode::Space {
                self.pressed.store(false, atomic::Ordering::Relaxed);
            }
        }
    }
}
