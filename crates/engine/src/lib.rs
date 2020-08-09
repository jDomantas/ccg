mod renderer;

pub use ggez;
use ggez::{Context, GameResult, event, graphics, timer};
pub use crate::renderer::{FrameRenderer, Icon, Renderer, Texture, Textures};

pub type Error = ggez::GameError;
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

pub const SCREEN_WIDTH: f32 = 1600.0;
pub const SCREEN_HEIGHT: f32 = 900.0;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum WindowMode {
    Windowed,
    Borderless,
}

struct CtxData {
    window_size: (f32, f32),
    physical_window_size: (f32, f32),
    mode: WindowMode,
    old_mouse_press: bool,
    current_mouse_press: bool,
    scroll_position: (f32, f32),
}

pub struct Ctx<'a> {
    ggez: &'a mut Context,
    data: &'a mut CtxData,
}

impl<'a> Ctx<'a> {
    pub fn is_mouse_click(&self) -> bool {
        !self.data.old_mouse_press && self.data.current_mouse_press
    }
    
    pub fn is_mouse_pressed(&self) -> bool {
        self.data.current_mouse_press
    }

    pub fn ggez(&mut self) -> &mut Context {
        self.ggez
    }

    fn update_mode(&mut self) -> Result {
        let window = ggez::graphics::window(self.ggez);
        let hidpi = window.get_hidpi_factor();
        let monitor = window.get_current_monitor();
        let position = monitor.get_position();
        let monitor_size = monitor.get_dimensions();
        match self.data.mode {
            WindowMode::Windowed => {
                self.data.physical_window_size = self.data.window_size;
                window.set_fullscreen(None);
                window.set_decorations(true);
                window.set_inner_size(winit::dpi::LogicalSize {
                    width: f64::from(self.data.window_size.0),
                    height: f64::from(self.data.window_size.1),
                });
                let outer_size = window
                    .get_outer_size()
                    .map(|s| s.to_physical(hidpi))
                    .unwrap_or(monitor_size);
                let position = winit::dpi::PhysicalPosition {
                    x: (monitor_size.width - outer_size.width) / 2.0,
                    y: (monitor_size.height - outer_size.height) / 2.0,
                }.to_logical(hidpi);
                dbg!(monitor_size, outer_size, position);
                window.set_position(position);
            }
            WindowMode::Borderless => {
                let dimensions = monitor.get_dimensions();
                self.data.physical_window_size = (dimensions.width as f32, dimensions.height as f32);
                window.set_fullscreen(None);
                window.set_decorations(false);
                window.set_inner_size(dimensions.to_logical(hidpi));
                window.set_position(position.to_logical(hidpi));
                // window.set_always_on_top(true);
            }
        }
        // let inner_size = window.get_inner_size().map(|s| (s.width as f32, s.height as f32)).unwrap_or(self.data.physical_window_size);
        // self.data.physical_window_size = inner_size;
        // println!("size: {:?}", inner_size);
        Ok(())
        // let fullscreen_type = match self.data.mode {
        //     WindowMode::Windowed => ggez::conf::FullscreenType::Windowed,
        //     WindowMode::Borderless => ggez::conf::FullscreenType::Desktop,
        // };
        // ggez::graphics::set_mode(self.ggez, ggez::conf::WindowMode {
        //     width: self.data.window_size.0,
        //     height: self.data.window_size.1,
        //     fullscreen_type,
        //     .. Default::default()
        // })?;
        // let window = ggez::graphics::window(self.ggez);
        // let size = window
        //     .get_inner_size()
        //     .expect("no window size")
        //     .to_physical(window.get_hidpi_factor());
        // self.data.physical_window_size = (size.width as f32, size.height as f32);
        // println!("set: {:?}, {:?}, physical size: {:?}", self.data.window_size, fullscreen_type, self.data.physical_window_size);
        // Ok(())
    }

    pub fn set_window_size(&mut self, width: f32, height: f32) -> Result {
        self.data.window_size = (width, height);
        self.update_mode()
    }

    pub fn window_size(&self) -> (f32, f32) {
        self.data.window_size
    }

    pub fn set_window_mode(&mut self, mode: WindowMode) -> Result {
        self.data.mode = mode;
        self.update_mode()
    }

    pub fn window_mode(&mut self) -> WindowMode {
        self.data.mode
    }

    pub fn mouse_position(&self) -> (f32, f32) {
        let mouse = ggez::input::mouse::position(self.ggez);
        let x = mouse.x / self.data.physical_window_size.0 * SCREEN_WIDTH;
        let y = mouse.y / self.data.physical_window_size.1 * SCREEN_HEIGHT;
        (x, y)
    }

    pub fn round_to_screen(&self, point: (f32, f32)) -> (f32, f32) {
        let scale_w = self.data.window_size.0 / SCREEN_WIDTH;
        let scale_h = self.data.window_size.1 / SCREEN_HEIGHT;
        let x = (point.0 * scale_w).round() / scale_w;
        let y = (point.1 * scale_h).round() / scale_h;
        (x, y)
    }

    pub fn scroll_position(&self) -> (f32, f32) {
        self.data.scroll_position
    }
}

pub trait Game {
    fn update(&mut self, ctx: &mut Ctx<'_>) -> Result;
    fn draw(&mut self, ctx: &mut Ctx<'_>) -> Result;
}

struct GameRunner {
    game: Box<dyn Game>,
    position_set: bool,
    ctx_data: CtxData,
}

const DEFAULT_PHYSICAL_WIDTH: f32 = 1600.0;
const DEFAULT_PHYSICAL_HEIGHT: f32 = 900.0;

impl GameRunner {
    fn new(game: Box<dyn Game>) -> Self {
        GameRunner {
            game,
            position_set: false,
            ctx_data: CtxData {
                window_size: (DEFAULT_PHYSICAL_WIDTH, DEFAULT_PHYSICAL_HEIGHT),
                physical_window_size: (DEFAULT_PHYSICAL_WIDTH, DEFAULT_PHYSICAL_HEIGHT),
                mode: WindowMode::Windowed,
                old_mouse_press: false,
                current_mouse_press: false,
                scroll_position: (0.0, 0.0),
            },
        }
    }
}

impl event::EventHandler for GameRunner {
    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.ctx_data.scroll_position.0 -= x;
        self.ctx_data.scroll_position.1 -= y;
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.position_set {
            self.position_set = true;
            let window = ggez::graphics::window(ctx);
            window.set_position(winit::dpi::LogicalPosition {
                x: 100.0,
                y: 50.0,
            });
        }
        if timer::ticks(ctx) % 180 == 0 {
            println!(
                "delta frame time: {:?}, average fps: {}",
                timer::delta(ctx),
                timer::fps(ctx),
            );
        }
        self.ctx_data.old_mouse_press = self.ctx_data.current_mouse_press;
        self.ctx_data.current_mouse_press = ggez::input::mouse::button_pressed(
            ctx,
            ggez::input::mouse::MouseButton::Left,
        );
        self.game.update(&mut Ctx { ggez: ctx, data: &mut self.ctx_data })
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        self.game.draw(&mut Ctx { ggez: ctx, data: &mut self.ctx_data })?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn run(create_game: &dyn Fn(&mut Context) -> Result<Box<dyn Game>>) -> Result {
    let resource_dir = std::path::PathBuf::from("./resources");

    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("ccg", "domantas")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Some sort of ccg"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(DEFAULT_PHYSICAL_WIDTH, DEFAULT_PHYSICAL_HEIGHT))
        .build()?;
        
    let game = create_game(ctx)?;
    ggez::graphics::set_screen_coordinates(ctx, ggez::graphics::Rect {
        x: 0.0,
        y: 0.0,
        w: SCREEN_WIDTH,
        h: SCREEN_HEIGHT,
    })?;
    let runner = &mut GameRunner::new(game);
    event::run(ctx, event_loop, runner)
}
