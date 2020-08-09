use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Card {
    pub icon: String,
    pub title: String,
    pub description: Vec<String>,
    pub effect: CardEffect,
}

#[derive(Deserialize, Debug)]
pub enum CardEffect {
    None,
    Enemy {
        icon: String,
        attack: u32,
        health: u32,
        coins: u32,
        health_reward: Option<u32>,
        buff_rewards: Option<Vec<Buff>>,
    },
    Buff(Buff),
    BossBuff(Buff),
    Heal {
        health: u32,
    },
    HealEnemy {
        health: u32,
    },
    Weapon {
        icon: String,
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
    NextAttackBonus { bonus: u32 },
    AttackBonus { bonus: u32 },
}

#[derive(Deserialize, Debug)]
pub struct Decks {
    pub draw: HashMap<String, u32>,
    pub trap: HashMap<String, u32>,
    pub treasure: HashMap<String, u32>,
    pub boss: String,
}
