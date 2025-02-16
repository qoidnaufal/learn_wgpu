use std::path::Path;
use std::io::Read;
use std::fs::File;

use image::GenericImageView;

use crate::color::{Color, Rgb};
use crate::Rgba;

use super::Gpu;

pub fn image_reader<P: AsRef<Path>>(path: P) -> Color<Rgba<u8>, u8> {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    let len = file.read_to_end(&mut buf).unwrap();
    let image = image::load_from_memory(&buf[..len]).unwrap();

    Color::new(image.dimensions(), &image.to_rgba8())
}

#[derive(Debug)]
pub struct TextureData {
    texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl TextureData {
    pub fn new(gpu: &Gpu, data: Color<Rgba<u8>, u8>) -> Self {
        let device = &gpu.device;
        let queue = &gpu.queue;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: data.dimensions().width,
                height: data.dimensions().height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                // | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Self::bind_group(device, &view);

        submit_texture(queue, texture.as_image_copy(), data);

        Self { texture, bind_group }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }
            ],
        })
    }

    pub fn bind_group(
        device: &wgpu::Device,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                }
            ],
        })
    }

    pub fn update_color(&self, queue: &wgpu::Queue, new_color: Rgb<u8>) {
        submit_texture(queue, self.texture.as_image_copy(), new_color.into());
    }
}

fn submit_texture(
    queue: &wgpu::Queue,
    texture: wgpu::TexelCopyTextureInfo,
    data: Color<Rgba<u8>, u8>
) {
    queue.write_texture(
        texture,
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * data.dimensions().width),
            rows_per_image: Some(data.dimensions().height),
        },
        wgpu::Extent3d {
            width: data.dimensions().width,
            height: data.dimensions().height,
            depth_or_array_layers: 1,
        }
    );
}
