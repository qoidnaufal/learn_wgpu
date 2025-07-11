use std::collections::HashMap;
use std::sync::Arc;

use aplite_reactive::{Effect, Get};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::application::ApplicationHandler;

use aplite_types::{Size, Rgba};
use aplite_renderer::{Renderer, RendererError};

use crate::prelude::ApliteResult;
use crate::context::Context;
use crate::error::ApliteError;
use crate::view::{IntoView, View, ViewId, VIEW_STORAGE};

#[derive(Debug)]
enum WinitSize {
    Logical(Size<u32>),
    Physical(PhysicalSize<u32>),
}

pub(crate) const DEFAULT_SCREEN_SIZE: Size<u32> = Size::new(800, 600);

pub struct WindowAttributes {
    title: &'static str,
    inner_size: Size<u32>,
    decorations: bool,
    transparent: bool,
    maximized: bool,
    resizable: bool,
}

impl Default for WindowAttributes {
    fn default() -> Self {
        Self {
            title: "GUI App",
            inner_size: DEFAULT_SCREEN_SIZE,
            decorations: true,
            transparent: false,
            maximized: false,
            resizable: true,
        }
    }
}

impl From<&WindowAttributes> for winit::window::WindowAttributes {
    fn from(w: &WindowAttributes) -> Self {
        Self::default()
            .with_inner_size::<winit::dpi::LogicalSize<u32>>(w.inner_size.into())
            .with_title(w.title)
            .with_decorations(w.decorations)
            .with_transparent(w.transparent)
            .with_maximized(w.maximized)
            .with_resizable(w.resizable)
    }
}

pub struct Aplite {
    renderer: Option<Renderer>,
    cx: Context,
    window: HashMap<WindowId, (ViewId, Arc<Window>)>,
    window_attributes: WindowAttributes,
    views: Vec<Box<dyn FnOnce(WindowId) -> Box<dyn IntoView>>>,

    #[cfg(feature = "render_stats")]
    stats: aplite_stats::Stats,
}

// user API
impl Aplite {
    pub fn new<IV: IntoView + 'static>(view_fn: impl FnOnce() -> IV + 'static) -> Self {
        let mut app = Self::new_empty();
        app.views.push(Box::new(|_| Box::new(view_fn())));
        app
    }

    pub fn new_empty() -> Self {
        Self {
            renderer: None,
            cx: Context::new(),
            window: HashMap::with_capacity(4),
            window_attributes: WindowAttributes::default(),
            views: Vec::with_capacity(4),

            #[cfg(feature = "render_stats")]
            stats: aplite_stats::Stats::new(),
        }
    }

    pub fn launch(mut self) -> ApliteResult {
        let event_loop = EventLoop::new()?;
        event_loop.run_app(&mut self)?;

        Ok(())
    }

    pub fn with_title(mut self, title: &'static str) -> Self {
        self.window_attributes.title = title;
        self
    }

    pub fn with_inner_size(mut self, width: u32, height: u32) -> Self {
        self.window_attributes.inner_size = (width, height).into();
        self
    }

    pub fn with_decorations_enabled(mut self, val: bool) -> Self {
        self.window_attributes.decorations = val;
        self
    }

    pub fn with_transparent(mut self, val: bool) -> Self {
        self.window_attributes.transparent = val;
        self
    }

    pub fn with_maximized(mut self, val: bool) -> Self {
        self.window_attributes.maximized = val;
        self
    }

    pub fn with_resizable(mut self, val: bool) -> Self {
        self.window_attributes.resizable = val;
        self
    }

    pub fn with_background_color(self, color: Rgba<u8>) -> Self {
        let _ = color;
        self
    }
}

// initialization
impl Aplite {
    fn initialize_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<(ViewId, Arc<Window>), ApliteError> {
        let attributes = &self.window_attributes;
        let window = event_loop.create_window(attributes.into())?;
        let window_id = window.id();

        let view_id = VIEW_STORAGE.with(|s| {
            let size = window
                .inner_size()
                .to_logical(window.scale_factor());

            let root = s.create_entity();
            let root_view = View::window(size.into());

            s.storage.borrow_mut().insert(root, root_view);
            self.cx.root_window.insert(root, window_id);

            if let Some(view_fn) = self.views.pop() {
                let view = view_fn(window_id);
                s.append_child(&root, view);

                self.cx.layout_the_whole_window(&root);

                #[cfg(feature = "debug_tree")] eprintln!("{:?}", s.tree.borrow());
            }

            root
        });

        Ok((view_id, Arc::new(window)))
    }

    fn initialize_renderer(&mut self, window: Arc<Window>) -> Result<(), ApliteError> {
        let renderer = Renderer::new(Arc::clone(&window))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn add_window(&mut self, view_id: ViewId, window: Arc<Window>) {
        let window_id = window.id();
        self.window.insert(window_id, (view_id, Arc::clone(&window)));

        let dirty = self.cx.dirty();
        Effect::new(move |_| {
            // FIXME: this should coresponds to root_id & window_id
            if dirty.get() { window.request_redraw() }
        });
    }
}

// window event
impl Aplite {
    fn handle_resize(&mut self, winit_size: WinitSize) {
        if let Some(renderer) = self.renderer.as_mut() {
            let size = match winit_size {
                WinitSize::Logical(size) => size,
                WinitSize::Physical(size) => {
                    let logical = size.to_logical::<u32>(renderer.scale_factor());
                    (logical.width, logical.height).into()
                },
            };
            renderer.resize(size);
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.set_scale_factor(scale_factor);
        }
    }

    fn handle_redraw_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some((_, window)) = self.window.get(window_id).cloned() {
            // FIXME: not sure if retained mode works like this
            self.submit_update(&window_id);

            #[cfg(feature = "render_stats")] let start = std::time::Instant::now();

            self.render(event_loop, window);

            #[cfg(feature = "render_stats")] self.stats.inc(start.elapsed())
        }
    }

    fn submit_update(&mut self, window_id: &WindowId) {
        if let Some(renderer) = self.renderer.as_mut() {
            let (root_id, _) = self.window.get(window_id).unwrap();
            if self.cx.dirty().get_untracked() {
                renderer.begin();
                self.cx.prepare_data(*root_id, renderer);
            }
            renderer.finish();
            self.cx.toggle_clean();
        }
    }

    fn render(&mut self, event_loop: &ActiveEventLoop, window: Arc<Window>) {
        if let Some(renderer) = self.renderer.as_mut() {
            if let Err(err) = renderer.render(Rgba::TRANSPARENT, window) {
                let size = renderer.screen_size().u32();
                match err {
                    RendererError::ShouldResize => self.handle_resize(WinitSize::Logical(size)),
                    RendererError::ShouldExit => event_loop.exit(),
                    _ => {}
                }
            }
        }
    }

    fn handle_close_request(&mut self, window_id: &WindowId, event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.remove(window_id) {
            drop(window);
            event_loop.exit();
        }
    }

    fn handle_click(&mut self, state: ElementState, button: MouseButton) {
        self.cx.handle_click(state, button);
    }

    fn handle_mouse_move(&mut self, window_id: &WindowId, pos: PhysicalPosition<f64>) {
        if let Some(renderer) = self.renderer.as_mut()
            && let Some((root, _)) = self.window.get(window_id) {
            let logical_pos = pos.to_logical::<f32>(renderer.scale_factor());
            self.cx.handle_mouse_move(root, (logical_pos.x, logical_pos.y));
        }
    }
}

impl ApplicationHandler for Aplite {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.initialize_window(event_loop) {
            Ok((view_id, window)) if self.initialize_renderer(Arc::clone(&window))
                .is_ok() => self.add_window(view_id, window),
            _ => event_loop.exit(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.handle_close_request(&window_id, event_loop),
            WindowEvent::RedrawRequested => self.handle_redraw_request(&window_id, event_loop),
            WindowEvent::Resized(s) => self.handle_resize(WinitSize::Physical(s)),
            WindowEvent::MouseInput { state, button, .. } => self.handle_click(state, button),
            WindowEvent::CursorMoved { position, .. } => self.handle_mouse_move(&window_id, position),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.set_scale_factor(scale_factor),
            _ => {}
        }
    }
}

