pub use egui_wgpu as renderer;
pub use egui_winit as platform;

use egui::ClippedPrimitive;
pub use platform::winit;
pub use renderer::wgpu;

pub use platform::State as Platform;
pub use renderer::renderer::Renderer;

use egui::Context as Ctx;
use winit::window;

/// Egui backend with winit platform and wgpu renderer
pub struct Backend {
    ctx: Ctx,
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

        let mut platform = Platform::new(window);
        platform.set_max_texture_side(device.limits().max_texture_dimension_2d as usize);
        platform.set_pixels_per_point(window.scale_factor() as f32);
        let renderer = Renderer::new(device, rt_format, None, 1);
        let ctx = Ctx::default();
        ctx.set_pixels_per_point(window.scale_factor() as f32);

        Self {
            ctx,
            platform,
            renderer,
            prims: None,
        }
    }

    // output indicates if egui wants exclusive access to this event
    pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> bool {
        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                self.platform.on_event(&self.ctx, event).consumed
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
            renderer::renderer::ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: window.scale_factor() as f32,
            }
        };

        let raw_input: egui::RawInput = self.platform.take_egui_input(window);
        let full_output = self.ctx.run(raw_input, |ctx| {
            build_ui(ctx);
        });
        self.platform
            .handle_platform_output(window, &self.ctx, full_output.platform_output);

        let clipped_primitives = self.ctx().tessellate(full_output.shapes);
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
        window: &'a window::Window,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let screen_descriptor = {
            let size = window.inner_size();
            renderer::renderer::ScreenDescriptor {
                size_in_pixels: [size.width, size.height],
                pixels_per_point: window.scale_factor() as f32,
            }
        };
        self.renderer.render(
            render_pass,
            self.prims.as_ref().unwrap(),
            &screen_descriptor,
        );
    }

    pub fn ctx(&self) -> &Ctx {
        &self.ctx
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
