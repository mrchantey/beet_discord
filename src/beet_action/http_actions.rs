use crate::prelude::*;
use beet::prelude::*;
use twilight_model::guild::Guild;


#[derive(Reflect)]
pub struct GetGuildParams {
	id: u64,
}

#[action(route = "get-guild")]
#[derive(Component)]
#[require(ParamsPartial=ParamsPartial::new::<GetGuildParams>())]
pub async fn GetGuildAction(cx: ActionContext<RequestParts>) -> Result<Guild> {
	let params = cx.input.params().parse_reflect::<GetGuildParams>()?;
	let get_guild = GetGuild::new(params.id.try_into()?);

	cx.caller
		.get_in_acestors_cloned::<DiscordHttpClient>()
		.await?
		.send(get_guild)
		.await?
		.xok()
}
