// www.codingames.com supports one file only :(

use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    let state = GameState {
        player: Player::from_stdin(),
        humans: parse_humans(),
        zombies: parse_zombies(),
    };

    eprintln!("Current : {}", state);

    let mut prediction = Prediction::make(&state, Strategy::closest_savable_human);
    let mut last_state_prediction = prediction.next();
    println!("{}", last_state_prediction.unwrap_or(&GameState::empty()).player);

    eprintln!("Commence...");

    // game loop
    loop {
        let state = GameState {
            player: Player::from_stdin(),
            humans: parse_humans(),
            zombies: parse_zombies(),
        };

        eprintln!("Current : {}", state);
        if let Some(prev_pred_state) = last_state_prediction {
            eprintln!("Pred was: {}", *prev_pred_state);
            eprintln!("{}", *prev_pred_state == state);
        }

        last_state_prediction = prediction.next();

        println!("{}", &last_state_prediction.unwrap_or(&GameState::empty()).player);
    }
}

// ----- Game Flow -----

struct Prediction {
    flow: Vec<GameState>,
    idx: usize,
}

impl Prediction {
    fn new() -> Prediction {
        Prediction { flow: vec![], idx: 0 }
    }
    fn make(start: &GameState, strategy: fn(&GameState) -> Player) -> Prediction {
        let mut pred = Prediction::new();
        pred.flow.push(start.simulate(strategy));
        while !pred.flow.last().unwrap().ended() {
            pred.flow.push(pred.flow.last().unwrap().simulate(strategy))
        }
        pred
    }

    fn next(&mut self) -> Option<&GameState> {
        if self.idx >= self.flow.len() {
            None
        } else {
            self.idx += 1;
            Some(&self.flow[self.idx - 1])
        }
    }
}


// ----- Game State -----

const MAX_X: i32 = 16000;
const MAX_Y: i32 = 9000;

#[derive(Debug, Clone, PartialEq)]
struct GameState {
    player: Player,
    humans: Vec<Human>,
    zombies: Vec<Zombie>,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = write!(f, "P: ({}), {}H, {}Z: ", self.player.pos, self.humans.len(), self.zombies.len());
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

    fn empty() -> GameState {
        GameState { player: Player::new_labeled(Vec2::new(), "???"), humans: vec![], zombies: vec![] }
    }

    fn simulate(&self, strategy: fn(&GameState) -> Player) -> GameState {
        let mut next_state = self.clone();
        next_state.clear_targets();
        next_state.zombies_set_targets();
        let player_target = strategy(&next_state);
        next_state.player.pos = move_from_to_capped(next_state.player.pos, player_target.pos, PLAYER_STEP);
        next_state.player.msg = player_target.msg;
        next_state.move_zombies();
        next_state.kill_zombies();
        next_state.kill_humans();
        next_state.calc_zombies_next_move();
        next_state
    }

    fn zombies_set_targets(&mut self) {
        let mut new_zombies = self.zombies.clone();  // old compilers :((((
        new_zombies.iter_mut().enumerate().for_each(
            |(z_idx, z)| {
                z.set_target(&self.player, &self.humans);
                if let Target::Human(h_idx) = z.target {
                    self.humans[h_idx].targeted_by = Some(z_idx);
                }
            }
        );
        self.zombies = new_zombies;
    }

    fn calc_zombies_next_move(&mut self) {
        let mut new_zombies = self.zombies.clone();  // old compilers :((((
        new_zombies.iter_mut().for_each(|z| z.set_next_move(&self.player, &self.humans));
        self.zombies = new_zombies;
    }

    fn move_zombies(&mut self) {
        for zombie in &mut self.zombies {
            zombie.pos = zombie.next_pos;
        }
    }

    fn kill_zombies(&mut self) {
        let new_zombies = self.zombies.clone();  // old compilers :((((
        self.zombies = new_zombies.iter().filter(|z| !z.check_within_player(&self.player)).cloned().collect();
    }

    fn kill_humans(&mut self) {
        let new_humans = self.humans.clone();  // old compilers :((((
        self.humans = new_humans.iter().filter(|h| !h.check_within_zombie(&self.zombies)).cloned().collect();
    }

    fn ended(&self) -> bool {
        self.humans.is_empty() || self.zombies.is_empty()
    }

    fn clear_targets(&mut self) {
        self.humans.iter_mut().for_each(|h| h.targeted_by = None);
        self.zombies.iter_mut().for_each(|z| { z.target = Target::Player; z.target_dist_sq = i32::MAX; });
    }
}



// ----- Strategies -----
struct Strategy;

impl Strategy {
    fn stay_still(state: &GameState) -> Player {
        Player { pos: state.player.pos, msg: "Zzz...".to_string() }
    }

    fn random_pos(_: &GameState) -> Player {
        Player {
            pos: Vec2 {
                x: rand_in_range(0, MAX_X),
                y: rand_in_range(0, MAX_Y),
            },
            msg: "Random Bullshit Go!!!".to_string()
        }
    }

    fn closest_savable_human(state: &GameState) -> Player {
        if state.humans.is_empty() {
            return Strategy::stay_still(state);
        }

        let mut msg = "Save human";
        let mut closest_from: Vec<_> = state.humans.iter().filter(|h| h.savable(&state.player, &state.zombies)).collect();
        if closest_from.is_empty() {
            closest_from = state.humans.iter().filter(|h| h.targeted_by == None).collect();
            if closest_from.is_empty() {
                closest_from = state.humans.iter().collect();
                msg = "Fuuuck!";
            }
        }

        let closest_human = closest_from.iter().min_by_key(
            |h| {
                if let Some(z_idx) = h.targeted_by {
                    state.zombies[z_idx].target_dist_sq
                } else {
                    dist_squared(h.pos, state.player.pos)
                }
            }
        );

        if let Some(h) = closest_human {
            Player::new_labeled(h.pos, msg)
        } else {
            Player::new_labeled(state.player.pos, msg)
        }
    }
}


// ----- Player -----

const PLAYER_RANGE: i32 = 2000;
const PLAYER_STEP: i32 = 1000;

#[derive(Debug, Clone)]
struct Player {
    pos: Vec2,
    msg: String,
}

impl Player {
    fn new(pos: Vec2) -> Self {
        Player { pos, msg: "".to_string() }
    }

    fn new_labeled(pos: Vec2, label: &str) -> Self {
        Player { pos, msg: label.to_string() }
    }

    fn from_stdin() -> Self {
        let input = parse_line();
        Player::new(Vec2 {x: input[0], y: input[1]})
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.msg.is_empty() {
            write!(f, "{}", self.pos)
        } else {
            write!(f, "{} {}", self.pos, self.msg)
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}



// ----- Humans -----

#[derive(Debug, Copy, Clone)]
struct Human {
    id: i32,
    pos: Vec2,
    targeted_by: Option<usize>,
}

impl Human {
    fn from_stdin() -> Self {
        let input = parse_line();
        Human {
            id: input[0],
            pos: Vec2 {x: input[1], y: input[2]},
            targeted_by: None
        }
    }

    fn check_within_zombie(&self, zombies: &[Zombie]) -> bool {
        zombies.iter().any(|z| z.pos == self.pos)
    }

    fn savable(&self, player: &Player, zombies: &[Zombie]) -> bool {
        if self.targeted_by.is_none() {
            return false;
        }

        true  // TODO:
    }
}

impl PartialEq for Human {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.pos == other.pos
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
    pos: Vec2,
    next_pos: Vec2,
    target: Target,
    target_dist_sq: i32,
}

impl Display for Zombie {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{id: {}, pos: ({}), next: ({})}}", self.id, self.pos, self.next_pos)
    }
}

impl PartialEq for Zombie {
    fn eq(&self, other: &Self) -> bool {
        (self.id, self.pos, self.next_pos) == (other.id, other.pos, other.next_pos)
    }
}

impl Zombie {
    fn from_stdin() -> Self {
        let input = parse_line();
        Zombie {
            id: input[0],
            pos: Vec2{x: input[1], y: input[2]},
            next_pos: Vec2{x: input[3], y: input[4]},
            target: Target::Player,
            target_dist_sq: i32::MAX,
        }
    }

    fn set_target(&mut self, player: &Player, humans: &[Human]) {
        self.target_dist_sq = dist_squared(self.next_pos, player.pos);
        self.target = Target::Player;
        for (idx, human) in humans.iter().enumerate() {
            let curr_dist = dist_squared(self.next_pos, human.pos);
            if curr_dist < self.target_dist_sq {
                self.target_dist_sq = curr_dist;
                self.target = Target::Human(idx);
            }
        }
    }

    fn check_within_player(&self, player: &Player) -> bool {
        dist_squared(self.next_pos, player.pos) <= PLAYER_RANGE * PLAYER_RANGE
    }

    fn set_next_move(&mut self, player: &Player, humans: &[Human]) {
        self.set_target(player, humans);  // TODO: is this needed?
        let mut target_pos = player.pos;
        if let Target::Human(idx) = self.target {
            target_pos = humans[idx].pos;
        }
        self.next_pos = move_from_to_capped(self.next_pos, target_pos, ZOMBIE_STEP);
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
    eprint!("{}", input_line);
    let strings = input_line.split(" ").collect::<Vec<_>>();
    strings.into_iter().map(atoi).collect()
}

fn read_line_as_i32() -> i32 {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    eprint!("{}", input_line);
    atoi(&input_line)
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct MathVec2<T> {
    x: T,
    y: T,
}

type Vec2 = MathVec2<i32>;
type Vec2f = MathVec2<f64>;

impl From<Vec2> for Vec2f {
    fn from(value: Vec2) -> Self {
        Self { x: value.x as f64, y: value.y as f64 }
    }
}

impl From<Vec2f> for Vec2 {
    fn from(value: Vec2f) -> Self {
        Self { x: value.x as i32, y: value.y as i32 }
    }
}

impl<T: AddAssign> AddAssign for MathVec2<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: AddAssign> Add for MathVec2<T> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<T> MathVec2<T>
where
    T: Copy + Mul<Output = T> + Add<Output = T> + MulAssign, f64: From<T>
{
    fn len(&self) -> f64
        where <T as Mul>::Output: Add
    {
        Into::<f64>::into(self.x*self.x + self.y*self.y).sqrt()
    }

    fn scale(&mut self, scalar: T) {
        self.x *= scalar;
        self.y *= scalar;
    }

    fn scaled(&self, scalar: T) -> Self {
        let mut res = *self;
        res.scale(scalar);
        res
    }
}

impl Vec2 {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Vec2f {
    fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn from_points(from: Vec2, to: Vec2) -> Self {
        Self { x: (to.x - from.x) as f64, y: (to.y - from.y) as f64 }
    }

    fn normalize(&mut self) {
        let len = self.len();
        self.x /= len;
        self.y /= len;
    }

    fn norm(&self) -> Self {
        let mut res = *self;
        res.normalize();
        res
    }
}

impl<T: Display> Display for MathVec2<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

fn sq<T>(num: T) -> T
where
    T: Mul<Output = T> + Copy
{
    num*num
}

fn dist_squared<T>(pt1: MathVec2<T>, pt2: MathVec2<T>) -> T
where
    T: Sub<Output = T> + Add<Output = T> + Mul<Output = T> + Copy
{
    sq(pt1.x-pt2.x) + sq(pt1.y-pt2.y)
}

fn dist<T>(pt1: MathVec2<T>, pt2: MathVec2<T>) -> f64
where
    T: Sub<Output = T> + Add<Output = T> + Mul<Output = T> + Copy, f64: From<T>
{
    Into::<f64>::into(dist_squared(pt1, pt2)).sqrt()
}

fn move_from_to_capped(from: Vec2, to: Vec2, cap: i32) -> Vec2 {
    if dist(from, to) <= cap as f64 {
        return to;
    }

    let dir = Vec2f::from_points(from, to).norm().scaled(cap as f64);
    from + dir.into()
}

fn rand() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos()
}

fn rand_in_range(min: i32, max_exclusive: i32) -> i32 {
    let range = max_exclusive - min;
    let rand = rand() % range as u32;
    min + rand as i32
}
