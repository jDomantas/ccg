#![allow(unused)]
#![warn(unused_must_use)]

pub mod card;
pub mod loader;
pub mod views;

use std::convert::TryInto;
use engine::{Ctx, FrameRenderer, Icon, Renderer, Result, Texture, SCREEN_HEIGHT, SCREEN_WIDTH};
use engine::ggez::{self, graphics::{Align, Text, TextFragment, Scale}};
use card::Decks;

pub struct GameData {
    decks: Decks,
}

struct TestGame {
    renderer: Renderer,
    data: GameData,
    view_stack: crate::views::ViewStack,
}

const CARD_WIDTH: f32 = 320.0;
const CARD_HEIGHT: f32 = 448.0;

impl engine::Game for TestGame {
    fn update(&mut self, ctx: &mut Ctx<'_>) -> Result {
        self.view_stack.update(&self.data, ctx, 1.0 / 60.0)
    }

    fn draw(&mut self, ctx: &mut Ctx<'_>) -> Result {
        // let (x, y) = ctx.mouse_position();
        let mut renderer = self.renderer.frame(ctx);
        // let card = self.data.decks.draw.iter().next().unwrap().texture;
        // renderer.draw_icon(Icon::new(0), 10.0, 10.0, CARD_WIDTH, CARD_HEIGHT)?;
        // renderer.draw(card, 10.0, 10.0, CARD_WIDTH, CARD_HEIGHT)?;
        self.view_stack.draw(&mut renderer)?;
        // ggez::graphics::draw_queued_text(ctx, ggez::graphics::DrawParam::default(), None, ggez::graphics::FilterMode::Linear)?;
        Ok(())
    }
}

fn main() {
    let result = engine::run(&|ctx| {
        let resources = match loader::load_resources(ctx) {
            Ok(resources) => {
                println!("loaded resources");
                resources
            }
            Err(e) => {
                panic!("failed to load resources: {}", e);
            }
        };
        ggez::graphics::set_screen_coordinates(ctx, ggez::graphics::Rect {
            x: 0.0,
            y: 0.0,
            w: SCREEN_WIDTH,
            h: SCREEN_HEIGHT,
        }).expect("failed to set screen coordintes");
        Ok(Box::new(TestGame {
            renderer: resources.renderer,
            data: GameData { decks: resources.decks },
            view_stack: views::ViewStack::new(views::MenuView::new(views::main::MainMenu)),
        }))
    });
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
