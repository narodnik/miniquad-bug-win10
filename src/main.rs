use async_channel::{Receiver, Sender};
use log::{debug, error, info};
use miniquad::{
    conf, window, BufferSource, BufferType, BufferUsage, EventHandler, RenderingBackend,
};
use smol::Task;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc, Arc, Mutex as SyncMutex,
    },
    thread,
};

pub type ExecutorPtr = Arc<smol::Executor<'static>>;

pub type AppPtr = Arc<App>;

pub struct App {
    pub render_api: RenderApiPtr,
    pub ex: ExecutorPtr,

    _signal: async_channel::Sender<()>,
    _thread: thread::JoinHandle<()>,
    tasks: SyncMutex<Vec<Task<()>>>,

    mesh1: SyncMutex<Option<MeshInfo>>,
    mesh2: SyncMutex<Option<MeshInfo>>,
    mesh3: SyncMutex<MeshInfo>,
}

impl App {
    pub fn new(render_api: RenderApiPtr, ex: ExecutorPtr) -> Arc<Self> {
        let (_signal, shutdown) = async_channel::unbounded::<()>();
        let ex2 = ex.clone();
        let _thread = thread::spawn(move || {
            if let Err(e) = smol::future::block_on(ex2.run(shutdown.recv())) {
                error!("smol exec: {e}");
            }
        });

        let mesh3 = Self::regen_mesh3(&render_api);

        Arc::new(Self {
            ex,
            render_api,
            _signal,
            _thread,
            tasks: SyncMutex::new(vec![]),
            mesh1: SyncMutex::new(None),
            mesh2: SyncMutex::new(None),
            mesh3: SyncMutex::new(mesh3),
        })
    }

    pub fn setup(self: Arc<Self>, resize_recvr: Receiver<()>) {
        self.tasks.lock().unwrap().push(self.ex.spawn(self.clone().start(resize_recvr)));
    }

    pub async fn start(self: Arc<Self>, resize_recvr: Receiver<()>) {
        debug!(target: "app", "App::start()");

        let me = Arc::downgrade(&self);
        let resize_task = self.ex.spawn(async move {
            loop {
                let Ok(()) = resize_recvr.recv().await else {
                    debug!(target: "ui::win", "Event relayer closed");
                    break
                };

                let Some(self_) = me.upgrade() else {
                    // Should not happen
                    panic!("self destroyed before modify_task was stopped!");
                };

                debug!(target: "ui::win", "Received window resize event");
                self_.draw().await;
            }
        });
        self.tasks.lock().unwrap().push(resize_task);

        std::thread::sleep(std::time::Duration::from_millis(2000));
        debug!(target: "app", "Sleeping 2000 ms...");

        self.draw().await;
    }

    pub async fn draw(&self) {
        debug!(target: "ui::win", "Window::draw()");

        let mut freed_buffers = vec![];

        let mesh3 = Self::regen_mesh3(&self.render_api);
        let old_mesh = std::mem::replace(&mut *self.mesh3.lock().unwrap(), mesh3.clone());
        freed_buffers.push(old_mesh.vertex_buffer);
        freed_buffers.push(old_mesh.index_buffer);

        let mesh2 = self.regen_mesh2();
        let old_mesh = std::mem::replace(&mut *self.mesh2.lock().unwrap(), Some(mesh2.clone()));
        if let Some(old) = old_mesh {
            freed_buffers.push(old.vertex_buffer);
            freed_buffers.push(old.index_buffer);
        }

        let mesh1 = self.regen_mesh1();
        let old_mesh = std::mem::replace(&mut *self.mesh1.lock().unwrap(), Some(mesh1.clone()));
        if let Some(old) = old_mesh {
            freed_buffers.push(old.vertex_buffer);
            freed_buffers.push(old.index_buffer);
        }

        for buff in freed_buffers {
            self.render_api.delete_buffer(buff);
        }

        debug!(target: "ui::win", "Window::draw() - replaced draw call");
    }

    fn regen_mesh1(&self) -> MeshInfo {
        let verts = vec![
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
        ];
        let indices = vec![0, 2, 1, 1, 2, 3];

        let num_elements = indices.len() as i32;
        let vertex_buffer = self.render_api.new_vertex_buffer(verts);
        let index_buffer = self.render_api.new_index_buffer(indices);

        MeshInfo { vertex_buffer, index_buffer, num_elements }
    }

    fn regen_mesh2(&self) -> MeshInfo {
        let verts = vec![
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
        ];
        let indices = vec![0, 2, 1, 1, 2, 3, 4, 6, 5, 5, 6, 7];

        let num_elements = indices.len() as i32;
        let vertex_buffer = self.render_api.new_vertex_buffer(verts);
        let index_buffer = self.render_api.new_index_buffer(indices);

        MeshInfo { vertex_buffer, index_buffer, num_elements }
    }

    fn regen_mesh3(render_api: &RenderApi) -> MeshInfo {
        let verts = vec![
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { pos: [0.0, 0.0], color: [0.0, 0.0, 0.0, 0.0], uv: [0.0, 0.0] },
        ];
        let indices = vec![
            0, 2, 1, 1, 2, 3, 4, 6, 5, 5, 6, 7, 8, 10, 9, 9, 10, 11, 12, 14, 13, 13, 14, 15, 16,
            18, 17, 17, 18, 19, 20, 22, 21, 21, 22, 23,
        ];

        let num_elements = indices.len() as i32;
        let vertex_buffer = render_api.new_vertex_buffer(verts);
        let index_buffer = render_api.new_index_buffer(indices);

        std::thread::sleep(std::time::Duration::from_micros(900));
        MeshInfo { vertex_buffer, index_buffer, num_elements }
    }
}

pub type GfxTextureId = u32;
pub type GfxBufferId = u32;

#[derive(Clone, Debug)]
pub struct MeshInfo {
    pub vertex_buffer: GfxBufferId,
    pub index_buffer: GfxBufferId,
    pub num_elements: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

static BUFFER_ID: AtomicU32 = AtomicU32::new(0);

pub type RenderApiPtr = Arc<RenderApi>;

pub struct RenderApi {
    method_req: mpsc::Sender<GraphicsMethod>,
}

impl RenderApi {
    pub fn new(method_req: mpsc::Sender<GraphicsMethod>) -> Arc<Self> {
        Arc::new(Self { method_req })
    }

    pub fn new_vertex_buffer(&self, verts: Vec<Vertex>) -> GfxBufferId {
        let gfx_buffer_id = BUFFER_ID.fetch_add(1, Ordering::SeqCst);
        //debug!(target: "gfx", "Req method: new_vertex_buffer(...{}, {gfx_buffer_id})", verts.len());
        assert_eq!(verts.len() % 4, 0);

        let method = GraphicsMethod::NewVertexBuffer((verts, gfx_buffer_id));
        let _ = self.method_req.send(method);

        gfx_buffer_id
    }

    pub fn new_index_buffer(&self, indices: Vec<u16>) -> GfxBufferId {
        let gfx_buffer_id = BUFFER_ID.fetch_add(1, Ordering::SeqCst);
        //debug!(target: "gfx", "Req method: new_index_buffer(...{}, {gfx_buffer_id})", indices.len());
        assert_eq!(indices.len() % 6, 0);

        let method = GraphicsMethod::NewIndexBuffer((indices, gfx_buffer_id));
        let _ = self.method_req.send(method);

        gfx_buffer_id
    }

    pub fn delete_buffer(&self, buffer: GfxBufferId) {
        //debug!(target: "gfx", "Req method: delete_buffer({buffer})");
        let method = GraphicsMethod::DeleteBuffer(buffer);
        let _ = self.method_req.send(method);
    }
}

#[derive(Clone, Debug)]
pub enum GraphicsMethod {
    NewVertexBuffer((Vec<Vertex>, GfxBufferId)),
    NewIndexBuffer((Vec<u16>, GfxBufferId)),
    DeleteBuffer(GfxBufferId),
}

struct Stage {
    #[allow(dead_code)]
    app: AppPtr,

    ctx: Box<dyn RenderingBackend>,
    buffers: HashMap<GfxBufferId, miniquad::BufferId>,

    method_rep: mpsc::Receiver<GraphicsMethod>,
    resize_sendr: Sender<()>,
}

impl Stage {
    pub fn new(
        app: AppPtr,
        method_rep: mpsc::Receiver<GraphicsMethod>,
        resize_sendr: Sender<()>,
    ) -> Self {
        let ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        Stage { app, ctx, buffers: HashMap::new(), method_rep, resize_sendr }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        //// Process as many methods as we can
        while let Ok(method) = self.method_rep.try_recv() {
            match method {
                GraphicsMethod::NewVertexBuffer((verts, gfx_buffer_id)) => {
                    let buffer = self.ctx.new_buffer(
                        BufferType::VertexBuffer,
                        BufferUsage::Immutable,
                        BufferSource::slice(&verts),
                    );
                    debug!(target: "gfx", "Invoked method: new_vertex_buffer(..., {gfx_buffer_id}) -> {buffer:?}");
                    self.buffers.insert(gfx_buffer_id, buffer);
                }
                GraphicsMethod::NewIndexBuffer((indices, gfx_buffer_id)) => {
                    let buffer = self.ctx.new_buffer(
                        BufferType::IndexBuffer,
                        BufferUsage::Immutable,
                        BufferSource::slice(&indices),
                    );
                    debug!(target: "gfx", "Invoked method: new_index_buffer(..., {gfx_buffer_id}) -> {buffer:?}");
                    self.buffers.insert(gfx_buffer_id, buffer);
                }
                GraphicsMethod::DeleteBuffer(gfx_buffer_id) => {
                    let buffer =
                        self.buffers.remove(&gfx_buffer_id).expect("couldn't find gfx_buffer_id");
                    debug!(target: "gfx", "Invoked method: delete_buffer({gfx_buffer_id} = {buffer:?})");
                    self.ctx.delete_buffer(buffer);
                }
            };
        }
    }

    fn draw(&mut self) {}

    fn resize_event(&mut self, _: f32, _: f32) {
        self.resize_sendr.try_send(()).unwrap();
        debug!("Resize triggered a draw event");
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )])
    .expect("logger");

    let ex = Arc::new(smol::Executor::new());

    let (method_req, method_rep) = mpsc::channel();
    // The UI actually needs to be running for this to reply back.
    // Otherwise calls will just hang.
    let render_api = RenderApi::new(method_req);

    let (resize_sendr, resize_recvr) = async_channel::unbounded();
    let app = App::new(render_api, ex.clone());
    app.clone().setup(resize_recvr);

    let mut conf = miniquad::conf::Conf {
        high_dpi: true,
        window_resizable: true,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            wayland_use_fallback_decorations: false,
            //blocking_event_loop: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let metal = std::env::args().nth(1).as_deref() == Some("metal");
    conf.platform.apple_gfx_api =
        if metal { conf::AppleGfxApi::Metal } else { conf::AppleGfxApi::OpenGl };

    miniquad::start(conf, move || Box::new(Stage::new(app, method_rep, resize_sendr)));

    debug!(target: "main", "Started GFX backend");
}
