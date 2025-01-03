use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::Window,
};
use std::cell::RefCell;

use crate::{
    error::Error,
    gpu::GpuResources,
    layout::{Button, Layout},
    renderer::GfxRenderer,
    types::{Size, Vector2},
};

thread_local! {
    pub static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub cursor: Cursor,
    pub window_size: Size<u32>,
}

impl Context {
    fn new() -> Self {
        Self {
            cursor: Cursor::new(),
            window_size: Size::new(0, 0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    pub action: winit::event::ElementState,
    pub button: winit::event::MouseButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub position: Vector2<f32>,
    pub state: MouseState,
    pub click: Vector2<f32>,
}

impl Cursor {
    fn new() -> Self {
        Self {
            position: Vector2::new(),
            state: MouseState {
                action: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Left,
            },
            click: Vector2::new(),
        }
    }

    pub fn set_state(&mut self,
        action: winit::event::ElementState,
        button: winit::event::MouseButton
    ) {
        self.state = MouseState { action, button };
    }
}

pub struct App<'a> {
    pub gfx: Option<GfxRenderer<'a>>,
    pub window: Option<Window>,
    pub layout: Layout,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            gfx: None,
            window: None,
            layout: Layout::new(),
        }
    }

    fn request_gpu(&self) -> Result<GpuResources, Error> {
        let gpu = GpuResources::request(self.window.as_ref().unwrap())?;
        gpu.configure();
        Ok(gpu)
    }

    fn request_redraw(&self) {
        self.window.as_ref().unwrap().request_redraw();
    }

    fn resize(&mut self) {
        self.gfx.as_mut().unwrap().resize();
    }

    fn id(&self) -> winit::window::WindowId {
        let gfx = self.gfx.as_ref().unwrap();
        gfx.gpu.id
    }

    fn update(&mut self) {
        let data = self.layout.vertices();
        self.gfx.as_mut().unwrap().update(&data);
    }

    fn render(&mut self) -> Result<(), Error> {
        self.gfx.as_mut().unwrap().render(self.layout.indices.len())
    }

    pub fn add_widget(&mut self, node: Button) -> &mut Self {
        self.layout.insert(node);
        self
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        window.set_title("My App");
        CONTEXT.with_borrow_mut(|ctx| ctx.window_size = Size::from(window.inner_size()));
        self.window = Some(window);

        self.layout.calculate();

        let gpu = self.request_gpu().unwrap();
        let gfx = GfxRenderer::new(gpu, &self.layout);
        let gfx: GfxRenderer<'a> = unsafe { std::mem::transmute(gfx) };
        self.gfx = Some(gfx);
    }

    fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {

        if self.id() == window_id {
            match event {
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    self.update();

                    match self.render() {
                        Ok(_) => {},
                        Err(Error::SurfaceRendering(surface_err)) => {
                            match surface_err {
                                wgpu::SurfaceError::Outdated
                                | wgpu::SurfaceError::Lost => self.resize(),
                                wgpu::SurfaceError::OutOfMemory => {
                                    log::error!("Out of Memory");
                                    event_loop.exit();
                                },
                                wgpu::SurfaceError::Timeout => {
                                    log::warn!("Surface Timeout")
                                },
                            }
                        }
                        Err(_) => panic!()
                    }
                }
                WindowEvent::Resized(new_size) => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.window_size = Size::from(new_size));
                    self.resize();
                }
                WindowEvent::MouseInput { state: action, button, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.set_state(action, button));

                    let initial = self.layout.vertices();
                    self.layout.handle_click();
                    if initial != self.layout.vertices() {
                        self.request_redraw();
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    CONTEXT.with_borrow_mut(|ctx| ctx.cursor.position = Vector2::from(position.cast()));

                    let initial = self.layout.vertices();
                    self.layout.set_position();
                    if initial != self.layout.vertices() {
                        self.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }
}

