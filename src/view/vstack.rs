use crate::callback::CALLBACKS;
use crate::layout::{Orientation, LayoutCtx};
use crate::{Pixel, Rgba};
use crate::element::{Attributes, Element, Shape, Style};

use super::{AnyView, IntoView, NodeId, View};

pub fn vstack(child_nodes: impl IntoIterator<Item = AnyView>) -> VStack {
    VStack::new(child_nodes)
}

pub struct VStack {
    id: NodeId,
    children: Vec<Box<dyn View>>,
    style: Style,
}

impl VStack {
    fn new(child_nodes: impl IntoIterator<Item = AnyView>) -> Self {
        let id = NodeId::new();
        let children = child_nodes.into_iter().collect();
        let style = Style::new(Rgba::DARK_GRAY, (0, 0), Shape::Rect);
        Self { id, children, style }
    }

    pub fn style<F: FnMut(&mut Style)>(mut self, mut f: F) -> Self {
        f(&mut self.style);
        self
    }

    // pub fn on_hover<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_hover.insert(self.id(), f.into()));
    //     self
    // }

    // pub fn on_click<F: FnMut(&mut Shape) + 'static>(self, f: F) -> Self {
    //     CALLBACKS.with_borrow_mut(|cbs| cbs.on_click.insert(self.id(), f.into()));
    //     self
    // }

    pub fn on_drag<F: FnMut(&mut Element) + 'static>(self, f: F) -> Self {
        CALLBACKS.with_borrow_mut(|cbs| cbs.on_drag.insert(self.id(), f.into()));
        self
    }
}

impl View for VStack {
    fn id(&self) -> NodeId { self.id }

    fn children(&self) -> Option<&[AnyView]> { Some(&self.children) }

    fn element(&self) -> Element { Element::filled(&self.style) }

    fn pixel(&self) -> Option<&Pixel<Rgba<u8>>> { None }

    fn layout(&self, cx: &mut LayoutCtx, attr: &mut Attributes) {
        cx.align_vertically();
        cx.assign_position(attr);
    }

    fn attributes(&self) -> Attributes {
        let mut size = self.style.get_dimensions();
        if !self.children.is_empty() {
            self.children.iter().for_each(|child| {
                let child_attr = child.attributes();
                let child_size = child_attr.dims;
                size.height += child_size.height;
                size.width = size.width.max(child_size.width + self.padding() * 2);
            });
            let child_len = self.children.len() as u32;
            size.height += self.padding() * 2 + self.spacing() * (child_len - 1);
        } else { size = (1, 1).into() }
        Attributes::new(size)
    }

    fn padding(&self) -> u32 { 20 }

    fn spacing(&self) -> u32 { 20 }

    fn orientation(&self) -> Orientation { Orientation::Vertical }
}

impl IntoView for VStack {
    type V = Self;
    fn into_view(self) -> Self::V { self }
}
