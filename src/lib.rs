pub use egui_wgpu as renderer;
pub use egui_winit as platform;

use egui::{ClippedPrimitive, Window};
pub use platform::winit;
pub use renderer::wgpu;

pub use platform::State as Platform;
pub use renderer::Renderer;

use egui::Context as Ctx;
use winit::window;

/// Egui backend with winit platform and wgpu renderer
pub struct Backend {
    platform: Platform,
    renderer: Renderer,
    prims: Option<Vec<ClippedPrimitive>>,
}

impl<'a> Backend {
    pub fn new(desc: BackendDescriptor) -> Self {
        let BackendDescriptor {
            device,
            rt_format,
            window,
        } = desc;
        let ctx = Ctx::default();
        let id = ctx.viewport_id();
        let pixels_per_point = window.scale_factor() as f32;
        let max_texture_side = device.limits().max_texture_dimension_2d as usize;
        let platform = Platform::new(
            ctx,
            id,
            window,
            Some(pixels_per_point),
            Some(max_texture_side),
        );
        let renderer = Renderer::new(device, rt_format, None, 1);

        Self {
            platform,
            renderer,
            prims: None,
        }
    }

    // output indicates if egui wants exclusive access to this event
    pub fn handle_event<T>(
        &mut self,
        window: &window::Window,
        event: &winit::event::Event<T>,
    ) -> bool {
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                self.platform.on_window_event(window, event).consumed
            }
            _ => false,
        }
    }

    //FIXME: better name for this
    pub fn draw_gui<F>(&'a mut self, desc: RenderDescriptor, build_ui: F)
    where
        F: FnOnce(&Ctx),
    {
        let RenderDescriptor {
            textures_to_update: _,
            window,
            device,
            queue,
            encoder,
        } = desc;

        let screen_descriptor = {
            let size = window.inner_size();
            renderer::ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: window.scale_factor() as f32,
            }
        };

        let raw_input: egui::RawInput = self.platform.take_egui_input(window);
        let full_output = self.ctx().run(raw_input, |ctx| {
            build_ui(ctx);
        });
        self.platform
            .handle_platform_output(window, full_output.platform_output);

        let pixels_per_point = self.ctx().pixels_per_point();
        let clipped_primitives = self.ctx().tessellate(full_output.shapes, pixels_per_point);
        self.prims = Some(clipped_primitives);

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            self.prims.as_ref().unwrap(),
            &screen_descriptor,
        );
        for (tex_id, img_delta) in full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, tex_id, &img_delta);
        }
        for tex_id in full_output.textures_delta.free {
            self.renderer.free_texture(&tex_id);
        }
    }

    pub fn render(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        width: u32,
        height: u32,
        pixels_per_point: f32,
    ) {
        let screen_descriptor = {
            renderer::ScreenDescriptor {
                size_in_pixels: [width, height],
                pixels_per_point,
            }
        };
        self.renderer.render(
            render_pass,
            self.prims.as_ref().unwrap(),
            &screen_descriptor,
        );
    }

    pub fn ctx(&self) -> &Ctx {
        &self.platform.egui_ctx()
    }

    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut Platform {
        &mut self.platform
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }
}

/// Backend creation descriptor
pub struct BackendDescriptor<'a> {
    /// Wgpu device
    pub device: &'a wgpu::Device,
    /// Render target format
    pub rt_format: wgpu::TextureFormat,
    pub window: &'a winit::window::Window,
}

pub struct RenderDescriptor<'a> {
    // TODO: turn into iterator
    pub textures_to_update: &'a [&'a egui::TextureId],
    pub window: &'a window::Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
}
