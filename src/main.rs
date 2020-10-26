use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use rand::Rng;

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

use rayon::prelude::*;

const TABLE_SIZE: usize = 9;
const TABLE_SIZE_MINUS_ONE: i64 = (TABLE_SIZE as i64) - 1;
const TESTS_COUNT: usize = 10000;
const MINMAX_DEPTH: usize = 16;

#[derive(Clone)]
struct Node {
    state: State,
}

impl Node {
    fn random() -> Self {
        let mut s = State::new();

        let mut rng = rand::thread_rng();

        for _ in 0..TABLE_SIZE_MINUS_ONE {
            let white_poss = s.possible_places();
            let white_chos = white_poss.choose(&mut rng).unwrap();

            s.place(white_chos.0, white_chos.1, Color::White);

            let black_poss = s.possible_places();
            let black_chos = black_poss.choose(&mut rng).unwrap();

            s.place(black_chos.0, black_chos.1, Color::Black);
        }

        Node { state: s }
    }

    fn with(&self, pos: Position, color: Color) -> Self {
        Node {
            state: self.state.with(pos, color),
        }
    }

    fn minimax(&self, depth: u16, max: bool) -> i32 {
        if depth == 0 || self.state.is_finished() {
            return self.cost();
        } else {
            if max {
                return self
                    .state
                    .possible_grows(Color::White)
                    .iter()
                    .map(|pos| {
                        let mut tmp = self.clone();
                        tmp.state.place(pos.0, pos.1, Color::White);
                        tmp.minimax(depth - 1, false)
                    })
                    .max()
                    .unwrap_or(self.cost());
            } else {
                return self
                    .state
                    .possible_grows(Color::Black)
                    .iter()
                    .map(|pos| {
                        let mut tmp = self.clone();
                        tmp.state.place(pos.0, pos.1, Color::Black);
                        tmp.minimax(depth - 1, true)
                    })
                    .min()
                    .unwrap_or(self.cost());
            }
        }
    }

    fn negamax(&self, depth: u16, sign: i8) -> i32 {
        if depth == 0 {
            return sign as i32 * self.cost();
        } else {
            self.state
                .possible_grows(if sign == 1 {
                    Color::White
                } else {
                    Color::Black
                })
                .iter()
                .map(|pos| {
                    -self
                        .clone()
                        .with(
                            *pos,
                            if sign == 1 {
                                Color::White
                            } else {
                                Color::Black
                            },
                        )
                        .negamax(depth - 1, -sign)
                })
                .max()
                .unwrap_or(self.cost())
        }
    }

    fn abnegamax(&self, depth: u16, mut alpha: i32, beta: i32, sign: i8) -> i32 {
        if depth == 0 {
            return self.cost();
        } else {
            for pos in self.state.possible_grows(if sign == 1 {
                Color::White
            } else {
                Color::Black
            }) {
                alpha = alpha.max(
                    -self
                        .with(
                            pos,
                            if sign == 1 {
                                Color::White
                            } else {
                                Color::Black
                            },
                        )
                        .abnegamax(depth - 1, -alpha, -beta, -sign),
                );
                if alpha >= beta {
                    return alpha;
                }
            }

            return alpha;
        }
    }

    fn cost(&self) -> i32 {
        self.state.cost()
    }

    fn get_optimal_moves(&mut self, depth: u16) -> Vec<(i32, Position)> {
        let mut foo: Vec<(i32, Position)> = self
            .state
            .possible_grows(Color::White)
            .par_iter()
            .map(|pos| (self.with(*pos, Color::White).abnegamax(depth - 1, std::i32::MIN, std::i32::MAX, -1), *pos))
            .collect();
  
            foo.par_sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        return foo.par_iter().take(5).map(|x| *x).collect();
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.state)?;
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Color {
    Empty,
    Black,
    White,
}

#[derive(Copy, Clone)]
struct Position(usize, usize);

#[derive(Debug, Copy, Clone)]
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

    fn with(&self, pos: Position, color: Color) -> Self {
        let mut tmp = self.clone();
        tmp.place(pos.0, pos.1, color);
        tmp
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
            .cartesian_product(0..TABLE_SIZE)
            .filter(|(x, y)| self.table[*x][*y] == Color::Empty)
            .map(|(x, y)| Position(x, y))
            .collect()
    }

    fn possible_grows(&self, color: Color) -> Vec<Position> {
        (0..TABLE_SIZE)
            .cartesian_product(0..TABLE_SIZE)
            .filter(|place| self.have_adjacment(place.0, place.1, color))
            .map(|(x, y)| Position(x, y))
            .collect()
    }

    fn is_finished(&self) -> bool {
        self.possible_grows(Color::Black).len() == 0 && self.possible_grows(Color::White).len() == 0
    }

    fn is_viable(&self) -> bool {
        let (whites, blacks) = (0..TABLE_SIZE).cartesian_product(0..TABLE_SIZE).fold(
            (0, 0),
            |(white, black), (x, y)| match self.table[x][y] {
                Color::White => (white + 1, black),
                Color::Black => (white, black + 1),
                _ => (white, black),
            },
        );

        (blacks > TABLE_SIZE_MINUS_ONE && whites > TABLE_SIZE_MINUS_ONE)
            || (blacks - whites).abs() < 2
    }

    // Count possible places to place stone and placed stones
    //      for both players and subtract black's count from white's count.
    //      White player want score to be as high and black player want as low.
    fn cost(&self) -> i32 {
        let mut white = 0;
        let mut black = 0;

        for i in 0..TABLE_SIZE {
            for j in 0..TABLE_SIZE {
                match self.table[i][j] {
                    Color::White => white += 1,
                    Color::Black => black += 1,
                    _ => {
                        if self.have_adjacment(i, j, Color::White) {
                            white += 1;
                        }
                        if self.have_adjacment(i, j, Color::Black) {
                            black += 1;
                        }
                    }
                }
            }
        }

        white - black
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  |")?;
        for i in 0..TABLE_SIZE {
            write!(f, "{}", std::char::from_u32('A' as u32 + i as u32).unwrap())?;
        }
        write!(f, "\n")?;
        writeln!(f, "{}", "-".repeat(TABLE_SIZE + 3))?;

        for i in 0..TABLE_SIZE {
            write!(f, "{:>2}|", i + 1)?;
            for j in 0..TABLE_SIZE {
                write!(
                    f,
                    "{}",
                    match self.table[i][j] {
                        Color::White => 'o',
                        Color::Black => 'x',
                        Color::Empty => '.',
                    }
                )?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

fn main() {
    println!("Table size: {}", TABLE_SIZE);
    println!("AlphaBetaNegamax depth: {}\n", MINMAX_DEPTH);

    let mut node = Node::random();
    let moves = node.get_optimal_moves(MINMAX_DEPTH as u16);

    println!("{}", node);

    println!("Best moves for white player:");

    for (i, (score, pos)) in moves.iter().enumerate() {
        println!(
            "{}. {}{} score: {}",
            i + 1,
            std::char::from_u32('A' as u32 + pos.1 as u32).unwrap(),
            pos.0 + 1,
            score
        );
    }
}

// fn main() {
//     println!("For table size: {}", TABLE_SIZE);
//     println!("Number of tests: {}", TESTS_COUNT);

//     // =============
//     // 1 Method
//     // =============
//     {
//         println!("=============");
//         println!("1 Method");
//         println!("=============");

//         let mut branches: Vec<usize> = Vec::new();
//         let mut depths = Vec::new();

//         let mut pb = ProgressBar::new(TESTS_COUNT as u64);
//         pb.set_style(
//             ProgressStyle::default_bar()
//                 .template("[{elapsed_precise}] [{bar:100.gray/white}]")
//                 .progress_chars("=>."),
//         );

//         for _ in 0..TESTS_COUNT {
//             let mut branches_local = Vec::new();
//             let mut depth = 0;
//             let mut state = State::new();
//             let mut rng = rand::thread_rng();

//             for _ in 0..TABLE_SIZE_MINUS_ONE {
//                 let black_poses = state.possible_places();
//                 branches_local.push(black_poses.len());

//                 let dec = black_poses[rng.gen_range(0, black_poses.len())];

//                 state.place(dec.0, dec.1, Color::Black);

//                 depth += 1;

//                 // ============================

//                 let white_poses = state.possible_places();
//                 branches_local.push(black_poses.len());

//                 let dec2 = white_poses[rng.gen_range(0, white_poses.len())];

//                 state.place(dec2.0, dec2.1, Color::White);

//                 depth += 1;
//             }

//             'endgame: loop {
//                 let black_grows = state.possible_grows(Color::Black);
//                 if black_grows.len() != 0 {
//                     branches_local.push(black_grows.len());
//                     depth += 1;

//                     let dec = black_grows[rng.gen_range(0, black_grows.len())];
//                     state.place(dec.0, dec.1, Color::Black);
//                 }
//                 if state.is_finished() {
//                     break 'endgame;
//                 }

//                 let white_grows = state.possible_grows(Color::White);
//                 if white_grows.len() != 0 {
//                     branches_local.push(black_grows.len());
//                     depth += 1;

//                     let dec = white_grows[rng.gen_range(0, white_grows.len())];
//                     state.place(dec.0, dec.1, Color::White);
//                 }
//                 if state.is_finished() {
//                     break 'endgame;
//                 }
//             }
//             depths.push(depth);
//             branches.push(branches_local.iter().sum::<usize>() / branches_local.len());
//             pb.inc(1);
//         }
//         pb.finish();
//         let branches_avg = branches.iter().sum::<usize>() / branches.len();
//         let depths_avg = depths.iter().sum::<usize>() / depths.len();
//         println!("Average branch count: {}", branches_avg);
//         println!("Average height of decision tree: {}", depths_avg);
//         println!("Game complexity: {}^{}", branches_avg, depths_avg);
//     }
//     // =============
//     // 2 Method
//     // =============
//     {
//         println!("=============");
//         println!("2 Method");
//         println!("=============");

//         let n: f64 = (TABLE_SIZE * TABLE_SIZE).pow(3) as f64;
//         let mut ress = Vec::new();

//         let mut pb = ProgressBar::new(TESTS_COUNT as u64);
//         pb.set_style(
//             ProgressStyle::default_bar()
//                 .template("[{elapsed_precise}] [{bar:100.gray/white}]")
//                 .progress_chars("=>."),
//         );

//         for _ in 0..TESTS_COUNT {
//             ress.push(
//                 ((0..10_000)
//                     .map(|_| State::random())
//                     .filter(|state| state.is_viable())
//                     .count() as f64
//                     / 100.0)
//                     * n,
//             );
//             pb.inc(1);
//         }
//         pb.finish();

//         println!(
//             "Game complexity: {}",
//             (ress.iter().sum::<f64>() / ress.len() as f64) as u64
//         );
//     }
// }
