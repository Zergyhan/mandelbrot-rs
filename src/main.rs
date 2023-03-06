#![forbid(unsafe_code)]

use rayon::prelude::*;
use log::error;
use num::complex::Complex;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;


fn iterate_mandelbrot_point(c: Complex<f64>, max_iterations: u32) -> f64 {
    let mut z = Complex::new(0.0, 0.0);
    let mut i = 0;
    while i < max_iterations && z.norm() < 2.0 {
        z = z * z + c;
        i += 1;
    }
    (i as f64) / (max_iterations as f64)
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let mut mandelbrot = Mandelbrot::new(200,WIDTH, HEIGHT);


    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            mandelbrot.draw(pixels.get_frame_mut());
            if let Err(err) = pixels.render() {
                error!("{:?}", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(winit::event::VirtualKeyCode::Escape)
                || input.key_pressed(winit::event::VirtualKeyCode::Q)
                || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // WASD
            if input.key_pressed(winit::event::VirtualKeyCode::W){
                mandelbrot.offset.im -= 0.05 * mandelbrot.zoom;
                mandelbrot.changed = true;
            }

            if input.key_pressed(winit::event::VirtualKeyCode::A){
                mandelbrot.offset.re -= 0.05 * mandelbrot.zoom;
                mandelbrot.changed = true;
            }

            if input.key_pressed(winit::event::VirtualKeyCode::S){
                mandelbrot.offset.im += 0.05 * mandelbrot.zoom;
                mandelbrot.changed = true;
            }

            if input.key_pressed(winit::event::VirtualKeyCode::D){
                mandelbrot.offset.re += 0.05 * mandelbrot.zoom;
                mandelbrot.changed = true;
            }

            // Zoom
            if input.key_pressed(winit::event::VirtualKeyCode::R) {
                mandelbrot.zoom /= 2.0;
                mandelbrot.changed = true;
            }
            if input.key_pressed(winit::event::VirtualKeyCode::F) {
                mandelbrot.zoom *= 2.0;
                mandelbrot.changed = true;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                match pixels.resize_surface(size.width, size.height) {
                    Ok(_) => {
                        mandelbrot.width = size.width;
                        mandelbrot.height = size.height;
                        mandelbrot.changed = true;
                        mandelbrot.resized = true;
                        println!("Resized to {}x{}", size.width, size.height)
                    }
                    Err(_) => {
                        error!("Failed to resize surface.");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                };
                match pixels.resize_buffer(size.width, size.height) {
                    Ok(_) => {}
                    Err(_) => {
                        error!("Failed to resize texture.");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                };
            }
            window.request_redraw();
        }
    });
}

struct Mandelbrot {
    max_iterations: u32,
    zoom: f64,
    offset: Complex<f64>,
    width: u32,
    height: u32,
    cache: Vec<f64>,
    changed: bool,
    resized: bool,
}

impl Mandelbrot {
    fn new(max_iterations:u32, width: u32, height: u32) -> Self {
        Self {
            max_iterations,
            zoom: 3.0,
            offset: Complex::new(-0.5, 0.0),
            width,
            height,
            cache: Vec::new(),
            changed: true,
            resized: true,
        }
    }
    fn draw(&mut self, screen: &mut [u8]) {
        if self.changed {
            self.update();
        }
        for (i, pixel) in screen.chunks_exact_mut(4).enumerate() {
            let x = i % self.width as usize;
            let y = i / self.width as usize;
            let colour_slider = self.cache[x + y * self.width as usize];
            let color = [
                (colour_slider * 255.0) as u8,
                (colour_slider * 255.0) as u8,
                (colour_slider * 255.0) as u8,
                255,
            ];
            pixel.copy_from_slice(&color);
        }
    }
    fn update(&mut self) {
        let ratio = self.width as f64 / self.height as f64;
        if self.resized {
            self.cache.clear();
            self.cache.resize((self.width * self.height) as usize, 0.0);
            self.resized = false;
        }
        (0..self.width*self.height).into_par_iter().map(|i| {
            let x = i % self.width;
            let y = i / self.width;
            let c = Complex::new(
                (x as f64 / self.width as f64 - 0.5) * ratio * self.zoom + self.offset.re,
                (y as f64 / self.height as f64 - 0.5) * self.zoom + self.offset.im,
            );
            iterate_mandelbrot_point(c, self.max_iterations)
        }).collect_into_vec(&mut self.cache);
        self.changed = false;
    }
}
