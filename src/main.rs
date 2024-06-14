// www.codingames.com supports one file only :(

use std::cmp::PartialEq;
use std::f64::consts::PI;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    let mut opt_last_state: Option<GameState> = None;

    // game loop
    loop {
        let mut state = GameState::new(
            Player::from_stdin(),
            parse_humans(),
            parse_zombies()
        );

        if opt_last_state.is_none() {
            opt_last_state = Some(state.clone());
        }
        else {
            let last_state = opt_last_state.as_ref().unwrap();
            state.calculate_new_score(last_state);
            opt_last_state = Some(state.clone());
            eprintln!("Score: {}", state.score);
        }

        const LOOKAHEAD_TURNS: i32 = 12;
        let mut sim_tree = SimTree::with_strategies(&[Strategy::save_humans, Strategy::go_kill]);
        let best_state = sim_tree.calculate_best_state(&state, LOOKAHEAD_TURNS);
        println!("{}", best_state.player);
    }
}

// ----- Game Flow -----

type StrategyFn = fn(&GameState) -> Player;

struct SimTree {
    strategies: Vec<StrategyFn>,
    best_score: i32,
    best_state: GameState,
}

impl SimTree {
    fn with_strategies(strategies: &[StrategyFn]) -> Self {
        Self { strategies: strategies.iter().cloned().collect(), best_score: -1, best_state: GameState::empty() }
    }

    fn calculate_best_state(&mut self, starting_state: &GameState, lookahead_turns: i32) -> GameState {
        for strategy in self.strategies.iter() {
            let state = starting_state.simulate(*strategy);
            let max_score = self.calc_max_score_inner_rec(&state, lookahead_turns);
            if max_score > self.best_score {
                self.best_score = max_score;
                self.best_state = state;
            }
        }

        self.best_state.clone()
    }

    fn calc_max_score_inner_rec(&self, state: &GameState, depth: i32) -> i32 {
        if state.ended() || depth == 0 {
            if state.humans.is_empty() || !state.clone().winnable() {
                return -1;
            }

            return state.score;
        }

        let mut max_score = state.score;
        for strategy in self.strategies.iter() {
            let new_state = state.simulate(*strategy);
            let score = self.calc_max_score_inner_rec(&new_state, depth - 1);
            if score > max_score {
                max_score = score;
            }
        }

        max_score
    }
}

// ----- Game State -----

const FIB: [i32; 30] = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89,144,233,377,610,987, 1597, 2584, 4181, 6765, 10946, 17711, 28657, 46368, 75025, 121393, 196418, 317811, 514229];
const ZOMBIE_PTS: i32 = 10;

#[derive(Debug, Clone, PartialEq)]
struct GameState {
    player: Player,
    humans: Vec<Human>,
    zombies: Vec<Zombie>,
    score: i32,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res = write!(f, "S: {}, P: ({}), {}H, {}Z: ", self.score, self.player.pos, self.humans.len(), self.zombies.len());
        for zombie in &self.zombies {
            res = res.and(write!(f, "{} ", zombie));
        }
        res
    }
}

impl GameState {
    fn new(player: Player, humans: Vec<Human>, zombies: Vec<Zombie>) -> Self {
        GameState { player, humans, zombies, score: 0 }
    }

    fn empty() -> GameState {
        GameState { player: Player::new_labeled(Vec2::new(), "???"), humans: vec![], zombies: vec![], score: 0 }
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
        for i in 0..self.zombies.len() {
            self.zombies[i].set_target(&self.player, &self.humans);
            if let Target::Human(h_idx) = self.zombies[i].target {
                self.humans[h_idx].set_target(&self.zombies, i);
            }
        }

        // DEBUG
        // eprint!("Zombie targets:\n\t");
        // for zombie in &self.zombies {
        //     let target_id = if let Target::Human(idx) = &zombie.target { self.humans[*idx].id } else { -1 };
        //     eprint!("{} -> {} | ", zombie.id, target_id)
        // }
        // eprintln!();
        //
        // eprint!("Humans targeted by:\n\t");
        // for human in &self.humans {
        //     let target_id = if let Some(idx) = &human.targeted_by { self.zombies[*idx].id } else { -1 };
        //     eprint!("{} -> {} | ", human.id, target_id)
        // }
        // eprintln!();
    }

    fn calc_zombies_next_move(&mut self) {
        for z in self.zombies.iter_mut() {
            z.set_next_move(&self.player, &self.humans);
        }
    }

    fn move_zombies(&mut self) {
        for zombie in &mut self.zombies {
            zombie.pos = zombie.next_pos;
        }
    }

    fn calc_score_for_zombie_kills(killed_zombies_count: usize, humans_alive_count: usize) -> i32 {
        let mut res_score = 0;
        let zombie_reward = ZOMBIE_PTS * sq(humans_alive_count as i32);
        for i in 1..=killed_zombies_count {
            res_score += zombie_reward * FIB[i+1];
        }
        res_score
    }

    fn kill_zombies(&mut self) {
        let before_cnt = self.zombies.len();
        self.zombies = self.zombies.iter().filter(|z| !z.check_within_player(&self.player)).cloned().collect();
        let after_cnt = self.zombies.len();
        let killed_cnt = before_cnt - after_cnt;
        self.score += Self::calc_score_for_zombie_kills(killed_cnt, self.humans.len());
    }

    fn kill_humans(&mut self) {
        self.humans = self.humans.iter().filter(|h| !h.check_within_zombie(&self.zombies)).cloned().collect();
    }

    fn ended(&self) -> bool {
        self.humans.is_empty() || self.zombies.is_empty()
    }

    fn clear_targets(&mut self) {
        self.humans.iter_mut().for_each(|h| h.targeted_by = None);
        self.zombies.iter_mut().for_each(|z| { z.target = Target::Player; z.target_dist_sq = i32::MAX; });
    }

    fn calculate_new_score(&mut self, previous_state: &GameState) {
        let zombie_kills = previous_state.zombies.len() - self.zombies.len();
        let move_got_score = Self::calc_score_for_zombie_kills(zombie_kills, previous_state.humans.len());
        eprintln!("Killed {} zombies and got {} more points.", zombie_kills, move_got_score);
        self.score = previous_state.score + move_got_score;
    }

    fn winnable(&mut self) -> bool {
        self.clear_targets();
        self.zombies_set_targets();
        self.zombies.is_empty() || self.humans.iter().any(|h| h.savable(&self.player, &self.zombies))
    }
}



// ----- Strategies -----
struct Strategy;

impl Strategy {
    fn save_humans(state: &GameState) -> Player {
        // TODO: No need to hug the humans
        if state.zombies.len() == 1 {
            if let Target::Human(h_idx) = state.zombies[0].target {
                return Player::new_labeled(state.humans[h_idx].pos, "Shoo");
            }
            return Player::new_labeled(state.zombies[0].pos, "Shoo");
        }

        let mut msg = "protec";
        let mut closest_from: Vec<_> = state.humans.iter().filter(|h| h.savable(&state.player, &state.zombies)).collect();
        //eprintln!("{} savable humans", closest_from.len());
        if closest_from.is_empty() {
            closest_from = state.humans.iter().filter(|h| h.targeted_by.is_none()).collect();
            //eprintln!("{} unknown state humans", closest_from.len());
            if closest_from.is_empty() {
                closest_from = state.humans.iter().collect();
                //eprintln!("No humans are savable :(");
                msg = "RIP";
            }
        }

        let closest_human = closest_from.iter().min_by_key(
            |h| {
                if let Some(z_idx) = h.targeted_by {
                    state.zombies[z_idx].target_dist_sq
                }
                else {
                    dist_squared(h.pos, state.player.pos)
                }
            }
        );

        if let Some(h) = closest_human {
            Player::new_labeled(h.pos, msg)
        }
        else {
            Player::new_labeled(state.player.pos, msg)
        }
    }

    fn go_kill(state: &GameState) -> Player {
        let msg = "KILL 'EM ALL";

        if state.zombies.len() == 1 {
            return Player::new_labeled(state.zombies[0].pos, msg);
        }

        let zombies_targeted_player: Vec<_> = state.zombies.iter().filter(|z| if let Target::Player = z.target { true } else { false }).collect();
        if zombies_targeted_player.len() == state.zombies.len() {
            let coord_sum: Vec2f = zombies_targeted_player.iter().map(|z| z.pos).fold(Vec2::new(), |a, b| a + b).into();
            let centroid = coord_sum / (zombies_targeted_player.len() as f64);
            return Player::new_labeled(centroid.into(), msg)
        }

        let closest_from: Vec<_> = state.humans.iter().filter(|h| h.savable(&state.player, &state.zombies)).collect();
        let closest_human = closest_from.iter().min_by_key(
            |h| {
                if let Some(z_idx) = h.targeted_by {
                    state.zombies[z_idx].target_dist_sq
                }
                else {
                    dist_squared(h.pos, state.player.pos)
                }
            }
        );

        if let Some(h) = closest_human {
            let human_ptr= *h as *const Human;
            let humans_start_ptr = &state.humans[0] as *const Human;
            let human_idx = unsafe{ human_ptr.offset_from(humans_start_ptr) as usize };
            let zombies_targeting_human: Vec<_> = state.zombies.iter().filter(|z| z.target == Target::Human(human_idx)).collect();
            if zombies_targeting_human.len() == 1 {
                return Player::new_labeled(zombies_targeting_human[0].pos, msg);
            }

            let farthest = (zombies_targeting_human.iter().max_by_key(|z| z.target_dist_sq).unwrap().target_dist_sq as f64).sqrt();
            let weight_fn = |dist: i32| farthest - (dist as f64).sqrt();
            let sum_weights = zombies_targeting_human.iter().map(|z| weight_fn(z.target_dist_sq)).fold(0f64, |a, b| a + b);
            let centroid_weighted: Vec2f = zombies_targeting_human.iter().map(|z| (<Vec2 as Into<Vec2f>>::into(z.pos).scaled(weight_fn(z.target_dist_sq))) / sum_weights).fold(Vec2f::new(), |a, b| a + b).into();

            return Player::new_labeled(centroid_weighted.into(), msg);
        }

        // fallback
        let closest_zombie = state.zombies.iter().min_by_key(|z| dist_squared(z.pos, state.player.pos)).unwrap().pos;
        Player::new_labeled(closest_zombie, msg)
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
        }
        else {
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

    fn set_target(&mut self, zombies: &[Zombie], idx: usize) {
        let zombie_dist = dist_squared(self.pos, zombies[idx].pos);
        if self.targeted_by.is_none() {
            self.targeted_by = Some(idx);
        }
        else {
            let target_idx = self.targeted_by.unwrap();
            let target_dist = zombies[target_idx].target_dist_sq;
            if zombie_dist < target_dist {
                self.targeted_by = Some(idx);
            }
        }
    }

    fn savable(&self, player: &Player, zombies: &[Zombie]) -> bool {
        match &self.targeted_by {
            None => { false }
            Some(z_idx) => {
                let zombie = &zombies[*z_idx];
                let hz = Vec2f::from_points(self.pos, zombie.pos);
                let hp = Vec2f::from_points(self.pos, player.pos);
                let angle = hp.angle_to(hz);
                let zombie_turns = f64::ceil(hz.len() / (ZOMBIE_STEP as f64)) as i32;
                if angle.abs() >= PI/2.0 {
                    let player_turns = f64::ceil((hp.len() - PLAYER_RANGE as f64) / (PLAYER_STEP as f64)) as i32;
                    player_turns <= zombie_turns
                }
                else {
                    let proj_zombie_step = angle.cos() * ZOMBIE_STEP as f64;
                    let z_projected_len = hp.len() - proj_zombie_step;
                    let zombie_delta = PLAYER_STEP as f64 - proj_zombie_step;
                    let player_turns = f64::ceil((z_projected_len - PLAYER_RANGE as f64) / zombie_delta) as i32;
                    player_turns <= zombie_turns
                }
            }
        }
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
    //eprint!("{}", input_line);
    let strings = input_line.split(" ").collect::<Vec<_>>();
    strings.into_iter().map(atoi).collect()
}

fn read_line_as_i32() -> i32 {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    //eprint!("{}", input_line);
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

impl<T> Mul for MathVec2<T>
where
    T: Mul<Output = T> + Add<Output = T>,
{
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl<T> Div<T> for MathVec2<T>
    where T: Div<Output = T> + Copy
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self { x: self.x/rhs, y: self.y/rhs }
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

    fn norm(mut self) -> Self {
        self.normalize();
        self
    }

    fn angle_to(self, other: Vec2f) -> f64 {
        f64::acos((self * other) / (self.len() * other.len()))
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
