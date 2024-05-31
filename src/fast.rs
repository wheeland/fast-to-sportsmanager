#![allow(non_snake_case)]

use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PlayerInfos {
    #[serde(default)]
    pub noLicense: u64,
    #[serde(default)]
    pub playerId: u64,
}

#[derive(Deserialize, Debug)]
pub struct RegisteredPlayers {
    #[serde(rename = "$value")]
    pub players: Vec<PlayerInfos>,
}

#[derive(Deserialize, Debug)]
pub struct DefinitivePhaseOpponentRanking {
    pub teamId: u64,
    pub relativeRank: u64,
}

#[derive(Deserialize, Debug)]
pub struct Ranking {
    pub definitivePhaseOpponentRanking: DefinitivePhaseOpponentRanking,
    pub rank: u64,
}

#[derive(Deserialize, Debug)]
pub struct PhaseRanking {
    #[serde(rename = "$value")]
    pub rankings: Vec<Ranking>,
}

#[derive(Deserialize, Debug)]
pub struct Game {
    pub scoreTeam1: u64,
    pub scoreTeam2: u64,
}

#[derive(Deserialize, Debug)]
pub struct TeamMatch {
    pub team1Id: u64,
    pub team2Id: u64,
    pub game: Game,
}

#[derive(Deserialize, Debug)]
pub struct Phase {
    pub phaseType: String,
    pub phaseRanking: PhaseRanking,
}

#[derive(Deserialize, Debug)]
pub struct Competition {
    #[serde(rename = "type")]
    pub competitionType: String,
    #[serde(default)]
    pub name: String,
    pub phase: Vec<Phase>,
}

#[derive(Deserialize, Debug)]
pub struct Tournament {
    pub name: String,
    pub competition: Vec<Competition>,
}

#[derive(Deserialize, Debug)]
pub struct Tournaments {
    #[serde(rename = "$value")]
    pub tournaments: Vec<Tournament>,
}

#[derive(Deserialize, Debug)]
pub struct Ffft {
    pub registeredPlayers: RegisteredPlayers,
    pub tournaments: Tournaments,
}
