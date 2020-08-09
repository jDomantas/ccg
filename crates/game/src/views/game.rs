use std::convert::TryInto;
use engine::{Ctx, FrameRenderer, Icon, Result, SCREEN_HEIGHT, SCREEN_WIDTH};
use engine::ggez::graphics::{Text, TextFragment, Scale};
use crate::GameData;
use crate::card::{BuffKind, Card, CardEffect, Creature, Decks};
use crate::views::{DrawKind, View, ViewChange};

const CARD_WIDTH: f32 = 320.0 / 2.5;
const CARD_HEIGHT: f32 = 448.0 / 2.5;

struct ActiveCreature {
    creature: Creature,
}

impl From<Creature> for ActiveCreature {
    fn from(creature: Creature) -> ActiveCreature {
        ActiveCreature { creature }
    }
}

impl ActiveCreature {
    fn attack_power(&self) -> u32 {
        let mut attack = self.creature.attack;
        for buff in &self.creature.buffs {
            match buff.kind {
                BuffKind::NextAttackBonus { damage } => attack += damage,
            }
        }
        if let Some(weapon) = &self.creature.weapon {
            attack += weapon.damage;
        }
        attack
    }

    fn spend_attack(&mut self) {
        self.creature.buffs.retain(|b| match b.kind {
            BuffKind::NextAttackBonus { .. } => false,
        });
        if let Some(weapon) = &mut self.creature.weapon {
            weapon.durability -= 1;
            if weapon.durability == 0 {
                self.creature.weapon = None;
            }
        }
    }

    fn draw(&self, x: f32, y: f32, renderer: &mut FrameRenderer<'_>) -> Result {
        renderer.draw_icon(self.creature.icon, x - 32.0, y - 32.0, 64.0, 64.0)?;
        for i in 0..self.creature.health {
            renderer.draw_icon(Icon::HEART, x - 32.0 + i as f32 * 4.0, y - 50.0, 16.0, 16.0)?;
        }
        for i in 0..self.attack_power() {
            renderer.draw_icon(Icon::SWORD, x - 32.0 + i as f32 * 4.0, y - 68.0, 16.0, 16.0)?;
        }
        Ok(())
    }
}

#[derive(Default, Copy, Clone)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }
}

struct VisibleCard {
    card: Card,
    pos: Rect,
    target_pos: Rect,
}

impl VisibleCard {
    fn new(card: Card, pos: Rect) -> VisibleCard {
        VisibleCard {
            card,
            pos,
            target_pos: pos,
        }
    }

    fn draw(&self, renderer: &mut FrameRenderer<'_>) -> Result {
        renderer.draw(
            self.card.texture,
            self.pos.x - self.pos.w / 2.0,
            self.pos.y - self.pos.h / 2.0,
            self.pos.w,
            self.pos.h,
        )
    }

    fn draw_back(&self, renderer: &mut FrameRenderer<'_>) -> Result {
        renderer.draw(
            renderer.textures().card_back,
            self.pos.x - self.pos.w / 2.0,
            self.pos.y - self.pos.h / 2.0,
            self.pos.w,
            self.pos.h,
        )
    }

    fn visual_rect(&self) -> Rect {
        Rect {
            x: self.pos.x - self.pos.w / 2.0,
            y: self.pos.y - self.pos.h / 2.0,
            w: self.pos.w,
            h: self.pos.h,
        }
    }

    fn update(&mut self, dt: f32) {
        let dx = self.target_pos.x - self.pos.x;
        let dy = self.target_pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let move_by = 400.0 * dt * (1.0 + (dist + 1.0).log(10.0));
        if dist <= move_by {
            self.pos = self.target_pos;
        } else {
            let (dx, dy) = (dx / dist * move_by, dy / dist * move_by);
            self.pos.x += dx;
            self.pos.y += dy;
        }
    }

    fn get_creature(&self) -> Option<Creature> {
        if let CardEffect::Enemy(c) = &self.card.effect {
            Some(c.clone())
        } else {
            None
        }
    }
}

struct Cell {
    position: (f32, f32),
    card: Option<VisibleCard>,
    fixed: bool,
    enemy: Option<ActiveCreature>,
}

impl Cell {
    fn drop_rect(&self) -> Rect {
        Rect {
            x: self.position.0 - CARD_WIDTH / 2.0,
            y: self.position.1 - 50.0,
            w: CARD_WIDTH,
            h: CARD_HEIGHT + 100.0,
        }
    }
}

struct Player {
    creature: ActiveCreature,
    cell: usize,
}

enum ActionState {
    None,
    Finished(f32),
    PlayerMove(f32),
    PlayerAttack(bool, f32),
    EnemyAttack(bool, usize, f32),
    AcceptBonus(f32, Icon),
}

struct Field {
    cells: Vec<Cell>,
    player: Option<Player>,
    action: ActionState,
}

impl Field {
    fn new() -> Field {
        let mut field = Field::new_pending(0);
        field.player = Some(Player {
            creature: ActiveCreature::from(Creature {
                icon: Icon::FIGHTER,
                max_health: Some(10),
                health: 10,
                attack: 4,
                coins: 0,
                weapon: None,
                buffs: Vec::new(),
            }),
            cell: 0,
        });
        field
    }

    fn new_pending(index: usize) -> Field {
        let mut a = index;
        let mut b = 2;
        let mut gen_y = || {
            a += 1;
            b += 1;
            ((a + (b + 3) * (b / 2)) * 65 % 150) as f32
        };
        let mut x = 170.0;
        let mut gen_x = || {
            let next = x + 180.0;
            std::mem::replace(&mut x, next)
        };
        Field {
            cells: vec![
                Cell { position: (gen_x(), 100.0 + 150.0), card: None, fixed: true, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: false, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: true, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: false, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: true, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: false, enemy: None },
                Cell { position: (gen_x(), 100.0 + gen_y()), card: None, fixed: true, enemy: None },
                Cell { position: (gen_x(), 100.0 + 150.0), card: None, fixed: false, enemy: None },
            ],
            player: None,
            action: ActionState::None,
        }
    }

    fn apply_effect(&mut self, card: &CardEffect) -> Option<Icon> {
        let player = self.player.as_mut().unwrap();
        match card {
            CardEffect::None => None,
            CardEffect::Enemy(_) => unreachable!(),
            CardEffect::Weapon(weapon) => {
                if player.creature.creature.coins >= weapon.price {
                    player.creature.creature.coins -= weapon.price;
                    player.creature.creature.weapon = Some(weapon.clone());
                    Some(Icon::SWORD)
                } else {
                    Some(Icon::CROSS)
                }
            }
            CardEffect::HealEnemy { health } => {
                let cells = &mut self.cells[player.cell..];
                for c in cells {
                    if let Some(creature) = &mut c.enemy {
                        creature.creature.heal(*health);
                        return Some(Icon::GREEN_HEART);
                    }
                }
                None
            }
            CardEffect::Buff(buff) => {
                player.creature.creature.buffs.push(buff.clone());
                Some(buff.icon)
            }
            CardEffect::Heal { health } => {
                player.creature.creature.heal(*health);
                Some(Icon::HEART)
            }
        }
    }

    fn update_action(&mut self, dt: f32) {
        match &mut self.action {
            ActionState::None => {
                self.action = ActionState::PlayerMove(0.0);
            }
            ActionState::Finished(t) => { *t += dt; }
            ActionState::PlayerMove(progress) => {
                *progress += dt / 1.2;
                if *progress >= 1.0 {
                    let player = self.player.as_mut().unwrap();
                    player.cell += 1;
                    if self.cells[player.cell].enemy.is_some() {
                        self.action = ActionState::PlayerAttack(false, 0.0);
                    } else {
                        if let Some(effect) = self.cells[player.cell].card.as_ref().map(|c| c.card.effect.clone()) {
                            if let Some(icon) = self.apply_effect(&effect) {
                                self.action = ActionState::AcceptBonus(0.0, icon);
                            } else {
                                self.action = ActionState::PlayerMove(0.0);
                            }
                        } else {
                            self.action = ActionState::PlayerMove(0.0);
                        }
                    }
                }
            }
            ActionState::PlayerAttack(hit, progress) => {
                let player = self.player.as_mut().unwrap();
                *progress += dt;
                if *progress >= 0.5 && !*hit {
                    *hit = true;
                    let enemy = self.cells[player.cell].enemy.as_mut().unwrap();
                    let damage = player.creature.attack_power();
                    enemy.creature.health = enemy.creature.health.saturating_sub(damage);
                    player.creature.spend_attack();
                    if enemy.creature.health == 0 {
                        player.creature.creature.coins += enemy.creature.coins;
                        self.cells[player.cell].enemy = None;
                    }
                }
                if *progress >= 1.0 {
                    if self.cells[player.cell].enemy.is_none() {
                        self.action = ActionState::PlayerMove(0.0);
                    } else {
                        self.action = ActionState::EnemyAttack(false, player.cell, 0.0);
                    }
                }
            }
            ActionState::EnemyAttack(hit, cell, progress) => {
                let enemy = self.cells[*cell].enemy.as_mut().unwrap();
                *progress += dt;
                if *progress >= 0.5 && !*hit {
                    *hit = true;
                    let player = self.player.as_mut().unwrap();
                    let damage = enemy.attack_power();
                    player.creature.creature.health = player.creature.creature.health.saturating_sub(damage);
                    enemy.spend_attack();
                    if player.creature.creature.health == 0 {
                        self.player = None;
                    }
                }
                if *progress >= 1.0 {
                    if self.player.is_none() {
                        self.action = ActionState::Finished(0.0);
                    } else {
                        self.action = ActionState::PlayerAttack(false, 0.0);
                    }
                }
            }
            ActionState::AcceptBonus(progress, _) => {
                *progress += dt;
                if *progress >= 0.6 {
                    self.action = ActionState::PlayerMove(0.0);
                }
            }
        }
        if let ActionState::PlayerMove(_) = self.action {
            if let Some(player) = &self.player {
                if player.cell == self.cells.len() - 1 {
                    self.action = ActionState::Finished(0.0);
                }
            }
        }
    }

    fn render(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        for (i, cell) in self.cells.iter().enumerate() {
            let icon = if cell.fixed && i > 0 { Icon::RED_CIRCLE } else { Icon::CIRCLE };
            renderer.draw_icon(icon, cell.position.0 - 32.0, cell.position.1 - 32.0, 64.0, 64.0)?;
            if let Some(card) = &cell.card {
                card.draw(renderer)?;
            }
        }
        for pairs in self.cells.windows(2) {
            let [a, b]: &[Cell; 2] = pairs.try_into().unwrap();
            let dx = b.position.0 - a.position.0;
            let dy = b.position.1 - a.position.1;
            let dist = (dx * dx + dy * dy).sqrt();
            let cnt = (dist / 25.0).round() as u32;
            for i in 0..=cnt {
                if i <= 1 || i >= cnt - 1 {
                    continue;
                }
                let x = a.position.0 + dx * (i as f32) / (cnt as f32);
                let y = a.position.1 + dy * (i as f32) / (cnt as f32);
                renderer.draw_icon(Icon::DOT, x - 32.0, y - 32.0, 64.0, 64.0)?;
            }
        }

        match self.action {
            ActionState::None |
            ActionState::Finished(_) |
            ActionState::PlayerMove(_) |
            ActionState::PlayerAttack(_, _) |
            ActionState::AcceptBonus(_, _) => {
                for cell in &self.cells {
                    if let Some(enemy) = &cell.enemy {
                        let pos = cell.position;
                        enemy.draw(pos.0 + 32.0 + 8.0, pos.1, renderer)?;
                    }
                }
            }
            ActionState::EnemyAttack(_, attacker, progress) => {
                for (index, cell) in self.cells.iter().enumerate() {
                    if let Some(enemy) = &cell.enemy {
                        if index != attacker {
                            let pos = cell.position;
                            enemy.draw(pos.0 + 32.0 + 8.0, pos.1, renderer)?;
                        } else {
                            let pos = cell.position;
                            let move_by = (progress * std::f32::consts::PI).sin() * 3.0 - 2.0;
                            let move_by = if move_by < 0.0 { 0.0 } else { move_by };
                            let swing_distance = 15.0;
                            let x = pos.0 - move_by * swing_distance;
                            let y = pos.1;
                            enemy.draw(x + 32.0 + 8.0, y, renderer)?;
                        }
                    }
                }
            }
        }

        match self.action {
            ActionState::None |
            ActionState::Finished(_) |
            ActionState::EnemyAttack(_, _, _) |
            ActionState::AcceptBonus(_, _) => {
                if let Some(player) = &self.player {
                    let pos = self.cells[player.cell].position;
                    player.creature.draw(pos.0 - 32.0 - 8.0, pos.1, renderer)?;
                }
            }
            ActionState::PlayerMove(progress) => {
                if let Some(player) = &self.player {
                    let a = self.cells[player.cell].position;
                    let b = self.cells[player.cell + 1].position;
                    let dx = b.0 - a.0;
                    let dy = b.1 - a.1;
                    let x = a.0 + dx * progress;
                    let y = a.1 + dy * progress;
                    player.creature.draw(x - 32.0 - 8.0, y, renderer)?;
                }
            }
            ActionState::PlayerAttack(_, progress) => {
                if let Some(player) = &self.player {
                    let pos = self.cells[player.cell].position;
                    let move_by = (progress * std::f32::consts::PI).sin() * 3.0 - 2.0;
                    let move_by = if move_by < 0.0 { 0.0 } else { move_by };
                    let swing_distance = 15.0;
                    let x = pos.0 - 32.0 + move_by * swing_distance;
                    let y = pos.1;
                    player.creature.draw(x - 8.0, y, renderer)?;
                }
            }
        }

        if let ActionState::AcceptBonus(progress, icon) = self.action {
            let cell = self.player.as_ref().unwrap().cell;
            let pos = self.cells[cell].position;
            let x = pos.0 + 32.0;
            let y = pos.1 - progress * 64.0;
            renderer.draw_icon(icon, x - 32.0, y - 32.0, 64.0, 64.0)?;
        }

        Ok(())
    }
}

struct Label {
    text: Text,
    position: (f32, f32),
    get_text: Box<dyn Fn(&GameState) -> String>,
}

impl Label {
    fn new(position: (f32, f32), get_text: impl Fn(&GameState) -> String + 'static) -> Label {
        Label {
            text: Text::new(TextFragment::new("").scale(Scale::uniform(32.0))),
            position,
            get_text: Box::new(get_text),
        }
    }
}

struct Button {
    bounds: Rect,
    icon: Icon,
    on_click: Box<dyn Fn(&mut GameState) -> ViewChange>,
}

impl Button {
    fn new(bounds: Rect, icon: Icon, on_click: impl Fn(&mut GameState) -> ViewChange + 'static) -> Button {
        Button {
            bounds,
            icon,
            on_click: Box::new(on_click),
        }
    }
}

pub struct GameState {
    field: Field,
    pending_fields: Vec<Field>,
    deck: Vec<Card>,
    trap_deck: Vec<Card>,
    discards: Vec<Card>,
    trap_discards: Vec<Card>,
    hand: Vec<VisibleCard>,
    drag: Option<VisibleCard>,
    preparing: bool,
    labels: Vec<Label>,
    buttons: Vec<Button>,
}

fn make_deck(cards: &[Card]) -> Vec<Card> {
    use rand::seq::SliceRandom;
    let mut cards = cards.iter().cloned().collect::<Vec<_>>();
    cards.shuffle(&mut rand::thread_rng());
    cards
}

impl GameState {
    pub fn new(decks: &Decks) -> GameState {
        let health_label = Label::new((50.0, 10.0), |state| {
            let player = if let Some(player) = &state.field.player {
                player
            } else {
                return String::new();
            };
            if let Some(limit) = player.creature.creature.max_health {
                format!(
                    "{}/{}",
                    player.creature.creature.health,
                    limit,
                )
            } else {
                format!("{}", player.creature.creature.health)
            }
        });
        let coins_label = Label::new((50.0, 50.0), |state| {
            let player = if let Some(player) = &state.field.player {
                player
            } else {
                return String::new();
            };
            format!("{}", player.creature.creature.coins)
        });
        let damage_label = Label::new((50.0, 50.0), |state| {
            let player = if let Some(player) = &state.field.player {
                player
            } else {
                return String::new();
            };
            format!("{}", player.creature.creature.coins)
        });
        let damage_label = Label::new((50.0, 90.0), |state| {
            let player = if let Some(player) = &state.field.player {
                player
            } else {
                return String::new();
            };
            let mut total = player.creature.attack_power();
            let mut attack_text = format!("{}", player.creature.creature.attack);
            total -= player.creature.creature.attack;
            if let Some(weapon) = &player.creature.creature.weapon {
                attack_text += &format!("+{}", weapon.damage);
                total -= weapon.damage;
            }
            if total > 0 {
                attack_text += &format!("+{}", total);
            }
            attack_text
        });
        let durability_label = Label::new((50.0, 130.0), |state| {
            let player = if let Some(player) = &state.field.player {
                player
            } else {
                return String::new();
            };
            if let Some(weapon) = &player.creature.creature.weapon {
                format!("{}", weapon.durability)
            } else {
                String::new()
            }
        });
        let deck_button = Button::new(
            Rect {
                x: 10.0,
                y: SCREEN_HEIGHT - 138.0,
                w: 128.0,
                h: 128.0,
            },
            Icon::DECK,
            |state| ViewChange::Push(Box::new(super::card_list::CardList::new(state.deck.clone()))),
        );
        let trap_deck_button = Button::new(
            Rect {
                x: 10.0,
                y: SCREEN_HEIGHT - 276.0,
                w: 128.0,
                h: 128.0,
            },
            Icon::TRAP_DECK,
            |state| ViewChange::Push(Box::new(super::card_list::CardList::new(state.trap_deck.clone()))),
        );
        let discard_button = Button::new(
            Rect {
                x: SCREEN_WIDTH - 138.0,
                y: SCREEN_HEIGHT - 138.0,
                w: 128.0,
                h: 128.0,
            },
            Icon::DECK,
            |state| ViewChange::Push(Box::new(super::card_list::CardList::new(state.discards.clone()))),
        );
        let trap_discard_button = Button::new(
            Rect {
                x: SCREEN_WIDTH - 138.0,
                y: SCREEN_HEIGHT - 276.0,
                w: 128.0,
                h: 128.0,
            },
            Icon::TRAP_DECK,
            |state| ViewChange::Push(Box::new(super::card_list::CardList::new(state.trap_discards.clone()))),
        );
        let mut state = GameState {
            field: Field::new(),
            pending_fields: vec![
                Field::new_pending(1),
                Field::new_pending(2),
                Field::new_pending(3),
            ],
            deck: make_deck(&decks.draw),
            trap_deck: make_deck(&decks.trap),
            discards: Vec::new(),
            trap_discards: Vec::new(),
            hand: Vec::new(),
            drag: None,
            preparing: true,
            labels: vec![health_label, coins_label, damage_label],
            buttons: vec![deck_button, trap_deck_button, discard_button, trap_discard_button],
        };
        let boss_cell = state.pending_fields.last_mut().unwrap().cells.last_mut().unwrap();
        let mut boss = VisibleCard::new(decks.boss.clone(), Rect {
            x: boss_cell.position.0,
            y: boss_cell.position.1 + 50.0 + CARD_HEIGHT / 2.0,
            w: CARD_WIDTH,
            h: CARD_HEIGHT,
        });
        boss_cell.card = Some(boss);
        boss_cell.enemy = boss_cell.card.as_ref().unwrap().get_creature().map(Into::into);
        state.layout_cards(true);
        state.draw_hand();
        state.draw_traps();
        state
    }

    fn layout_cards(&mut self, place: bool) {
        let hand_width = self.hand.len() as f32 * CARD_WIDTH + (self.hand.len() - 1) as f32 * CARD_WIDTH * 0.1;
        let start_x = SCREEN_WIDTH / 2.0 - hand_width / 2.0;
        for (i, card) in self.hand.iter_mut().enumerate() {
            let x = i as f32 * CARD_WIDTH * 1.1 + start_x;
            let y = SCREEN_HEIGHT - CARD_HEIGHT - 10.0;
            card.target_pos = Rect {
                x: x + CARD_WIDTH / 2.0,
                y: y + CARD_HEIGHT / 2.0,
                w: CARD_WIDTH,
                h: CARD_HEIGHT,
            };
            if place {
                card.pos = card.target_pos;
            }
        }
    }

    fn draw_hand(&mut self) {
        loop {
            if self.hand.len() >= 6 {
                break;
            }
            if let Some(card) = self.deck.pop() {
                let mut card = VisibleCard::new(card, Rect {
                    x: 10.0 + CARD_WIDTH / 2.0,
                    y: SCREEN_HEIGHT - 10.0 - CARD_HEIGHT / 2.0,
                    w: CARD_WIDTH,
                    h: CARD_HEIGHT,
                });
                card.target_pos = card.pos;
                self.hand.insert(0, card);
            } else {
                break;
            }
        }
    }

    fn draw_traps(&mut self) {
        for cell in self.field.cells.iter_mut().skip(1) {
            if cell.fixed {
                if let Some(card) = self.trap_deck.pop() {
                    let mut card = VisibleCard::new(card, Rect {
                        x: 10.0 + CARD_WIDTH / 2.0,
                        y: SCREEN_HEIGHT - 10.0 - CARD_HEIGHT / 2.0,
                        w: CARD_WIDTH,
                        h: CARD_HEIGHT,
                    });
                    card.target_pos.x = cell.position.0;
                    card.target_pos.y = cell.position.1 + 50.0 + CARD_HEIGHT / 2.0;
                    cell.card = Some(card);
                    cell.enemy = cell.card.as_ref().unwrap().get_creature().map(Into::into);
                }
            }
        }
    }

    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        self.field.render(renderer)?;
        for card in &self.hand {
            card.draw(renderer)?;
        }
        if let Some(card) = &self.drag {
            card.draw(renderer)?;
        }

        if let Some(player) = &self.field.player {
            renderer.draw_icon(Icon::HEART, 10.0, 10.0, 32.0, 32.0)?;
            renderer.draw_icon(Icon::COIN, 10.0, 50.0, 32.0, 32.0)?;
            renderer.draw_icon(Icon::SWORD, 10.0, 90.0, 32.0, 32.0)?;
            if let Some(weapon) = &player.creature.creature.weapon {
                renderer.draw_icon(Icon::SWORD, 10.0, 130.0, 32.0, 32.0)?;
            }
        }
        for label in &self.labels {
            engine::ggez::graphics::queue_text(renderer.ggez(), &label.text, [label.position.0, label.position.1], Some(engine::ggez::graphics::BLACK));
        }
        for button in &self.buttons {
            renderer.draw_icon(button.icon, button.bounds.x, button.bounds.y, button.bounds.w, button.bounds.h)?;
        }
        Ok(())
    }
}

impl View for GameState {
    fn draw_kind(&self) -> DrawKind {
        DrawKind::Opaque
    }

    fn update(&mut self, data: &GameData, ctx: &mut Ctx<'_>, dt: f32) -> Result<ViewChange> {
        if !self.preparing {
            self.field.update_action(dt * 3.0);
            match self.field.action {
                ActionState::Finished(t) if t >= 0.5 && self.field.player.is_some() && self.pending_fields.len() > 0 => {
                    let mut player = self.field.player.take().unwrap();
                    player.cell = 0;
                    self.field = self.pending_fields.remove(0);
                    self.field.player = Some(player);
                    self.preparing = true;
                    self.draw_hand();
                    self.draw_traps();
                }
                _ => {}
            }
        }

        let (mouse_x, mouse_y) = ctx.mouse_position();
        if ctx.is_mouse_click() {
            let buttons = std::mem::take(&mut self.buttons);
            for button in &buttons {
                if button.bounds.contains(mouse_x, mouse_y) {
                    let change = (button.on_click)(self);
                    if !matches!(change, ViewChange::None) {
                        self.buttons = buttons;
                        return Ok(change);
                    }
                }
            }
            self.buttons = buttons;
        }

        let mouse_pressed = ctx.is_mouse_pressed();
        match &mut self.drag {
            Some(card) if mouse_pressed => {
                card.pos.x = mouse_x;
                card.pos.y = mouse_y;
                card.target_pos = card.pos;
            }
            Some(card) => {
                let x = card.pos.x;
                let y = card.pos.y;
                if self.preparing {
                    for cell in &mut self.field.cells {
                        if cell.card.is_some() || cell.fixed {
                            continue;
                        }
                        let rect = cell.drop_rect();
                        if rect.contains(x, y) {
                            card.target_pos.x = cell.position.0;
                            card.target_pos.y = cell.position.1 + 50.0 + CARD_HEIGHT / 2.0;
                            cell.card = self.drag.take();
                            cell.enemy = cell.card.as_ref().unwrap().get_creature().map(Into::into);
                            break;
                        }
                    }
                }
                if let Some(card) = self.drag.take() {
                    self.hand.push(card);
                    self.hand.sort_by(|a, b| a.pos.x.partial_cmp(&b.pos.x).unwrap());
                }
            }
            None if mouse_pressed => {
                for (index, card) in self.hand.iter().enumerate() {
                    let rect = card.visual_rect();
                    if rect.contains(mouse_x, mouse_y) {
                        self.drag = Some(self.hand.remove(index));
                        break;
                    }
                }
                if self.drag.is_none() && self.preparing {
                    for cell in &mut self.field.cells {
                        if cell.fixed {
                            continue;
                        }
                        if let Some(card) = &cell.card {
                            let rect = card.visual_rect();
                            if rect.contains(mouse_x, mouse_y) {
                                self.drag = cell.card.take();
                                cell.enemy = None;
                                break;
                            }
                        }
                    }
                }
            }
            None => {}
        }
        self.layout_cards(false);
        for cell in &mut self.field.cells {
            if let Some(card) = &mut cell.card {
                card.update(dt);
            }
        }
        for card in &mut self.hand {
            card.update(dt);
        }

        if engine::ggez::input::keyboard::is_key_pressed(
            ctx.ggez(),
            engine::ggez::input::keyboard::KeyCode::Space,
        ) {
            if self.preparing {
                println!("finished preparation");
            }
            self.preparing = false;
        }

        let mut labels = std::mem::take(&mut self.labels);
        for label in &mut labels {
            label.text.fragments_mut()[0].text = (label.get_text)(self);
        }
        self.labels = labels;
        Ok(super::ViewChange::None)
    }

    fn draw(&mut self, renderer: &mut FrameRenderer<'_>) -> Result {
        self.draw(renderer)?;
        engine::ggez::graphics::draw_queued_text(
            renderer.ggez(),
            engine::ggez::graphics::DrawParam::default(),
            None,
            engine::ggez::graphics::FilterMode::Linear,
        )?;
        Ok(())
    }
}
