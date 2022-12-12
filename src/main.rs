use std::{io, collections::VecDeque};
use rand::Rng;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Debug, PartialEq, Eq)]
enum Owner {
    Neutral,
    Me,
    Enemy,
}

impl Default for Owner {
    fn default() -> Self {
        Owner::Neutral
    }
}

impl From<i32> for Owner {
    fn from(n: i32) -> Self {
        match n {
            -1 => Owner::Neutral,
            0 => Owner::Enemy,
            1 => Owner::Me,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Default)]
struct Location {
    scrap_amount: i32,
    owner: Owner,
    units: i32,
    recycler: bool,
    can_build: bool,
    can_spawn: bool,
    in_range_of_recycler: bool,
}

struct Game {
    width: usize,
    height: usize,
    grid: Vec<Vec<Location>>,
    my_matter: i32,
    enemy_matter: i32,
    my_robots: Vec<(usize, usize)>,
    grid_dist_to_outside: Vec<Vec<i32>>,
}

fn bool_from_i32(n: i32) -> bool {
    match n {
        0 => false,
        _ => true,
    }
}

enum Action {
    Move { amount: usize, fromX: usize, fromY: usize, toX: usize, toY: usize },
    Build { x: usize, y: usize },
    Spawn { amount: i32, x: usize, y: usize },
    Wait,
    Message { text: String },
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Self::Move { amount, fromX, fromY, toX, toY } =>
                format!("MOVE {amount} {fromX} {fromY} {toX} {toY}"),
            Self::Build { x, y } =>
                format!("BUILD {x} {y}"),
            Self::Spawn { amount, x, y } =>
                format!("SPAWN {amount} {x} {y}"),
            Self::Wait =>
                format!("WAIT"),
            Self::Message { text } =>
                format!("MESSAGE {text}")
        }.to_string()
    }
}

fn print_actions(actions: Vec<Action>) {
    println!("{}", actions.into_iter().map(|action| action.to_string()).collect::<Vec<String>>().join(";"));
}

impl Game {
    fn new() -> Self {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let width = parse_input!(inputs[0], usize);
        let height = parse_input!(inputs[1], usize);
        let mut grid = Vec::new();
        for i in 0..height {
            let mut row = Vec::new();
            for j in 0..width {
                row.push(Location::default());
            }
            grid.push(row);
        }

        Game {
            width,
            height,
            grid,
            my_matter: 0,
            enemy_matter: 0,
            my_robots: Vec::new(),
            grid_dist_to_outside: vec![vec![-1; width]; height]
        }
    }

    fn neighbors(&self, i: usize, j: usize) -> Vec<(usize, usize)> {
        let (i, j) = (i as i32, j as i32);
        [(i, j+1), (i+1, j), (i, j-1), (i-1, j)]
            .into_iter()
            .filter(
                |(i2, j2)|
                    *i2 >= 0 &&
                    *i2 < self.height as i32 &&
                    *j2 >= 0 &&
                    *j2 < self.width as i32
            )
            .map(|(i2, j2)| (*i2 as usize, *j2 as usize))
            .collect()
    }

    fn set_from_input(&mut self) {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        self.my_matter = parse_input!(inputs[0], i32);
        self.enemy_matter = parse_input!(inputs[1], i32);
        self.my_robots.clear();
        let mut outside_coords = Vec::new();
        for i in 0..self.height {
            for j in 0..self.width {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();
                let inputs = input_line.split(" ").collect::<Vec<_>>();
                self.grid[i][j].scrap_amount = parse_input!(inputs[0], i32);
                self.grid[i][j].owner = parse_input!(inputs[1], i32).into(); // 1 = me, 0 = foe, -1 = neutral
                self.grid[i][j].units = parse_input!(inputs[2], i32);
                self.grid[i][j].recycler = bool_from_i32(parse_input!(inputs[3], i32));
                self.grid[i][j].can_build = bool_from_i32(parse_input!(inputs[4], i32));
                self.grid[i][j].can_spawn = bool_from_i32(parse_input!(inputs[5], i32));
                self.grid[i][j].in_range_of_recycler = bool_from_i32(parse_input!(inputs[6], i32));

                if self.grid[i][j].owner == Owner::Me && self.grid[i][j].units > 0 {
                    self.my_robots.push((i, j));
                }

                if self.grid[i][j].owner != Owner::Me && self.grid[i][j].scrap_amount > 0 {
                    outside_coords.push((i, j));
                    self.grid_dist_to_outside[i][j] = 0;
                }
                else {
                    self.grid_dist_to_outside[i][j] = -1;
                }
            }
        }

        let mut to_visit: VecDeque<(usize, usize)> = outside_coords.clone().into();
        while to_visit.len() > 0 {
            let (i, j) = to_visit.pop_front().unwrap();
            let current_dist = self.grid_dist_to_outside[i][j];
            let unvisited_neighbors: Vec<(usize, usize)> = self.neighbors(i, j)
                .into_iter()
                .filter(|(i2, j2)| self.grid_dist_to_outside[*i2][*j2] < 0)
                .collect();
            for (i2, j2) in unvisited_neighbors {
                self.grid_dist_to_outside[i2][j2] = current_dist + 1;
                to_visit.push_back((i2, j2));
            }
        }

        eprintln!("{}", self.grid_dist_to_outside.iter().map(|row| row.iter().map(|val| val.to_string()).collect::<Vec<String>>().join(" ")).collect::<Vec<String>>().join("\n"))
    }

    fn compute_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        // MOVING ROBOTS
        for &(i, j) in self.my_robots.iter() {
            let n_units = self.grid[i][j].units as usize;
            let neighbors: Vec<(usize, usize)> = self.neighbors(i, j)
                .into_iter()
                .filter(|(i2, j2)| self.grid[*i2][*j2].scrap_amount > 0)
                .collect();
            eprintln!("MY ROBOTS: {:?}, n_units: {}, neighbors: {:?}", (i, j), n_units, neighbors);
            let min_dist = neighbors
                .iter()
                .map(|(i2, j2)| self.grid_dist_to_outside[*i2][*j2])
                .min()
                .unwrap();
            let mut min_dist_destinations = Vec::new();
            for (i2, j2) in neighbors {
                if self.grid_dist_to_outside[i2][j2] == min_dist {
                    min_dist_destinations.push((i2, j2));
                }
            }
            eprintln!("min_dist: {}, min_dist_destinations: {:?}", min_dist, min_dist_destinations);
            for (k, (i2, j2)) in min_dist_destinations.iter().enumerate() {
                let amount = n_units / min_dist_destinations.len() + if k < n_units % min_dist_destinations.len() {1} else {0};
                if amount == 0 {
                    break;
                }
                actions.push(Action::Move { amount, fromX: j, fromY: i, toX: *j2, toY: *i2 });
            }
        }
        // SPAWNING ROBOTS
        let mut frontier: Vec<(usize, usize)> = Vec::new();
        for i in 0..self.height {
            for j in 0..self.width {
                if self.grid[i][j].owner == Owner::Me && self.neighbors(i, j).into_iter().any(|(i2, j2)| self.grid[i2][j2].owner != Owner::Me && self.grid[i2][j2].scrap_amount > 0) {
                    frontier.push((i, j));
                }
            }
        }

        let mut rng = rand::thread_rng();
        for _ in 0..self.my_matter / 10 {
            let k = rng.gen_range(0..frontier.len());
            let (i, j) = frontier[k];
            actions.push(Action::Spawn { amount: 1, x: j, y: i });
        }

        actions
    }
}




fn main() {
    let mut game = Game::new();
    loop {
        game.set_from_input();
        let actions = game.compute_actions();
        print_actions(actions);
    }
}
