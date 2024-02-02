use egui::{self};
use egui_winit::winit::{event::KeyEvent, keyboard::NamedKey};
use eww::{wgpu, winit};
use std::sync::Arc;

use winit::{
    event::{ElementState, Event, WindowEvent, WindowEvent::KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    window::{Window, WindowBuilder},
};

#[derive(Default)]
struct GuiState {
    pub name: String,
    pub age: u8,
}

use futures::executor::block_on;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("eww basic example")
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);

    let mut wgpu = block_on(WgpuCtx::init(&window));

    let mut backend = eww::Backend::new(eww::BackendDescriptor {
        device: &wgpu.device,
        rt_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        window: &window,
    });

    let mut gui_state = GuiState {
        ..Default::default()
    };

    event_loop.run(move |event, elwt| {
        backend.handle_event(&window, &event);
        match event {
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::RedrawRequested => {
                    render(&wgpu, &window, &mut backend, &mut gui_state)
                }
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::Resized(new_size) => {
                    resize(&mut wgpu, *new_size);
                    window.request_redraw();
                }
                _ => {}
            },
            _ => {}
        }
    });
}

fn resize(wgpu: &mut WgpuCtx, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
        wgpu.config.width = new_size.width;
        wgpu.config.height = new_size.height;
        wgpu.surface.configure(&wgpu.device, &wgpu.config);
    }
}

fn render(wgpu: &WgpuCtx, window: &Window, backend: &mut eww::Backend, gui_state: &mut GuiState) {
    let output = wgpu.surface.get_current_texture().unwrap();
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = wgpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    let clear_color = wgpu::Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };

    let render_desc = eww::RenderDescriptor {
        textures_to_update: &[],
        window,
        device: &wgpu.device,
        queue: &wgpu.queue,
        encoder: &mut encoder,
    };

    backend.draw_gui(render_desc, |ctx| {
        build_gui(ctx, gui_state);
    });
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        backend.render(window, &mut render_pass);
    }

    wgpu.queue.submit(Some(encoder.finish()));
    output.present();
}

fn build_gui(ctx: &egui::Context, gui_state: &mut GuiState) {
    egui::Window::new("eww basic example").show(ctx, |ui| {
        ui.heading("My egui Application");
        ui.horizontal(|ui| {
            let name_label = ui.label("Your name: ");
            ui.text_edit_singleline(&mut gui_state.name)
                .labelled_by(name_label.id);
        });
        ui.add(egui::Slider::new(&mut gui_state.age, 0..=120).text("age"));
        if ui.button("Click each year").clicked() {
            gui_state.age += 1;
        }
        ui.label(format!("Hello '{}', age {}", gui_state.name, gui_state.age));
    });
}

struct WgpuCtx<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
}

impl<'a> WgpuCtx<'a> {
    async fn init(window: &Arc<Window>) -> WgpuCtx<'a> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }
}
