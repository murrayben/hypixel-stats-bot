use reqwest::Client;
use serde::Deserialize;
use std::cmp;

use crate::commands::utils;

#[derive(Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
struct Bedwars {
    #[serde(alias = "Experience")]
    experience: Option<f32>,
    final_kills_bedwars: Option<i32>,
    final_deaths_bedwars: Option<i32>,
    wins_bedwars: Option<i32>,
    losses_bedwars: Option<i32>,
}

impl Default for Bedwars {
    fn default() -> Self {
        Bedwars {
            experience: Some(0.0),
            final_kills_bedwars: Some(0),
            final_deaths_bedwars: Some(0),
            wins_bedwars: Some(0),
            losses_bedwars: Some(0),
        }
    }
}


#[derive(Deserialize, Debug)]
struct Player {
    stats: Option<Stats>,
}

#[derive(Deserialize, Debug)]
struct PlayerData {
    success: bool,
    player: Option<Player>,
    cause: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Stats {
    #[serde(alias = "Bedwars")]
    bedwars: Option<Bedwars>,
}

pub async fn get_stats<'a>(
    ign: &str,
    client: &Client,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let uuid_res = utils::get_uuid(ign, client).await;
    if let Err(_) = uuid_res {
        return Ok(format!("{}: Nicked!", bold_and_underline(ign, 3)));
    }
    let url = format!(
        "https://api.hypixel.net/v2/player?uuid={}&key={}",
        uuid_res.unwrap(), api_key
    );
    let request = client.get(&url).build().unwrap();
    let resp = client.execute(request).await?.json::<PlayerData>().await?;

    if let None = resp.player {
        return Ok(format!("{}: Nicked!", bold_and_underline(ign, 3)));
    }

    if !resp.success {
        Err(resp.cause.unwrap_or(String::from("Unknown cause")).into())
    } else {
        let player = resp.player.unwrap();
        let stats = player.stats.unwrap();
        if let Some(bedwars_stats) = stats.bedwars {
            let level = xp_to_level(bedwars_stats.experience.unwrap_or_default() as i32);
            let level_str = format!("Level {:.2}", level);
            let wins = bedwars_stats.wins_bedwars.unwrap_or_default();
            let final_kills = bedwars_stats.final_kills_bedwars.unwrap_or_default();
            let wlr: f32 = wins as f32 / bedwars_stats.losses_bedwars.unwrap_or_default() as f32;
            let fkdr: f32 = final_kills as f32 / bedwars_stats.final_deaths_bedwars.unwrap_or_default() as f32;
            let fkdr_str = format!("FKDR: {:.2}", fkdr);
            let level_colour_code = if level > 300.0 {
                3
            } else if level > 200.0 {
                2
            } else if level > 100.0 {
                1
            } else {
                -1
            };
            let fkdr_colour_code = if fkdr > 3.0 {
                3
            } else if fkdr > 2.0 {
                2
            } else if fkdr > 1.0 {
                1
            } else {
                -1
            };
            let ign_colour_code = cmp::max(level_colour_code, fkdr_colour_code);
            Ok(
                format!(
                    "{}: {}, Wins: {}, WLR: {:.2}, Finals: {}, {}",
                    bold_and_underline(ign, ign_colour_code),
                    bold_and_underline(&level_str, level_colour_code),
                    wins, wlr, final_kills,
                    bold_and_underline(&fkdr_str, fkdr_colour_code)
                )
            )
        } else {
            Err("Unable to retrieve bedwars stats".into())
        }
    }
}

fn xp_to_level(xp: i32) -> f32 {
    if xp == 0 {
        return 0.0;
    }

    const FIRST_FOUR: [i32; 4] = [500, 1000, 2000, 3500];
    const FIRST_FOUR_TOTAL: i32 = 7000;
    const TOTAL_PRESTIGE: i32 = FIRST_FOUR_TOTAL + 96 * 5000;
    
    let mut level = 0.0;
    let prestiges: i32 = xp / TOTAL_PRESTIGE;
    level += 100.0 * prestiges as f32;

    let mut extra_xp: i32 = xp - (prestiges * TOTAL_PRESTIGE);
    for level_xp in FIRST_FOUR {
        if extra_xp < level_xp {
            level += (extra_xp as f32) / (level_xp as f32);
            extra_xp = 0;
            break;
        }
        level += 1.0;
        extra_xp -= level_xp;
    }
    if extra_xp > 0 {
        level += (extra_xp as f32) / 5000.0
    }
    level
}


fn bold_and_underline(text: &str, colour: i32) -> String {
    if colour == -1 {
        return text.to_string();
    }
    let colour_code = match colour {
        3 => 31,
        2 => 33,
        1 => 34,
        _ => 37,
    };
    format!("\u{001B}[1;4;{}m{}\u{001B}[0m", colour_code, text)
}