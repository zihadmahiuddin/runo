use convert_case::{Case, Converter};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentType, Context, CreateComponents, Interaction,
        InteractionResponseType, MessageComponentInteraction,
    },
    Event,
};
use runo::{
    card::{Card, CardColor},
    turn::{PlayAction, TurnAction},
    uno::Uno,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use crate::{Data, UnoGame};

use super::select_menu::{CardSelectMenu, ColorSelectMenu, SelectMenu};

#[derive(Debug, Display, EnumString, EnumIter)]
pub enum UnoButton {
    PlayCard,
    ViewHand,
    Draw,
    Uno,
    Callout,
}

impl UnoButton {
    fn custom_id(&self) -> String {
        let converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Snake);
        converter.convert(format!("{self}"))
    }

    fn label(&self) -> String {
        let converter = Converter::new()
            .from_case(Case::Pascal)
            .to_case(Case::Title);
        converter.convert(format!("{self}"))
    }
    pub fn create_action_row(c: &mut CreateComponents) -> &mut CreateComponents {
        c.create_action_row(|ar| {
            for (index, variant) in Self::iter().enumerate() {
                ar.create_button(|b| {
                    b.label(variant.label())
                        .style(if index == 0 {
                            ButtonStyle::Primary
                        } else {
                            ButtonStyle::Secondary
                        })
                        .custom_id(variant.custom_id())
                });
            }
            ar
        })
    }

    async fn process(ctx: &Context, interaction: &MessageComponentInteraction, data: &Data) {
        let Some(button_type) = Self::iter().find(|x| x.custom_id() == interaction.data.custom_id) else {
            return;
        };
        let mut matches = data.matches.lock().await;

        let Some(game) = matches.get_mut(&interaction.channel_id) else {
            return;
        };

        match game {
            UnoGame::Pending { .. } => todo!(),
            UnoGame::Ongoing { game, .. } => match button_type {
                Self::PlayCard => handle_play_card(ctx, interaction, game).await,
                Self::ViewHand => handle_view_hand(ctx, interaction, game).await,
                Self::Draw => handle_draw(ctx, interaction, game).await,
                Self::Uno => handle_say_uno(ctx, interaction, game).await,
                Self::Callout => handle_callout(ctx, interaction, game).await,
            },
        };
    }

    pub async fn handle_event<'a>(ctx: &Context, event: &Event<'_>, data: &Data) {
        if let Event::InteractionCreate {
            interaction: Interaction::MessageComponent(component_interaction),
        } = event
        {
            if let ComponentType::Button = component_interaction.data.component_type {
                Self::process(ctx, component_interaction, data).await
            }
        }
    }
}

async fn handle_play_card(
    ctx: &Context,
    interaction: &MessageComponentInteraction,
    game: &mut Uno,
) {
    let Some(player) = game.get_player(&interaction.user.id.0) else {
        return
    };

    let player_cards = player.hand.clone();
    let mut card_select_menu = CardSelectMenu::new(player_cards.as_slice());
    let interaction = card_select_menu
        .await_selection(ctx, interaction)
        .await
        .unwrap()
        .unwrap();

    let chosen_card = card_select_menu.get_selection().unwrap();

    match chosen_card {
        Card::Colored(_, _) => {
            let result = game.play_turn(TurnAction::Play(PlayAction::ColoredCard(
                chosen_card.clone(),
            )));

            let delete_original_response = interaction.delete_original_interaction_response(ctx);

            let create_followup_message = interaction.create_followup_message(ctx, |irf| {
                irf.content(format!("You chose: {}, result: {:?}", chosen_card, result))
                    .ephemeral(true)
            });

            let (delete_original_response, create_followup_message) =
                tokio::join!(delete_original_response, create_followup_message,);
            delete_original_response.unwrap();
            create_followup_message.unwrap();
        }
        _ => {
            let colors = CardColor::iter().collect::<Vec<_>>();

            let mut color_select_menu = ColorSelectMenu::new(&colors);
            color_select_menu
                .await_selection(ctx, &interaction)
                .await
                .unwrap();
            let color = color_select_menu.get_selection().unwrap();

            let play_action = match chosen_card {
                Card::Colored(_, _) => unreachable!(),
                Card::Wild => PlayAction::WildDraw(color.clone()),
                Card::WildDraw => PlayAction::WildDraw(color.clone()),
            };

            let result = game.play_turn(TurnAction::Play(play_action));
            interaction
                .create_interaction_response(ctx, |ir| {
                    ir.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|ird| {
                            ird.content(format!("You chose: {}, result: {:?}", chosen_card, result))
                                .ephemeral(true)
                        })
                })
                .await
                .unwrap();
        }
    }
}

async fn handle_view_hand(ctx: &Context, interaction: &MessageComponentInteraction, game: &Uno) {
    let Some(player) = game.get_player(&interaction.user.id.0) else {
        return
    };

    let cards_string = player
        .hand
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    interaction
        .create_interaction_response(ctx, |ir| {
            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|ird| {
                    ird.content(format!("Your hand: {cards_string}."))
                        .ephemeral(true)
                })
        })
        .await
        .unwrap();
}

async fn handle_draw(ctx: &Context, interaction: &MessageComponentInteraction, game: &mut Uno) {
    let result = game.play_turn(TurnAction::Draw);
    interaction
        .create_interaction_response(ctx, |ir| {
            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|ird| {
                    ird.content(format!("You chose to draw, result: {:?}", result))
                        .ephemeral(true)
                })
        })
        .await
        .unwrap();
}

async fn handle_say_uno(ctx: &Context, interaction: &MessageComponentInteraction, game: &mut Uno) {
    let result = game.play_turn(TurnAction::Uno);
    interaction
        .create_interaction_response(ctx, |ir| {
            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|ird| {
                    ird.content(format!("You chose to say UNO, result: {:?}", result))
                        .ephemeral(true)
                })
        })
        .await
        .unwrap();
}

async fn handle_callout(ctx: &Context, interaction: &MessageComponentInteraction, game: &mut Uno) {
    let result = game.play_turn(TurnAction::Callout);
    interaction
        .create_interaction_response(ctx, |ir| {
            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|ird| {
                    ird.content(format!("You chose do a callout, result: {:?}", result))
                        .ephemeral(true)
                })
        })
        .await
        .unwrap();
}
