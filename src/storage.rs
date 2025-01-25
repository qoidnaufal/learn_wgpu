use std::collections::HashMap;
use math::{Size, Vector2};
use crate::context::{MouseAction, CONTEXT};
use crate::renderer::Gfx;
use crate::view::{NodeId, View};
use crate::texture::{image_reader, TextureData};
use crate::shapes::Shape;
use crate::error::Error;
use crate::callback::CALLBACKS;
use crate::IntoView;

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
pub struct WidgetStorage {
    pub nodes: Vec<NodeId>,
    pub shapes: HashMap<NodeId, Shape>,
    pub children: HashMap<NodeId, Vec<NodeId>>,
    pub parent: HashMap<NodeId, NodeId>,
    pub changed_ids: Vec<NodeId>,
}

impl WidgetStorage {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            shapes: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            changed_ids: Vec::new(),
        }
    }

    pub fn insert(&mut self, node: impl IntoView) -> &mut Self {
        let node = node.into_view();
        node.layout();
        let node_id = node.id();
        let shape = node.shape();
        self.nodes.push(node_id);
        self.shapes.insert(node_id, shape);
        if let Some(children) = node.children() {
            children.iter().for_each(|child_view| {
                let child_id = child_view.id();
                let child_shape = child_view.shape();
                self.nodes.push(child_id);
                self.shapes.insert(child_id, child_shape);
                self.parent.insert(child_id, node_id);
            });
            if let Some(child_storage) = self.children.get_mut(&node_id) {
                child_storage.extend(children.iter().map(|v| v.id()));
            } else {
                self.children.insert(node_id, children.iter().map(|v| v.id()).collect());
            }
        }
        self
    }

    pub fn has_changed(&self) -> bool {
        !self.changed_ids.is_empty()
    }

    pub fn prepare(
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
                    shape.color.into()
                    // ImageData {
                    //     dimension: (1, 1).into(),
                    //     data: Color::from(shape.color).to_vec(),
                    // }
                };
                let v = shape.v_buffer(*node_id, device);
                let i = shape.i_buffer(*node_id, device);
                let u = shape.u_buffer(*node_id, device);
                let t = TextureData::new(
                    device,
                    queue,
                    bg_layout,
                    u,
                    image_data,
                );

                gfx.v_buffer.insert(*node_id, v);
                gfx.i_buffer.insert(*node_id, i);
                gfx.textures.insert(*node_id, t);
            }
        });
    }

    pub fn detect_hover(&self) {
        let hovered = self.shapes.iter().find(|(_, shape)| {
            shape.is_hovered()
        });
        if let Some((id, _)) = hovered {
            CONTEXT.with_borrow_mut(|ctx| {
                if ctx.cursor.click.obj.is_none() {
                    ctx.cursor.hover.prev = ctx.cursor.hover.curr;
                    ctx.cursor.hover.curr = Some(*id);
                }
            })
        } else {
            CONTEXT.with_borrow_mut(|ctx| {
                ctx.cursor.hover.prev = ctx.cursor.hover.curr.take();
            });
        }
    }

    pub fn handle_hover(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(ref prev_id) = CONTEXT.with_borrow_mut(|ctx| ctx.cursor.hover.prev.take()) {
            let shape = self.shapes.get_mut(prev_id).unwrap();
            if shape.revert_color() {
                self.changed_ids.push(*prev_id);
            }
        }
        if let Some(ref hover_id) = cursor.hover.curr {
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
            self.changed_ids.push(*hover_id);
        }
    }

    pub fn handle_click(&mut self) {
        let cursor = CONTEXT.with_borrow(|ctx| ctx.cursor);
        if let Some(ref click_id) = cursor.click.obj {
            let shape = self.shapes.get_mut(click_id).unwrap();
            CALLBACKS.with_borrow_mut(|callbacks| {
                if let Some(on_click) = callbacks.on_click.get_mut(click_id) {
                    on_click(shape);
                    self.changed_ids.push(*click_id);
                }
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let shape = self.shapes.get_mut(hover_id).unwrap();
                CALLBACKS.with_borrow_mut(|callbacks| {
                    if let Some(on_hover) = callbacks.on_hover.get_mut(hover_id) {
                        on_hover(shape);
                        self.changed_ids.push(*hover_id);
                    }
                });
            }
        }
    }

    pub fn layout(&mut self) {
        let window_size: Size<f32> = CONTEXT.with_borrow(|ctx| ctx.window_size.into());
        // this should be something like &mut LayoutCtx
        let mut used_space = Size::new(0, 0);

        self.nodes.iter().for_each(|node_id| {
            // if let Some(children) = self.children.get(node_id) {
            //     children.iter().for_each(|child_id| {
            //         let parent_id = self.parent[child_id];
            //         let parent_shape = self.shapes.get(&parent_id).unwrap();
            //         let child_shape = self.shapes.get_mut(child_id).unwrap();
            //         // child_shape.scale();
            //     });
            // }

            let shape = self.shapes.get_mut(node_id).unwrap();
            shape.scale();
            let pos: Vector2<f32> = CONTEXT.with_borrow(|cx| cx.layout.get_position(node_id).unwrap().clone()).into();
            let used = Size::<f32>::new(pos.x, pos.y) / window_size;
            let x = (used.width + shape.transform[0].x) - 1.0;   // -1.0 is the left edge of the screen coordinate
            let y = 1.0 - (used.height + shape.transform[1].y); //  1.0 is the top  edge of the screen coordinate
            let translate = Vector2 { x, y };
            shape.set_translate(translate);
            self.changed_ids.push(*node_id);
            used_space.height += shape.dimensions.height;
        });
    }
}

