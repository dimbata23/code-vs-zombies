// www.codingames.com supports one file only :(

use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io;

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    let mut last_prediction = None;

    // game loop
    loop {
        let state = GameState {
            player: Player::from_stdin(),
            humans: parse_humans(),
            zombies: parse_zombies(),
        };

        eprintln!("Current : {}", state);
        if let Some(pred_state) = &last_prediction {
            eprintln!("Pred was: {}", *pred_state);
            eprintln!("{}", *pred_state == state);
        }

        last_prediction = Some(state.simulate(calculate_next_move));

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

#[derive(Debug, Clone, PartialEq)]
struct GameState {
    player: Player,
    humans: Vec<Human>,
    zombies: Vec<Zombie>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = write!(f, "P: ({}, {}), {}H, {}Z: ", self.player.x, self.player.y, self.humans.len(), self.zombies.len());
        for zombie in &self.zombies {
            res = res.and(write!(f, "{} ", zombie));
        }
        res
    }
}

impl GameState {
    fn new(player: Player, humans: Vec<Human>, zombies: Vec<Zombie>) -> Self {
        GameState { player, humans, zombies }
    }

    fn simulate(&self, strategy: fn(&GameState) -> Player) -> GameState {
        let mut next_state = self.clone();
        next_state.move_zombies();
        let player_target = strategy(&self);
        (next_state.player.x, next_state.player.y) = move_from_to_capped((self.player.x, self.player.y), (player_target.x, player_target.y), PLAYER_STEP);
        next_state.kill_zombies();
        next_state.kill_humans();
        next_state.calc_zombies_next_move();
        next_state
    }

    fn calc_zombies_next_move(&mut self) {
        let mut new_zombies = self.zombies.clone();  // old compilers :((((
        new_zombies.iter_mut().for_each(|x| x.set_target(&self.player, &self.humans));
        new_zombies.iter_mut().for_each(|z| z.set_next_move(&self.player, &self.humans));
        self.zombies = new_zombies;
    }

    fn move_zombies(&mut self) {
        for zombie in &mut self.zombies {
            (zombie.x, zombie.y) = (zombie.next_x, zombie.next_y);
        }
    }

    fn kill_zombies(&mut self) {
        let mut new_zombies = self.zombies.clone();  // old compilers :((((
        new_zombies.iter_mut().for_each(|z| z.check_within_player(&self.player));
        self.zombies = new_zombies.iter().filter(|z| !z.dead).cloned().collect();
    }

    fn kill_humans(&mut self) {
        let mut new_humans = self.humans.clone();  // old compilers :((((
        new_humans.iter_mut().for_each(|h| h.check_within_zombie(&self.zombies));
        self.humans = new_humans.iter().filter(|h| !h.dead).cloned().collect();
    }
}



// ----- Move logic -----
fn calculate_next_move(state: &GameState) -> Player {
    //Player::new(0, 0)
    Player::new(state.humans[0].x, state.humans[0].y)
}



// ----- Player -----

const PLAYER_RANGE: i32 = 2000;
const PLAYER_STEP: i32 = 1000;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
struct Human {
    id: i32,
    x: i32,
    y: i32,
    dead: bool,
}

impl Human {
    fn from_stdin() -> Self {
        let input = parse_line();
        Human {
            id: input[0],
            x: input[1],
            y: input[2],
            dead: false,
        }
    }

    fn check_within_zombie(&mut self, zombies: &[Zombie]) {
        self.dead = zombies.iter().any(|z| (z.x, z.y) == (self.x, self.y));
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

const ZOMBIE_STEP: i32 = 400;

#[derive(Debug, Copy, Clone)]
struct Zombie {
    id: i32,
    x: i32,
    y: i32,
    next_x: i32,
    next_y: i32,
    target: Target,
    dead: bool,
}

impl Display for Zombie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{id: {}, pos: ({}, {}), next: ({}, {})}}", self.id, self.x, self.y, self.next_x, self.next_y)
    }
}

impl PartialEq for Zombie {
    fn eq(&self, other: &Self) -> bool {
        (self.id, self.x, self.y, self.next_x, self.next_y) == (other.id, other.x, other.y, other.next_x, other.next_y)
    }
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
            dead: false,
        }
    }

    fn set_target(&mut self, player: &Player, humans: &[Human]) {
        let mut sq_dist = dist_squared((self.next_x, self.next_y), (player.x, player.y));
        self.target = Target::Player;
        for (idx, human) in humans.iter().enumerate() {
            let curr_dist = dist_squared((self.next_x, self.next_y), (human.x, human.y));
            if curr_dist < sq_dist {
                sq_dist = curr_dist;
                self.target = Target::Human(idx);
            }
        }
    }

    fn check_within_player(&mut self, player: &Player) {
        self.dead = dist_squared((self.next_x, self.next_y), (player.x, player.y)) <= PLAYER_RANGE * PLAYER_RANGE;
    }

    fn set_next_move(&mut self, player: &Player, humans: &[Human]) {
        self.set_target(player, humans);
        let (mut target_x, mut target_y) = (player.x, player.y);
        if let Target::Human(idx) = self.target {
            (target_x, target_y) = (humans[idx].x, humans[idx].y);
        }
        (self.next_x, self.next_y) = move_from_to_capped((self.next_x, self.next_y), (target_x, target_y), ZOMBIE_STEP);
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

#[derive(Debug, Copy, Clone, PartialEq)]
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

fn dist_squared((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> i32 {
    (x1-x2) * (x1-x2) + (y1-y2) * (y1-y2)
}

fn dist((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> f32 {
    (dist_squared((x1, y1), (x2, y2)) as f32).sqrt()
}

fn move_from_to_capped((x1, y1): (i32, i32), (x2, y2): (i32, i32), cap: i32) -> (i32, i32) {
    if dist((x1, y1), (x2, y2)) <= cap as f32 {
        return (x2, y2);
    }

    let dir = scale(norm(vec((x1, y1), (x2, y2))), cap);
    add((x1, y1), dir)
}

fn vec((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> (i32, i32) {
    (x2 - x1, y2 - y1)
}

fn norm((x, y): (i32, i32)) -> (f32, f32) {
    let len = ((x*x + y*y) as f32).sqrt();
    (x as f32 / len, y as f32 / len)
}

fn scale((x, y): (f32, f32), scalar: i32) -> (f32, f32) {
    (x * scalar as f32, y * scalar as f32)
}

fn add((x1, y1): (i32, i32), (x2, y2): (f32, f32)) -> (i32, i32) {
    (((x1 as f32) + x2) as i32, ((y1 as f32) + y2) as i32)
}
