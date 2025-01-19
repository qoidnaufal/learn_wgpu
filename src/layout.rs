use std::collections::HashMap;
use math::{Size, Vector2};
use crate::buffer::Gfx;
use crate::color::Color;
use crate::widget::{NodeId, Widget};
use crate::texture::{image_reader, ImageData, TextureData};
use crate::shapes::Shape;
use crate::error::Error;
use crate::callback::CALLBACKS;
use crate::app::CONTEXT;

pub fn cast_slice<A: Sized, B: Sized>(p: &[A]) -> Result<&[B], Error> {
    if align_of::<B>() > align_of::<A>()
        && (p.as_ptr() as *const () as usize) % align_of::<B>() != 0 {
        return Err(Error::PointersHaveDifferentAlignmnet);
    }
    unsafe {
        let len = size_of_val::<[A]>(p) / size_of::<B>();
        Ok(core::slice::from_raw_parts(p.as_ptr() as *const B, len))
    }
}

#[derive(Debug)]
pub struct Layout {
    pub nodes: Vec<NodeId>,
    pub shapes: HashMap<NodeId, Shape>,
    pub has_changed: bool,
    last_changed_id: Option<NodeId>,
    used_space: Size<u32>,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            used_space: Size::new(0, 0),
            has_changed: false,
            last_changed_id: None,
        }
    }

    pub fn insert(&mut self, node: impl Widget) -> &mut Self {
        let id = node.id();
        let shape = node.shape();
        self.nodes.push(id);
        self.shapes.insert(id, shape);
        self
    }

    pub fn process_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bg_layout: &wgpu::BindGroupLayout,
        gfx: &mut Gfx,
    ) {
        self.nodes.iter().for_each(|node_id| {
            if let Some(shape) = self.shapes.get(node_id) {
                let image_data = if let Some(ref src) = shape.src {
                    image_reader(src)
                } else {
                    ImageData {
                        dimension: (1, 1).into(),
                        data: Color::from(shape.color).to_vec(),
                    }
                };
                let v_buffer = shape.v_buffer(device);
                let i_buffer = shape.i_buffer(device);
                let u_buffer = shape.u_buffer(device);
                let texture = TextureData::new(
                    device,
                    queue,
                    bg_layout,
                    u_buffer,
                    image_data.dimension,
                    &image_data.data,
                    *node_id,
                );

                gfx.v_buffer.push(v_buffer);
                gfx.i_buffer.push(i_buffer);
                gfx.textures.push(texture);
            }
        });
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().find(|(_, shape)| {
            shape.is_hovered()
        });
        if let Some((id, _)) = hovered {
            CONTEXT.with_borrow_mut(|ctx| {
                if let Some(click_id) = ctx.cursor.click.obj {
                    ctx.cursor.hover.obj = Some(click_id);
                } else {
                    ctx.cursor.hover.obj = Some(*id);
                }
            })
        } else {
            CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.obj = None)
        }
    }

    pub fn handle_hover(&mut self, queue: &wgpu::Queue, gfx: &Gfx) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let (Some(ref hover_id), Some(ref change_id), None) = (
            cursor.hover.obj, self.last_changed_id, cursor.click.obj
        ) { if hover_id == change_id { return; } }

        if let Some(ref change_id) = self.last_changed_id.take() {
            if cursor.hover.obj.is_some_and(|hover_id| hover_id != *change_id) || cursor.hover.obj.is_none() {
                let shape = self.shapes.get_mut(change_id).unwrap();
                shape.revert_color();
                if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *change_id) {
                    texture.change_color(queue, shape.color);
                }
                self.has_changed = true;
            }
        }
        if let Some(ref hover_id) = cursor.hover.obj {
            let shape = self.shapes.get_mut(hover_id).unwrap();

            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                    on_hover(shape);
                }
                if cursor.is_dragging(*hover_id) {
                    if let Some(on_drag) = callbacks.on_drag.get_mut(hover_id) {
                        on_drag(shape);
                    }
                }
            });
            
            if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *hover_id) {
                texture.change_color(queue, shape.color);
            }
            self.has_changed = true;
            self.last_changed_id = Some(*hover_id);
        }
    }

    pub fn handle_click(&mut self, queue: &wgpu::Queue, gfx: &Gfx) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let Some(ref click_id) = cursor.click.obj {
            let shape = self.shapes.get_mut(click_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_click) = callbacks.on_click.get_mut(click_id) {
                    on_click(shape);
                }
            });
            if let Some(texture) = gfx.textures.iter().find(|t| t.node_id == *click_id) {
                texture.change_color(queue, shape.color);
            }
            self.has_changed = true;
            self.last_changed_id = Some(*click_id)
        }
    }

    pub fn calculate(&mut self) {
        let window_size: Size<f32> = CONTEXT.with_borrow(|ctx| ctx.window_size.into());

        self.nodes.iter().for_each(|id| {
            if let Some(shape) = self.shapes.get_mut(id) {
                let s = Size::<f32>::from(shape.dimensions) / window_size / 2.0;
                let used = Size::<f32>::from(self.used_space) / window_size;
                let tx = (used.width + s.width) - 1.0;
                let ty = 1.0 - (s.height + used.height);
                shape.set_transform(Vector2 { x: tx, y: ty }, s);
                self.used_space.height += shape.dimensions.height;
            }
        });
    }
}

