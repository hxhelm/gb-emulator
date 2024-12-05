#![deny(clippy::all)]
#![allow(unused)]

use error_iter::ErrorIter as _;
use log::{error, info};
use pixels::{Error, Pixels, SurfaceTexture};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{Window, WindowId};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOX_SIZE: i16 = 64;

#[derive(Debug)]
pub struct App {
    pixels: Option<Pixels>,
    terminated: Arc<AtomicBool>,
    window: Option<Arc<Window>>,
    world: World,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
#[derive(Debug)]
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl App {
    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Hello Pixels")
                            .with_inner_size(size)
                            .with_min_inner_size(size)
                            .with_resizable(false),
                    )
                    .unwrap(),
            )
        };

        self.window = Some(window.clone());

        self.pixels = {
            let window_size = window.inner_size();
            let window_clone = window.clone();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window_clone);
            match Pixels::new(WIDTH, HEIGHT, surface_texture) {
                Ok(pixels) => {
                    window.request_redraw();

                    Some(pixels)
                }
                Err(err) => {
                    log_error("pixels::new", err);
                    event_loop.exit();

                    None
                }
            }
        };
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init_window(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
                self.terminated.store(true, Ordering::Relaxed);
            }

            WindowEvent::Resized(size) => {
                if self.pixels.is_none() {
                    return;
                }

                if let Err(err) = self
                    .pixels
                    .as_mut()
                    .unwrap()
                    .resize_surface(size.width, size.height)
                {
                    log_error("pixels.resize_surface", err);
                    event_loop.exit();
                }
            }

            WindowEvent::RedrawRequested => {
                if self.terminated.load(Ordering::Relaxed) {
                    event_loop.exit();
                }

                if self.pixels.is_none() {
                    return;
                }

                // Update internal state
                self.world.update();

                // Draw the current frame
                self.world.draw(self.pixels.as_mut().unwrap().frame_mut());
                if let Err(err) = self.pixels.as_ref().unwrap().render() {
                    log_error("pixels.render", err);
                    event_loop.exit();
                } else {
                    // Queue a redraw for the next frame
                    self.window.as_ref().unwrap().request_redraw();
                }
            }

            _ => (),
        }
    }

    // When the pointer leaves the window, this is called incessantly!
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}
}

impl App {
    pub fn init(terminated: Arc<AtomicBool>) -> Self {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut app = App {
            pixels: None,
            window: None,
            world: World::new(),
            terminated: terminated.clone(),
        };

        event_loop.run_app(&mut app).expect("Failed to run app");
        app
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
