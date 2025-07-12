use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    id: Option<String>,
    error_message: Option<String>,
}

pub async fn get_uuid(ign: &str, client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", ign);
    let request = client.get(&url).build().unwrap();

    let resp = match client.execute(request).await?.json::<User>().await {
        Ok(resp) => resp,
        Err(err) => {
            return Err(err.into());
        }
    };

    if let Some(err) = resp.error_message {
        return Err(err.into());
    }

    if let Some(uuid) = resp.id {
        return Ok(uuid);
    } else {
        return Err("No player data found".into());
    }
}
