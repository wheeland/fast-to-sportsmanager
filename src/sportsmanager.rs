use serde_derive::Serialize;

use crate::{
    fast::TeamMatchResult,
    itsf::ItsfPlayer,
    model::{self, Team},
};

#[derive(Serialize, Debug)]
pub struct Spieler {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@vorname")]
    pub vorname: String,
    #[serde(rename = "@nachname")]
    pub nachname: String,
}

impl Spieler {
    pub fn from_itsf(player: &ItsfPlayer) -> Self {
        Self {
            name: format!("{} {}", player.first_name, player.last_name),
            vorname: player.first_name.clone(),
            nachname: player.last_name.clone(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Meldung {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@platz")]
    pub platz: u64,
    pub spieler: Vec<Spieler>,
}

impl Meldung {
    pub fn from_team(rank: u64, team: &Team) -> Self {
        let spieler1 = Spieler::from_itsf(&team.player1);
        let spieler2 = team.player2.as_ref().map(Spieler::from_itsf);
        let mut name = spieler1.name.clone();
        let mut spieler = vec![spieler1];
        if let Some(spieler2) = spieler2 {
            name += &format!(" / {}", spieler2.name);
            spieler.push(spieler2);
        }

        Self {
            name,
            platz: rank,
            spieler,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Satz {
    #[serde(rename = "@heim")]
    pub heim: u64,
    #[serde(rename = "@gast")]
    pub gast: u64,
}

#[derive(Serialize, Debug)]
pub struct Spiel {
    #[serde(rename = "@heim")]
    pub heim: String,
    #[serde(rename = "@gast")]
    pub gast: String,
    #[serde(rename = "@no")]
    pub no: u64,
    pub satz: Vec<Satz>,
}

impl Spiel {
    pub fn from(no: u64, m: &model::Match) -> Self {
        let heim = Meldung::from_team(0, &m.team1).name;
        let gast = Meldung::from_team(0, &m.team2).name;
        let (s1, s2) = match m.result {
            TeamMatchResult::Draw => (1, 1),
            TeamMatchResult::Win1 => (1, 0),
            TeamMatchResult::Win2 => (0, 1),
        };
        let satz = vec![Satz { heim: s1, gast: s2 }];
        Self {
            heim,
            gast,
            satz,
            no,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Runde {
    #[serde(rename = "@no")]
    pub no: u64,
    pub spiel: Vec<Spiel>,
}

#[derive(Serialize, Debug)]
pub struct Disziplin {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@system")]
    pub system: String,
    pub meldung: Vec<Meldung>,
    pub runde: Vec<Runde>,
}

impl Disziplin {
    pub fn swiss(name: &str) -> Self {
        Self {
            name: String::from(name),
            system: String::from("Schweizer System"),
            meldung: Vec::new(),
            runde: Vec::new(),
        }
    }

    pub fn ko(name: &str) -> Self {
        Self {
            name: String::from(name),
            system: String::from("KO-System"),
            meldung: Vec::new(),
            runde: Vec::new(),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Sport {
    pub disziplin: Disziplin,
}
