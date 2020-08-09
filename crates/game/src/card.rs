use engine::{Icon, Texture};

#[derive(Debug, Clone)]
pub struct Weapon {
    pub icon: Icon,
    pub damage: u32,
    pub durability: u32,
    pub price: u32,
}

#[derive(Debug, Clone)]
pub enum BuffKind {
    NextAttackBonus { damage: u32 },
}

#[derive(Debug, Clone)]
pub struct Buff {
    pub icon: Icon,
    pub kind: BuffKind,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub icon: Icon,
    pub health: u32,
    pub max_health: Option<u32>,
    pub attack: u32,
    pub coins: u32,
    pub weapon: Option<Weapon>,
    pub buffs: Vec<Buff>,
}

impl Creature {
    pub fn heal(&mut self, amount: u32) {
        self.health += amount;
        if let Some(limit) = self.max_health {
            self.health = std::cmp::min(self.health, limit);
        }
    }
}

#[derive(Debug, Clone)]
pub enum CardEffect {
    None,
    Heal { health: u32 },
    HealEnemy { health: u32 },
    Buff(Buff),
    Weapon(Weapon),
    Enemy(Creature),
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
    pub texture: Texture,
    pub effect: CardEffect,
}

#[derive(Debug, Clone)]
pub struct Decks {
    pub draw: Vec<Card>,
    pub trap: Vec<Card>,
    pub boss: Card,
}
