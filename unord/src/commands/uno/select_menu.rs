use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use poise::{
    async_trait,
    serenity_prelude::{
        CollectComponentInteraction, Context, CreateInteractionResponseData,
        InteractionResponseType, MessageComponentInteraction, ShardMessenger,
    },
};
use runo::card::{Card, CardColor};

use super::AsEmoji;

#[async_trait]
pub trait SelectMenu<T>: Sync {
    fn custom_id() -> String;

    fn create_interaction_response_data<'a>(
        &self,
        _ctx: &Context,
        _ird: &'a mut CreateInteractionResponseData,
    );

    fn create_collect_component_interaction(
        &self,
        _shard: impl AsRef<ShardMessenger>,
        _interaction: &MessageComponentInteraction,
    ) -> CollectComponentInteraction;

    async fn await_selection(
        &mut self,
        ctx: &Context,
        interaction: &MessageComponentInteraction,
    ) -> Result<Option<Arc<MessageComponentInteraction>>> {
        interaction
            .create_interaction_response(ctx, |ir| {
                ir.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|ird| {
                        self.create_interaction_response_data(ctx, ird);
                        ird
                    })
            })
            .await?;

        let collect_component_interaction =
            self.create_collect_component_interaction(ctx, interaction);
        let select_interaction = collect_component_interaction.await;

        match select_interaction {
            Some(select_interaction) => {
                if select_interaction.data.custom_id != Self::custom_id() {
                    // TODO: Error
                    return Ok(Some(select_interaction));
                }

                self.on_collected(&select_interaction.data.values);
                return Ok(Some(select_interaction));
            }
            // TODO: Error
            None => Ok(select_interaction),
        }
    }

    fn on_collected(&mut self, selections: &[String]);

    fn get_selection(&self) -> Option<T>;
}

pub struct CardSelectMenu<'cards> {
    available_cards: &'cards [Card],
    selected_index: Option<usize>,
}

impl<'cards> CardSelectMenu<'cards> {
    pub fn new(available_cards: &'cards [Card]) -> Self {
        Self {
            available_cards,
            selected_index: None,
        }
    }
}

#[async_trait]
impl<'cards> SelectMenu<&'cards Card> for CardSelectMenu<'cards> {
    fn custom_id() -> String {
        "select_menu_card".to_string()
    }

    fn on_collected(&mut self, selections: &[String]) {
        if let Some(selected_index) = selections
            .iter()
            .find_map(|str_selection| str_selection.parse::<usize>().ok())
        {
            self.selected_index = Some(selected_index);
        }
    }

    fn get_selection(&self) -> Option<&'cards Card> {
        self.selected_index
            .and_then(|index| self.available_cards.get(index))
    }

    fn create_interaction_response_data<'a>(
        &self,
        _ctx: &Context,
        ird: &'a mut CreateInteractionResponseData,
    ) {
        ird.content("Select the card you want to play:")
            .components(|c| {
                c.create_action_row(|ar| {
                    ar.create_select_menu(|sm| {
                        sm.custom_id(Self::custom_id())
                            .min_values(1)
                            .max_values(1)
                            .options(|o| {
                                for (index, card) in self.available_cards.iter().enumerate() {
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
            .ephemeral(true);
    }

    fn create_collect_component_interaction(
        &self,
        shard: impl AsRef<ShardMessenger>,
        interaction: &MessageComponentInteraction,
    ) -> CollectComponentInteraction {
        CollectComponentInteraction::new(shard.as_ref())
            .timeout(Duration::from_secs(60))
            .author_id(interaction.user.id)
            .channel_id(interaction.channel_id)
            .collect_limit(1)
            .filter(move |select_menu_interaction| {
                select_menu_interaction.data.custom_id == Self::custom_id()
            })
    }
}

pub struct ColorSelectMenu<'colors> {
    available_colors: &'colors [CardColor],
    selected_index: Option<usize>,
}

impl<'cards> ColorSelectMenu<'cards> {
    pub fn new(available_colors: &'cards [CardColor]) -> Self {
        Self {
            available_colors,
            selected_index: None,
        }
    }
}

#[async_trait]
impl<'cards> SelectMenu<&'cards CardColor> for ColorSelectMenu<'cards> {
    fn custom_id() -> String {
        "select_menu_card_color".to_string()
    }

    fn on_collected(&mut self, selections: &[String]) {
        if let Some(selected_index) = selections
            .iter()
            .find_map(|str_selection| str_selection.parse::<usize>().ok())
        {
            self.selected_index = Some(selected_index);
        }
    }

    fn get_selection(&self) -> Option<&'cards CardColor> {
        self.selected_index
            .and_then(|index| self.available_colors.get(index))
    }

    fn create_interaction_response_data<'a>(
        &self,
        _ctx: &Context,
        ird: &'a mut CreateInteractionResponseData,
    ) {
        ird.ephemeral(true)
            .content("Select the color you want to set the wild card to:")
            .components(|c| {
                c.create_action_row(|ar| {
                    ar.create_select_menu(|sm| {
                        sm.custom_id(Self::custom_id())
                            .min_values(1)
                            .max_values(1)
                            .options(|o| {
                                for (index, color) in self.available_colors.iter().enumerate() {
                                    o.create_option(|o| {
                                        o.label(color.to_string())
                                            .value(index)
                                            .emoji(color.as_emoji())
                                    });
                                }
                                o
                            })
                    })
                })
            });
    }

    fn create_collect_component_interaction(
        &self,
        shard: impl AsRef<ShardMessenger>,
        interaction: &MessageComponentInteraction,
    ) -> CollectComponentInteraction {
        CollectComponentInteraction::new(shard.as_ref())
            .timeout(Duration::from_secs(60))
            .author_id(interaction.user.id)
            .channel_id(interaction.channel_id)
            .collect_limit(1)
            .filter(move |select_menu_interaction| {
                select_menu_interaction.data.custom_id == Self::custom_id()
            })
    }
}
