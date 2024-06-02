use std::{cell::Cell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{
    fast,
    itsf::{ItsfPlayer, ItsfPlayerDb},
};

pub struct Team {
    pub id: u64,
    pub player1: ItsfPlayer,
    pub player2: Option<ItsfPlayer>,
}

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

pub struct Match {
    pub result: fast::TeamMatchResult,
    pub team1: TeamRc,
    pub team2: TeamRc,
}

impl Debug for Match {
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
    pub matches: Vec<Match>,
}

pub struct Competition<'a> {
    pub source: &'a fast::Competition,
    pub teams: HashMap<u64, Rc<Team>>,
    pub phases: Vec<Phase<'a>>,
    pub subcomps: Cell<Vec<Rc<Self>>>,
    pub is_subcomp: Cell<bool>,
}

impl<'a> Competition<'a> {
    pub fn new(competition: &'a fast::Competition, players: &ItsfPlayerDb) -> Rc<Self> {
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

        Rc::new(Self {
            source: competition,
            teams,
            phases,
            subcomps: Cell::new(Vec::new()),
            is_subcomp: Cell::new(false),
        })
    }

    fn is_qualification_for(&self, other: &Competition) -> bool {
        other
            .teams
            .iter()
            .all(|(_, other_team)| self.teams.iter().any(|(_, team)| team.is_same(other_team)))
    }

    pub fn maybe_add_subcompetition(&self, other: &Rc<Competition<'a>>) -> bool {
        let self_ptr = self as *const _;
        let other_ptr = &**other as *const _;
        if self_ptr != other_ptr && self.is_qualification_for(other) {
            let mut subcomps = self.subcomps.take();
            subcomps.push(other.clone());
            self.subcomps.set(subcomps);

            other.is_subcomp.set(true);

            true
        } else {
            false
        }
    }

    pub fn subcomps(&self) -> Vec<Rc<Self>> {
        let subcomps = self.subcomps.take();
        let ret = subcomps.clone();
        self.subcomps.set(subcomps);
        ret
    }
}
