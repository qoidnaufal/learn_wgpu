use crate::callback::CALLBACKS;
use crate::layout::{Attributes, LayoutCtx};
use crate::color::{Pixel, Rgba};
use crate::element::Element;
use crate::style::{Style, Shape};

use super::{AnyView, IntoView, NodeId, View};

pub fn stack(child_nodes: impl IntoIterator<Item = AnyView>) -> Stack {
    Stack::new(child_nodes)
}

pub struct Stack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
    style: Style,
}

impl Stack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        let style = Style::new(Rgba::DARK_GRAY, (1, 1), Shape::Rect);
        Self { id, children, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
    }

    pub fn on_hover<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
        self
    }

    pub fn on_click<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
        self
    }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for Stack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn element(&self) -> Element { Element::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx) -> Attributes {
        let style = self.style();
        cx.insert_orientation(self.id, style.orientation());
        cx.insert_spacing(self.id, style.spacing());
        cx.insert_padding(self.id, style.padding());
        
        cx.set_orientation(&self.id);
        cx.set_spacing(&self.id);
        cx.set_padding(&self.id);
        cx.assign_position(&self.id)
    }

    fn style(&self) -> Style { self.style }
}

impl IntoView for Stack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
