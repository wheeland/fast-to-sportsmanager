use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, rc::Rc};

use crate::{
    fast,
    itsf::{ItsfPlayer, ItsfPlayerDb},
};

#[derive(Clone)]
pub struct Team {
    pub id: u64,
    pub player1: ItsfPlayer,
    pub player2: Option<ItsfPlayer>,
}

impl Hash for Team {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.player1.hash(state);
        self.player2.hash(state);
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.player1 == other.player1 && self.player2 == other.player2
    }
}

impl Eq for Team {}

impl Team {
    pub fn new(competition_team: &fast::CompetitionTeam, players: &ItsfPlayerDb) -> Option<Self> {
        let id = competition_team.id;
        let player1 = players.get(competition_team.team.player1Id);
        let player2 = players.get(competition_team.team.player2Id);

        player1.map(|player1| Self {
            id,
            player1: player1.clone(),
            player2: player2.cloned(),
        })
    }

    pub fn is_same(&self, other: &Team) -> bool {
        self.player1 == other.player1 && self.player2 == other.player2
    }
}

impl Debug for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.player2 {
            Some(player2) => f.write_fmt(format_args!(
                "({} {}, {} {})",
                self.player1.first_name,
                self.player1.last_name,
                player2.first_name,
                player2.last_name
            )),
            None => f.write_fmt(format_args!(
                "({} {})",
                self.player1.first_name, self.player1.last_name
            )),
        }
    }
}

pub type TeamRc = Rc<Team>;

pub struct Match<'a> {
    pub source: &'a fast::TeamMatch,
    pub result: fast::TeamMatchResult,
    pub team1: TeamRc,
    pub team2: TeamRc,
}

impl Debug for Match<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.result {
            fast::TeamMatchResult::Draw => {
                f.write_fmt(format_args!("{:?} - {:?}", *self.team1, *self.team2))
            }
            fast::TeamMatchResult::Win1 => {
                f.write_fmt(format_args!("{:?} > {:?}", *self.team1, *self.team2))
            }
            fast::TeamMatchResult::Win2 => {
                f.write_fmt(format_args!("{:?} > {:?}", *self.team2, *self.team1))
            }
        }
    }
}

pub struct Phase<'a> {
    pub source: &'a fast::Phase,
    pub phase_type: String,
    pub ranking: Vec<(TeamRc, u64)>,
    pub matches: Vec<Match<'a>>,
}

pub struct Competition<'a> {
    pub source: &'a fast::Competition,
    pub teams: HashMap<u64, Rc<Team>>,
    pub phases: Vec<Phase<'a>>,
    pub subcomps: Vec<Rc<RefCell<Self>>>,
    pub is_subcomp: bool,
}

impl<'a> Competition<'a> {
    pub fn new(competition: &'a fast::Competition, players: &ItsfPlayerDb) -> Rc<RefCell<Self>> {
        let mut teams = HashMap::new();
        let mut phases = Vec::new();

        for competition_team in &competition.competitionTeam {
            if let Some(team) = Team::new(competition_team, players) {
                teams.insert(team.id, Rc::new(team));
            }
        }

        for phase in &competition.phase {
            let phase_type = phase.phaseType.clone();
            let mut ranking = Vec::new();
            let mut matches = Vec::new();

            for phase_ranking in &phase.phaseRanking.rankings {
                let team = phase_ranking.definitivePhaseOpponentRanking.teamId;
                if let Some(team) = teams.get(&team) {
                    ranking.push((team.clone(), phase_ranking.rank));
                }
            }

            for phase_match in &phase.teamMatch {
                let team1 = teams.get(&phase_match.team1Id);
                let team2 = teams.get(&phase_match.team2Id);
                if let Some((team1, team2)) = team1.zip(team2) {
                    matches.push(Match {
                        source: phase_match,
                        result: phase_match.result(),
                        team1: team1.clone(),
                        team2: team2.clone(),
                    });
                }
            }

            phases.push(Phase {
                source: phase,
                phase_type,
                ranking,
                matches,
            });
        }

        Rc::new(RefCell::new(Self {
            source: competition,
            teams,
            phases,
            subcomps: Vec::new(),
            is_subcomp: false,
        }))
    }

    fn is_qualification_for(&self, other: &Competition) -> bool {
        other
            .teams
            .iter()
            .all(|(_, other_team)| self.teams.iter().any(|(_, team)| team.is_same(other_team)))
    }

    pub fn maybe_add_subcompetition(&mut self, other: &Rc<RefCell<Competition<'a>>>) -> bool {
        let mut other_mut = other.borrow_mut();

        let self_ptr = self.source as *const _;
        let other_ptr = other_mut.source as *const _;

        if self_ptr != other_ptr && self.is_qualification_for(&other_mut) {
            assert!(!self.is_subcomp);
            self.subcomps.push(other.clone());
            other_mut.is_subcomp = true;

            true
        } else {
            false
        }
    }

    pub fn rankings(&self) -> HashMap<Team, u64> {
        let ranking_phase = self
            .phases
            .iter()
            .filter(|phase| !phase.matches.is_empty())
            .next();
        let ranking_phase = ranking_phase.unwrap();

        ranking_phase
            .ranking
            .iter()
            .map(|(team, rank)| ((**team).clone(), *rank))
            .collect()
    }

    pub fn adjust_final_rankings(&mut self, qualification_rankings: &HashMap<Team, u64>) {
        for phase in &mut self.phases {
            let mut first_rank = 4;

            while first_rank < phase.ranking.len() {
                let last_rank = (first_rank * 2).min(phase.ranking.len());
                phase.ranking[first_rank..last_rank].sort_by(|a, b| {
                    let a = *qualification_rankings.get(&*a.0).unwrap();
                    let b = *qualification_rankings.get(&*b.0).unwrap();
                    a.cmp(&b)
                });
                first_rank *= 2;
            }

            for (new_rank, (_, rank)) in &mut phase.ranking.iter_mut().enumerate() {
                *rank = (new_rank + 1) as u64;
            }
        }
    }
}
