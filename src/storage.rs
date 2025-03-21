use std::collections::HashMap;
use util::Vector2;

use crate::context::{Cursor, LayoutCtx, MouseAction};
use crate::renderer::{Gfx, Renderer};
use crate::shapes::Attributes;
use crate::view::NodeId;
use crate::callback::CALLBACKS;
use crate::Rgba;

#[derive(Debug)]
pub struct WidgetStorage {
    pub nodes: Vec<NodeId>,
    pub children: HashMap<NodeId, Vec<NodeId>>,
    pub parent: HashMap<NodeId, NodeId>,
    pub attribs: HashMap<NodeId, Attributes>,
    pub cached_color: HashMap<NodeId, Rgba<u8>>,
    pub layout: LayoutCtx,
    pending_update: Vec<NodeId>,
}

impl Default for WidgetStorage {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            attribs: HashMap::new(),
            cached_color: HashMap::new(),
            layout: LayoutCtx::new(),
            pending_update: Vec::new(),
        }
    }
}

impl WidgetStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_children(&mut self, node_id: NodeId, child_id: NodeId) {
        self
            .children
            .entry(node_id)
            .or_insert(vec![child_id])
            .push(child_id);
    }

    pub fn insert_parent(&mut self, node_id: NodeId, parent_id: NodeId) {
        self.parent.insert(node_id, parent_id);
    }

    pub fn get_parent(&self, node_id: NodeId) -> Option<&NodeId> {
        self.parent.get(&node_id)
    }

    // fn get_children(&self, node_id: NodeId) -> Option<&Vec<NodeId>> {
    //     self.children.get(&node_id)
    // }

    pub fn is_root(&self, node_id: NodeId) -> bool {
        self.parent.get(&node_id).is_none()
    }

    pub fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.clear();
        renderer.update();
    }

    pub fn detect_hover(&self, cursor: &mut Cursor, gfx: &Gfx) {
        // let start = std::time::Instant::now();
        let hovered = self.nodes.iter().enumerate().filter_map(|(idx, node_id)| {
            let shape = &gfx.shapes.data[idx];
            let attr = &self.attribs[node_id];
            if shape.is_hovered(cursor, attr) {
                Some(node_id)
            } else { None }
        }).min();
        // eprintln!("{:?}", start.elapsed());
        if let Some(id) = hovered {
            if cursor.click.obj.is_none() {
                cursor.hover.prev = cursor.hover.curr;
                cursor.hover.curr = Some(*id);
            }
        } else {
            cursor.hover.prev = cursor.hover.curr.take();
        }
    }

    pub fn handle_hover(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if cursor.is_hovering_same_obj() && cursor.click.obj.is_none() {
            return;
        }
        if let Some(ref prev_id) = cursor.hover.prev.take() {
            if let Some(cached) = self.cached_color.get(prev_id) {
                let idx = self.nodes.iter().position(|node_id| node_id == prev_id).unwrap();
                gfx.shapes.update(idx, |shape| shape.revert_color(*cached));
                self.pending_update.push(*prev_id);
            }
        }
        if let Some(ref hover_id) = cursor.hover.curr {
            let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
            gfx.shapes.update(idx, |shape| {
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(hover_id, shape);
                    if cursor.is_dragging(*hover_id) {
                        callbacks.handle_drag(hover_id, shape);
                        if let Some(attribs) = self.attribs.get_mut(hover_id) {
                            gfx.transforms.update(shape.transform as usize, |transform| {
                                attribs.set_position(cursor, transform);
                            });
                        }
                    }
                });
            });
            self.pending_update.push(*hover_id);
        }
    }

    pub fn handle_click(&mut self, cursor: &mut Cursor, gfx: &mut Gfx) {
        if let Some(ref click_id) = cursor.click.obj {
            let idx = self.nodes.iter().position(|node_id| node_id == click_id).unwrap();
            let shape = gfx.shapes.data.get_mut(idx).unwrap();
            let attr = &self.attribs[click_id];
            cursor.click.delta = cursor.click.pos - Vector2::<f32>::from(attr.pos);
            CALLBACKS.with_borrow_mut(|callbacks| {
                callbacks.handle_click(click_id, shape);
                self.pending_update.push(*click_id);
            });
        }
        if cursor.state.action == MouseAction::Released {
            if let Some(ref hover_id) = cursor.hover.curr {
                let idx = self.nodes.iter().position(|node_id| node_id == hover_id).unwrap();
                let shape = gfx.shapes.data.get_mut(idx).unwrap();
                CALLBACKS.with_borrow_mut(|callbacks| {
                    callbacks.handle_hover(hover_id, shape);
                    self.pending_update.push(*hover_id);
                });
            }
        }
    }
}

