use std::{collections::HashMap, fs};

use scraper::{ElementRef, Html, Selector};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItsfPlayer {
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

fn parse_player_info_from(html: &Html) -> Result<ItsfPlayer, String> {
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
        first_name,
        last_name,
    })
}

fn download_player_info_from(url: &str) -> Result<ItsfPlayer, String> {
    let body = reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    let itsf = Html::parse_document(&body);
    parse_player_info_from(&itsf)
}

pub fn download_player_info(itsf_id: u64) -> Result<ItsfPlayer, String> {
    let url = format!(
        "https://www.tablesoccer.org/page/player&numlic={:08}",
        itsf_id
    );
    download_player_info_from(&url)
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

    pub fn register(&mut self, id: u64, lic: u64) -> bool {
        if self.players.contains_key(&id) || id == 0 {
            return false;
        }

        if let Some(player) = download_player_info(lic).ok() {
            if !player.first_name.is_empty() && !player.last_name.is_empty() {
                println!(
                    "Player ID {} registered with ITSF lic {}: {} {}",
                    id, lic, player.first_name, player.last_name
                );
                self.players.insert(id, player);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get(&self, id: u64) -> Option<&ItsfPlayer> {
        self.players.get(&id)
    }
}
