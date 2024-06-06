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


#[derive(Debug)]
struct GameState {
    player: Player,
    humans: Vec<Human>,
    zombies: Vec<Zombie>,
}

impl GameState {
    fn new(player: Player, humans: Vec<Human>, zombies: Vec<Zombie>) -> Self {
        GameState { player, humans, zombies }
    }

    // TODO: test
    fn calc_zombie_targets(&mut self) {
        for zombie in &mut self.zombies {
            zombie.target_idx = closest_human_idx(zombie.next_x, zombie.next_y, &self.humans);
        }
    }
}



// ----- Move logic -----
fn calculate_next_move(state: &GameState) -> Move {
    return Move::new(state.humans[0].x, state.humans[0].y)
}

struct Move {
    x: i32,
    y: i32,
    msg: String,
}

impl Move {
    fn new(x: i32, y: i32) -> Self {
        Move { x, y, msg: "".to_string() }
    }

    fn new_labeled(x: i32, y: i32, label: &str) -> Self {
        Move { x, y, msg: label.to_string() }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.msg.is_empty() {
            write!(f, "{} {}", self.x, self.y)
        } else {
            write!(f, "{} {} {}", self.x, self.y, self.msg)
        }
    }
}



// ----- Player -----

#[derive(Debug)]
struct Player {
    x: i32,
    y: i32,
}

impl Player {
    fn from_stdin() -> Self {
        let input = parse_line();
        Player { x: input[0], y: input[1] }
    }
}



// ----- Humans -----

#[derive(Debug)]
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

#[derive(Debug)]
struct Zombie {
    id: i32,
    x: i32,
    y: i32,
    next_x: i32,
    next_y: i32,
    target_idx: usize,
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
            target_idx: 0,
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

fn closest_human_idx(x: i32, y: i32, humans: &Vec<Human>) -> usize {
    let mut idx = 0usize;
    let mut sq_dist = i32::MAX;

    for (i, human) in humans.iter().enumerate() {
        let curr_dist = dist_squared(x, y, human.x, human.y);
        if curr_dist < sq_dist {
            idx = i;
            sq_dist = sq_dist;
        }
    }

    idx
}

fn dist_squared(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1-x2) * (x1-x2) + (y1-y2) * (y1-y1)
}

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    (dist_squared(x1, y1, x2, y2) as f32).sqrt()
}
