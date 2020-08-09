use engine::{Ctx, FrameRenderer, Result, ggez::graphics::Text};
use crate::{Decks, GameData, card::Card};
use super::{DrawKind, View, ViewChange};

const CARD_WIDTH: f32 = 320.0;
const CARD_HEIGHT: f32 = 448.0;

fn select_treasure(cards: &[Card]) -> Vec<Card> {
    let target_count = 3;
    let mut picks: Vec<Card> = Vec::new();
    while picks.len() < target_count {
        let candidates = cards.iter().filter(|c| picks.iter().all(|p| p.id != c.id)).count();
        if candidates == 0 {
            break;
        }
        let idx = rand::random::<usize>() % candidates;
        let pick = cards.iter().filter(|c| picks.iter().all(|p| p.id != c.id)).nth(idx).unwrap().clone();
        picks.push(pick);
    }
    picks
}

pub struct CardSelect {
    cards: Vec<(Card, f32)>,
    decks: Decks,
}

impl CardSelect {
    pub fn new(decks: Decks) -> CardSelect {
        let cards = select_treasure(&decks.treasure);
        CardSelect {
            cards: cards.into_iter().map(|c| (c, 0.0)).collect(),
            decks,
        }
    }

    pub fn new_unsorted(cards: Vec<Card>, decks: Decks) -> CardSelect {
        CardSelect {
            cards: cards.into_iter().map(|c| (c, 0.0)).collect(),
            decks,
        }
    }
}

impl CardSelect {
    fn card_positions(&self) -> impl Iterator<Item = (f32, f32)> + 'static {
        let center = (self.cards.len().saturating_sub(1) as f32) / 2.0;
        (0..self.cards.len()).map(move |idx| ((idx as f32 - center) * CARD_WIDTH * 1.3 + 800.0, 500.0))
    }
}

impl View for CardSelect {
    fn draw_kind(&self) -> DrawKind {
        DrawKind::Opaque
    }

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result<ViewChange> {
        let (x, y) = ctx.mouse_position();
        let click = ctx.is_mouse_click();
        for ((cx, cy), (card, dy)) in self.card_positions().zip(self.cards.iter_mut()) {
            let inside = x >= cx - CARD_WIDTH / 2.0 &&
                x <= cx + CARD_WIDTH / 2.0 &&
                y >= cy - CARD_HEIGHT / 2.0 &&
                y <= cy + CARD_HEIGHT / 2.0;
            if inside {
                *dy += dt * 10.0;
            } else {
                *dy -= dt * 10.0;
            }
            if *dy < 0.0 {
                *dy = 0.0;
            } else if *dy > 1.0 {
                *dy = 1.0;
            }
            if inside && click {
                self.decks.draw.push(card.clone());
                return Ok(ViewChange::Replace(Box::new(super::GameState::new(&self.decks))));
            }
        }
        Ok(ViewChange::None)
    }

    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        for ((cx, cy), (card, dy)) in self.card_positions().zip(self.cards.iter()) {
            let y = cy - *dy * 30.0;
            renderer.draw(
                card.texture,
                cx - CARD_WIDTH / 2.0,
                y - CARD_HEIGHT / 2.0,
                CARD_WIDTH,
                CARD_HEIGHT,
            )?;
        }
        Ok(())
    }
}
