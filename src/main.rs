use std::{env, fs, process::ExitCode};

use itsf::ItsfPlayerDb;

mod analysis;
mod fast;
mod itsf;

const CACHE: &'static str = "player_cache.json";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("FAST .xml file name missing");
        println!("Usage: {} file.xml", args[0]);
        return ExitCode::from(1);
    }

    let xml = fs::read_to_string(&args[1]).expect("Unable to read file");
    let ffft: fast::Ffft = serde_xml_rs::from_str(&xml).expect("Failed to parse XML");

    // download player info
    let mut players = ItsfPlayerDb::try_load_cache(CACHE);
    for player in &ffft.registeredPlayers.players {
        players.register(player.playerId, player.noLicense);
        players.save_cache(CACHE);
    }

    // analyze data
    for tournament in &ffft.tournaments.tournaments {
        for competition in &tournament.competition {
            println!(
                "Competition {} {} {}",
                competition.competitionType, competition.name, competition.sex,
            );

            let comp = analysis::Competition::new(competition, &players);
            for phase in comp.phases {
                println!("  Phase {}", phase.phase_type);
                println!("    Ranking:");
                for (team, rank) in &phase.ranking {
                    println!("      {} {:?}", rank, *team);
                }
                println!("    Matches:");
                for m in &phase.matches {
                    println!("      {:?}", m);
                }
            }
        }
    }

    // println!("{:#?}", ffft);

    return ExitCode::SUCCESS;
}
