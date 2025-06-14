use std::collections::HashMap;

use aplite_reactive::Effect;
use aplite_types::{Size, Vector2};

use aplite_renderer::ImageData;
use aplite_renderer::Render;
use aplite_renderer::Renderer;

pub mod layout;
pub(crate) mod properties;
pub(crate) mod cursor;
pub(crate) mod tree;

use properties::{AspectRatio, Properties};
use tree::{Entity, NodeId, Tree};
use cursor::{Cursor, MouseAction, MouseButton};
use layout::{
    LayoutContext,
    Orientation,
};

pub(crate) enum UpdateMode {
    HoverColor(NodeId),
    ClickColor(NodeId),
    RevertColor(NodeId),
    Transform(NodeId),
    Size(NodeId),
}

type ImageFn = Box<dyn Fn() -> ImageData>;
type StyleFn = Box<dyn Fn(&mut Properties)>;
type ActionFn = Box<dyn Fn()>;

pub struct Context {
    current: Option<NodeId>,
    pub(crate) tree: Tree<NodeId>,
    pub(crate) properties: Vec<Properties>,
    image_fn: HashMap<NodeId, ImageFn>,
    style_fn: HashMap<NodeId, StyleFn>,
    callbacks: HashMap<NodeId, ActionFn>,
    pub(crate) cursor: Cursor,
    pending_update: Vec<UpdateMode>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            current: None,
            tree: Default::default(),
            properties: Vec::with_capacity(1024),
            image_fn: HashMap::new(),
            style_fn: HashMap::new(),
            callbacks: HashMap::new(),
            cursor: Cursor::new(),
            pending_update: Vec::with_capacity(10),
        }
    }
}

impl Context {
    pub(crate) fn new(size: Size<u32>) -> Self {
        let mut cx = Self::default();
        cx.tree.insert(NodeId::root(), None);
        cx.properties.push(Properties::window_properties(size));
        cx
    }

    // pub(crate) fn add_view<F, IV>(&mut self, view_fn: F)
    // where
    //     F: Fn() -> IV + 'static,
    //     IV: crate::view::IntoView + 'static,
    // {
    //     Effect::new(move |_| {
    //         view_fn().into_view(self, |_| {});
    //     });
    // }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                         Debug                            //
//                                                          //
// ........................................................ //
// ........................................................ //

#[cfg(feature = "debug_tree")]
impl Context {
    pub(crate) fn debug_tree(&self) {
        self.print_children_from(NodeId::root());
    }

    pub(crate) fn print_children_from(&self, start: NodeId) {
        eprintln!(" > {start:?}: {:?}", self.get_window_properties().name().unwrap_or_default());
        if start == NodeId::root() {
            self.recursive_print(None, 0);
        } else {
            self.recursive_print(Some(start), 0);
        }
    }

    fn recursive_print(&self, start: Option<NodeId>, indent: usize) {
        let acc = 3;
        if let Some(current) = start {
            if let Some(children) = self.tree.get_all_children(&current) {
                children.iter().for_each(|child| {
                    let prop = self.get_node_data(child);
                    let data = prop.size();
                    let name = prop.name().unwrap_or_default();
                    if self.tree.get_parent(child).is_some_and(|p| self.tree.get_parent(p).is_some()) {
                        for i in 0..(indent - acc)/acc {
                            let c = acc - i;
                            eprint!("{:c$}|", "");
                        }
                        let j = acc - 1;
                        eprintln!("{:j$}╰─ {child:?}: {name:?} | {data:?}", "");
                    } else {
                        eprintln!("{:indent$}╰─ {child:?}: {name:?} | {data:?}", "");
                    }
                    if self.tree.get_first_child(child).is_some() {
                        self.recursive_print(Some(*child), indent + acc);
                    }
                });
            }
        } else {
            self.tree.get_all_ancestor()
                .iter()
                .for_each(|node| {
                    let prop = self.get_node_data(*node);
                    let data = prop.size();
                    let name = prop.name().unwrap_or_default();
                    eprintln!(" > {node:?}: {name:?} | {data:?}");
                    if self.tree.get_first_child(node).is_some() {
                        self.recursive_print(Some(**node), indent + acc);
                    }
                });
        }
    }
}

// window
impl Context {
    pub(crate) fn get_window_properties(&self) -> &Properties {
        &self.properties[0]
    }

    pub(crate) fn update_window_properties<F: Fn(&mut Properties)>(&mut self, f: F) {
        if let Some(prop) = self.properties.get_mut(0) {
            f(prop);
        }
    }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                          Data                            //
//                                                          //
// ........................................................ //
// ........................................................ //

impl Context {
    pub(crate) fn create_entity(&self) -> NodeId {
        self.tree.create_entity()
    }

    pub(crate) fn current_entity(&self) -> Option<NodeId> {
        self.current
    }

    pub(crate) fn set_current_entity(&mut self, entity: Option<NodeId>) {
        self.current = entity;
    }

    pub(crate) fn insert(
        &mut self,
        node_id: NodeId,
        parent: Option<NodeId>,
        properties: Properties,
    ) {
        self.tree.insert(node_id, parent);
        self.properties.push(properties);
    }

    pub(crate) fn add_image<F: Fn() -> ImageData + 'static>(&mut self, node_id: NodeId, f: F) {
        self.image_fn.insert(node_id, Box::new(f));
    }

    pub(crate) fn add_style_fn<F: Fn(&mut Properties) + 'static>(&mut self, node_id: NodeId, style_fn: F) {
        self.style_fn.insert(node_id, Box::new(style_fn));
    }

    pub(crate) fn add_callbacks<F: Fn() + 'static>(&mut self, node_id: NodeId, callback: F) {
        self.callbacks.insert(node_id, Box::new(callback));
    }

    pub(crate) fn get_node_data(&self, node_id: &NodeId) -> &Properties {
        &self.properties[node_id.index()]
    }

    pub(crate) fn get_node_data_mut(&mut self, node_id: &NodeId) -> &mut Properties {
        &mut self.properties[node_id.index()]
    }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                          Layout                          //
//                                                          //
// ........................................................ //
// ........................................................ //

impl Context {
    pub(crate) fn layout(&mut self) {
        let ancestors = self.tree
            .get_all_ancestor()
            .iter()
            .map(|node_id| **node_id)
            .collect::<Vec<_>>();

        ancestors
            .iter()
            .for_each(|node_id| {
                self.calculate_size_recursive(node_id);
            });

        self.recursive_layout(&NodeId::root());
    }

    pub(crate) fn recursive_layout(&mut self, node_id: &NodeId) {
        let children = LayoutContext::new(node_id, self).calculate();
        if node_id !=&NodeId::root() { self.pending_update.push(UpdateMode::Transform(*node_id)) }
        if let Some(children) = children {
            children.iter().for_each(|child| self.recursive_layout(child));
        }
    }

    fn calculate_size_recursive(&mut self, node_id: &NodeId) -> Size<u32> {
        let prop = *self.get_node_data(node_id);
        let padding = prop.padding();
        let mut size = prop.size();

        let mut resized = false;

        if let Some(children) = self.tree.get_all_children(node_id) {
            children.iter().for_each(|child_id| {
                let child_size = self.calculate_size_recursive(child_id);
                match prop.orientation() {
                    Orientation::Vertical => {
                        size.add_height(child_size.height());
                        size.set_width(size.width().max(child_size.width() + padding.horizontal()));
                    }
                    Orientation::Horizontal => {
                        size.set_height(size.height().max(child_size.height() + padding.vertical()));
                        size.add_width(child_size.width());
                    }
                }
            });
            let child_len = children.len() as u32;
            let stretch = prop.spacing() * (child_len - 1);
            match prop.orientation() {
                Orientation::Vertical => {
                    size.add_height(padding.vertical() + stretch);
                },
                Orientation::Horizontal => {
                    size.add_width(padding.horizontal() + stretch);
                },
            }
        }

        if let AspectRatio::Defined(tuple) = prop.image_aspect_ratio() {
            if let Some(parent) = self.tree.get_parent(node_id) {
                match self.get_node_data(parent).orientation() {
                    Orientation::Vertical => size.adjust_height(tuple.into()),
                    Orientation::Horizontal => size.adjust_width(tuple.into()),
                }
            } else {
                size.adjust_width(tuple.into());
            }
            self.get_node_data_mut(node_id).set_size(size);
            self.pending_update.push(UpdateMode::Size(*node_id));
        }

        let final_size = size
            .max(prop.min_width(), prop.min_height())
            .min(prop.max_width(), prop.max_height());

        resized |= final_size != prop.size();

        if resized {
            self.get_node_data_mut(node_id).set_size(final_size);
            self.pending_update.push(UpdateMode::Size(*node_id));
        }
        final_size
    }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                          Cursor                          //
//                                                          //
// ........................................................ //
// ........................................................ //

impl Context {
    pub(crate) fn handle_mouse_move(&mut self, pos: impl Into<Vector2<f32>>) {
        if self.properties.len() <= 1 { return }
        self.cursor.hover.pos = pos.into();

        #[cfg(feature = "cursor_stats")] let start = std::time::Instant::now();
        self.detect_scope();

        if let Some(scope) = self.cursor.scope {
            self.detect_hovered_child(scope);
        } else {
            self.cursor.hover.prev = self.cursor.hover.curr.take();
        }
        #[cfg(feature = "cursor_stats")] eprintln!("{:?}", start.elapsed());

        self.handle_hover();
    }

    fn detect_scope(&mut self) {
        if let Some(current) = self.cursor.hover.curr.as_ref() {
            if let Some(scope) = self.cursor.scope.as_ref() {
                if self.tree.is_member_of(current, scope) { return }
            }
        }
        self.cursor.scope = self
            .tree
            .iter()
            .skip(1)
            .filter_map(|node| {
                if self.get_node_data(node.id()).is_hovered(self.cursor.hover.pos) {
                    Some(*node.id())
                } else {
                    None
                }
            }).max();
    }

    fn detect_hovered_child(&mut self, scope: NodeId) {
        let mut curr = scope;
        while let Some(children) = self.tree.get_all_children(&curr) {
            if let Some(hovered) = children.iter().find(|child| {
                self.get_node_data(child).is_hovered(self.cursor.hover.pos)
            }) {
                curr = *hovered;
            } else {
                break
            }
        }

        if self.cursor.click.obj.is_none() {
            self.cursor.hover.prev = self.cursor.hover.curr;
            self.cursor.hover.curr = Some(curr);
        }
    }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                     Event Handling                       //
//                                                          //
// ........................................................ //
// ........................................................ //

impl Context {
    pub(crate) fn handle_hover(&mut self) {
        if self.cursor.is_idling() || self.cursor.is_unscoped() { return }

        if let Some(prev_id) = self.cursor.hover.prev.take() {
            self.pending_update.push(UpdateMode::RevertColor(prev_id));
        }
        if let Some(hover_id) = self.cursor.hover.curr {
            self.pending_update.push(UpdateMode::HoverColor(hover_id));
            let dragable = self.get_node_data(&hover_id).is_dragable();
            if self.cursor.is_dragging(&hover_id) && dragable {
                self.handle_drag(&hover_id);
            }
        }
    }

    fn handle_drag(&mut self, hover_id: &NodeId) {
        let pos = self.cursor.hover.pos - self.cursor.click.offset;
        self.get_node_data_mut(hover_id).set_position(pos.into());
        self.recursive_layout(hover_id);
    }

    pub(crate) fn handle_click(&mut self, action: impl Into<MouseAction>, button: impl Into<MouseButton>) {
        self.cursor.set_click_state(action.into(), button.into());
        if let Some(click_id) = self.cursor.click.obj {
            if let Some(callback) = self.callbacks.get(&click_id) {
                callback();
            }
            let props = self.get_node_data(&click_id);
            self.cursor.click.offset = self.cursor.click.pos - Vector2::<f32>::from(props.pos());
            self.pending_update.push(UpdateMode::ClickColor(click_id));
        }
        if self.cursor.state.action == MouseAction::Released {
            if let Some(hover_id) = self.cursor.hover.curr {
                self.pending_update.push(UpdateMode::HoverColor(hover_id));
            }
        }
    }
}

// ........................................................ //
// ........................................................ //
//                                                          //
//                        Render                            //
//                                                          //
// ........................................................ //
// ........................................................ //

impl Context {
    pub(crate) fn has_changed(&self) -> bool {
        !self.pending_update.is_empty()
    }

    pub(crate) fn submit_update(&mut self, renderer: &mut Renderer) {
        self.pending_update.iter().for_each(|mode| {
            match mode {
                UpdateMode::HoverColor(node_id) => {
                    if let Some(color) = self.get_node_data(node_id).hover_color() {
                        renderer.update_element_color(node_id.index() - 1, color);
                    }
                },
                UpdateMode::ClickColor(node_id) => {
                    if let Some(color) = self.get_node_data(node_id).click_color() {
                        renderer.update_element_color(node_id.index() - 1, color);
                    }
                }
                UpdateMode::RevertColor(node_id) => {
                    let color = self.get_node_data(node_id).fill_color();
                    renderer.update_element_color(node_id.index() - 1, color);
                }
                UpdateMode::Transform(node_id) => {
                    let rect = self.get_node_data(node_id).rect();
                    renderer.update_element_transform(node_id.index() - 1, rect);
                }
                UpdateMode::Size(node_id) => {
                    let rect = self.get_node_data(node_id).rect();
                    renderer.update_element_size(node_id.index() - 1, rect.size());
                }
            }
        });
        self.pending_update.clear();
        renderer.write_data();
    }
}

impl Render for Context {
    fn render(&self, renderer: &mut Renderer) {
        self.tree.iter().skip(1).for_each(|node| {
            if let Some(image_fn) = self.image_fn.get(node.id()) {
                // if node.id().index() == 3 {
                //     let info = renderer.push_image(image_fn);
                //     let prop = self.get_node_data(node.id());
                //     renderer.add_component(prop, Some(info));
                // } else {
                //     let info = renderer.push_atlas(image_fn);
                //     let prop = self.get_node_data(node.id());
                //     renderer.add_component(prop, info);
                // }

                if let Some(info) = renderer.push_atlas(image_fn) {
                    let prop = self.get_node_data(node.id());
                    renderer.add_component(prop, Some(info));
                } else {
                    let info = renderer.push_image(image_fn);
                    let prop = self.get_node_data(node.id());
                    renderer.add_component(prop, Some(info));
                }
            } else {
                let prop = self.get_node_data(node.id());
                renderer.add_component(prop, None);
            }
        });
    }
}
