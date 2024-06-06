// www.codingames.com supports one file only :(

use std::fmt::{Display, Formatter};
use std::io;

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    // game loop
    loop {
        let state = GameState {
            player: Player::from_stdin(),
            humans: parse_humans(),
            zombies: parse_zombies(),
        };
        eprintln!("{:?}", state);

        let next_move = calculate_next_move(&state);
        println!("{}", next_move)
    }
}

// ----- Game Flow -----

struct Prediction {
    flow: Vec<GameState>,
}

impl Prediction {
    fn new() -> Prediction {
        Prediction { flow: vec![] }
    }
    fn make(start: &GameState, strategy: fn(&GameState) -> Player) -> Prediction {
        let mut pred = Prediction::new();
        pred.flow.push(start.simulate(strategy));
        // TODO: calc all possible
        pred
    }
}


// ----- Game State -----

#[derive(Debug, Clone)]
struct GameState {
    player: Player,
    humans: Vec<Human>,
    zombies: Vec<Zombie>,
}

impl GameState {
    fn new(player: Player, humans: Vec<Human>, zombies: Vec<Zombie>) -> Self {
        GameState { player, humans, zombies }
    }

    fn simulate(&self, strategy: fn(&GameState) -> Player) -> GameState {
        let mut next_state = self.clone();
        next_state.player = strategy(&self);
        // TODO: make calculations
        next_state
    }

    // TODO: test
    fn calc_zombie_targets(&mut self) {
        // TODO: fix mutable/immutable borrows
        self.zombies.iter_mut().for_each(|x| x.set_closest_human(&self));
    }
}



// ----- Move logic -----
fn calculate_next_move(state: &GameState) -> Player {
    return Player::new(state.humans[0].x, state.humans[0].y)
}



// ----- Player -----

#[derive(Debug, Clone)]
struct Player {
    x: i32,
    y: i32,
    msg: String,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player { x, y, msg: "".to_string() }
    }

    fn new_labeled(x: i32, y: i32, label: &str) -> Self {
        Player { x, y, msg: label.to_string() }
    }

    fn from_stdin() -> Self {
        let input = parse_line();
        Player::new(input[0], input[1])
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.msg.is_empty() {
            write!(f, "{} {}", self.x, self.y)
        } else {
            write!(f, "{} {} {}", self.x, self.y, self.msg)
        }
    }
}



// ----- Humans -----

#[derive(Debug, Copy, Clone)]
struct Human {
    id: i32,
    x: i32,
    y: i32,
}

impl Human {
    fn from_stdin() -> Self {
        let input = parse_line();
        Human {
            id: input[0],
            x: input[1],
            y: input[2],
        }
    }
}

fn parse_humans() -> Vec<Human> {
    let mut res = vec![];
    let human_count = read_line_as_i32();
    res.reserve(human_count as usize);
    for _ in 0..human_count {
        res.push(Human::from_stdin());
    }
    res
}



// ----- Zombies -----

#[derive(Debug, Copy, Clone)]
struct Zombie {
    id: i32,
    x: i32,
    y: i32,
    next_x: i32,
    next_y: i32,
    target: Target,
}

impl Zombie {
    fn from_stdin() -> Self {
        let input = parse_line();
        Zombie {
            id: input[0],
            x: input[1],
            y: input[2],
            next_x: input[3],
            next_y: input[4],
            target: Target::Player,
        }
    }

    fn set_closest_human(&mut self, state: &GameState) {
        let mut sq_dist = dist_squared(self.next_x, self.next_y, state.player.x, state.player.y);
        self.target = Target::Player;
        for (idx, human) in state.humans.iter().enumerate() {
            let curr_dist = dist_squared(self.next_x, self.next_y, human.x, human.y);
            if curr_dist < sq_dist {
                sq_dist = curr_dist;
                self.target = Target::Human(idx);
            }
        }
    }
}

fn parse_zombies() -> Vec<Zombie> {
    let mut res = vec![];
    let zombie_count = read_line_as_i32();
    res.reserve(zombie_count as usize);
    for _ in 0..zombie_count {
        res.push(Zombie::from_stdin());
    }
    res
}

#[derive(Debug, Copy, Clone)]
enum Target {
    Player,         // the player
    Human(usize),   // human idx
}


// ----- Utils -----

fn atoi(str: &str) -> i32 {
    str.trim().parse().unwrap_or(0)
}

fn parse_line() -> Vec<i32> {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let strings = input_line.split(" ").collect::<Vec<_>>();
    strings.into_iter().map(atoi).collect()
}

fn read_line_as_i32() -> i32 {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    atoi(&input_line)
}

fn dist_squared(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1-x2) * (x1-x2) + (y1-y2) * (y1-y1)
}

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    (dist_squared(x1, y1, x2, y2) as f32).sqrt()
}
