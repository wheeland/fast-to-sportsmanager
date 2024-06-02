use std::{env, fs, process::ExitCode};

use itsf::ItsfPlayerDb;

mod analysis;
mod fast;
mod itsf;

const CACHE: &'static str = "player_cache.json";

fn write_competition(outfile: &str, comp: &analysis::Competition) {
    let mut out = String::new();

    assert!(comp.phases.len() > 0, "Competition has no phases");
    let match_phase = comp
        .phases
        .iter()
        .filter(|phase| !phase.matches.is_empty())
        .next();
    let phase = match_phase.unwrap_or(comp.phases.first().unwrap());

    for (team, rank) in &phase.ranking {
        out += &format!("{:4} {:?}\n", rank, team);
    }
    out += "\n";
    for m in &phase.matches {
        out += &format!("{:?}\n", m);
    }

    fs::write(outfile, out).expect("Failed to write file");
}

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
    let mut competitions = Vec::new();
    for tournament in &ffft.tournaments.tournaments {
        for competition in &tournament.competition {
            let comp = analysis::Competition::new(competition, &players);

            for other_comp in &competitions {
                comp.maybe_add_subcompetition(other_comp);
                other_comp.maybe_add_subcompetition(&comp);
            }

            competitions.push(comp);
        }
    }

    competitions.retain(|c| !c.is_subcomp.get());

    // write output files, grouped by root competitions
    for (index, comp) in competitions.iter().enumerate() {
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
        let comp_name = comp_name.trim().to_string();

        let _ = fs::create_dir(&comp_name);

        write_competition(&format!("{}/qualifications.txt", comp_name), comp);
        for (id, sub) in comp.subcomps().iter().enumerate() {
            write_competition(
                &format!("{}/{} {}.txt", comp_name, id + 1, sub.source.name),
                sub,
            );
        }
    }

    return ExitCode::SUCCESS;
}
