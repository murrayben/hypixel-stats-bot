use reqwest::Client;
use serde::Deserialize;

use crate::commands::utils;

#[derive(Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
struct Bedwars {
    #[serde(alias = "Experience")]
    experience: Option<i32>,
    final_kills_bedwars: Option<i32>,
    final_deaths_bedwars: Option<i32>,
    wins_bedwars: Option<i32>,
    losses_bedwars: Option<i32>,
}

impl Default for Bedwars {
    fn default() -> Self {
        Bedwars {
            experience: Some(0),
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
        return Ok(format!("**__{}__**: Nicked!", ign));
    }
    let url = format!(
        "https://api.hypixel.net/v2/player?uuid={}&key={}",
        uuid_res.unwrap(), api_key
    );
    let request = client.get(&url).build().unwrap();
    let resp = client.execute(request).await?.json::<PlayerData>().await?;

    if !resp.success {
        Err(resp.cause.unwrap_or(String::from("Unknown cause")).into())
    } else {
        let player = resp.player.unwrap();
        let stats = player.stats.unwrap();
        if let Some(bedwars_stats) = stats.bedwars {
            let level = xp_to_level(bedwars_stats.experience.unwrap_or_default());
            let wins = bedwars_stats.wins_bedwars.unwrap_or_default();
            let final_kills = bedwars_stats.final_kills_bedwars.unwrap_or_default();
            let wlr: f32 = wins as f32 / bedwars_stats.losses_bedwars.unwrap_or_default() as f32;
            let fkdr: f32 = final_kills as f32 / bedwars_stats.final_deaths_bedwars.unwrap_or_default() as f32;
            Ok(format!("**__{}__**: Level **{:.2}**, Wins: **{}**, WLR: **{:.2}**, Finals: **{}**, FKDR: **{:.2}**", ign, level, wins, wlr, final_kills, fkdr))
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