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
    fn new(rank: u64, spieler: Vec<Spieler>) -> Self {
        let mut name = spieler[0].name.clone();
        if let Some(spieler2) = spieler.get(1) {
            name += &format!(" / {}", spieler2.name);
        }
        Self {
            name,
            platz: rank,
            spieler,
        }
    }

    pub fn from_players(rank: u64, players: &[ItsfPlayer]) -> Self {
        assert!(players.len() == 1 || players.len() == 2);
        let spieler1 = Spieler::from_itsf(&players[0]);
        let spieler = match players.get(1) {
            None => vec![spieler1],
            Some(p) => vec![spieler1, Spieler::from_itsf(p)],
        };
        Self::new(rank, spieler)
    }

    pub fn from_team(rank: u64, team: &Team) -> Self {
        let spieler1 = Spieler::from_itsf(&team.player1);
        let spieler = match &team.player2 {
            None => vec![spieler1],
            Some(p) => vec![spieler1, Spieler::from_itsf(p)],
        };
        Self::new(rank, spieler)
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
    pub fn from(no: u64, heim: &str, gast: &str, score: (u64, u64)) -> Self {
        Self {
            heim: heim.to_string(),
            gast: gast.to_string(),
            satz: vec![Satz {
                heim: score.0,
                gast: score.1,
            }],
            no,
        }
    }

    pub fn from_match(no: u64, m: &model::Match) -> Self {
        Self::from(
            no,
            &Meldung::from_team(0, &m.team1).name,
            &Meldung::from_team(0, &m.team2).name,
            match m.result {
                TeamMatchResult::Draw => (1, 1),
                TeamMatchResult::Win1 => (1, 0),
                TeamMatchResult::Win2 => (0, 1),
            },
        )
    }
}

#[derive(Serialize, Debug)]
pub struct Runde {
    #[serde(rename = "@no")]
    pub no: u64,
    pub spiel: Vec<Spiel>,
}

impl Runde {
    pub fn new(no: u64) -> Self {
        Self {
            no,
            spiel: Vec::new(),
        }
    }
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
