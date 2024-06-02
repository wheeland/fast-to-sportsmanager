use std::{collections::HashMap, env, fs, process::ExitCode};

use itsf::ItsfPlayerDb;

mod analysis;
mod fast;
mod itsf;
mod sportsmanager;

const CACHE: &'static str = "player_cache.json";

enum CompetitionType {
    Swiss,
    KO,
}

fn write_competition(outfile: &str, comp: &analysis::Competition, ty: CompetitionType) {
    assert!(comp.phases.len() > 0, "Competition has no phases");
    let match_phase = comp
        .phases
        .iter()
        .filter(|phase| !phase.matches.is_empty())
        .next();
    let phase = match_phase.unwrap_or(comp.phases.first().unwrap());

    let mut disziplin = sportsmanager::Disziplin {
        name: comp.source.name.clone(),
        system: sportsmanager::Disziplin::KO.to_string(),
        meldung: Vec::new(),
        runde: Vec::new(),
    };

    for (team, rank) in &phase.ranking {
        disziplin
            .meldung
            .push(sportsmanager::Meldung::from(*rank, team));
    }

    let matches: Vec<sportsmanager::Spiel> = phase
        .matches
        .iter()
        .enumerate()
        .map(|(id, m)| sportsmanager::Spiel::from(id as u64, m))
        .collect();

    match ty {
        CompetitionType::Swiss => {
            let mut matches_per_team = HashMap::new();
            for (team, rank) in &phase.ranking {
                let team_name = sportsmanager::Meldung::from(*rank, team).name;
                matches_per_team.insert(team_name, 0);
            }

            let mut runde = sportsmanager::Runde {
                no: 1,
                spiel: Vec::new(),
            };

            for m in matches {
                let mut next_round = false;
                next_round |= *matches_per_team.get(&m.heim).unwrap_or(&0) >= runde.no;
                next_round |= *matches_per_team.get(&m.gast).unwrap_or(&0) >= runde.no;
                if next_round {
                    let no = runde.no + 1;
                    disziplin.runde.push(runde);
                    runde = sportsmanager::Runde {
                        no,
                        spiel: Vec::new(),
                    };
                }

                *matches_per_team.get_mut(&m.heim).unwrap() += 1;
                *matches_per_team.get_mut(&m.gast).unwrap() += 1;

                runde.spiel.push(m);
            }

            if !runde.spiel.is_empty() {
                disziplin.runde.push(runde);
            }
        }
        CompetitionType::KO => {}
    }

    let sport = sportsmanager::Sport { disziplin };
    let out = quick_xml::se::to_string(&sport).expect("Failed to serialize sportsmanager xml");
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

        write_competition(
            &format!("{}/qualifications.xml", comp_name),
            comp,
            CompetitionType::Swiss,
        );

        for (id, sub) in comp.subcomps().iter().enumerate() {
            write_competition(
                &format!("{}/{} {}.xml", comp_name, id + 1, sub.source.name),
                sub,
                CompetitionType::KO,
            );
        }
    }

    return ExitCode::SUCCESS;
}
