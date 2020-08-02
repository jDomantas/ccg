use ggez::Context;
use ggez::graphics::{DrawParam, Image, Rect};
use ggez::nalgebra::Point2;
use crate::{Ctx, CtxData, Result};

pub struct Renderer {
    icons: Image,
    textures: Vec<Image>,
    indices: Textures,
}

impl Renderer {
    pub fn new(icons: Image, textures: Vec<Image>, indices: Textures) -> Self {
        Renderer { icons, textures, indices }
    }

    pub fn frame<'a>(&'a mut self, ctx: &'a mut Ctx<'_>) -> FrameRenderer<'a> {
        FrameRenderer {
            renderer: self,
            ctx: ctx.ggez,
            ctx_data: ctx.data,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Icon {
    index: u32,
}

impl Icon {
    pub const CIRCLE: Icon = Icon { index: 1 };
    pub const DOT: Icon = Icon { index: 2 };
    pub const SQUARE: Icon = Icon { index: 3 };
    pub const SWORD: Icon = Icon { index: 4 };
    pub const HEART: Icon = Icon { index: 5 };
    pub const SHIELD: Icon = Icon { index: 6 };
    pub const FIGHTER: Icon = Icon { index: 7 };
    pub const BEHOLDER: Icon = Icon { index: 8 };
    pub const CARD: Icon = Icon { index: 9 };
    pub const PLAY: Icon = Icon { index: 10 };
    pub const CARD_BACK: Icon = Icon { index: 11 };
    pub const COIN: Icon = Icon { index: 12 };
    pub const CROSS: Icon = Icon { index: 13 };
    pub const BANG: Icon = Icon { index: 14 };
    pub const RED_CIRCLE: Icon = Icon { index: 15 };
    pub const BLUE_BEHOLDER: Icon = Icon { index: 16 };
    pub const GREEN_HEART: Icon = Icon { index: 17 };
    pub const BROKEN: Icon = Icon { index: 18 };

    pub const fn new(index: u32) -> Icon {
        Icon { index }
    }
}

pub struct Textures {
    pub button: Texture,
    pub button_hover: Texture,
    pub button_selected: Texture,
    pub card_back: Texture,
}

#[derive(Debug, Copy, Clone)]
pub struct Texture {
    index: u32,
}

impl Texture {
    pub const fn new(index: u32) -> Texture {
        Texture { index }
    }
}

pub struct FrameRenderer<'a> {
    renderer: &'a mut Renderer,
    ctx: &'a mut Context,
    ctx_data: &'a mut CtxData,
}

impl<'a> FrameRenderer<'a> {
    pub fn textures(&self) -> &Textures {
        &self.renderer.indices
    }

    pub fn ggez(&mut self) -> &mut Context {
        self.ctx
    }

    pub fn ctx(&mut self) -> Ctx<'_> {
        Ctx {
            ggez: self.ctx,
            data: self.ctx_data,
        }
    }

    pub fn draw(&mut self, texture: Texture, x: f32, y: f32, width: f32, height: f32) -> Result {
        let texture = &self.renderer.textures[texture.index as usize];
        ggez::graphics::draw(
            self.ctx,
            texture,
            DrawParam::new()
                .dest(Point2::new(x, y))
                .scale([
                    width / f32::from(texture.width()),
                    height / f32::from(texture.height()),
                ])
                .src(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 }),
        )
    }

    pub fn draw_icon(&mut self, icon: Icon, x: f32, y: f32, width: f32, height: f32) -> Result {
        let row = (icon.index / 8) as f32;
        let col = (icon.index % 8) as f32;
        let draw = DrawParam::new()
            .dest(Point2::new(x, y))
            .scale([width / 16.0, height / 16.0])
            .src(Rect {
                x: col / 8.0,
                y: row / 8.0,
                w: 0.125,
                h: 0.125,
            });
        ggez::graphics::draw(self.ctx, &self.renderer.icons, draw)
    }
}
