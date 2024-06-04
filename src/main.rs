// www.codingames.com supports one file only :(

use std::fmt::{Display, Formatter};
use std::io;

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    // game loop
    loop {
        let player = Player::from_stdin();
        let humans = parse_humans();
        let zombies = parse_zombies();

        eprintln!("{:?}", player);
        eprintln!("{:?}", humans);
        eprintln!("{:?}", zombies);

        let next_move = calculate_next_move(player, &humans, &zombies);
        println!("{}", next_move)
    }
}



// ----- Move logic -----
fn calculate_next_move(player: Player, humans: &Vec<Human>, zombies: &Vec<Zombie>) -> Move {
    return Move::new(humans[0].x, humans[0].y)
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
    let strs = input_line.split(" ").collect::<Vec<_>>();
    strs.into_iter().map(atoi).collect()
}

fn read_line_as_i32() -> i32 {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    atoi(&input_line)
}
