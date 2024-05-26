use tf_demo_parser::{demo::parser::analyser::Death, MatchState};

use crate::player::Player;

const KILL_INTERVAL: f64 = 15.0; // P-REC default = 15.0 seconds
const TICK: f64 = 0.015; // Amount of seconds per tick

#[derive(Default, Clone, Debug)]
pub struct Killstreak {
    pub kills: Kills,
    pub start_tick: u32,
    pub end_tick: u32,
    pub length: f64, // seconds
}

#[derive(Default, Clone, Debug)]
pub struct Killstreaks {
    pub killstreaks: Vec<Killstreak>,
}

#[derive(Default, Clone, Debug)]
pub struct Kill {
    tick: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Kills {
    pub kills: Vec<Kill>,
}

impl<'a> Kills {
    pub fn new(player: &Player, state: &MatchState) -> Self {
        let mut kills = Self::default();
        let deaths: Vec<Death> = state
            .deaths
            .iter()
            .filter(|death| death.killer == player.user_id && death.killer != death.victim)
            .cloned()
            .collect();
        for death in deaths {
            kills.kills.push(Kill {
                tick: u32::from(death.tick),
            });
        }
        return kills;
    }
}

impl Killstreaks {
    pub fn new(player: &Player, state: &MatchState) -> Self {
        let kills = Kills::new(&player, &state);
        let killstreaks = Self::from_kills(&kills);
        return killstreaks;
    }

    fn from_kills(kills: &Kills) -> Killstreaks {
        let mut killstreaks = Killstreaks::default();
        let mut streak = Killstreak::default();
        let mut last_kill = &kills.kills[0];

        streak.start_tick = last_kill.tick;

        for current_kill in &kills.kills[1..] {
            let time_between_kills =
                (f64::from(current_kill.tick) - f64::from(last_kill.tick)) * TICK;
            streak.kills.kills.push(last_kill.clone());

            if time_between_kills <= KILL_INTERVAL {
                streak.end_tick = current_kill.tick
            } else {
                if streak.kills.kills.len() >= 4 {
                    streak.length =
                        (f64::from(streak.end_tick) - f64::from(streak.start_tick)) * TICK;
                    killstreaks.killstreaks.push(streak.clone());
                }

                streak = Killstreak::default();
                streak.start_tick = current_kill.tick;
            }
            last_kill = current_kill;
        }

        return killstreaks;
    }
}
