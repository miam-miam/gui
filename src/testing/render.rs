use crate::TestHarness;
use gui_core::glazier::kurbo::Affine;
use gui_core::vello::peniko::Color;
use gui_core::vello::{RenderParams, Renderer, RendererOptions, SceneFragment};
use gui_core::{Component, SceneBuilder, ToComponent};
use std::default::Default;
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    Extent3d, Maintain, MapMode, Queue, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView,
};

const RGBA_SIZE: usize = std::mem::size_of::<u32>();

impl<T: ToComponent> TestHarness<T> {
    fn get_dev_queue(&self, dev_id: usize) -> (&Device, &Queue) {
        let device = &self.window_state.render.devices[dev_id].device;
        let queue = &self.window_state.render.devices[dev_id].queue;
        (device, queue)
    }

    fn create_texture(&self, dev_id: usize, width: u32, height: u32) -> Texture {
        let (device, _) = self.get_dev_queue(dev_id);

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture_desc = TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::STORAGE_BINDING,
            label: None,
            view_formats: &[TextureFormat::Rgba8UnormSrgb],
        };
        device.create_texture(&texture_desc)
    }

    fn render_to_texture(
        &mut self,
        texture_view: &TextureView,
        dev_id: usize,
        render_params: &RenderParams,
    ) {
        let queue = &self.window_state.render.devices[dev_id].queue;
        let device = &self.window_state.render.devices[dev_id].device;

        let renderer_options = RendererOptions {
            surface_format: Some(TextureFormat::Rgba8UnormSrgb),
            timestamp_period: queue.get_timestamp_period(),
        };

        let mut sb = SceneBuilder::for_scene(&mut self.window_state.scene);
        let mut fragment = SceneFragment::new();
        let mut component = SceneBuilder::for_fragment(&mut fragment);
        self.window_state.component.render(
            &mut component,
            &mut self.window_state.handle,
            &mut self.window_state.global_positions[..],
            &mut self.window_state.active_widget,
            &self.window_state.hovered_widgets[..],
        );
        sb.append(
            &fragment,
            Some(Affine::translate(
                self.window_state.global_positions[0].origin().to_vec2(),
            )),
        );

        self.window_state
            .renderer
            .get_or_insert_with(|| Renderer::new(device, &renderer_options).unwrap())
            .render_to_texture(
                device,
                queue,
                &self.window_state.scene,
                texture_view,
                render_params,
            )
            .unwrap();
        device.poll(Maintain::Wait);
    }

    fn copy_texture_to_buffer(
        &mut self,
        texture: &Texture,
        dev_id: usize,
        byte_aligned_width: u32,
        height: u32,
    ) -> Buffer {
        let (device, queue) = self.get_dev_queue(dev_id);

        let output_buffer_size = (byte_aligned_width * height) as BufferAddress;
        let output_buffer_desc = BufferDescriptor {
            size: output_buffer_size,
            usage: BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(byte_aligned_width),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );

        queue.submit(Some(encoder.finish()));
        output_buffer
    }

    fn output_buffer_to_own_buffer(
        &mut self,
        output_buffer: &Buffer,
        dev_id: usize,
        byte_aligned_width: u32,
        width: u32,
        height: u32,
    ) {
        let (device, _) = self.get_dev_queue(dev_id);

        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before awaiting
        // the future. Otherwise, the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(Maintain::Wait);
        pollster::block_on(rx.receive()).unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let capacity_needed = height as usize * width as usize * RGBA_SIZE;
        if let Some(additional) = capacity_needed.checked_sub(self.image_buffer.capacity()) {
            self.image_buffer.reserve(additional);
        }
        for row in 0..height {
            let start = (row * byte_aligned_width) as usize;
            let end = start + RGBA_SIZE * width as usize;
            self.image_buffer.extend_from_slice(&data[start..end])
        }
        drop(data);
        output_buffer.unmap();
    }

    pub(crate) fn render(&mut self) {
        let (width, height) = (
            self.window_state.size.width as u32,
            self.window_state.size.height as u32,
        );

        let dev_id = pollster::block_on(self.window_state.render.device(None)).unwrap();

        let texture = self.create_texture(dev_id, width, height);
        let texture_view = texture.create_view(&Default::default());

        let render_params = RenderParams {
            base_color: Color::WHITE,
            width,
            height,
        };

        self.render_to_texture(&texture_view, dev_id, &render_params);

        let byte_aligned_width = (RGBA_SIZE as u32 * width).next_multiple_of(256);

        let output_buffer =
            self.copy_texture_to_buffer(&texture, dev_id, byte_aligned_width, height);

        self.output_buffer_to_own_buffer(&output_buffer, dev_id, byte_aligned_width, width, height);
    }
}
