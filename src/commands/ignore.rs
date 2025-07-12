use crate::IgnoreList;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::client::Context;
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub async fn run<'a>(
    options: &[ResolvedOption<'a>],
    ctx: &Context
) -> String {
    let mut data = ctx.data.write().await;
    let ignore_list_map = data.get_mut::<IgnoreList>().unwrap();
    let ignore_list = ignore_list_map.entry("IgnoreList".into()).or_insert(Vec::new());
    let mut options_iter = options.iter();
    if let Some(ResolvedOption {
        value: ResolvedValue::String(action),
        ..
    }) = options_iter.next() {
        if let Some(ResolvedOption {
            value: ResolvedValue::String(ign),
            ..
        }) = options_iter.next() {
            match *action {
                "add" => {
                    ignore_list.push(String::from(*ign));
                    format!("Successfully added {} to the ignore list", *ign)
                },
                "remove" => {
                    let index = ignore_list.iter().position(|x| x == *ign);
                    match index {
                        Some(i) => {
                            ignore_list.remove(i);
                            format!("Successfully removed {} from the ignore list", *ign)
                        },
                        None => {
                            format!("Could not find {} in the ignore list", *ign)
                        }
                    }
                },
                "list" => {
                    format!("The current ignore list is:\n{}", ignore_list.join("\n"))
                }
                _ => String::from("Please provide a valid action")
            }
        } else {
            String::from("Please provide a valid IGN")
        }
    } else {
        String::from("Please provide a valid action and IGN")
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ignore")
        .description("Ignore player from stats bot")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "action", "Action")
                .required(true)
                .add_string_choice("Add", "add")
                .add_string_choice("Remove", "remove")
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "ign", "In-game name")
                .required(true),
        )
}
