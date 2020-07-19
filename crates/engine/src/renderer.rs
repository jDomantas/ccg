use ggez::{Context, GameResult};
use ggez::graphics::{DrawParam, Image, Rect, spritebatch::SpriteBatch};
use ggez::nalgebra::Point2;

pub struct Renderer {
    spritebatch: SpriteBatch,
}

impl Renderer {
    pub fn new(ctx: &mut Context, spritesheet: &str) -> GameResult<Self> {
        let image = Image::new(ctx, spritesheet)?;
        let mut pixels = image.to_rgba8(ctx)?;
        for pixel in pixels.chunks_mut(4) {
            if pixel == &[163, 73, 164, 255] || pixel == &[200, 191, 231, 255] {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 0;
            }
        }
        let image = Image::from_rgba8(
            ctx,
            image.width(),
            image.height(),
            &pixels,
        )?;
        let mut spritebatch = SpriteBatch::new(image);
        spritebatch.set_filter(ggez::graphics::FilterMode::Nearest);
        Ok(Renderer { spritebatch })
    }

    pub fn frame(&mut self) -> FrameRenderer<'_> {
        self.spritebatch.clear();
        FrameRenderer { renderer: self }
    }
}

pub struct FrameRenderer<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> FrameRenderer<'a> {
    pub fn add_image(&mut self, x: f32, y: f32, width: f32, height: f32, index: u32) {
        let row = (index / 8) as f32;
        let col = (index % 8) as f32;
        let draw = DrawParam::new()
            .dest(Point2::new(x, y))
            .scale([width / 16.0, height / 16.0])
            .src(Rect {
                x: col / 8.0,
                y: row / 8.0,
                w: 0.125,
                h: 0.125,
            });
        self.renderer.spritebatch.add(draw);
    }

    pub fn draw(self, ctx: &mut Context) -> GameResult<()> {
        let draw = DrawParam::new().dest(Point2::new(0.0, 0.0));
        ggez::graphics::draw(ctx, &self.renderer.spritebatch, draw)?;
        Ok(())
    }
}
