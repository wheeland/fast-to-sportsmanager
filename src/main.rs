use std::{collections::HashMap, fs, path::Path};

use clap::Parser;

use crate::sportsmanager::Spieler;

mod coral;
mod sportsmanager;

fn create_dir(path: &str) {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir(path).expect("Failed to create output directory");
    } else if !path.is_dir() {
        panic!("Not a directory: {}", path.display());
    }
}

fn write_disziplin(outfile: &str, disziplin: sportsmanager::Disziplin) {
    let sport = sportsmanager::Sport { disziplin };
    let out = quick_xml::se::to_string(&sport).expect("Failed to serialize sportsmanager xml");
    fs::write(outfile, out).expect("Failed to write file");
}

/// Generates tournament XML files that can be imported into sportsmanager
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    /// Input Coral JSON file
    input: String,

    /// Output foler
    output_directory: String,
}

fn main() {
    let args = CLI::parse();

    create_dir(&args.output_directory);
    let json = fs::read_to_string(&args.input).expect("Unable to read file");
    let coral: coral::Coral = serde_json::from_str(&json).expect("Failed to parse input JSON file");

    let players: HashMap<_, _> = coral
        .players
        .iter()
        .map(|coral_player| (coral_player.code.clone(), coral_player.name.clone()))
        .collect();

    let get_meldung = |rank: u64, codes: &[String]| {
        assert!(codes.len() == 1 || codes.len() == 2);

        let spieler: Vec<Spieler> = codes
            .iter()
            .map(|code| Spieler::from_name(&players.get(code).unwrap()))
            .collect();

        sportsmanager::Meldung::new(rank, spieler)
    };

    for comp in coral.competitions {
        let folder = args.output_directory.clone() + "/" + &comp.name.trim();
        create_dir(&folder);

        for phase in &comp.phases {
            let mut disziplin = match phase.system.as_str() {
                "swiss" | "round_robin" => sportsmanager::Disziplin::swiss(&phase.name),
                "sko" => sportsmanager::Disziplin::ko(&phase.name),
                _ => panic!("Invalid phase system: '{}'", phase.system),
            };

            println!("Processing '{}' / '{}'", comp.name, phase.name);

            for standing in &phase.standings {
                disziplin
                    .meldung
                    .push(get_meldung(standing.rank as _, &standing.players));
            }

            let mut runden = HashMap::new();
            for m in &phase.matches {
                if m.home.len() != m.away.len() {
                    continue;
                }

                let heim = get_meldung(0, &m.home);
                let gast = get_meldung(0, &m.away);

                let score = match m.winner {
                    Some(1) => (1, 0),
                    Some(2) => (0, 1),
                    _ => (0, 0),
                };
                let spiel =
                    sportsmanager::Spiel::from(m.number as _, &heim.name, &gast.name, score);

                let runde_no = m.round as _;
                let runde = runden
                    .entry(runde_no)
                    .or_insert(sportsmanager::Runde::new(runde_no));

                runde.spiel.push(spiel);
            }

            disziplin.runde = runden.into_values().collect();
            disziplin.runde.sort_by_key(|runde| runde.no);

            write_disziplin(&format!("{}/{}.xml", folder, phase.name), disziplin);
        }
    }
}
