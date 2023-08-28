use crate::{
    quake3a::{CauseOfDeath, KillEntry, LogEntry},
    Error, Result,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::{io::BufRead, str::FromStr};

// Quake 3 Arena logs are organized with one entry per line, each of which is:
//
// <min>:<ss> <message>
//
// Note: player names may contain whitespace
lazy_static! {
    static ref INIT_GAME_ENTRY_PATTERN: Regex = Regex::new(
        r"^\s*\d+:\d+ InitGame:"
    ).unwrap();

    // Kill messages:
    //
    // Kill: <a> <b> <c>: <player1> killed <player2> by <type>
    static ref KILL_ENTRY_PATTERN: Regex = Regex::new(
        r"^\s*\d+:\d+ Kill: \d+ \d+ \d+: (.+) killed (.+) by (\S+)"
    ).unwrap();
}

/// Iterator yielding kill entries from Quake 3 Arena logs.
pub struct KillLog<L> {
    log: L,
}

impl<L> KillLog<L> {
    /// Create a new iterator from raw log data.
    pub fn new(log: L) -> Self {
        Self { log }
    }
}

impl<L> Iterator for KillLog<L>
where
    L: BufRead,
{
    type Item = Result<LogEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        // Parse and return the next kill log entry. For simplicity we use
        // `BufRead::read_line` to read the whole line to memory one by one. The
        // downside is that this requires at least one allocation per call to
        // `next`.
        let mut line = String::new();
        loop {
            line.clear();
            match self.log.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => (),
                Err(err) => {
                    return Some(Err(err.into()));
                }
            }

            if INIT_GAME_ENTRY_PATTERN.is_match(&line) {
                return Some(Ok(LogEntry::InitGame));
            } else if let Some(captures) = KILL_ENTRY_PATTERN.captures(&line) {
                let Ok(cause) = CauseOfDeath::from_str(&captures[3]) else {
                    return Some(Err(Error::InvalidCauseOfDeath(captures[3].to_owned())));
                };

                return Some(Ok(LogEntry::Kill(KillEntry::new(
                    &captures[1],
                    &captures[2],
                    cause,
                ))));
            }
        }
    }
}
