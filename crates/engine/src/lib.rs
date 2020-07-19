mod renderer;

pub use ggez;
use ggez::{Context, GameResult, event, graphics, timer};
pub use crate::renderer::{FrameRenderer, Renderer};

pub trait Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult;
    fn draw(&mut self, ctx: &mut Context) -> GameResult;
}

struct GameRunner {
    game: Box<dyn Game>,
    position_set: bool,
}

impl GameRunner {
    fn new(game: Box<dyn Game>) -> Self {
        GameRunner {
            game,
            position_set: false,
        }
    }
}

impl event::EventHandler for GameRunner {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !self.position_set {
            self.position_set = true;
            let window = ggez::graphics::window(ctx);
            window.set_position(winit::dpi::LogicalPosition {
                x: 100.0,
                y: 50.0,
            });
        }
        if timer::ticks(ctx) % 60 == 0 {
            println!(
                "delta frame time: {:?}, average fps: {}",
                timer::delta(ctx),
                timer::fps(ctx),
            );
        }
        self.game.update(ctx)
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);
        self.game.draw(ctx)?;
        graphics::present(ctx)?;
        Ok(())
    }
}

pub fn run(create_game: &dyn Fn(&mut Context) -> GameResult<Box<dyn Game>>) -> GameResult {
    let resource_dir = std::path::PathBuf::from("./resources");

    let (ctx, event_loop) = &mut ggez::ContextBuilder::new("ccg", "domantas")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Some sort of ccg"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(1600.0, 900.0))
        .build()?;

    let game = create_game(ctx)?;
    let runner = &mut GameRunner::new(game);
    event::run(ctx, event_loop, runner)
}
