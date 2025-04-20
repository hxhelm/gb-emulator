#![deny(clippy::all)]
#![allow(unused)]

use crossbeam_channel::Receiver;
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

use super::{PixelData, LCD_HEIGHT, LCD_WIDTH};

const BOX_SIZE: i16 = 32;

#[derive(Debug)]
pub struct App {
    // TODO: may receive events other than Frames
    frame_receiver: Receiver<PixelData>,
    pixels: Option<Pixels<'static>>,
    terminated: Arc<AtomicBool>,
    window: Option<Arc<Window>>,
}

impl App {
    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let size = LogicalSize::new((LCD_WIDTH * 3) as f64, (LCD_HEIGHT * 3) as f64);
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
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            match Pixels::new(LCD_WIDTH as u32, LCD_HEIGHT as u32, surface_texture) {
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

    fn draw_new_frame(&mut self) {
        let Ok(new_frame) = self.frame_receiver.try_recv() else {
            return;
        };

        while let Ok(new_frame) = self.frame_receiver.try_recv() {}

        let pixels_frame = self.pixels.as_mut().unwrap().frame_mut();
        let new_pixels = new_frame.0;

        for (i, pixel) in pixels_frame.chunks_exact_mut(4).enumerate() {
            let rgba = match new_pixels[i] {
                0 => [0x9b, 0xbc, 0x0f, 0xff],
                1 => [0x8b, 0xac, 0x0f, 0xff],
                2 => [0x30, 0x62, 0x30, 0xff],
                3 => [0x0f, 0x38, 0x0f, 0xff],
                _ => [0x00, 0x00, 0x00, 0xff],
            };

            pixel.copy_from_slice(&rgba);
        }
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

                self.draw_new_frame();

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
    pub fn init(terminated: Arc<AtomicBool>, frame_receiver: Receiver<PixelData>) -> Self {
        Self {
            frame_receiver,
            pixels: None,
            window: None,
            terminated: terminated.clone(),
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self).expect("Failed to run app");
    }
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
