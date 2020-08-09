pub mod config;

use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use engine::{Texture, Textures};
use engine::ggez::{self, Context, GameResult};
use engine::ggez::graphics::{Canvas, Image, Text, TextFragment};
use crate::card;
use ggez::graphics::{Scale, Rect};

pub struct GameResources {
    pub decks: card::Decks,
    pub renderer: engine::Renderer,
}

#[derive(Default)]
struct TextureSet {
    textures: Vec<Image>,
}

impl TextureSet {
    fn add(&mut self, texture: Image) -> Texture {
        self.textures.push(texture);
        Texture::new(self.textures.len() as u32 - 1)
    }
}

const CARD_WIDTH: u16 = 320;
const CARD_HEIGHT: u16 = 448;

pub fn load_resources(ctx: &mut Context) -> GameResult<GameResources> {
    let mut texture_set = TextureSet::default();
    let icons = Image::new(ctx, "/temp.png")?;
    let mut icons = make_transparent(ctx, &icons)?;
    icons.set_filter(ggez::graphics::FilterMode::Nearest);
    let card_base = Image::new(ctx, "/card-base.png")?;

    let card_back = Image::new(ctx, "/card-back.png")?;
    let card_back = render_card_back(ctx, &card_base, &card_back)?;
    let card_back = texture_set.add(card_back);

    let button = texture_set.add(Image::new(ctx, "/button/regular.png")?);
    let button_hover = texture_set.add(Image::new(ctx, "/button/hover.png")?);
    let button_selected = texture_set.add(Image::new(ctx, "/button/selected.png")?);

    let cards = load_cards(ctx, &card_base, &icons, &mut texture_set.textures)?;
    let decks = load_decks(&cards);

    let renderer = engine::Renderer::new(icons, texture_set.textures, Textures {
        card_back,
        button,
        button_hover,
        button_selected,
    });

    Ok(GameResources {
        decks,
        renderer,
    })
}

fn load_cards(ctx: &mut Context, base: &Image, icons: &Image, textures: &mut Vec<Image>) -> GameResult<HashMap<String, card::Card>> {
    let mut renderer = CardRenderer {
        icons,
        base,
    };
    let mut cards = HashMap::new();
    for entry in std::fs::read_dir("./data/cards")? {
        let entry = entry?;
        let path = entry.path();
        let name = PathBuf::from(entry.file_name());
        let name = name.file_stem().and_then(|s| s.to_str()).expect("bad card name");
        println!("loading card {}", path.display());
        let card_ron = std::fs::read_to_string(path)?;
        let card: config::Card = ron::from_str(&card_ron)
            .map_err(|e| ggez::GameError::ResourceLoadError(
                format!("could not deserialize card: {}", e)
            ))?;
        let image = renderer.render_card(ctx, &card)?;
        let texture = engine::Texture::new(textures.len() as u32);
        textures.push(image);
        cards.insert(name.to_owned(), card::Card {
            id: name.to_owned(),
            texture,
            effect: convert_effect(&card.effect),
        });
    }
    Ok(cards)
}

fn load_decks(cards: &HashMap<String, card::Card>) -> card::Decks {
    let decks = std::fs::read_to_string("./data/decks.ron").unwrap();
    let config: config::Decks = ron::from_str(&decks).unwrap();
    let mut draw = Vec::new();
    let mut trap = Vec::new();
    let mut treasure = Vec::new();
    for (id, &count) in &config.draw {
        if let Some(card) = cards.get(id) {
            for _ in 0..count {
                draw.push(card.clone());
            }
        } else {
            panic!("card not defined: {}", id);
        }
    }
    for (id, &count) in &config.trap {
        if let Some(card) = cards.get(id) {
            for _ in 0..count {
                trap.push(card.clone());
            }
        } else {
            panic!("card not defined: {}", id);
        }
    }
    for (id, &count) in &config.treasure {
        if let Some(card) = cards.get(id) {
            for _ in 0..count {
                treasure.push(card.clone());
            }
        } else {
            panic!("card not defined: {}", id);
        }
    }
    let boss = if let Some(card) = cards.get(&config.boss) {
        card.clone()
    } else {
        panic!("card not defined: {}", config.boss);
    };
    card::Decks { draw, trap, treasure, boss }
}

struct CardRenderer<'a> {
    icons: &'a Image,
    base: &'a Image,
}

fn icon_index(icon: &str) -> u32 {
    match icon {
        "sword" => 4,
        "heart" => 5,
        "shield" => 6,
        "beholder" => 8,
        "coin" => 12,
        "cross" => 13,
        "bang" => 14,
        "blue-beholder" => 16,
        "green-heart" => 17,
        "broken" => 18,
        "disarm" => 22,
        "red-sword" => 23,
        _ => panic!("invalid icon: {:?}", icon)
    }
}

fn convert_effect(effect: &config::CardEffect) -> card::CardEffect {
    match *effect {
        config::CardEffect::None => {
            card::CardEffect::None
        }
        config::CardEffect::Enemy { ref icon, attack, health, coins, health_reward, ref buff_rewards } => {
            card::CardEffect::Enemy(card::Creature {
                icon: engine::Icon::new(icon_index(icon)),
                health,
                max_health: None,
                attack,
                coins,
                health_reward: health_reward.unwrap_or(0),
                buff_rewards: buff_rewards.iter().flat_map(|x| x.iter()).map(convert_buff).collect(),
                weapon: None,
                buffs: Vec::new(),
            })
        }
        config::CardEffect::Buff(ref buff) => {
            card::CardEffect::Buff(convert_buff(buff))
        }
        config::CardEffect::Heal { health } => {
            card::CardEffect::Heal { health }
        }
        config::CardEffect::HealEnemy { health } => {
            card::CardEffect::HealEnemy { health }
        }
        config::CardEffect::Weapon { ref icon, damage, durability, price } => {
            card::CardEffect::Weapon(card::Weapon {
                icon: engine::Icon::new(icon_index(icon)),
                damage,
                durability,
                price,
            })
        }
        config::CardEffect::Disarm => card::CardEffect::Disarm,
        config::CardEffect::BossBuff(ref buff) => card::CardEffect::BossBuff(convert_buff(buff)),
    }
}

fn convert_buff(buff: &config::Buff) -> card::Buff {
    match *buff {
        config::Buff::NextAttackBonus { bonus } => {
            card::Buff {
                icon: engine::Icon::SWORD,
                kind: card::BuffKind::NextAttackBonus { damage: bonus },
            }
        }
        config::Buff::AttackBonus { bonus } => {
            card::Buff {
                icon: engine::Icon::SWORD,
                kind: card::BuffKind::AttackBonus { damage: bonus },
            }
        }
    }
}

impl CardRenderer<'_> {
    fn render_card(&mut self, ctx: &mut Context, config: &config::Card) -> GameResult<Image> {
        let card_render = ggez::graphics::Canvas::new(
            ctx,
            CARD_WIDTH,
            CARD_HEIGHT,
            ggez::conf::NumSamples::One,
        )?;
        ggez::graphics::set_canvas(ctx, Some(&card_render));
        ggez::graphics::set_screen_coordinates(ctx, Rect {
            x: 0.0,
            y: 0.0,
            w: f32::from(CARD_WIDTH),
            h: f32::from(CARD_HEIGHT),
        })?;
        ggez::graphics::draw(ctx, self.base, ggez::graphics::DrawParam::new()
            .src(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 })
            .dest([0.0, 0.0]))?;
        let icon = icon_index(&config.icon);
        let col = icon % 8;
        let row = icon / 8;
        ggez::graphics::draw(ctx, self.icons, ggez::graphics::DrawParam::new()
            .src(Rect {
                x: col as f32 / 8.0,
                y: row as f32 / 8.0,
                w: 1.0 / 8.0,
                h: 1.0 / 8.0,
            })
            .dest([(f32::from(CARD_WIDTH) / 2.0 - 80.0), 80.0])
            .scale([10.0, 10.0]))?; // 160x160
        let title = Text::new(TextFragment::new(config.title.as_str())
            .scale(Scale { x: 60.0, y: 60.0 }));
        let width = title.width(ctx);
        let x = (f32::from(CARD_WIDTH) - width as f32) / 2.0;
        ggez::graphics::draw(ctx, &title, ggez::graphics::DrawParam::new()
            .dest([x, 20.0])
            .color(ggez::graphics::BLACK))?;
        let mut y = 70.0 + 160.0 + 10.0;
        for line in &config.description {
            let line = Text::new(TextFragment::new(line.as_str())
                .scale(Scale { x: 50.0, y: 50.0 }));
            let width = line.width(ctx);
            let x = (f32::from(CARD_WIDTH) - width as f32) / 2.0;
            ggez::graphics::draw(ctx, &line, ggez::graphics::DrawParam::new()
                .dest([x, y])
                .color(ggez::graphics::BLACK))?;
            y += line.height(ctx) as f32;
        }
        ggez::graphics::present(ctx)?;
        ggez::graphics::set_canvas(ctx, None);
        unwrap_canvas(ctx, card_render)
    }
}

fn render_card_back(ctx: &mut Context, base: &Image, back: &Image) -> GameResult<Image> {
    let card_render = ggez::graphics::Canvas::new(
        ctx,
        CARD_WIDTH,
        CARD_HEIGHT,
        ggez::conf::NumSamples::One,
    )?;
    ggez::graphics::set_canvas(ctx, Some(&card_render));
    ggez::graphics::set_screen_coordinates(ctx, Rect {
        x: 0.0,
        y: 0.0,
        w: f32::from(CARD_WIDTH),
        h: f32::from(CARD_HEIGHT),
    })?;
    ggez::graphics::draw(ctx, base, ggez::graphics::DrawParam::new()
        .src(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 })
        .dest([0.0, 0.0]))?;
    ggez::graphics::draw(ctx, back, ggez::graphics::DrawParam::new()
        .src(Rect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 })
        .dest([0.0, 0.0]))?;
    ggez::graphics::present(ctx)?;
    ggez::graphics::set_canvas(ctx, None);
    unwrap_canvas(ctx, card_render)
}

fn unwrap_canvas(ctx: &mut Context, canvas: Canvas) -> GameResult<Image> {
    let image = canvas.into_inner();
    let mut data = image.to_rgba8(ctx)?;
    let (first, second) = data.split_at_mut(usize::from(image.width()) * usize::from(image.height()) * 2);
    let first = first.chunks_mut(usize::from(image.width() * 4));
    let second = second.chunks_mut(usize::from(image.width() * 4));
    for (a, b) in first.zip(second.rev()) {
        let mut c = [0; (CARD_WIDTH * 4) as usize];
        c.copy_from_slice(a);
        a.copy_from_slice(b);
        b.copy_from_slice(&c);
    }
    let image = Image::from_rgba8(ctx, image.width(), image.height(), &data)?;
    Ok(image)
}

fn make_transparent(ctx: &mut Context, image: &Image) -> GameResult<Image> {
    let mut pixels = image.to_rgba8(ctx)?;
    for pixel in pixels.chunks_mut(4) {
        if pixel == &[163, 73, 164, 255] || pixel == &[200, 191, 231, 255] {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            pixel[3] = 0;
        }
    }
    Image::from_rgba8(
        ctx,
        image.width(),
        image.height(),
        &pixels,
    )
}
