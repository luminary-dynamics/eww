use egui::{self};
use eww::{wgpu, winit};

use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[derive(Default)]
struct GuiState {
    pub name: String,
    pub age: u8,
}

use futures::executor::block_on;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("eww basic example")
        .build(&event_loop)
        .unwrap();

    let mut wgpu = block_on(WgpuCtx::init(&window));

    let mut backend = eww::Backend::new(eww::BackendDescriptor {
        device: &wgpu.device,
        rt_format: wgpu::TextureFormat::Bgra8UnormSrgb,
        window: &window,
    });

    let mut gui_state = GuiState {
        ..Default::default()
    };

    event_loop.run(move |event, _, control_flow| {
        backend.handle_event(&event);

        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                render(&wgpu, &window, &mut backend, &mut gui_state)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    resize(&mut wgpu, *new_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    resize(&mut wgpu, **new_inner_size);
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
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
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

struct WgpuCtx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
}

impl WgpuCtx {
    async fn init(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }
}
