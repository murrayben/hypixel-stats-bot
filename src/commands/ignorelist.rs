use crate::IgnoreList;
use serenity::builder::CreateCommand;
use serenity::client::Context;

pub async fn run(
    ctx: &Context
) -> String {
    let mut data = ctx.data.write().await;
    let ignore_list_map = data.get_mut::<IgnoreList>().unwrap();
    let ignore_list = ignore_list_map.entry("IgnoreList".into()).or_insert(Vec::new());
    if ignore_list.len() > 0 {
        format!("The current list of players that have their stats hidden is:\n- {}\n", ignore_list.join("\n- "))
    } else {
        String::from("No players currently have their stats hidden!")
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ignorelist")
        .description("Print out the list of ignored players")
}
