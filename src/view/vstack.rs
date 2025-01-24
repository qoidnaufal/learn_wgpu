use math::Size;

use crate::callback::CALLBACKS;
use crate::Rgb;
use crate::shapes::{Shape, ShapeKind};

use super::{AnyView, IntoView, NodeId, View};

pub fn vstack(child_nodes: impl IntoIterator<Item = impl IntoView>) -> VStack {
    VStack::new(child_nodes)
}

pub struct VStack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
}

impl VStack {
    fn new(child_nodes: impl IntoIterator<Item = impl IntoView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().map(IntoView::into_any).collect();
        Self { id, children }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        let mut size = Size::new(0, 0);
        if !self.children.is_empty() {
            self.children.iter().for_each(|c| {
                let shape = c.shape();
                size.width += shape.dimensions.width;
                size.height = size.height.max(shape.dimensions.height);
            });
        } else {
            size = (1, 1).into();
        }
        Shape::filled(Rgb::BLACK, ShapeKind::FilledRectangle, size)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for VStack {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn children(&self) -> Option<&[AnyView]> {
        Some(&self.children)
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V {
        self
    }
}
