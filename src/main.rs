use crate::quake3a::LogEntry;
use clap::{command, value_parser, Arg};
use quake3a::{KillLog, MatchScoreboard};
use std::{
    cmp::Reverse,
    collections::BTreeMap,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
    process, result,
};
use thiserror::Error;

mod quake3a;

#[derive(Debug, Error)]
pub enum Error {
    #[error("'{0}' is not a valid cause of death")]
    InvalidCauseOfDeath(String),

    #[error("Found a kill entry but the match hasn't started yet")]
    NoMatch,

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

type Result<T> = result::Result<T, Error>;

fn main() {
    let args = command!()
        .arg(
            Arg::new("PATH")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();
    let path = args.get_one::<PathBuf>("PATH").unwrap();

    if let Err(err) = process_log(path) {
        eprintln!("Error while processing log: {}", err);
        process::exit(1);
    }
}

fn process_log<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut scoreboard = None;
    for entry in KillLog::new(BufReader::new(File::open(path)?)) {
        match entry? {
            LogEntry::InitGame => {
                print_report(&scoreboard);
                println!();
                scoreboard = Some(MatchScoreboard::new());
            }
            LogEntry::Kill(kill) => match &mut scoreboard {
                Some(scoreboard) => scoreboard.record(kill),
                None => return Err(Error::NoMatch),
            },
        }
    }
    print_report(&scoreboard);

    Ok(())
}

fn print_report(scoreboard: &Option<MatchScoreboard>) {
    let Some(scoreboard) = scoreboard else {
        return;
    };

    println!("Match ended with {} kills", scoreboard.total_kills());

    let mut player_scores = scoreboard.player_scores().collect::<Vec<_>>();
    player_scores.sort_by_key(|&(_, player)| Reverse(player.score()));
    for (name, player) in player_scores {
        println!(
            "- {name} ({}/{}):\n    Kills:",
            player.n_kills(),
            player.n_deaths()
        );
        for (ty, count) in player.kills() {
            println!("        - {}: {}", ty, count);
        }
        println!("    Deaths:");
        for (ty, count) in player.deaths() {
            println!("        - {}: {}", ty, count);
        }
        if player.n_suicides() > 0 {
            println!("    Suicides: {}", player.n_suicides());
        }
        println!("    Final score: {}", player.score());
    }

    println!("-> By cause of death:");
    let mut by_cause_of_death = BTreeMap::<_, usize>::new();
    for (_, player) in scoreboard.player_scores() {
        for (cause, count) in player.deaths() {
            *by_cause_of_death.entry(cause).or_default() += count;
        }
    }
    let mut by_cause_of_death = by_cause_of_death.into_iter().collect::<Vec<_>>();
    by_cause_of_death.sort_by_key(|&(_, count)| Reverse(count));
    for (cause, count) in by_cause_of_death {
        println!("    - {cause}: {count}");
    }
}
