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
        return Ok(format!("{} is nicked!", ign));
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
            let mut level: f32 = 0.0;
            let mut experience = bedwars_stats.experience.unwrap_or_default();
            if experience > 7000 {
                experience -= 7000;
                level += 4.0;
                level += (experience as f32) / 5000.0;
            } else if experience > 3500 {
                experience -= 3500;
                level += 3.0;
                level += (experience as f32) / 3500.0;
            } else if experience > 1500 {
                experience -= 1500;
                level += 2.0;
                level += (experience as f32) / 2000.0;
            } else if experience > 500 {
                experience -= 500;
                level += 1.0;
                level += (experience as f32) / 1000.0;
            } else {
                level += (experience as f32) / 500.0;
            }
            let wins = bedwars_stats.wins_bedwars.unwrap_or_default();
            let final_kills = bedwars_stats.final_kills_bedwars.unwrap_or_default();
            let wlr: f32 = wins as f32 / bedwars_stats.losses_bedwars.unwrap_or_default() as f32;
            let fkdr: f32 = final_kills as f32 / bedwars_stats.final_deaths_bedwars.unwrap_or_default() as f32;
            Ok(format!("{}: Level {:.2}, Wins: {}, WLR: {:.2}, Finals: {}, FKDR: {:.2}", ign, level, wins, wlr, final_kills, fkdr))
        } else {
            Err("Unable to retrieve bedwars stats".into())
        }
    }
}
    