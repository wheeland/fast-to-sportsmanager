use std::{collections::HashMap, fs};

use scraper::{ElementRef, Html, Selector};
use serde_derive::{Deserialize, Serialize};

use crate::fast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItsfPlayer {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
}

fn get_div_with_class<'a>(root: &'a Html, class: &'static str) -> Vec<ElementRef<'a>> {
    let div_selector = Selector::parse("div").unwrap();
    root.select(&div_selector)
        .filter(|div| div.value().attr("class") == Some(class))
        .collect()
}

fn is_uppercase(word: &str) -> bool {
    word.chars().all(|c| !c.is_lowercase())
}

fn to_normalcase(word: &str) -> String {
    let mut result = String::new();

    for ch in word.chars().enumerate() {
        if ch.0 == 0 {
            result.push(ch.1);
        } else {
            for ch in ch.1.to_lowercase() {
                result.push(ch);
            }
        }
    }

    result
}

fn parse_player_info_from(id: u64, html: &Html) -> Result<ItsfPlayer, String> {
    let nomdujoueur = get_div_with_class(html, "nomdujoueur");
    let nomdujoueur = nomdujoueur.first().ok_or("can't find div nomdujoueur")?;
    let name = nomdujoueur
        .text()
        .next()
        .ok_or("can't find text in nomdujoueur div")?;

    let last_name = name
        .split(' ')
        .filter(|word| !word.is_empty() && is_uppercase(word))
        .map(to_normalcase)
        .collect::<Vec<String>>()
        .join(" ");

    let first_name = name
        .split(' ')
        .filter(|word| !word.is_empty() && !is_uppercase(word))
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(ItsfPlayer {
        id,
        first_name,
        last_name,
    })
}

fn download_player_info_from(id: u64, url: &str) -> Result<ItsfPlayer, String> {
    let body = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    let itsf = Html::parse_document(&body);
    parse_player_info_from(id, &itsf)
}

pub fn download_player_info(itsf_id: u64) -> Result<ItsfPlayer, String> {
    let url = format!(
        "https://www.tablesoccer.org/page/player&numlic={:08}",
        itsf_id
    );
    download_player_info_from(itsf_id, &url)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItsfPlayerDb {
    players: HashMap<u64, ItsfPlayer>,
}

impl ItsfPlayerDb {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn try_load_cache(path: &str) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or(Self::new()),
            Err(_) => Self::new(),
        }
    }

    pub fn save_cache(&self, path: &str) {
        if let Ok(s) = serde_json::to_string(self) {
            fs::write(path, s).expect("Failed to write player cache file");
        }
    }

    pub fn register(&mut self, player_infos: &fast::PlayerInfos) {
        let id = player_infos.id();
        assert!(id > 0);

        if !self.players.contains_key(&id) {
            let player = match &player_infos.player {
                Some(player) => ItsfPlayer {
                    id,
                    first_name: player.person.firstName.clone(),
                    last_name: player.person.lastName.clone(),
                },
                None => {
                    assert!(player_infos.noLicense > 0);
                    let player = download_player_info(player_infos.noLicense)
                        .expect("Failed to get ITSF player");
                    println!(
                        "Downloaded player data for {}: {} {}",
                        player_infos.noLicense, player.first_name, player.last_name
                    );
                    player
                }
            };
            self.players.insert(id, player);
        }
    }

    pub fn get(&self, id: u64) -> Option<&ItsfPlayer> {
        self.players.get(&id)
    }
}
