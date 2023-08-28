use crate::quake3a::{CauseOfDeath, KillEntry, KillLog, LogEntry, MatchScoreboard};
use maplit::btreeset;
use pretty_assertions::assert_eq;
use std::{
    collections::BTreeSet,
    io::{BufReader, Cursor},
};

#[test]
fn empty_log() {
    let mut kill_log = KillLog::new(BufReader::new(Cursor::new("")));

    assert!(kill_log.next().is_none());
}

#[test]
fn parse_init_game_entry() {
    let mut kill_log = KillLog::new(BufReader::new(Cursor::new(
        r"  0:00 InitGame: \sv_floodProtect\1\sv_maxPing\0\sv_minPing\0\sv_maxRate\10000\sv_minRate\0\sv_hostname\Code Miner Server\g_gametype\0\sv_privateClients\2\sv_maxclients\16\sv_allowDownload\0\dmflags\0\fraglimit\20\timelimit\15\g_maxGameClients\0\capturelimit\8\version\ioq3 1.36 linux-x86_64 Apr 12 ",
    )));

    assert_eq!(kill_log.next().unwrap().unwrap(), LogEntry::InitGame);
    assert!(kill_log.next().is_none());
}

#[test]
fn parse_kill_entry() {
    let mut kill_log = KillLog::new(BufReader::new(Cursor::new(
        r"  2:11 Kill: 2 4 6: Dono da Bola killed Zeh by MOD_ROCKET",
    )));

    assert_eq!(
        kill_log.next().unwrap().unwrap(),
        LogEntry::Kill(KillEntry::new(
            "Dono da Bola",
            "Zeh",
            CauseOfDeath::MOD_ROCKET,
        ))
    );
    assert!(kill_log.next().is_none());
}

#[test]
fn parse_skip_invalid_entry() {
    let mut kill_log = KillLog::new(BufReader::new(Cursor::new(
        r" 26  0:00 ------------------------------------------------------------",
    )));

    assert!(kill_log.next().is_none());
}

#[test]
fn record_world_kill() {
    let mut scoreboard = MatchScoreboard::new();
    scoreboard.record(KillEntry::new(
        "<world>",
        "Isgalamido",
        CauseOfDeath::MOD_TRIGGER_HURT,
    ));

    assert_eq!(scoreboard.total_kills(), 1);

    let mut players = scoreboard.player_scores().collect::<Vec<_>>();
    assert_eq!(players.len(), 1);

    let (name, player) = players.pop().unwrap();
    assert_eq!(name, "Isgalamido");
    assert_eq!(player.n_deaths(), 1);
    assert_eq!(
        player.deaths().collect::<BTreeSet<_>>(),
        btreeset![(CauseOfDeath::MOD_TRIGGER_HURT, 1)],
    );
    assert_eq!(player.n_kills(), 0);
    assert_eq!(player.kills().collect::<BTreeSet<_>>(), BTreeSet::new());
    assert_eq!(player.n_suicides(), 0);
    assert_eq!(player.score(), -1);
}

#[test]
fn record_suicide() {
    let mut scoreboard = MatchScoreboard::new();
    scoreboard.record(KillEntry::new(
        "Isgalamido",
        "Isgalamido",
        CauseOfDeath::MOD_ROCKET_SPLASH,
    ));

    assert_eq!(scoreboard.total_kills(), 1);

    let mut players = scoreboard.player_scores().collect::<Vec<_>>();
    assert_eq!(players.len(), 1);

    let (name, player) = players.pop().unwrap();
    assert_eq!(name, "Isgalamido");
    assert_eq!(player.n_deaths(), 1);
    assert_eq!(
        player.deaths().collect::<BTreeSet<_>>(),
        btreeset![(CauseOfDeath::MOD_ROCKET_SPLASH, 1)],
    );
    assert_eq!(player.n_kills(), 1);
    assert_eq!(
        player.kills().collect::<BTreeSet<_>>(),
        btreeset![(CauseOfDeath::MOD_ROCKET_SPLASH, 1)],
    );
    assert_eq!(player.n_suicides(), 1);
    assert_eq!(player.score(), -1);
}

#[test]
fn record_player_kill() {
    let mut scoreboard = MatchScoreboard::new();
    scoreboard.record(KillEntry::new(
        "Dono da Bola",
        "Zeh",
        CauseOfDeath::MOD_ROCKET,
    ));

    assert_eq!(scoreboard.total_kills(), 1);

    let mut players = scoreboard.player_scores().collect::<Vec<_>>();
    players.sort_by_key(|&(name, _)| name);
    assert_eq!(players.len(), 2);

    let (name, player) = players.pop().unwrap();
    assert_eq!(name, "Zeh");
    assert_eq!(player.n_deaths(), 1);
    assert_eq!(
        player.deaths().collect::<BTreeSet<_>>(),
        btreeset![(CauseOfDeath::MOD_ROCKET, 1)],
    );
    assert_eq!(player.n_kills(), 0);
    assert_eq!(player.kills().collect::<BTreeSet<_>>(), BTreeSet::new());
    assert_eq!(player.n_suicides(), 0);
    assert_eq!(player.score(), 0);

    let (name, player) = players.pop().unwrap();
    assert_eq!(name, "Dono da Bola");
    assert_eq!(player.n_deaths(), 0);
    assert_eq!(player.deaths().collect::<BTreeSet<_>>(), BTreeSet::new(),);
    assert_eq!(player.n_kills(), 1);
    assert_eq!(
        player.kills().collect::<BTreeSet<_>>(),
        btreeset![(CauseOfDeath::MOD_ROCKET, 1)],
    );
    assert_eq!(player.n_suicides(), 0);
    assert_eq!(player.score(), 1);
}
