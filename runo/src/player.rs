use crate::card::Card;

#[allow(unused)]
#[derive(Debug)]
pub struct Player {
    pub id: u64,
    name: String,
    pub hand: Vec<Card>,
    pub uno_performed: bool,
}

impl Player {
    pub fn new(id: u64, name: String, cards: Vec<Card>) -> Self {
        Self {
            id,
            name,
            hand: cards,
            uno_performed: false,
        }
    }

    pub fn cards_count(&self) -> usize {
        self.hand.len()
    }

    pub fn card_index(&self, card: &Card) -> Option<usize> {
        self.hand.iter().position(|x| x == card)
    }

    pub fn add_card(&mut self, card: Card) {
        self.hand.push(card);
        self.uno_performed = false;
    }

    pub fn remove_card(&mut self, index: usize) {
        self.hand.remove(index);
        self.uno_performed = false;
    }

    pub fn uno(&mut self) {
        self.uno_performed = true;
    }

    // pub fn uno_performed(&self) -> bool {
    //     self.uno_performed
    // }
}
