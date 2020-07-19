#![allow(unused)]

use std::convert::TryInto;
use engine::ggez::{self, Context, GameResult, graphics::{Text, TextFragment, Scale}};
use engine::{FrameRenderer, Renderer};
use ggez::graphics::Align;

const SCREEN_WIDTH: f32 = 1600.0;
const SCREEN_HEIGHT: f32 = 900.0;

const CARD_WIDTH: f32 = 120.0;
const CARD_HEIGHT: f32 = CARD_WIDTH * 16.0 / 10.0;

#[derive(Clone)]
struct Creature {
    texture_icon: u32,
    health: u32,
    attack: u32,
    bonus_attack: u32,
    coins: u32,
    weapon: Option<Weapon>,
}

impl Creature {
    fn attack_power(&self) -> u32 {
        self.attack + self.bonus_attack + self.weapon.as_ref().map(|w| w.bonus_attack).unwrap_or(0)
    }

    fn spend_attack(&mut self) {
        self.bonus_attack = 0;
        if let Some(weapon) = &mut self.weapon {
            weapon.durability -= 1;
            if weapon.durability == 0 {
                self.weapon = None;
            }
        }
    }

    fn render(&self, x: f32, y: f32, ctx: &mut Context, renderer: &mut FrameRenderer<'_>) {
        renderer.add_image(x - 32.0, y - 32.0, 64.0, 64.0, self.texture_icon);
        for i in 0..self.health {
            renderer.add_image(x - 32.0 + i as f32 * 4.0, y - 50.0, 16.0, 16.0, 5);
        }
        for i in 0..self.attack_power() {
            renderer.add_image(x - 32.0 + i as f32 * 4.0, y - 68.0, 16.0, 16.0, 4);
        }
    }
}

#[derive(Clone)]
struct Weapon {
    durability: u32,
    bonus_attack: u32,
    price: u32,
}

#[derive(Clone)]
enum CardEffect {
    Creature(Creature),
    BuffCreatureHealth(u32),
    Weapon(Weapon),
    AttackBonus(u32),
    Heal(u32),
}

impl CardEffect {
    fn apply_in(&self, field: &mut Field) -> Option<u32> {
        let player = field.player.as_mut().unwrap();
        match self {
            CardEffect::Creature(_) => unreachable!(),
            CardEffect::Weapon(weapon) => {
                if player.creature.coins >= weapon.price {
                    player.creature.coins -= weapon.price;
                    player.creature.weapon = Some(weapon.clone());
                    Some(4)
                } else {
                    Some(13)
                }
            }
            CardEffect::BuffCreatureHealth(amount) => {
                let cells = &mut field.cells[player.cell..];
                for c in cells {
                    if let Some(creature) = &mut c.enemy {
                        creature.health += amount;
                        return Some(17);
                    }
                }
                None
            }
            CardEffect::AttackBonus(amount) => {
                player.creature.bonus_attack += amount;
                Some(14)
            }
            CardEffect::Heal(amount) => {
                player.creature.health += amount;
                Some(5)
            }
        }
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

struct Card {
    title: ggez::graphics::Text,
    description: ggez::graphics::Text,
    icon: u32,
    effect: CardEffect,
    pos: Rect,
    target_pos: Rect,
}

impl Card {
    fn new(title: &str, description: &str, icon: u32, effect: CardEffect) -> Card {
        let title = Text::new(TextFragment::new(title)
            .scale(Scale::uniform(20.0)));
        let mut description = Text::new(TextFragment::new(description)
            .scale(Scale::uniform(20.0)));
        description.set_bounds([CARD_WIDTH * 0.75, 10000.0], Align::Left);
        Card {
            title,
            description,
            icon,
            effect,
            pos: Rect::default(),
            target_pos: Rect::default(),
        }
    }

    fn render(&self, ctx: &mut Context, renderer: &mut FrameRenderer<'_>) {
        renderer.add_image(
            self.pos.x - self.pos.w / 2.0 * 8.0 / 5.0,
            self.pos.y - self.pos.h / 2.0,
            self.pos.w * 16.0 / 10.0,
            self.pos.h,
            9,
        );
        renderer.add_image(
            self.pos.x - self.pos.w / 2.0 + self.pos.w * 0.2,
            self.pos.y - self.pos.h / 2.0 + self.pos.h * 0.1875,
            self.pos.w * 6.0 / 10.0,
            self.pos.h * 6.0 / 16.0,
            self.icon,
        );
        ggez::graphics::queue_text(
            ctx,
            &self.title,
            [
                (self.pos.x - self.pos.w / 2.0 + self.pos.w * 0.15).round(),
                (self.pos.y - self.pos.h / 2.0 + self.pos.h * 0.09).round(),
            ],
            Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)),
        );
        ggez::graphics::queue_text(
            ctx,
            &self.description,
            [
                (self.pos.x - self.pos.w / 2.0 + self.pos.w * 0.15).round(),
                (self.pos.y + self.pos.h * 0.09).round(),
            ],
            Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)),
        );
    }

    fn render_back(&self, ctx: &mut Context, renderer: &mut FrameRenderer<'_>) {
        renderer.add_image(
            self.pos.x - self.pos.w / 2.0 * 8.0 / 5.0,
            self.pos.y - self.pos.h / 2.0,
            self.pos.w * 16.0 / 10.0,
            self.pos.h,
            11,
        );
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
        if let CardEffect::Creature(c) = &self.effect {
            Some(c.clone())
        } else {
            None
        }
    }
}

struct Cell {
    position: (f32, f32),
    card: Option<Card>,
    fixed: bool,
    enemy: Option<Creature>,
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
    creature: Creature,
    cell: usize,
}

enum ActionState {
    None,
    Finished(f32),
    PlayerMove(f32),
    PlayerAttack(bool, f32),
    EnemyAttack(bool, usize, f32),
    AcceptBonus(f32, u32),
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
            creature: Creature {
                texture_icon: 7,
                health: 10,
                attack: 4,
                bonus_attack: 0,
                coins: 0,
                weapon: None,
            },
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
                        if let Some(effect) = self.cells[player.cell].card.as_ref().map(|c| c.effect.clone()) {
                            if let Some(icon) = effect.apply_in(self) {
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
                    enemy.health = enemy.health.saturating_sub(player.creature.attack + player.creature.bonus_attack);
                    player.creature.spend_attack();
                    if enemy.health == 0 {
                        player.creature.coins += enemy.coins;
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
                    player.creature.health = player.creature.health.saturating_sub(enemy.attack + enemy.bonus_attack);
                    enemy.spend_attack();
                    if player.creature.health == 0 {
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

    fn render(&mut self, ctx: &mut Context, renderer: &mut FrameRenderer<'_>) {
        for (i, cell) in self.cells.iter().enumerate() {
            let icon = if cell.fixed && i > 0 { 15 } else { 1 };
            renderer.add_image(cell.position.0 - 32.0, cell.position.1 - 32.0, 64.0, 64.0, icon);
            if let Some(card) = &cell.card {
                card.render(ctx, renderer);
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
                renderer.add_image(x - 32.0, y - 32.0, 64.0, 64.0, 2);
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
                        enemy.render(pos.0 + 32.0 + 8.0, pos.1, ctx, renderer);
                    }
                }
            }
            ActionState::EnemyAttack(_, attacker, progress) => {
                for (index, cell) in self.cells.iter().enumerate() {
                    if let Some(enemy) = &cell.enemy {
                        if index != attacker {
                            let pos = cell.position;
                            enemy.render(pos.0 + 32.0 + 8.0, pos.1, ctx, renderer);
                        } else {
                            let pos = cell.position;
                            let move_by = (progress * std::f32::consts::PI).sin() * 3.0 - 2.0;
                            let move_by = if move_by < 0.0 { 0.0 } else { move_by };
                            let swing_distance = 15.0;
                            let x = pos.0 - move_by * swing_distance;
                            let y = pos.1;
                            enemy.render(x + 32.0 + 8.0, y, ctx, renderer);
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
                    player.creature.render(pos.0 - 32.0 - 8.0, pos.1, ctx, renderer);
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
                    player.creature.render(x - 32.0 - 8.0, y, ctx, renderer);
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
                    player.creature.render(x - 8.0, y, ctx, renderer);
                }
            }
        }

        if let ActionState::AcceptBonus(progress, icon) = self.action {
            let cell = self.player.as_ref().unwrap().cell;
            let pos = self.cells[cell].position;
            let x = pos.0 + 32.0;
            let y = pos.1 - progress * 64.0;
            renderer.add_image(x - 32.0, y - 32.0, 64.0, 64.0, icon);
        }
    }
}

struct GameState {
    field: Field,
    pending_fields: Vec<Field>,
    deck: Vec<Card>,
    trap_deck: Vec<Card>,
    hand: Vec<Card>,
    drag: Option<Card>,
    preparing: bool,
    health_text: Text,
    coins_text: Text,
    attack_text: Text,
    durability_text: Text,
}

fn create_boss() -> Card {
    Card::new("boss", "20 health, 7 attack", 16, CardEffect::Creature(Creature {
        texture_icon: 16,
        health: 20,
        attack: 7,
        bonus_attack: 0,
        coins: 0,
        weapon: None,
    }))
}

impl GameState {
    fn new() -> GameState {
        fn create_text() -> Text {
            Text::new(TextFragment::new("?").scale(Scale::uniform(32.0)))
        }
        let mut state = GameState {
            field: Field::new(),
            pending_fields: vec![
                Field::new_pending(1),
                Field::new_pending(2),
                Field::new_pending(3),
            ],
            deck: create_deck(),
            trap_deck: create_trap_deck(),
            hand: Vec::new(),
            drag: None,
            preparing: true,
            health_text: create_text(),
            coins_text: create_text(),
            attack_text: create_text(),
            durability_text: create_text(),
        };
        let boss_cell = state.pending_fields.last_mut().unwrap().cells.last_mut().unwrap();
        let mut boss = create_boss();
        boss.target_pos.x = boss_cell.position.0;
        boss.target_pos.y = boss_cell.position.1 + 50.0 + CARD_HEIGHT / 2.0;
        boss.target_pos.w = CARD_WIDTH;
        boss.target_pos.h = CARD_HEIGHT;
        boss.pos = boss.target_pos;
        boss_cell.card = Some(boss);
        boss_cell.enemy = boss_cell.card.as_ref().unwrap().get_creature();
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
        for (i, card) in self.deck.iter_mut().enumerate() {
            let x = 10.0;
            let y = SCREEN_HEIGHT - 10.0 - CARD_HEIGHT - i as f32 * 10.0;
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
        for (i, card) in self.trap_deck.iter_mut().enumerate() {
            let x = 20.0 + CARD_WIDTH;
            let y = SCREEN_HEIGHT - 10.0 - CARD_HEIGHT - i as f32 * 10.0;
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
                self.hand.insert(0, card);
            } else {
                break;
            }
        }
    }

    fn draw_traps(&mut self) {
        for cell in self.field.cells.iter_mut().skip(1) {
            if cell.fixed {
                if let Some(mut card) = self.trap_deck.pop() {
                    card.target_pos.x = cell.position.0;
                    card.target_pos.y = cell.position.1 + 50.0 + CARD_HEIGHT / 2.0;
                    cell.card = Some(card);
                    cell.enemy = cell.card.as_ref().unwrap().get_creature();
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        let dt = ggez::timer::delta(ctx).as_secs_f32();
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
        let mouse_pressed = ggez::input::mouse::button_pressed(ctx, ggez::input::mouse::MouseButton::Left);
        let mouse = ggez::input::mouse::position(ctx);
        match &mut self.drag {
            Some(card) if mouse_pressed => {
                card.pos.x = mouse.x;
                card.pos.y = mouse.y;
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
                            cell.enemy = cell.card.as_ref().unwrap().get_creature();
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
                    if rect.contains(mouse.x, mouse.y) {
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
                            if rect.contains(mouse.x, mouse.y) {
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
        for card in &mut self.deck {
            card.update(dt);
        }

        if ggez::input::keyboard::is_key_pressed(ctx, ggez::input::keyboard::KeyCode::Space) {
            if self.preparing {
                println!("finished preparation");
            }
            self.preparing = false;
        }

        if let Some(player) = &self.field.player {
            let mut attack_text = format!("{}", player.creature.attack);
            if let Some(weapon) = &player.creature.weapon {
                self.durability_text.fragments_mut()[0].text = format!("{}", weapon.durability);
                attack_text += &format!("+{}", weapon.bonus_attack);
            }
            if player.creature.bonus_attack > 0 {
                attack_text += &format!("+{}", player.creature.bonus_attack);
            }
            self.attack_text.fragments_mut()[0].text = attack_text;
            self.health_text.fragments_mut()[0].text = format!("{}", player.creature.health);
            self.coins_text.fragments_mut()[0].text = format!("{}", player.creature.coins);
        }
    }

    fn render(&mut self, ctx: &mut Context, renderer: &mut FrameRenderer<'_>) {
        self.field.render(ctx, renderer);
        for card in &self.hand {
            card.render(ctx, renderer);
        }
        if let Some(card) = &self.drag {
            card.render(ctx, renderer);
        }
        for card in &self.deck {
            card.render_back(ctx, renderer);
        }
        for card in &self.trap_deck {
            card.render_back(ctx, renderer);
        }
        if let Some(player) = &self.field.player {
            renderer.add_image(10.0, 10.0, 32.0, 32.0, 5);
            ggez::graphics::queue_text(ctx, &self.health_text, [50.0, 10.0], Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)));
            renderer.add_image(10.0, 50.0, 32.0, 32.0, 12);
            ggez::graphics::queue_text(ctx, &self.coins_text, [50.0, 50.0], Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)));
            renderer.add_image(10.0, 90.0, 32.0, 32.0, 4);
            ggez::graphics::queue_text(ctx, &self.attack_text, [50.0, 90.0], Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)));
            if let Some(weapon) = &player.creature.weapon {
                renderer.add_image(10.0, 130.0, 32.0, 32.0, 4);
                ggez::graphics::queue_text(ctx, &self.durability_text, [50.0, 130.0], Some(ggez::graphics::Color::new(0.0, 0.0, 0.0, 1.0)));
            }
        }
    }
}

fn create_deck() -> Vec<Card> {
    vec![
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("big bonk", "+5 to next attack", 14, CardEffect::AttackBonus(5)),
        Card::new("big bonk", "+5 to next attack", 14, CardEffect::AttackBonus(5)),
        Card::new("pls help", "+5 health", 5, CardEffect::Heal(5)),
        Card::new("pls help", "+5 health", 5, CardEffect::Heal(5)),
        Card::new("sword", "5 attack, 2 hits, 6 coins", 4, CardEffect::Weapon(Weapon {
            bonus_attack: 5,
            durability: 2,
            price: 6,
        })),
        Card::new("beholder", "5 attack, 10 health, 5 coins", 8, CardEffect::Creature(Creature {
            texture_icon: 8,
            attack: 5,
            health: 10,
            bonus_attack: 0,
            coins: 5,
            weapon: None,
        })),
        Card::new("beholder", "5 attack, 10 health, 5 coins", 8, CardEffect::Creature(Creature {
            texture_icon: 8,
            attack: 5,
            health: 10,
            bonus_attack: 0,
            coins: 5,
            weapon: None,
        })),
        Card::new("beholder", "5 attack, 10 health, 5 coins", 8, CardEffect::Creature(Creature {
            texture_icon: 8,
            attack: 5,
            health: 10,
            bonus_attack: 0,
            coins: 5,
            weapon: None,
        })),
    ]
}

fn create_trap_deck() -> Vec<Card> {
    vec![
        Card::new("bonk", "+2 to next attack", 14, CardEffect::AttackBonus(2)),
        Card::new("heal foe", "+3 to next enemy health", 17, CardEffect::BuffCreatureHealth(3)),
    ]
}

struct Game {
    state: GameState,
    renderer: Renderer,
}

impl engine::Game for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.state.update(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut frame = self.renderer.frame();
        self.state.render(ctx, &mut frame);
        frame.draw(ctx)?;
        ggez::graphics::draw_queued_text(ctx, ggez::graphics::DrawParam::default(), None, ggez::graphics::FilterMode::Linear)?;
        Ok(())
    }
}

fn main() {
    let result = engine::run(&|ctx| {
        let renderer = Renderer::new(ctx, "/temp.png")?;
        Ok(Box::new(Game {
            state: GameState::new(),
            renderer,
        }))
    });
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
