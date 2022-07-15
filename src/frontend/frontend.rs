use imgui::*;
use std::time::Instant;

use imgui_wgpu::{
  Renderer,
  RendererConfig
};

use winit::{
  event::*,
  event_loop::{ControlFlow, EventLoop},
  window::Window,
  dpi::PhysicalSize,
  dpi::LogicalSize
};

use pollster::block_on;

use wgpu::{Backends, CommandEncoderDescriptor, Device, Instance, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};

use imgui_winit_support::{HiDpiMode, WinitPlatform};

pub struct Frontend {
  event_loop: EventLoop<()>,
  instance: Instance,
  window: Window,
  size: PhysicalSize<u32>,
  surface: Surface,
  renderer: Renderer,
  imgui: Context,
  platform: WinitPlatform,
  device: Device,
  queue: Queue
}

impl Frontend {
  pub fn new() -> Frontend {
    let event_loop = EventLoop::new();
    let instance = Instance::new(Backends::PRIMARY);
    let (window, size, surface) = {
      let window = Window::new(&event_loop).unwrap();
      window.set_inner_size(LogicalSize {
        width: 1280.0,
        height: 720.0,
      });
      window.set_title(&format!("natsukashii v{}", env!("CARGO_PKG_VERSION")));
      let size = window.inner_size();

      let surface = unsafe { instance.create_surface(&window) };

      (window, size, surface)
    };

    let highdpi_factor = window.scale_factor();
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: Some(&surface),
      force_fallback_adapter: false
    })).unwrap();

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

    let surface_desc = SurfaceConfiguration {
      usage: TextureUsages::RENDER_ATTACHMENT,
      format: TextureFormat::Bgra8UnormSrgb,
      width: size.width as u32,
      height: size.height as u32,
      present_mode: PresentMode::Fifo
    };

    surface.configure(&device, &surface_desc);

    let mut imgui = Context::create();
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(
      imgui.io_mut(),
      &window,
      HiDpiMode::Default,
    );
    imgui.set_ini_filename(None);

    let font_size = (13.0 * highdpi_factor) as f32;
    imgui.io_mut().font_global_scale = (1.0 / highdpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::DefaultFontData {
      config: Some(FontConfig {
        oversample_h: 1,
        pixel_snap_h: true,
        size_pixels: font_size,
        ..Default::default()
      }),
    }]);

    let renderer_config = RendererConfig {
      texture_format: surface_desc.format,
      ..Default::default()
    };

    let mut renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

    Frontend {
      event_loop,
      instance,
      window,
      size,
      surface,
      renderer,
      imgui,
      platform,
      device,
      queue
    }
  }

  fn present(&mut self, mut last_frame: Instant) {
    let clear_color = wgpu::Color {
      r: 0.1,
      g: 0.2,
      b: 0.3,
      a: 1.0,
    };

    let now = Instant::now();
    self.imgui.io_mut().update_delta_time(now - last_frame);

    last_frame = now;

    let frame = match self.surface.get_current_texture() {
      Ok(frame) => frame,
      Err(e) => {
        eprintln!("dropped frame: {:?}", e);
        return;
      }
    };
    self.platform
      .prepare_frame(self.imgui.io_mut(), &self.window)
      .expect("Failed to prepare frame");

    let ui = self.imgui.frame();

    let mut encoder =
      self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

    self.platform.prepare_render(&ui, &self.window);

    let view = frame
      .texture
      .create_view(&TextureViewDescriptor::default());
    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
      label: None,
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(clear_color),
          store: true,
        },
      })],
      depth_stencil_attachment: None,
    });

    self.renderer
      .render(ui.render(), &self.queue, &self.device, &mut rpass)
      .expect("Rendering failed");

    drop(rpass);

    self.queue.submit(Some(encoder.finish()));

    frame.present();
  }

  pub fn run(&mut self) {
    let mut last_frame = Instant::now();

    self.event_loop.run(move |event, _, control_flow| {
      *control_flow = if cfg!(feature = "metal-auto-capture") {
        ControlFlow::Exit
      } else {
        ControlFlow::Poll
      };

      match event {
        Event::WindowEvent {
          event: WindowEvent::Resized(_),
          ..
        } => {
          let size = self.window.inner_size();
          let surface_desc = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: PresentMode::Fifo
          };

          self.surface.configure(&self.device, &surface_desc);
        }
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          *control_flow = ControlFlow::Exit;
        }
        Event::MainEventsCleared => self.window.request_redraw(),
        Event::RedrawEventsCleared => {
          self.present(last_frame);
        }
        _ => {}
      }

      self.platform.handle_event(self.imgui.io_mut(), &self.window, &event);
    });
  }
}