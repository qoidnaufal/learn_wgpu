use std::marker::PhantomData;

use wgpu::util::DeviceExt;

use crate::NodeId;
use crate::texture::TextureData;
use crate::shapes::{Shape, Transform};

#[derive(Debug)]
pub struct Buffer<T> {
    pub buffer: wgpu::Buffer,
    pub count: u32,
    usage: wgpu::BufferUsages,
    len: usize,
    label: String,
    _phantom: PhantomData<T>
}

impl<T> Buffer<T> {
    // pub fn v(device: &wgpu::Device, data: &[u8], node_id: NodeId) -> Self {
    //     Self::new(device, wgpu::BufferUsages::VERTEX, data, node_id)
    // }

    pub fn i(device: &wgpu::Device, data: &[u8], node_id: NodeId) -> Self {
        Self::new(device, wgpu::BufferUsages::INDEX, data, node_id)
    }

    pub fn u(device: &wgpu::Device, data: &[u8], node_id: NodeId) -> Self {
        Self::new(device, wgpu::BufferUsages::UNIFORM, data, node_id)
    }

    // pub fn s(device: &wgpu::Device, data: &[u8], node_id: NodeId) -> Self {
    //     Self::new(device, wgpu::BufferUsages::STORAGE, data, node_id)
    // }

    fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, data: &[u8], node_id: NodeId) -> Self {
        let len = data.len();
        let label = node_id.to_string();
        let count = (len / size_of_val(&usage)) as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents: data,
            usage: usage | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            count,
            usage,
            len,
            label,
            _phantom: PhantomData,
        }
    }

    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(..)
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, offset: usize, data: &[u8]) {
        if data.len() > self.len {
            self.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(self.label.as_str()),
                contents: data,
                usage: self.usage | wgpu::BufferUsages::COPY_DST,
            });
            self.len = data.len();
            self.count = (self.len / size_of_val(&self.usage)) as u32;
        } else {
            queue.write_buffer(
                &self.buffer,
                offset as wgpu::BufferAddress,
                data,
            );
        }
    }
}

pub struct Gfx {
    pub i: Buffer<Vec<u32>>,
    pub u: Buffer<Transform>,
    pub t: TextureData,
    pub bg: wgpu::BindGroup,
}

impl Gfx {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        shape: &Shape,
        node_id: NodeId
    ) -> Self {
        let i = shape.i_buffer(node_id, device);
        let u = shape.u_buffer(node_id, device);
        let t = TextureData::new(device, queue, shape.image_data());
        let bg = t.bind_group(device, bg_layout, &u.buffer);

        Self { i, u, t, bg }
    }
}
