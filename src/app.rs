use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;
use winit::application::ApplicationHandler;
use util::{Size, Vector2};

use crate::context::Cursor;
use crate::renderer::Renderer;
use crate::storage::WidgetStorage;
use crate::error::Error;
use crate::IntoView;

pub fn launch<F, IV>(f: F) -> Result<(), Error>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    let event_loop = EventLoop::new()?;
    let mut app = App::new(f);
    event_loop.run_app(&mut app)?;

    Ok(())
}

struct Stats {
    counter: u32,
    time: std::time::Duration,
}

impl Stats {
    fn new() -> Self {
        Self {
            counter: 0,
            time: std::time::Duration::from_nanos(0),
        }
    }

    fn inc(&mut self, d: std::time::Duration) {
        self.time += d;
        self.counter += 1;
    }
}

impl Drop for Stats {
    fn drop(&mut self) {
        let avg = self.time / self.counter;
        eprintln!("average update time: {avg:?}");
    }
}

struct App<F> {
    renderer: MaybeUninit<Renderer>,
    storage: WidgetStorage,
    cursor: Cursor,
    window: HashMap<WindowId, Arc<Window>>,
    stats: Stats,
    view_fn: Option<F>,
}

impl<F, IV> App<F>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    pub fn new(view_fn: F) -> Self {
        Self {
            renderer: MaybeUninit::uninit(),
            window: HashMap::new(),
            storage: WidgetStorage::new(),
            cursor: Cursor::new(),
            stats: Stats::new(),
            view_fn: Some(view_fn),
        }
    }

    fn request_redraw(&self, window_id: winit::window::WindowId) {
        if let Some(window) = self.window.get(&window_id) {
            window.request_redraw();
        }
    }

    fn resize(&mut self, size: Size<u32>) {
        unsafe { self.renderer.assume_init_mut().resize(size) }
    }

    fn update(&mut self) {
        let renderer = unsafe { self.renderer.assume_init_mut() };
        self.storage.submit_update(renderer);
    }

    fn render(&mut self) -> Result<(), Error> {
        unsafe { self.renderer.assume_init_mut().render() }
    }
}

fn create_window(event_loop: &winit::event_loop::ActiveEventLoop) -> Arc<Window> {
    let window = event_loop.create_window(Default::default()).unwrap();
    window.set_title("My App");
    Arc::new(window)
}

impl<F, IV> ApplicationHandler for App<F>
where
    F: Fn() -> IV + 'static,
    IV: IntoView + 'static,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(view_fn) = self.view_fn.take() {
            let window = create_window(event_loop);
            let renderer = Renderer::new(window.clone(), &mut self.storage, view_fn);
            self.window.insert(window.id(), window);
            self.renderer.write(renderer);
        }
        // eprintln!("{:?}", self.storage.nodes);
        // self.storage.children.iter().for_each(|(node_id, vec)| {
        //     eprintln!("{node_id:?} | {vec:?}")
        // });
        // eprintln!("{:#?}", unsafe { &self.renderer.assume_init_ref().gfx.shapes.data });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let size = unsafe { self.renderer.assume_init_mut().gpu.size() };
        match event {
            WindowEvent::CloseRequested => {
                eprintln!();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let start = std::time::Instant::now();
                self.update();
                let elapsed = start.elapsed();
                self.stats.inc(elapsed);

                match self.render() {
                    Ok(_) => {},
                    Err(Error::SurfaceRendering(surface_err)) => {
                        match surface_err {
                            wgpu::SurfaceError::Outdated
                            | wgpu::SurfaceError::Lost => {
                                eprintln!("surface lost / outdated");
                                self.resize(size);
                            },
                            wgpu::SurfaceError::OutOfMemory
                            | wgpu::SurfaceError::Other => {
                                eprintln!("Out of Memory / other error");
                                event_loop.exit();
                            },
                            wgpu::SurfaceError::Timeout => {
                                eprintln!("Surface Timeout")
                            },
                        }
                    }
                    Err(_) => panic!()
                }
            }
            WindowEvent::Resized(new_size) => {
                self.resize(Size::new(new_size.width, new_size.height));
            }
            WindowEvent::MouseInput { state: action, button, .. } => {
                let gfx = unsafe { &mut self.renderer.assume_init_mut().gfx };
                self.cursor.set_click_state(action.into(), button.into());
                self.storage.handle_click(&mut self.cursor, gfx);
                if self.storage.has_changed() {
                    self.request_redraw(window_id);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let gfx = unsafe { &mut self.renderer.assume_init_mut().gfx };
                self.cursor.hover.pos = Vector2::new(position.x as _, position.y as _);
                self.storage.detect_hover(&mut self.cursor, gfx);
                self.storage.handle_hover(&mut self.cursor, gfx);
                if self.storage.has_changed() {
                    self.request_redraw(window_id);
                }
            }
            _ => {}
        }
    }
}

