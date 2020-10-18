use rand::distributions::{Distribution, Uniform};
use rand::Rng;

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

const TABLE_SIZE: usize = 9;
const TABLE_SIZE_MINUS_ONE: i64 = (TABLE_SIZE as i64) - 1;
const TESTS_COUNT: usize = 10000;

#[derive(Copy, Clone, PartialEq, Debug)]
enum Color {
    Empty,
    Black,
    White,
}

#[derive(Copy, Clone)]
struct Position(usize, usize);

#[derive(Debug)]
struct State {
    table: [[Color; TABLE_SIZE]; TABLE_SIZE],
}

impl State {
    fn new() -> Self {
        State {
            table: [[Color::Empty; TABLE_SIZE]; TABLE_SIZE],
        }
    }

    fn random() -> Self {
        let mut tmp = State::new();
        let mut rng = rand::thread_rng();
        let range = Uniform::from(0..3);

        for column in tmp.table.iter_mut() {
            for element in column.iter_mut() {
                *element = match range.sample(&mut rng) {
                    0 => Color::Empty,
                    1 => Color::White,
                    _ => Color::Black,
                };
            }
        }

        tmp
    }

    fn place(&mut self, x: usize, y: usize, color: Color) {
        self.table[x][y] = color;
    }

    fn get_field(&self, x: i64, y: i64) -> Option<Color> {
        if x < 0 || x > TABLE_SIZE_MINUS_ONE as i64 || y < 0 || y > TABLE_SIZE_MINUS_ONE as i64 {
            None
        } else {
            Some(self.table[x as usize][y as usize])
        }
    }

    fn have_adjacment(&self, x: usize, y: usize, color: Color) -> bool {
        let ortho = [(-1, -1), (-1, 1), (1, -1), (1, 1)]
            .clone()
            .iter()
            .filter_map(|coords| self.get_field(coords.0 + x as i64, coords.1 + y as i64))
            .filter(|clr| *clr == color)
            .count();

        let diagonal = [(-1, 0), (1, 0), (0, -1), (0, 1)]
            .clone()
            .iter()
            .filter_map(|coords| self.get_field(coords.0 + x as i64, coords.1 + y as i64))
            .filter(|clr| *clr == color)
            .count();

        (ortho >= 2 || diagonal >= 2) && self.table[x][y] == Color::Empty
    }

    fn possible_places(&self) -> Vec<Position> {
        (0..TABLE_SIZE)
            //.zip(0..TABLE_SIZE)
            .cartesian_product(0..TABLE_SIZE)
            .filter(|(x, y)| self.table[*x][*y] == Color::Empty)
            .map(|(x, y)| Position(x, y))
            .collect()
    }

    fn possible_grows(&self, color: Color) -> Vec<Position> {
        (0..TABLE_SIZE)
            //.zip(0..TABLE_SIZE)
            .cartesian_product(0..TABLE_SIZE)
            .filter(|place| self.have_adjacment(place.0, place.1, color))
            .map(|(x, y)| Position(x, y))
            .collect()
    }

    fn is_finished(&self) -> bool {
        self.possible_grows(Color::Black).len() == 0 && self.possible_grows(Color::White).len() == 0
    }

    fn is_viable(&self) -> bool {
        let blacks: i64 = self
            .table
            .iter()
            .map(|column| {
                column
                    .iter()
                    .filter(|color| **color == Color::Black)
                    .count() as i64
            })
            .sum::<i64>();
        let whites: i64 = self
            .table
            .iter()
            .map(|column| {
                column
                    .iter()
                    .filter(|color| **color == Color::White)
                    .count() as i64
            })
            .sum::<i64>();

        (blacks > TABLE_SIZE_MINUS_ONE && whites > TABLE_SIZE_MINUS_ONE)
            || (blacks - whites).abs() < 2
    }
}

fn main() {
    println!("For table size: {}", TABLE_SIZE);
    println!("Number of tests: {}", TESTS_COUNT);

    // =============
    // 1 Method
    // =============
    {
        println!("=============");
        println!("1 Method");
        println!("=============");

        let mut branches: Vec<usize> = Vec::new();
        let mut depths = Vec::new();

        let mut pb = ProgressBar::new(TESTS_COUNT as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:100.gray/white}]")
                .progress_chars("=>."),
        );

        for _ in 0..TESTS_COUNT {
            let mut branches_local = Vec::new();
            let mut depth = 0;
            let mut state = State::new();
            let mut rng = rand::thread_rng();

            for _ in 0..TABLE_SIZE_MINUS_ONE {
                let black_poses = state.possible_places();
                branches_local.push(black_poses.len());

                let dec = black_poses[rng.gen_range(0, black_poses.len())];

                state.place(dec.0, dec.1, Color::Black);

                depth += 1;

                // ============================

                let white_poses = state.possible_places();
                branches_local.push(black_poses.len());

                let dec2 = white_poses[rng.gen_range(0, white_poses.len())];

                state.place(dec2.0, dec2.1, Color::White);

                depth += 1;
            }

            'endgame: loop {
                let black_grows = state.possible_grows(Color::Black);
                if black_grows.len() != 0 {
                    branches_local.push(black_grows.len());
                    depth += 1;

                    let dec = black_grows[rng.gen_range(0, black_grows.len())];
                    state.place(dec.0, dec.1, Color::Black);
                }
                if state.is_finished() {
                    break 'endgame;
                }

                let white_grows = state.possible_grows(Color::White);
                if white_grows.len() != 0 {
                    branches_local.push(black_grows.len());
                    depth += 1;

                    let dec = white_grows[rng.gen_range(0, white_grows.len())];
                    state.place(dec.0, dec.1, Color::White);
                }
                if state.is_finished() {
                    break 'endgame;
                }
            }
            depths.push(depth);
            branches.push(branches_local.iter().sum::<usize>() / branches_local.len());
            pb.inc(1);
        }
        pb.finish();
        let branches_avg = branches.iter().sum::<usize>() / branches.len();
        let depths_avg = depths.iter().sum::<usize>() / depths.len();
        println!("Average branch count: {}", branches_avg);
        println!("Average height of decision tree: {}", depths_avg);
        println!("Game complexity: {}^{}", branches_avg, depths_avg);
    }
    // =============
    // 2 Method
    // =============
    {
        println!("=============");
        println!("2 Method");
        println!("=============");

        let n: f64 = (TABLE_SIZE * TABLE_SIZE).pow(3) as f64;
        let mut ress = Vec::new();

        let mut pb = ProgressBar::new(TESTS_COUNT as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:100.gray/white}]")
                .progress_chars("=>."),
        );

        for _ in 0..TESTS_COUNT {
            ress.push(
                ((0..10_000)
                    .map(|_| State::random())
                    .filter(|state| state.is_viable())
                    .count() as f64
                    / 100.0)
                    * n,
            );
            pb.inc(1);
        }
        pb.finish();

        println!(
            "Game complexity: {}",
            (ress.iter().sum::<f64>() / ress.len() as f64) as u64
        );
    }
}
