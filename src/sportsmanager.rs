use serde_derive::Serialize;

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
    pub fn from_name(name: &str) -> Self {
        let names: Vec<_> = name.split(" ").collect();
        let vorname = names[names.len()-1].to_string();
        let nachname = names[0..names.len()-1].join(" ");
        Self {
            name: vorname.clone() + " " + &nachname,
            vorname,
            nachname,
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
    pub fn new(rank: u64, spieler: Vec<Spieler>) -> Self {
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
