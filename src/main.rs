use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn get_games() -> Result<Vec<String>> {
    let file = File::open("games.txt")?;
    let iterator = BufReader::new(file).lines();
    let mut lines = Vec::new();
    for line in iterator {
        lines.push(line?.to_lowercase());
    }

    Ok(lines)
}

async fn get_mc_score(client: &Client, game: &str) -> Result<usize> {
    let game = game.replace(" ", "-");
    let response = client
        .get(&format!(
            "https://www.metacritic.com/game/pc/{}/critic-reviews",
            game
        ))
        .send()
        .await?;

    let response = response.error_for_status()?;
    let body = response.text().await?;
    let html = Html::parse_document(&body);
    let selector = Selector::parse(&format!(
        r#"a[class="metascore_anchor"][href="/game/pc/{}/critic-reviews"]"#,
        game
    ))
    .map_err(|_| anyhow!("blah"))?;

    let element = html
        .select(&selector)
        .next()
        .ok_or_else(|| anyhow!("Cannot find a CSS selector"))?;

    Ok(element
        .children()
        .nth(1)
        .unwrap()
        .children()
        .nth(1)
        .unwrap()
        .first_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .parse::<usize>()
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let games = get_games().context("Error getting the game list")?;
    let client = Client::new();
    let mut f = File::create("result.csv")?;
    for game in games {
        println!("{}", &game);
        let mc_score = get_mc_score(&client, &game)
            .await
            .context(format!("Getting MC score for {}", game));

        match mc_score {
            Ok(score) => {
                f.write_all(format!("{},{}\n", game, score).as_bytes())?;
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
    Ok(())
}
