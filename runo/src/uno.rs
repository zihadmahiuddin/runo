use std::{collections::BTreeMap, fmt::Debug};

use rand::{thread_rng, Rng};

use crate::card::{Card, ColoredCard, PlayedCard};
use crate::deck::Deck;
use crate::error::{Result, UnoError};
use crate::player::Player;
use crate::turn::{PlayAction, TurnAction, TurnActionResult};

#[derive(Debug)]
pub struct Uno {
    deck: Deck,
    players: BTreeMap<u64, Player>,
    winners: Vec<Player>,
    current_turn_player_id_index: usize,
    player_order_reversed: bool,
    last_played_card: PlayedCard,
}

impl Uno {
    pub fn new(player_names: Vec<String>) -> Result<Self> {
        let mut rng = thread_rng();
        let mut players_info = BTreeMap::new();

        for player_name in player_names {
            loop {
                let id = rng.gen();
                if let std::collections::btree_map::Entry::Vacant(e) = players_info.entry(id) {
                    e.insert(player_name);
                    break;
                }
            }
        }

        Self::new_with_ids(players_info)
    }

    pub fn new_with_ids(players_info: BTreeMap<u64, String>) -> Result<Self> {
        if players_info.len() < 2 {
            return Err(UnoError::NotEnoughPlayers);
        }
        if players_info.len() > 10 {
            return Err(UnoError::TooManyPlayers);
        }

        let mut deck = Deck::new();
        let mut players = BTreeMap::new();

        deck.shuffle();

        for (player_id, player_name) in players_info {
            let cards = deck.draw_cards(7);
            let player = Player::new(player_id, player_name, cards);
            players.insert(player_id, player);
        }

        let winners = Vec::with_capacity(players.len());

        let _player_ids = players.keys().copied().collect::<Vec<_>>();
        // let current_turn_player_id_index = thread_rng().gen_range(0..player_ids.len());
        let current_turn_player_id_index = 0;

        let Card::Colored(color, last_played_card) = deck
            .draw_colored_card()
            .expect("There is always at least one card at this point.") else {
            panic!("Expected to get a colored card.");
        };

        let last_played_card = last_played_card.into_played_card(color);

        Ok(Uno {
            deck,
            players,
            winners,
            current_turn_player_id_index,
            last_played_card,
            player_order_reversed: false,
        })
    }

    pub fn play_turn(&mut self, turn_action: TurnAction) -> (TurnActionResult, bool) {
        let current_turn_player_id = self.get_current_turn_player_id();

        let player = self
            .players
            .get_mut(&current_turn_player_id)
            .expect("The player with the current turn must always exist.");

        let turn_action_result = match turn_action {
            TurnAction::Play(play_action) => match play_action {
                PlayAction::ColoredCard(card) => {
                    if let Some(hand_card_index) = player.card_index(&card) {
                        player.remove_card(hand_card_index);
                        match card {
                            Card::Colored(color, card) => {
                                let result = match card {
                                    ColoredCard::Skip => {
                                        self.move_turn_n_players_ahead(2);
                                        TurnActionResult::Skip
                                    }
                                    ColoredCard::Reverse => {
                                        self.perform_reverse();
                                        TurnActionResult::Reverse
                                    }
                                    ColoredCard::Draw => {
                                        self.draw_cards_to_player(
                                            &self.get_next_turn_player_id(),
                                            2,
                                        );
                                        self.move_turn_n_players_ahead(1);
                                        TurnActionResult::Draw
                                    }
                                    ColoredCard::Number(_) => {
                                        self.move_turn_n_players_ahead(1);
                                        TurnActionResult::Neutral
                                    }
                                };
                                self.last_played_card = card.into_played_card(color);
                                result
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        TurnActionResult::CardNotInHand
                    }
                }
                PlayAction::Wild(color) => {
                    self.last_played_card = PlayedCard::Wild(color);
                    self.move_turn_n_players_ahead(1);
                    TurnActionResult::Wild
                }
                PlayAction::WildDraw(color) => {
                    self.last_played_card = PlayedCard::WildDraw(color);
                    self.draw_cards_to_player(&self.get_next_turn_player_id(), 4);
                    self.move_turn_n_players_ahead(1);
                    TurnActionResult::WildDraw
                }
            },
            TurnAction::Callout => {
                let called_out_player_ids = self.perform_callout();
                if called_out_player_ids.is_empty() {
                    TurnActionResult::CalloutFailed
                } else {
                    TurnActionResult::CalledOut(called_out_player_ids)
                }
            }
            TurnAction::Uno => {
                if self.perform_uno() {
                    TurnActionResult::UnoSuccessful
                } else {
                    TurnActionResult::UnoFailed
                }
            }
            TurnAction::Draw => {
                self.draw_cards_to_player(&current_turn_player_id, 2);
                TurnActionResult::SelfDraw
            }
        };

        let player = self
            .players
            .get(&current_turn_player_id)
            .expect("The player with the current turn must always exist.");

        let won = player.cards_count() == 0;
        if won {
            self.winners.push(
                self.players
                    .remove(&current_turn_player_id)
                    .expect("The player just won."),
            );
        }

        (turn_action_result, won)
    }

    pub fn get_player_ids(&self) -> Vec<u64> {
        self.players.keys().copied().collect()
    }

    pub fn get_player(&self, player_id: &u64) -> Option<&Player> {
        self.players.get(player_id)
    }

    pub fn get_player_mut(&mut self, player_id: &u64) -> Option<&mut Player> {
        self.players.get_mut(player_id)
    }

    pub fn get_current_turn_player_id(&self) -> u64 {
        self.get_nth_turn_player_id(0)
    }

    pub fn get_next_turn_player_id(&self) -> u64 {
        self.get_nth_turn_player_id(1)
    }

    pub fn get_last_played_card(&self) -> &PlayedCard {
        &self.last_played_card
    }

    fn draw_cards_to_player(&mut self, player_id: &u64, count: usize) {
        let cards = self.deck.draw_cards(count);

        let player = self
            .get_player_mut(player_id)
            .expect("Player has disappeared...");

        for card in cards {
            player.add_card(card);
        }
    }

    fn perform_reverse(&mut self) {
        let current_turn_player_id = self.get_current_turn_player_id();
        self.player_order_reversed = !self.player_order_reversed;
        let reversed_player_ids = self.get_order_aware_player_ids();
        self.current_turn_player_id_index = reversed_player_ids
            .iter()
            .position(|x| x == &&current_turn_player_id)
            .expect("This must exist since only the order has been reversed, nothing was removed.");
    }

    fn perform_callout(&mut self) -> Vec<u64> {
        let mut called_out_player_ids = vec![];
        let current_turn_player_id = self.get_current_turn_player_id();

        for player in self.players.values() {
            if player.id == current_turn_player_id {
                continue;
            }

            if player.cards_count() == 1 {
                called_out_player_ids.push(player.id);
            }
        }

        if called_out_player_ids.is_empty() {
            self.draw_cards_to_player(&current_turn_player_id, 2);
        } else {
            for called_out_player_id in &called_out_player_ids {
                self.draw_cards_to_player(called_out_player_id, 2);
            }
        }

        called_out_player_ids
    }

    fn perform_uno(&mut self) -> bool {
        let current_turn_player_id = self.get_current_turn_player_id();
        let current_player = self
            .get_player_mut(&current_turn_player_id)
            .expect("Current player must always exist.");

        if current_player.cards_count() == 1 {
            current_player.uno();
            true
        } else {
            self.draw_cards_to_player(&current_turn_player_id, 2);
            false
        }
    }

    fn get_order_aware_player_ids(&self) -> Vec<&u64> {
        let keys_iter = self.players.keys();
        let player_ids: Vec<&u64> = if self.player_order_reversed {
            keys_iter.rev().collect()
        } else {
            keys_iter.collect()
        };
        player_ids
    }

    fn get_nth_turn_player_id(&self, n: usize) -> u64 {
        let player_ids = self.get_order_aware_player_ids();
        return **player_ids
            .iter()
            .cycle()
            .nth(self.current_turn_player_id_index + n)
            .expect("Cycle always returns something...right?");
    }

    fn move_turn_n_players_ahead(&mut self, n: usize) {
        for _ in 0..n {
            self.current_turn_player_id_index = if self.player_order_reversed {
                if self.current_turn_player_id_index == 0 {
                    self.players.len() - 1
                } else {
                    self.current_turn_player_id_index - 1
                }
            } else if self.current_turn_player_id_index == self.players.len() - 1 {
                0
            } else {
                self.current_turn_player_id_index + 1
            };
        }
    }

    // fn get_random_player_id(players: &BTreeMap<u64, Player>) -> u64 {
    //     let mut rng = thread_rng();
    //     players
    //         .keys()
    //         .choose(&mut rng)
    //         .map(|x| *x)
    //         .expect("A match without players?")
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_player_names(count: usize) -> Vec<String> {
        let mut player_names = Vec::new();
        for i in 0..count {
            player_names.push(format!("Player {}", i + 1));
        }
        player_names
    }

    fn create_players_info(count: usize) -> BTreeMap<u64, String> {
        let mut players_info = BTreeMap::new();
        for i in 0..count {
            players_info.insert(i as u64, format!("Player {}", i + 1));
        }
        players_info
    }

    #[test]
    fn return_ok_if_enough_players() {
        let result = Uno::new(create_player_names(2));
        assert!(matches!(result, Result::Ok(_)));
    }

    #[test]
    fn return_err_if_not_enough_players() {
        let error = Uno::new(create_player_names(1)).unwrap_err();
        assert!(matches!(error, UnoError::NotEnoughPlayers));
    }

    #[test]
    fn return_err_if_too_many_players() {
        let error = Uno::new(create_player_names(11)).unwrap_err();
        assert!(matches!(error, UnoError::TooManyPlayers));
    }

    #[test]
    fn all_players_start_with_7_cards() {
        let uno = Uno::new(create_player_names(4)).unwrap();
        for (_id, player) in uno.players {
            assert_eq!(player.cards_count(), 7);
        }
    }

    #[test]
    fn next_player_turn_works_when_first_player() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        // Set current turn to first player, random by default
        uno.current_turn_player_id_index = 0;

        uno.move_turn_n_players_ahead(1);

        assert_eq!(uno.get_current_turn_player_id(), 1);
    }

    #[test]
    fn next_player_turn_works_when_last_player() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        // Set current turn to last player, random by default
        uno.current_turn_player_id_index = 3;

        uno.move_turn_n_players_ahead(1);

        assert_eq!(uno.get_current_turn_player_id(), 0);
    }

    #[test]
    fn next_player_turn_works_when_other_player() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        // Set current turn to some other player, random by default
        uno.current_turn_player_id_index = 1;

        uno.move_turn_n_players_ahead(1);

        assert_eq!(uno.get_current_turn_player_id(), 2);
    }

    #[test]
    fn skip_player_turn_works() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        // Set current turn to first player, random by default
        uno.current_turn_player_id_index = 0;

        uno.move_turn_n_players_ahead(2);

        assert_eq!(uno.current_turn_player_id_index, 2);
    }

    #[test]
    fn perform_reverse_works() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        // Set current turn to first player, random by default
        uno.current_turn_player_id_index = 0;

        uno.perform_reverse();

        assert_eq!(uno.current_turn_player_id_index, 3);
    }

    #[test]
    fn perform_uno_does_not_work_unless_one_card_left() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        let uno_successful = uno.perform_uno();

        let player = uno
            .get_player(&uno.get_current_turn_player_id())
            .expect("Player must exist.");

        assert_eq!(player.cards_count(), 9);

        assert!(!uno_successful);
    }

    #[test]
    fn perform_uno_works_if_one_card_left() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();
        let player = uno
            .get_player_mut(&uno.get_current_turn_player_id())
            .expect("Player must exist.");

        player.hand.truncate(1);

        assert!(uno.perform_uno())
    }

    #[test]
    fn perform_callout_does_not_work_if_no_players_can_be_called_out() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        let called_out_player_ids = uno.perform_callout();

        let player = uno
            .get_player(&uno.get_current_turn_player_id())
            .expect("Player must exist.");

        assert_eq!(player.cards_count(), 9);

        assert_eq!(called_out_player_ids.len(), 0);

        let next_player = uno
            .get_player(&uno.get_next_turn_player_id())
            .expect("Next player must exist.");
        assert_eq!(next_player.cards_count(), 7);
    }

    #[test]
    fn perform_callout_works_if_players_can_be_called_out() {
        let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

        let next_player = uno
            .get_player_mut(&uno.get_next_turn_player_id())
            .expect("Next player must exist.");
        next_player.hand.truncate(1);

        assert_eq!(uno.perform_callout().len(), 1);

        let player = uno
            .get_player(&uno.get_current_turn_player_id())
            .expect("Player must exist.");
        assert_eq!(player.cards_count(), 7);

        let next_player = uno
            .get_player(&uno.get_next_turn_player_id())
            .expect("Next player must exist.");
        assert_eq!(next_player.cards_count(), 3);
    }
}
