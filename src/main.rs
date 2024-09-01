use std::{collections::HashMap, fs};

use clap::{Parser, Subcommand};

mod fast;
mod itsf;
mod model;
mod sportsmanager;

const CACHE: &'static str = "player_cache.json";

enum CompetitionType {
    Swiss,
    KO,
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
        let spiel = sportsmanager::Spiel::from(id as u64, m);
        let no = match ty {
            CompetitionType::Swiss => m.source.matchDepth,
            CompetitionType::KO => match m.source.matchDepth {
                0 => 19998, // 3rd place match
                1 => 19999, // finals
                _ => 19999 - m.source.matchDepth,
            },
        };

        let runde = runden.entry(no).or_insert(sportsmanager::Runde {
            no,
            spiel: Vec::new(),
        });

        runde.spiel.push(spiel);
    }

    let mut runden: Vec<sportsmanager::Runde> = runden.into_values().collect();
    runden.sort_by_key(|runde| runde.no);
    for runde in runden {
        disziplin.runde.push(runde);
    }

    let sport = sportsmanager::Sport { disziplin };
    let out = quick_xml::se::to_string(&sport).expect("Failed to serialize sportsmanager xml");
    fs::write(outfile, out).expect("Failed to write file");
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
        Command::FAST { input_xml } => {
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
                let comp_name = comp_name.trim().to_string();
        
                let _ = fs::create_dir(&comp_name);
        
                write_competition(
                    &format!("{}/qualifications.xml", comp_name),
                    &comp,
                    CompetitionType::Swiss,
                );
        
                for (id, sub) in comp.subcomps.iter().enumerate() {
                    let sub = sub.borrow();
        
                    write_competition(
                        &format!("{}/{} {}.xml", comp_name, id + 1, sub.source.name),
                        &sub,
                        CompetitionType::KO,
                    );
                }
            }
        }
    }
}
