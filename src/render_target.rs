use egui_wgpu::wgpu;
use egui_winit::egui;

pub struct RenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    texture_id: egui::TextureId,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        renderer: &mut crate::Renderer,
        texture_format: wgpu::TextureFormat,
        size: [u32; 2],
    ) -> Self {
        let extent = wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        };

        let descriptor = wgpu::TextureDescriptor {
            label: Some("egui_wgpu_render_target_texture"),
            view_formats: &[],
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        let texture = device.create_texture(&descriptor);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gui_wgpu_texture_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("egui_wgpu_render_target_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

        let _bind_group_layout =
            device.create_bind_group_layout(&gui_wgpu_texture_layout_descriptor);

        let texture_id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);

        Self {
            texture,
            view,

            texture_id,
        }
    }

    //pub fn ui(&mut self, ui: &mut egui::Ui, size: impl Into<egui::Vec2>) -> egui::Response {
    //ui.image(self.texture_id, size)
    //}

    pub fn texture_id(&self) -> egui::TextureId {
        self.texture_id
    }
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
