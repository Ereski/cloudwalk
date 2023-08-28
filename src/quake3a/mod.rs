use std::collections::BTreeMap;
use strum_macros::{Display, EnumString};

mod parser;
#[cfg(test)]
mod test;

pub use parser::KillLog;

/// An Quake 3 Arena log entry.
#[derive(Debug, PartialEq, Eq)]
pub enum LogEntry {
    /// New match started.
    InitGame,

    /// A player was killed.
    Kill(KillEntry),
}

/// Kill log entry containing the player that performed the kill, the player
/// that was killed and the instrument of killing.
#[derive(Debug, PartialEq, Eq)]
pub struct KillEntry {
    player: String,
    killed: String,
    cause: CauseOfDeath,
}

impl KillEntry {
    /// Create a new kill log entry.
    pub fn new<P, K>(player: P, killed: K, cause: CauseOfDeath) -> Self
    where
        P: Into<String>,
        K: Into<String>,
    {
        Self {
            player: player.into(),
            killed: killed.into(),
            cause,
        }
    }
}

/// The valid causes of death:
/// <https://github.com/id-Software/Quake-III-Arena/blob/master/code/game/bg_public.h#L571-L604>.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Display, EnumString)]
#[allow(non_camel_case_types)]
pub enum CauseOfDeath {
    MOD_BFG_SPLASH,
    MOD_BFG,
    MOD_CHAINGUN,
    MOD_CRUSH,
    MOD_FALLING,
    MOD_GAUNTLET,
    MOD_GRAPPLE,
    MOD_GRENADE_SPLASH,
    MOD_GRENADE,
    MOD_JUICED,
    MOD_KAMIKAZE,
    MOD_LAVA,
    MOD_LIGHTNING,
    MOD_MACHINEGUN,
    MOD_NAIL,
    MOD_PLASMA_SPLASH,
    MOD_PLASMA,
    MOD_PROXIMITY_MINE,
    MOD_RAILGUN,
    MOD_ROCKET_SPLASH,
    MOD_ROCKET,
    MOD_SHOTGUN,
    MOD_SLIME,
    MOD_SUICIDE,
    MOD_TARGET_LASER,
    MOD_TELEFRAG,
    MOD_TRIGGER_HURT,
    MOD_UNKNOWN,
    MOD_WATER,
}

/// Scoreboard for a single match.
#[derive(Default)]
pub struct MatchScoreboard {
    players: BTreeMap<String, PlayerScore>,
    total_kills: usize,
}

impl MatchScoreboard {
    /// Create a blank scoreboard.
    pub fn new() -> Self {
        Self::default()
    }

    /// The total number of kills this match, including suicides and `<world>`
    /// kills.
    pub fn total_kills(&self) -> usize {
        self.total_kills
    }

    /// Get an iterator walking through all players and their individual scores.
    pub fn player_scores(&self) -> impl Iterator<Item = (&str, &PlayerScore)> {
        self.players
            .iter()
            .map(|(name, score)| (name.as_str(), score))
    }

    /// Record a kill.
    pub fn record(&mut self, kill: KillEntry) {
        let was_killed_by_world = kill.player == "<world>";
        let killed_self = kill.killed == kill.player;
        if !was_killed_by_world {
            self.players
                .entry(kill.player)
                .or_default()
                .record_kill(kill.cause, killed_self);
        }
        self.players
            .entry(kill.killed)
            .or_default()
            .record_death(kill.cause, was_killed_by_world || killed_self);
        self.total_kills += 1;
    }
}

/// The score of a single player.
#[derive(Default)]
pub struct PlayerScore {
    deaths: BTreeMap<CauseOfDeath, usize>,
    kills: BTreeMap<CauseOfDeath, usize>,
    n_kills: usize,
    n_deaths: usize,
    n_suicides: usize,
    score: isize,
}

impl PlayerScore {
    /// The number of times this player has died, including suicides.
    pub fn n_deaths(&self) -> usize {
        self.n_deaths
    }

    pub fn deaths(&self) -> impl Iterator<Item = (CauseOfDeath, usize)> + '_ {
        self.deaths.iter().map(|(cause, n)| (*cause, *n))
    }

    /// The number of times this player has died, including suicides.
    pub fn n_kills(&self) -> usize {
        self.n_kills
    }

    pub fn kills(&self) -> impl Iterator<Item = (CauseOfDeath, usize)> + '_ {
        self.kills.iter().map(|(cause, n)| (*cause, *n))
    }

    /// The number of times this player has killed itself.
    pub fn n_suicides(&self) -> usize {
        self.n_suicides
    }

    /// This player's total score. The score is calculated as:
    ///
    /// ```text
    /// n_kills - n_suicides - n_world_deaths
    /// ```
    ///
    /// Where `n_world_deaths` is the number of deaths caused by the `<world>`.
    pub fn score(&self) -> isize {
        self.score
    }

    fn record_death(&mut self, cause: CauseOfDeath, subtract_score: bool) {
        *self.deaths.entry(cause).or_default() += 1;
        self.n_deaths += 1;
        if subtract_score {
            self.score -= 1;
        }
    }

    fn record_kill(&mut self, cause: CauseOfDeath, killed_self: bool) {
        *self.kills.entry(cause).or_default() += 1;
        self.n_kills += 1;
        if killed_self {
            self.n_suicides += 1;
        } else {
            self.score += 1;
        }
    }
}
