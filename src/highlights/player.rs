use tf_demo_parser::demo::parser::analyser::Class;
use tf_demo_parser::demo::parser::MatchState;

#[derive(Default)]
pub struct Player {
    username: String,
    pub user_id: u16,
    pub class: &'static str,
}

impl Player {
    pub fn new(state: &MatchState, username: String) -> Self {
        let mut player = Self::default();
        player.username = username;
        player.get_id(state);
        player.get_class(state);
        return player;
    }

    fn get_class(&mut self, state: &MatchState) {
        for (user_id, user_info) in &state.users {
            if u16::from(*user_id) == self.user_id {
                let mut max_value: u8 = 0;
                let mut max_class: Option<Class> = None;

                for (class, value) in user_info.classes.iter() {
                    if value > max_value {
                        max_value = value;
                        max_class = Some(class);
                    }
                }
                self.class = string_from_class(max_class.unwrap());
                return;
            }
        }
        self.class = string_from_class(Class::Other);
    }

    fn get_id(&mut self, state: &MatchState) {
        for (user_id, user_info) in &state.users {
            if user_info.name == self.username {
                self.user_id = u16::from(*user_id);
                return;
            }
        }
        self.user_id = 0;
    }
}

fn string_from_class(class: Class) -> &'static str {
    match class {
        Class::Scout => "scout",
        Class::Soldier => "soldier",
        Class::Pyro => "pyro",
        Class::Demoman => "demoman",
        Class::Heavy => "heavy",
        Class::Engineer => "engineer",
        Class::Medic => "medic",
        Class::Sniper => "sniper",
        Class::Spy => "spy",
        _ => "other",
    }
}
