use engine::{Ctx, FrameRenderer, Result};
use crate::{GameData, card::Card};
use super::{DrawKind, View, ViewChange};

const CARD_WIDTH: f32 = 320.0 / 1.5;
const CARD_HEIGHT: f32 = 448.0 / 1.5;

pub struct CardList {
    cards: Vec<Card>,
    start_scroll: Option<f32>,
    scroll_cap: f32,
    y: f32,
}

impl CardList {
    pub fn new(mut cards: Vec<Card>) -> CardList {
        cards.sort_by(|a, b| a.id.cmp(&b.id));
        CardList::new_unsorted(cards)
    }

    pub fn new_unsorted(cards: Vec<Card>) -> CardList {
        let rows = (cards.len() + 5) / 6;
        let mut scroll_cap = rows as f32 * (CARD_HEIGHT + CARD_WIDTH * 0.2) - CARD_HEIGHT * 2.0;
        if scroll_cap < 0.0 {
            scroll_cap = 0.0;
        }
        CardList {
            cards,
            start_scroll: None,
            y: 0.0,
            scroll_cap,
        }
    }
}

impl View for CardList {
    fn draw_kind(&self) -> DrawKind {
        DrawKind::OnTop
    }

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result<ViewChange> {
        let scroll = ctx.scroll_position().1 * 100.0;
        self.y = match self.start_scroll {
            None => {
                self.start_scroll = Some(scroll);
                0.0
            }
            Some(s) if s > scroll => {
                self.start_scroll = Some(scroll);
                0.0
            }
            Some(s) if scroll - s > self.scroll_cap => {
                self.start_scroll = Some(scroll - self.scroll_cap);
                self.scroll_cap
            }
            Some(s) => scroll - s,
        };
        Ok(if ctx.is_mouse_click() {
            ViewChange::Pop
        } else {
            ViewChange::None
        })
    }

    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        renderer.draw_fade(0.8)?;
        for (i, card) in self.cards.iter().enumerate() {
            let row = i / 6;
            let col = i % 6;
            let x = (col as f32 - 2.5) * CARD_WIDTH * 1.2 + 800.0;
            let y = (row as f32) * (CARD_HEIGHT + CARD_WIDTH * 0.2) + CARD_HEIGHT / 2.0 + 100.0;
            renderer.draw(
                card.texture,
                x - CARD_WIDTH / 2.0,
                y - CARD_HEIGHT / 2.0 - self.y,
                CARD_WIDTH,
                CARD_HEIGHT,
            )?;
        }
        Ok(())
    }
}
