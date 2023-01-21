use std::collections::{BTreeMap, HashSet};

use poise::serenity_prelude::UserId;
use runo::error::UnoError;

use crate::{commands::uno::button::UnoButton, Context, Error, UnoGame};

pub mod button;

#[poise::command(slash_command, prefix_command, subcommands("create", "join", "start"))]
pub async fn uno(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Create a new UNO match in the current channel
#[poise::command(prefix_command, slash_command)]
pub async fn create(ctx: Context<'_>) -> Result<(), Error> {
    let create_match_result = create_match(ctx).await;

    match create_match_result {
        CreateMatchResult::Created => {
            ctx.send(|m| {
                m.reply(true)
                    .content("Successfully created a match with you in it!")
            })
            .await?;
        }
        CreateMatchResult::AlreadyExists(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true)
                    .content(
                        format!(
                            "There is already a pending match in this channel, try `/uno join` first. The following users are in the match now:\n{players}"
                        )
                    )
            })
            .await?;
        }
        CreateMatchResult::AlreadyJoined(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true)
                    .content(
                        format!(
                            "You are already in the pending match in this channel, wait for it to start. The following users are in the match:\n{players}"
                        )
                    )
            })
            .await?;
        }
        CreateMatchResult::AlreadyStarted(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true).content(format!(
                    "There is already an ongoing match in this channel with the following users:\n{players}"
                ))
            })
            .await?;
        }
    }

    Ok(())
}

/// Join the pending UNO match in the current channel
#[poise::command(prefix_command, slash_command)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let join_match_result = join_match(ctx).await;

    match join_match_result {
        JoinMatchResult::DoesNotExist => {
            ctx.send(|m| {
                m.reply(true)
                    .content("No pending match found in this channel, try `/uno create` first.")
            })
            .await?;
        }
        JoinMatchResult::AlreadyJoined(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true)
                    .content(
                        format!(
                            "You are already in the pending match in this channel, wait for it to start. The following users are in the match:\n{players}"
                        )
                    )
            })
            .await?;
        }
        JoinMatchResult::Joined(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true)
                    .content(
                        format!(
                            "You have been added to the pending match in this channel. The following users are in the match now:\n{players}"
                        )
                    )
            })
            .await?;
        }
        JoinMatchResult::AlreadyStarted(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true).content(format!(
                    "There is already an ongoing match in this channel with the following users:\n{players}"
                ))
            })
            .await?;
        }
    }

    Ok(())
}

/// Start the pending UNO match in the current channel
#[poise::command(prefix_command, slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
    let start_match_result = start_match(ctx).await;

    match start_match_result {
        StartMatchResult::AlreadyStarted(player_ids) => {
            let players = join_player_list_to_string(player_ids.iter());
            ctx.send(|m| {
                m.reply(true).content(format!(
                    "A match is already running in this channel with the following users:\n{players}"
                ))
            })
            .await?;
        }
        StartMatchResult::DoesNotExist => {
            ctx.send(|m| {
                m.reply(true).content(
                    "There's no pending match in this channel. Use `/uno create` to create one.",
                )
            })
            .await?;
        }
        StartMatchResult::NotHost => {
            ctx.send(|m| {
                m.reply(true)
                    .content("A pending match can only be started by the host.")
            })
            .await?;
        }
        StartMatchResult::Started => {
            ctx.send(|m| {
                m.reply(true)
                    .content("Match started! Nothing else will happen yet tho...or will it?")
                    .components(|c| UnoButton::create_action_row(c))
            })
            .await?;
        }
        StartMatchResult::UnoError(uno_error) => {
            ctx.send(|m| {
                m.reply(true)
                    .content(format!("Failed to start match: {uno_error}"))
            })
            .await?;
        }
    }

    Ok(())
}

enum CreateMatchResult {
    AlreadyExists(HashSet<UserId>),
    AlreadyJoined(HashSet<UserId>),
    AlreadyStarted(HashSet<UserId>),
    Created,
}

enum JoinMatchResult {
    AlreadyJoined(HashSet<UserId>),
    AlreadyStarted(HashSet<UserId>),
    DoesNotExist,
    Joined(HashSet<UserId>),
}

enum StartMatchResult {
    AlreadyStarted(HashSet<UserId>),
    DoesNotExist,
    NotHost,
    Started,
    UnoError(UnoError),
}

async fn create_match(ctx: Context<'_>) -> CreateMatchResult {
    let mut hash_map = ctx.data().matches.lock().await;
    if let Some(existing_match) = hash_map.get_mut(&ctx.channel_id()) {
        let player_ids = existing_match.get_player_ids();
        match existing_match {
            UnoGame::Pending { .. } => {
                if player_ids.contains(&ctx.author().id) {
                    CreateMatchResult::AlreadyJoined(player_ids)
                } else {
                    CreateMatchResult::AlreadyExists(player_ids)
                }
            }
            UnoGame::Ongoing { .. } => CreateMatchResult::AlreadyStarted(player_ids),
        }
    } else {
        let mut queued_users = BTreeMap::new();
        queued_users.insert(ctx.author().id, ctx.author().tag());

        hash_map.insert(
            ctx.channel_id(),
            UnoGame::Pending {
                channel_id: ctx.channel_id(),
                host: ctx.author().id,
                queued_users,
            },
        );
        CreateMatchResult::Created
    }
}

async fn join_match(ctx: Context<'_>) -> JoinMatchResult {
    let mut hash_map = ctx.data().matches.lock().await;
    if let Some(existing_match) = hash_map.get_mut(&ctx.channel_id()) {
        match existing_match {
            UnoGame::Pending { queued_users, .. } => {
                if let std::collections::btree_map::Entry::Vacant(e) =
                    queued_users.entry(ctx.author().id)
                {
                    e.insert(ctx.author().tag());
                    JoinMatchResult::Joined(existing_match.get_player_ids())
                } else {
                    JoinMatchResult::AlreadyJoined(existing_match.get_player_ids())
                }
            }
            UnoGame::Ongoing { .. } => {
                JoinMatchResult::AlreadyStarted(existing_match.get_player_ids())
            }
        }
    } else {
        JoinMatchResult::DoesNotExist
    }
}

async fn start_match(ctx: Context<'_>) -> StartMatchResult {
    let mut hash_map = ctx.data().matches.lock().await;
    if let Some(existing_match) = hash_map.get_mut(&ctx.channel_id()) {
        match existing_match {
            UnoGame::Ongoing { .. } => {
                StartMatchResult::AlreadyStarted(existing_match.get_player_ids())
            }
            UnoGame::Pending {
                host: started_by, ..
            } => {
                if started_by != &ctx.author().id {
                    StartMatchResult::NotHost
                } else {
                    match existing_match.into_ongoing() {
                        Ok(_) => StartMatchResult::Started,
                        Err(err) => StartMatchResult::UnoError(err),
                    }
                }
            }
        }
    } else {
        StartMatchResult::DoesNotExist
    }
}

fn join_player_list_to_string<'a>(player_list_iter: impl Iterator<Item = &'a UserId>) -> String {
    player_list_iter
        .map(|id| format!("<@{id}>"))
        .collect::<Vec<_>>()
        .join("\n")
}
