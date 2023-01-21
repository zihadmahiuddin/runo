use std::{marker::PhantomData, str::FromStr, time::Duration};

use convert_case::{Case, Converter};
use poise::{
    serenity_prelude::{
        ButtonStyle, ChannelId, CollectComponentInteraction, ComponentType, Context,
        CreateComponents, CreateInteractionResponse, CreateInteractionResponseData, Interaction,
        InteractionResponseType, MessageComponentInteraction, ReactionType,
    },
    Event,
};
use runo::{
    card::{Card, CardColor},
    player::Player,
    turn::{PlayAction, TurnAction},
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumIter, EnumString};

use crate::{Data, UnoGame};

#[derive(Debug, Display, EnumString, EnumIter)]
#[allow(clippy::upper_case_acronyms)]
enum UnoButtonType {
    PlayCard,
    ViewHand,
    Draw,
    UNO,
    Callout,
}

impl UnoButtonType {
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
}

pub struct UnoButton;

impl UnoButton {
    pub fn create_action_row(c: &mut CreateComponents) -> &mut CreateComponents {
        c.create_action_row(|ar| {
            for (index, variant) in UnoButtonType::iter().enumerate() {
                ar.create_button(|b| {
                    b.label(variant.label())
                        .style(if index == 0 {
                            ButtonStyle::Primary
                        } else {
                            ButtonStyle::Secondary
                        })
                        .custom_id(variant.custom_id())
                    // TODO: move ID to constants or something
                });
            }
            ar
        })
    }

    async fn process(ctx: &Context, interaction: &MessageComponentInteraction, data: &Data) {
        let Some(button_type) = UnoButtonType::iter().find(|x| x.custom_id() == interaction.data.custom_id) else {
            return;
        };
        let mut matches = data.matches.lock().await;

        let Some(game) = matches.get_mut(&interaction.channel_id) else {
            return;
        };

        match button_type {
            UnoButtonType::PlayCard => {
                match game {
                    UnoGame::Ongoing { game, .. } => {
                        let Some(player) = game.get_player(&interaction.user.id.0) else {
                        return;
                    };

                        interaction
                            .create_interaction_response(ctx, |ir| {
                                ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|ird| {
                                        ird.content("Select the card you want to play.")
                                            .components(|c| {
                                                c.create_action_row(|ar| {
                                                    ar.create_select_menu(|sm| {
                                                        sm.custom_id("select_card_to_play")
                                                            .min_values(1)
                                                            .max_values(1)
                                                            .options(|o| {
                                                                for (index, card) in
                                                                    player.hand.iter().enumerate()
                                                                {
                                                                    o.create_option(|o| {
                                                                        o.label(card.to_string())
                                                                            .value(index)
                                                                            .emoji(card.as_emoji())
                                                                    });
                                                                }
                                                                o
                                                            })
                                                    })
                                                })
                                            })
                                            .ephemeral(true)
                                    })
                            })
                            .await
                            .unwrap();

                        let player_cards = player.hand.clone();
                        let cards_count = player.cards_count();

                        let Some(interaction) = CollectComponentInteraction::new(&ctx.shard)
                        .timeout(Duration::from_secs(10))
                        .author_id(interaction.user.id)
                        .channel_id(interaction.channel_id)
                        .collect_limit(1)
                        .filter(move |select_menu_interaction| {
                            dbg!(select_menu_interaction);
                            // TODO: make it a const
                            if select_menu_interaction.data.custom_id == "select_card_to_play" {
                                if let Some(first_value) =
                                    select_menu_interaction.data.values.first()
                                {
                                    let first_value_usize = first_value
                                        .parse::<usize>()
                                        .expect("It should always be a valid index...");
                                    first_value_usize < cards_count
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }).await else {
                        return
                    };

                        dbg!(&interaction);

                        let chosen_card = {
                            interaction.data.values.first().map(|first_value| &player_cards[first_value.parse::<usize>().expect("It should always be a valid index since we only sent that before.")])
                        };

                        let Some(chosen_card) = chosen_card else {
                        return
                    };

                        match chosen_card {
                            Card::Colored(_, _) => {
                                let result = game.play_turn(TurnAction::Play(
                                    PlayAction::ColoredCard(chosen_card.clone()),
                                ));
                                interaction
                                    .create_interaction_response(ctx, |ir| {
                                        ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                            .interaction_response_data(|ird| {
                                                ird.content(format!(
                                                    "You chose: {}, result: {:?}, win: {}",
                                                    chosen_card, result.0, result.1
                                                ))
                                                .ephemeral(true)
                                            })
                                    })
                                    .await
                                    .unwrap();
                            }
                            _ => {
                                let colors = CardColor::iter().collect::<Vec<_>>();

                                interaction
                                .create_interaction_response(ctx, |ir| {
                                    ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                        .interaction_response_data(|ird| {
                                            ird
                                                .ephemeral(true)
                                                .content("Select the color you want to set the wild card to.")
                                                .components(|c| {
                                                    c.create_action_row(|ar| {
                                                        ar.create_select_menu(|sm| {
                                                            sm.custom_id("select_wild_card_color")
                                                                .min_values(1)
                                                                .max_values(1)
                                                                .options(|o| {
                                                                    for (index, color) in CardColor::iter()
                                                                        .enumerate()
                                                                    {
                                                                        o.create_option(|o| {
                                                                            o.label(
                                                                                format!("{color}"),
                                                                            )
                                                                            .value(index)
                                                                            .emoji(color.as_emoji())
                                                                        });
                                                                    }
                                                                    o
                                                                })
                                                        })
                                                    })
                                                })
                                        })
                                })
                                .await
                                .unwrap();

                                let Some(interaction) = CollectComponentInteraction::new(&ctx.shard)
                                .timeout(Duration::from_secs(10))
                                .author_id(interaction.user.id)
                                .channel_id(interaction.channel_id)
                                .collect_limit(1)
                                .filter(move |select_menu_interaction| {
                                    dbg!(select_menu_interaction);
                                    // TODO: make it a const
                                    if select_menu_interaction.data.custom_id == "select_wild_card_color" {
                                        if let Some(first_value) =
                                            select_menu_interaction.data.values.first()
                                        {
                                            let first_value_usize = first_value
                                                .parse::<usize>()
                                                .expect("It should always be a valid index...");
                                            first_value_usize < CardColor::COUNT
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }).await else {
                                return
                            };

                                let Some(first_value) = interaction.data.values.first() else {
                                return
                            };
                                let index = first_value
                                    .parse::<usize>()
                                    .expect("It should always be a valid index...");
                                let color = &colors[index];

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
                                                ird.content(format!(
                                                    "You chose: {}, result: {:?}, win: {}",
                                                    chosen_card, result.0, result.1
                                                ))
                                                .ephemeral(true)
                                            })
                                    })
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    UnoGame::Pending { .. } => todo!(),
                }
            }
            UnoButtonType::ViewHand => match game {
                UnoGame::Ongoing { game, .. } => {
                    let Some(player) = game.get_player(&interaction.user.id.0) else {
                        return;
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
                UnoGame::Pending { .. } => todo!(),
            },
            UnoButtonType::Draw => match game {
                UnoGame::Pending { .. } => todo!(),
                UnoGame::Ongoing { game, .. } => {
                    let result = game.play_turn(TurnAction::Draw);
                    interaction
                        .create_interaction_response(ctx, |ir| {
                            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|ird| {
                                    ird.content(format!(
                                        "You chose to draw, result: {:?}, win: {}",
                                        result.0, result.1
                                    ))
                                    .ephemeral(true)
                                })
                        })
                        .await
                        .unwrap();
                }
            },
            UnoButtonType::UNO => match game {
                UnoGame::Pending { .. } => todo!(),
                UnoGame::Ongoing { game, .. } => {
                    let result = game.play_turn(TurnAction::Uno);
                    interaction
                        .create_interaction_response(ctx, |ir| {
                            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|ird| {
                                    ird.content(format!(
                                        "You chose to say UNO, result: {:?}, win: {}",
                                        result.0, result.1
                                    ))
                                    .ephemeral(true)
                                })
                        })
                        .await
                        .unwrap();
                }
            },
            UnoButtonType::Callout => match game {
                UnoGame::Pending { .. } => todo!(),
                UnoGame::Ongoing { game, .. } => {
                    let result = game.play_turn(TurnAction::Callout);
                    interaction
                        .create_interaction_response(ctx, |ir| {
                            ir.kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|ird| {
                                    ird.content(format!(
                                        "You chose do a callout, result: {:?}, win: {}",
                                        result.0, result.1
                                    ))
                                    .ephemeral(true)
                                })
                        })
                        .await
                        .unwrap();
                }
            },
        }
    }

    pub async fn handle_event<'a>(ctx: &Context, event: &Event<'_>, data: &Data) {
        match event {
            Event::InteractionCreate { interaction } => match interaction {
                Interaction::MessageComponent(component_interaction) => match component_interaction
                    .data
                    .component_type
                {
                    ComponentType::Button => Self::process(ctx, component_interaction, data).await,
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
}

struct CardSelectMenu<T, F> {
    t_marker: PhantomData<T>,
    f_marker: PhantomData<F>,
}

impl<T, F> CardSelectMenu<T, F> {
    const SELECT_MENU_ID: &str = "select_card_to_play";

    async fn wait_for_selection<'a, Fn>(
        ctx: &Context,
        interaction: &MessageComponentInteraction,
        f: Fn,
    ) -> T
    where
        for<'b> Fn: FnOnce(
            &'b mut CreateInteractionResponseData<'a>,
        ) -> &'b mut CreateInteractionResponseData<'a>,
        T: FromStr,
    {
        interaction
            .create_interaction_response(ctx, |ir| {
                ir.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|ird| {
                        f(ird)
                            .content("Select the card you want to play.")
                            .components(|c| {
                                c.create_action_row(|ar| {
                                    ar.create_select_menu(|sm| {
                                        sm.custom_id(Self::SELECT_MENU_ID)
                                            .min_values(1)
                                            .max_values(1)
                                            .options(|o| {
                                                // for (index, card) in player.hand.iter().enumerate()
                                                // {
                                                //     o.create_option(|o| {
                                                //         o.label(card.to_string())
                                                //             .value(index)
                                                //             .emoji(card.as_emoji())
                                                //     });
                                                // }
                                                o
                                            })
                                    })
                                })
                            })
                            .ephemeral(true)
                    })
            })
            .await
            .unwrap();

        // let cards_count = player.cards_count();

        // let m = interaction.get_interaction_response(ctx).await.unwrap();
        //
        // let select_interaction = m
        //     .await_component_interaction(&ctx)
        //     .timeout(Duration::from_secs(60))
        //     .author_id(interaction.user.id)
        //     .channel_id(interaction.channel_id)
        //     .collect_limit(1)
        //     .filter(move |select_menu_interaction| {
        //         if select_menu_interaction.data.custom_id == Self::SELECT_MENU_ID {
        //             if let Some(first_value) = select_menu_interaction.data.values.first() {
        //                 let first_value_usize = first_value
        //                     .parse::<usize>()
        //                     .expect("It should always be a valid index...");
        //                 first_value_usize <= cards_count - 1
        //             } else {
        //                 false
        //             }
        //         } else {
        //             false
        //         }
        //     })
        //     .await
        //     .unwrap();

        // let chosen_card = {
        //     if let Some(first_value) = select_interaction.data.values.first() {
        //         player
        //             .hand
        //             .iter()
        //             .skip(first_value.parse::<usize>().expect(
        //                 "It should always be a valid index since we only sent that before.",
        //             ))
        //             .next()
        //     } else {
        //         None
        //     }
        // };
        todo!()
    }
}

struct ColorSelectMenu;

trait AsEmoji {
    fn as_emoji(&self) -> ReactionType;
}

impl AsEmoji for CardColor {
    fn as_emoji(&self) -> ReactionType {
        match self {
            CardColor::Red => ReactionType::Unicode("🟥".to_string()),
            CardColor::Green => ReactionType::Unicode("🟩".to_string()),
            CardColor::Blue => ReactionType::Unicode("🟦".to_string()),
            CardColor::Yellow => ReactionType::Unicode("🟨".to_string()),
        }
    }
}

impl AsEmoji for Card {
    fn as_emoji(&self) -> ReactionType {
        match self {
            Card::Colored(color, _) => color.as_emoji(),
            _ => ReactionType::Unicode("⬛".to_string()),
        }
    }
}
