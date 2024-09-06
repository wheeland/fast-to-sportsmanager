use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tournament {
    pub name: String,
    pub organization: String,
}

#[derive(Debug, Deserialize)]
pub struct Standing {
    pub rank: i32,
    pub players: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct Match {
    pub number: i32,
    pub round: i32,
    pub group: i32,
    pub home: Vec<String>,
    pub away: Vec<String>,
    pub winner: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Phase {
    pub name: String,
    pub system: String,
    pub stage: i32,
    pub matches: Vec<Match>,
    pub standings: Vec<Standing>,

}

#[derive(Debug, Deserialize)]
pub struct Competition {
    pub name: String,
    #[serde(rename(deserialize = "type"))]
    pub ty: String,
    pub category: String,
    pub phases: Vec<Phase>,
    pub standings: Vec<Standing>,
}

#[derive(Debug, Deserialize)]
pub struct Player {
    pub name: String,
    pub code: String,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Coral {
    pub tournament: Tournament,
    pub players: Vec<Player>,
    pub competitions: Vec<Competition>,
}

impl Coral {
}
