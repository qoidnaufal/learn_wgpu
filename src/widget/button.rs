use crate::{
    color::Rgb,
    shapes::{Shape, ShapeKind},
};
use super::{NodeId, Widget, CALLBACKS};

pub fn button() -> Button {
    Button::new()
}

#[derive(Debug, Clone, Copy)]
pub struct Button {
    id: NodeId,
}

impl Button {
    fn new() -> Self {
        let id = NodeId::new();
        Self { id }
    }

    fn id(&self) -> NodeId {
        self.id
    }

    fn shape(&self) -> Shape {
        Shape::filled(Rgb::RED, ShapeKind::FilledRectangle)
    }

    pub fn on_hover<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Shape) + 'static>(&self, f: F) -> &Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl Widget for Button {
    fn id(&self) -> NodeId {
        self.id()
    }

    fn shape(&self) -> Shape {
        self.shape()
    }
}

impl Widget for &Button {
    fn id(&self) -> NodeId {
        (*self).id()
    }

    fn shape(&self) -> Shape {
        (*self).shape()
    }
}
