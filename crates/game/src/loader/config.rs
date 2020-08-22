use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Icon {
    Circle,
    Dot,
    Square,
    Sword,
    Heart,
    Shield,
    Fighter,
    Beholder,
    Card,
    Play,
    CardBack,
    Coin,
    Cross,
    Bang,
    RedCircle,
    BlueBeholder,
    GreenHeart,
    Broken,
    Deck,
    TrapDeck,
    Black,
    Disarm,
    RedSword,
    Bow,
    Fighter2,
    Chicken,
}

impl From<Icon> for engine::Icon {
    fn from(icon: Icon) -> engine::Icon {
        match icon {
            Icon::Circle => engine::Icon::CIRCLE,
            Icon::Dot => engine::Icon::DOT,
            Icon::Square => engine::Icon::SQUARE,
            Icon::Sword => engine::Icon::SWORD,
            Icon::Heart => engine::Icon::HEART,
            Icon::Shield => engine::Icon::SHIELD,
            Icon::Fighter => engine::Icon::FIGHTER,
            Icon::Beholder => engine::Icon::BEHOLDER,
            Icon::Card => engine::Icon::CARD,
            Icon::Play => engine::Icon::PLAY,
            Icon::CardBack => engine::Icon::CARD_BACK,
            Icon::Coin => engine::Icon::COIN,
            Icon::Cross => engine::Icon::CROSS,
            Icon::Bang => engine::Icon::BANG,
            Icon::RedCircle => engine::Icon::RED_CIRCLE,
            Icon::BlueBeholder => engine::Icon::BLUE_BEHOLDER,
            Icon::GreenHeart => engine::Icon::GREEN_HEART,
            Icon::Broken => engine::Icon::BROKEN,
            Icon::Deck => engine::Icon::DECK,
            Icon::TrapDeck => engine::Icon::TRAP_DECK,
            Icon::Black => engine::Icon::BLACK,
            Icon::Disarm => engine::Icon::DISARM,
            Icon::RedSword => engine::Icon::RED_SWORD,
            Icon::Bow => engine::Icon::BOW,
            Icon::Fighter2 => engine::Icon::FIGHTER_2,
            Icon::Chicken => engine::Icon::CHICKEN,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Card {
    pub icon: Icon,
    pub title: String,
    pub description: Vec<String>,
    pub effect: CardEffect,
}

#[derive(Deserialize, Debug)]
pub enum CardEffect {
    None,
    Enemy {
        icon: Icon,
        attack: u32,
        health: u32,
        #[serde(default)]
        max_health: Option<u32>,
        #[serde(default)]
        rewards: Vec<CardEffect>,
    },
    Buff {
        icon: Icon,
        kind: Buff,
        expiration: BuffExpiration,
    },
    BossBuff {
        icon: Icon,
        kind: Buff,
        expiration: BuffExpiration,
    },
    Heal {
        health: u32,
    },
    Armor {
        amount: u32,
    },
    Coins {
        amount: u32,
    },
    Attack {
        use_base: bool,
        bonus: u32,
    },
    HealEnemy {
        health: u32,
    },
    Weapon {
        icon: Icon,
        damage: u32,
        durability: u32,
    },
    Buy {
        price: u32,
        effect: Box<CardEffect>,
    },
    Disarm,
}

#[derive(Deserialize, Debug)]
pub enum Buff {
    AttackBonus { bonus: u32 },
    /// Applied to owner after it attacks something
    OnAttack { effect: Box<CardEffect> },
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum BuffExpiration {
    Permanent,
    AfterAttack,
    AfterBeingHit,
}

#[derive(Deserialize, Debug)]
pub struct Decks {
    pub draw: HashMap<String, u32>,
    pub trap: HashMap<String, u32>,
    pub treasure: HashMap<String, u32>,
    pub boss: String,
}
