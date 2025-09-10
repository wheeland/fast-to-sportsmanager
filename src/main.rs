use std::{collections::HashMap, fs, path::Path};

use clap::{Parser, Subcommand};

use crate::{itsf::ItsfPlayer, sportsmanager::Spieler};

mod coral;
mod fast;
mod itsf;
mod model;
mod sportsmanager;

const CACHE: &'static str = "player_cache.json";

fn create_dir(path: &str) {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir(path).expect("Failed to create output directory");
    } else if !path.is_dir() {
        panic!("Not a directory: {}", path.display());
    }
}

enum CompetitionType {
    Swiss,
    KO,
}

fn write_disziplin(outfile: &str, disziplin: sportsmanager::Disziplin) {
    let sport = sportsmanager::Sport { disziplin };
    let out = quick_xml::se::to_string(&sport).expect("Failed to serialize sportsmanager xml");
    fs::write(outfile, out).expect("Failed to write file");
}

fn write_competition(outfile: &str, comp: &model::Competition, ty: CompetitionType) {
    assert!(comp.phases.len() > 0, "Competition has no phases");
    let match_phase = comp
        .phases
        .iter()
        .filter(|phase| !phase.matches.is_empty())
        .next();
    let phase = match_phase.unwrap_or(comp.phases.first().unwrap());

    let mut disziplin = match ty {
        CompetitionType::Swiss => sportsmanager::Disziplin::swiss(&comp.source.name),
        CompetitionType::KO => sportsmanager::Disziplin::ko(&comp.source.name),
    };

    for (team, rank) in &phase.ranking {
        disziplin
            .meldung
            .push(sportsmanager::Meldung::from_team(*rank, team));
    }

    let mut runden = HashMap::new();

    for (id, m) in phase.matches.iter().enumerate() {
        let spiel = sportsmanager::Spiel::from_match(id as u64, m);
        let no = match ty {
            CompetitionType::Swiss => m.source.matchDepth,
            CompetitionType::KO => match m.source.matchDepth {
                0 => 19998, // 3rd place match
                1 => 19999, // finals
                _ => 19999 - m.source.matchDepth,
            },
        };

        let runde = runden.entry(no).or_insert(sportsmanager::Runde::new(no));
        runde.spiel.push(spiel);
    }

    let mut runden: Vec<sportsmanager::Runde> = runden.into_values().collect();
    runden.sort_by_key(|runde| runde.no);
    for runde in runden {
        disziplin.runde.push(runde);
    }

    write_disziplin(outfile, disziplin);
}

/// Generates tournament XML files that can be imported into sportsmanager
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Clears the ITSF player cache, meaning that all players will be parsed from the ITSF
    /// website the next time a tournament is processed
    ClearPlayerCache,

    /// Generates sportsmanager tournament XMLs from a FAST outfrom.xml
    FAST {
        input_xml: String,
        directory: String,
    },

    /// Generates sportsmanager tournament XMLs from a Coral JSON export
    Coral {
        input_json: String,
        directory: String,
    },
}

fn main() {
    let args = CLI::parse();

    match args.command {
        Command::ClearPlayerCache => {
            if std::path::Path::new(CACHE).exists() {
                std::fs::remove_file(CACHE).expect(&format!("Failed to remove {}", CACHE));
            }
        }
        Command::FAST {
            input_xml,
            directory,
        } => {
            create_dir(&directory);
            let xml = fs::read_to_string(&input_xml).expect("Unable to read file");
            let ffft: fast::Ffft = serde_xml_rs::from_str(&xml).expect("Failed to parse XML");

            // download player info
            let mut players = itsf::ItsfPlayerDb::try_load_cache(CACHE);
            for player in &ffft.registeredPlayers.players {
                players.register(player);
                players.save_cache(CACHE);
            }

            // analyze data
            let mut competitions = Vec::new();
            for tournament in &ffft.tournaments.tournaments {
                for competition in &tournament.competition {
                    let comp = model::Competition::new(competition, &players);

                    for other_comp in &competitions {
                        comp.borrow_mut().maybe_add_subcompetition(other_comp);
                        other_comp.borrow_mut().maybe_add_subcompetition(&comp);
                    }

                    competitions.push(comp);
                }
            }

            competitions.retain(|c| !c.borrow().is_subcomp);

            for comp in &competitions {
                let rankings = comp.borrow().rankings();
                for sub in &comp.borrow().subcomps {
                    sub.borrow_mut().adjust_final_rankings(&rankings);
                }
            }

            // write output files, grouped by root competitions
            for (index, comp) in competitions.iter().enumerate() {
                let comp = comp.borrow();

                let mut sex = comp.source.sex.clone();
                if !sex.is_empty() {
                    sex = format!(" ({})", sex);
                }

                let comp_name = format!(
                    "{} - {} {}{}",
                    index + 1,
                    comp.source.competitionType,
                    comp.source.name,
                    sex
                );

                let folder = directory.clone() + "/" + comp_name.trim();
                create_dir(&folder);

                write_competition(
                    &format!("{}/qualifications.xml", folder),
                    &comp,
                    CompetitionType::Swiss,
                );

                for (id, sub) in comp.subcomps.iter().enumerate() {
                    let sub = sub.borrow();

                    write_competition(
                        &format!("{}/{} {}.xml", folder, id + 1, sub.source.name),
                        &sub,
                        CompetitionType::KO,
                    );
                }
            }
        }
        Command::Coral {
            input_json,
            directory,
        } => {
            create_dir(&directory);
            let json = fs::read_to_string(&input_json).expect("Unable to read file");
            let coral: coral::Coral =
                serde_json::from_str(&json).expect("Failed to parse input JSON file");

            // download player info
            let mut player_db = itsf::ItsfPlayerDb::try_load_cache(CACHE);

            enum Player {
                Registered(ItsfPlayer),
                Unknown(String),
            }
            let players: HashMap<String, Player> = coral
                .players
                .iter()
                .map(|coral_player| {
                    let player = if let Some(itsf_player) =
                        coral_player.license.as_ref().and_then(|id| {
                            player_db.register_id(id.parse().expect("player ITSF ID not a number"))
                        }) {
                        Player::Registered(itsf_player.clone())
                    } else {
                        Player::Unknown(coral_player.name.clone())
                    };
                    (coral_player.code.clone(), player)
                })
                .collect();

            player_db.save_cache(CACHE);

            let get_meldung = |rank: u64, codes: &[String]| {
                assert!(codes.len() == 1 || codes.len() == 2);

                let spieler: Vec<Spieler> = codes
                    .iter()
                    .map(|code| match players.get(code).unwrap() {
                        Player::Registered(itsf) => Spieler::from_itsf(itsf),
                        Player::Unknown(name) => Spieler::from_name(name),
                    })
                    .collect();

                sportsmanager::Meldung::new(rank, spieler)
            };

            for comp in coral.competitions {
                let folder = directory.clone() + "/" + &comp.name.trim();
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
                        let spiel = sportsmanager::Spiel::from(
                            m.number as _,
                            &heim.name,
                            &gast.name,
                            score,
                        );

                        let runde_no = m.round as _;
                        let runde = runden.entry(runde_no).or_insert(sportsmanager::Runde {
                            no: runde_no,
                            spiel: Vec::new(),
                        });

                        runde.spiel.push(spiel);
                    }

                    disziplin.runde = runden.into_values().collect();
                    disziplin.runde.sort_by_key(|runde| runde.no);

                    write_disziplin(&format!("{}/{}.xml", folder, phase.name), disziplin);
                }
            }
        }
    }
}
