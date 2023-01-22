use std::collections::BTreeMap;

use runo::{
    card::{Card, CardColor, ColoredCard, PlayedCard},
    turn::{PlayAction, TurnAction, TurnActionResult},
    uno::{PlayTurnResult, Uno},
};

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
fn play_turn_works_if_card_in_hand() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We add a "Red 1" card to the player so that we can test for it below
    player.hand[0] = Card::Colored(CardColor::Red, ColoredCard::Number(1));

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(CardColor::Red, ColoredCard::Number(1)),
    )));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Neutral,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::Colored(CardColor::Red, ColoredCard::Number(1))
    ));
}

#[test]
fn play_turn_fails_if_card_not_in_hand() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // Change every "Red 1" card to "Green 1" so that we can later test that a "Red 1" card
    // does not exist.
    for card in &mut player.hand {
        if let Card::Colored(color, colored_card) = card {
            if color == &CardColor::Red {
                if let ColoredCard::Number(1) = colored_card {
                    *card = Card::Colored(CardColor::Green, ColoredCard::Number(1));
                }
            }
        }
    }

    let last_played_card_before = uno.get_last_played_card().clone();

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(CardColor::Red, ColoredCard::Number(1)),
    )));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::CardNotInHand,
            won: false
        }
    );

    let last_played_card = uno.get_last_played_card();

    assert_eq!(last_played_card, &last_played_card_before);
}

#[test]
fn play_turn_skips_player_properly() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();
    let current_turn_player_id = uno.get_current_turn_player_id();
    let expected_next_player_id = if current_turn_player_id == 2 {
        0
    } else if current_turn_player_id == 3 {
        1
    } else {
        current_turn_player_id + 2
    };

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We change the first card of the player to "Red Skip" so we can test for it below
    player.hand[0] = Card::Colored(CardColor::Red, ColoredCard::Skip);

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(CardColor::Red, ColoredCard::Skip),
    )));

    assert_eq!(uno.get_current_turn_player_id(), expected_next_player_id);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Skip,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::Colored(CardColor::Red, ColoredCard::Skip)
    ));
}

#[test]
fn play_turn_performs_reverse_properly() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

    let current_turn_player_id = uno.get_current_turn_player_id();
    let prev_player_id = if current_turn_player_id == 0 {
        3
    } else {
        current_turn_player_id - 1
    };

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We change the first card of the player to "Green Reverse" so we can test for it below
    player.hand[0] = Card::Colored(CardColor::Green, ColoredCard::Reverse);

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(CardColor::Green, ColoredCard::Reverse),
    )));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Reverse,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::Colored(CardColor::Green, ColoredCard::Reverse)
    ));

    assert_eq!(uno.get_next_turn_player_id(), prev_player_id);
}

#[test]
fn play_turn_performs_draw_properly() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();
    let current_turn_player_id = uno.get_current_turn_player_id();
    let expected_next_player_id = if current_turn_player_id == 3 {
        0
    } else {
        current_turn_player_id + 1
    };

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We change the first card of the player to "Green Draw" so we can test for it below
    player.hand[0] = Card::Colored(CardColor::Green, ColoredCard::Draw);

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(CardColor::Green, ColoredCard::Draw),
    )));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Draw,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::Colored(CardColor::Green, ColoredCard::Draw)
    ));

    let next_player_id = uno.get_current_turn_player_id();

    assert_eq!(next_player_id, expected_next_player_id);

    let next_player = uno
        .get_player_mut(&next_player_id)
        .expect("The next player has disappeared.");

    assert_eq!(next_player.cards_count(), 9);
}

#[test]
fn play_turn_performs_wild_properly() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();
    let current_turn_player_id = uno.get_current_turn_player_id();
    let expected_next_player_id = if current_turn_player_id == 3 {
        0
    } else {
        current_turn_player_id + 1
    };

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We change the first card of the player to "Wild" so we can test for it below
    player.hand[0] = Card::Wild;

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::Wild(CardColor::Red)));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Wild,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::Wild(CardColor::Red)
    ));

    let next_player_id = uno.get_current_turn_player_id();

    assert_eq!(next_player_id, expected_next_player_id);
}

#[test]
fn play_turn_performs_wild_draw_properly() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();
    let current_turn_player_id = uno.get_current_turn_player_id();
    let expected_next_player_id = if current_turn_player_id == 3 {
        0
    } else {
        current_turn_player_id + 1
    };

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    // We change the first card of the player to "Green Draw" so we can test for it below
    player.hand[0] = Card::WildDraw;

    let turn_action_result =
        uno.play_turn(TurnAction::Play(PlayAction::WildDraw(CardColor::Yellow)));

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::WildDraw,
            won: false
        }
    );

    assert!(matches!(
        uno.get_last_played_card(),
        PlayedCard::WildDraw(CardColor::Yellow)
    ));

    let next_player_id = uno.get_current_turn_player_id();

    assert_eq!(next_player_id, expected_next_player_id);

    let next_player = uno
        .get_player_mut(&next_player_id)
        .expect("The next player has disappeared.");

    assert_eq!(next_player.cards_count(), 11);
}

#[test]
fn turn_uno_works_if_only_one_card() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    player.hand.truncate(1);

    let turn_action_result = uno.play_turn(TurnAction::Uno);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::UnoSuccessful,
            won: false
        }
    );
}

#[test]
fn turn_uno_does_not_work_if_more_than_one_card() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let player = uno
        .get_player_mut(&uno.get_current_turn_player_id())
        .expect("Current player must exist.");

    player.hand.truncate(4);

    let turn_action_result = uno.play_turn(TurnAction::Uno);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::UnoFailed,
            won: false
        }
    );
}

#[test]
fn turn_callout_works_if_players_eligible() {
    let mut uno = Uno::new_with_ids(create_players_info(4)).unwrap();

    let next_player = uno.get_player_mut(&1).expect("Next player must exist.");
    next_player.hand.truncate(1);

    let next_next_player = uno.get_player_mut(&2).expect("Next player must exist.");
    next_next_player.hand.truncate(1);

    let turn_action_result = uno.play_turn(TurnAction::Callout);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::CalledOut(vec![1, 2]),
            won: false
        }
    );
}

#[test]
fn turn_callout_does_not_work_if_no_players_eligible() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let turn_action_result = uno.play_turn(TurnAction::Callout);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::CalloutFailed,
            won: false
        }
    );
}

#[test]
fn turn_winning_works_properly() {
    let mut uno = Uno::new(create_player_names(4)).unwrap();

    let color = match uno.get_last_played_card() {
        PlayedCard::Colored(color, _) => color,
        PlayedCard::Wild(color) => color,
        PlayedCard::WildDraw(color) => color,
    }
    .clone();

    let current_player_id = uno.get_current_turn_player_id();

    let player = uno
        .get_player_mut(&current_player_id)
        .expect("Current player must exist.");
    player.hand.truncate(0);
    player.add_card(Card::Colored(color.clone(), ColoredCard::Skip));

    let turn_action_result = uno.play_turn(TurnAction::Play(PlayAction::ColoredCard(
        Card::Colored(color, ColoredCard::Skip),
    )));

    let player = uno
        .get_player_mut(&current_player_id)
        .expect("Current player must exist.");
    assert_eq!(player.cards_count(), 0);

    assert_eq!(
        turn_action_result,
        PlayTurnResult {
            turn_action_result: TurnActionResult::Skip,
            won: true
        }
    );
}
