use super::prelude::*;

async fn get_face_by_id(ctx: &Context, id: UserId) -> Result<String, String> {
    match id.to_user_cached(ctx).await {
        Some(user) => Ok(user.face()),
        None => Err(format!("user: Identifier does not belong to a valid user: {}", id)),
    }
}

/// Get the url to a user's avatar.
#[command]
#[usage = "`avatar_url [@user/<user-id>]`"]
pub async fn avatar_url(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(ctx, msg.author.face()).await?;
        return Ok(());
    }

    let face = match args.single::<UserId>() {
        Ok(id) => get_face_by_id(ctx, id).await?,
        Err(_) => return Err(From::from("user: Provided an incorrect user identifier or mention")),
    };

    msg.channel_id.say(ctx, face).await?;

    Ok(())
}
