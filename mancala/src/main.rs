use mcts::*;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::time::Instant;
use std::{collections::HashMap, fmt::Display};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Player {
    A,
    B,
}

impl Player {
    fn other(&self) -> Self {
        match self {
            Player::A => Player::B,
            Player::B => Player::A,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Mancala {
    player: Player,
    board: Vec<Vec<u8>>,
    store: [u8; 2],
}

impl Mancala {
    fn new() -> Self {
        Mancala {
            player: Player::A,
            board: vec![vec![4, 4, 4, 4, 4, 4], vec![4, 4, 4, 4, 4, 4]],
            store: [0, 0],
        }
    }
}

impl Game for Mancala {
    type Player = Player;
    type Action = usize;

    fn get_current_player(&self) -> Self::Player {
        self.player
    }

    fn list_actions(&self) -> Vec<Self::Action> {
        self.board[match self.player {
            Player::A => 0,
            Player::B => 1,
        }]
        .iter()
        .enumerate()
        .filter(|(_, &y)| y != 0)
        .map(|(i, _)| i)
        .collect()
    }

    fn do_action(mut self, action: Self::Action) -> Result<Self, InvalidAction> {
        let (side_a, side_b) = self.board.split_at_mut(1);
        let this_side;
        let other_side;
        let store = self
            .store
            .get_mut(match self.player {
                Player::A => 0,
                Player::B => 1,
            })
            .unwrap();
        match self.player {
            Player::A => {
                this_side = side_a.into_iter().next().unwrap();
                other_side = side_b.into_iter().next().unwrap();
            }
            Player::B => {
                this_side = side_b.into_iter().next().unwrap();
                other_side = side_a.into_iter().next().unwrap();
            }
        };

        if *this_side.get(action).ok_or(InvalidAction)? == 0 {
            Err(InvalidAction)
        } else {
            let mut seeds = this_side[action];
            this_side[action] = 0;

            for (i, hole) in this_side[action + 1..].iter_mut().enumerate() {
                *hole += 1;
                seeds -= 1;
                if seeds == 0 {
                    self.player = self.player.other();
                    let opposite_index = 4 - i - action;
                    if other_side[opposite_index] != 0 && *hole == 1 {
                        let total = other_side[opposite_index] + 1;
                        *hole = 0;
                        *other_side.get_mut(opposite_index).unwrap() = 0;
                        *store += total;
                    }
                    break;
                }
            }

            if seeds != 0 {
                'sowing: loop {
                    seeds -= 1;
                    *store += 1;
                    if seeds == 0 {
                        break;
                    }

                    for hole in other_side.iter_mut() {
                        *hole += 1;
                        seeds -= 1;
                        if seeds == 0 {
                            self.player = self.player.other();
                            break 'sowing;
                        }
                    }

                    for (i, hole) in this_side.iter_mut().enumerate() {
                        *hole += 1;
                        seeds -= 1;
                        if seeds == 0 {
                            self.player = self.player.other();
                            let opposite_index = 5 - i;
                            if other_side[opposite_index] != 0 && *hole == 1 {
                                let total = other_side[opposite_index] + 1;
                                *hole = 0;
                                *other_side.get_mut(opposite_index).unwrap() = 0;
                                *store += total;
                            }
                            break 'sowing;
                        }
                    }
                }
            }

            Ok(self)
        }
    }

    fn get_player_final_scores(&self) -> Option<HashMap<Self::Player, f64>> {
        if self
            .board
            .iter()
            .any(|side| side.iter().all(|&hole| hole == 0))
        {
            let mut scores = HashMap::with_capacity(2);
            let a_score = self.store[0] + self.board[0].iter().sum::<u8>();
            let b_score = self.store[1] + self.board[1].iter().sum::<u8>();

            if a_score > b_score {
                scores.insert(Player::A, 1.);
                scores.insert(Player::B, 0.);
            } else if a_score < b_score {
                scores.insert(Player::B, 1.);
                scores.insert(Player::A, 0.);
            } else {
                scores.insert(Player::A, 0.5);
                scores.insert(Player::B, 0.5);
            }

            Some(scores)
        } else {
            None
        }
    }
}

impl Display for Mancala {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.board[0]
            .iter()
            .zip(self.board[1].iter().rev())
            .map(|(x, y)| x.to_string() + " " + &y.to_string())
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, " {} \n{}\n {} ", self.store[1], s, self.store[0])
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let game = Mancala::new();
    let mut search = SearchInfo::new(game);
    let mut game = search.get_game();
    println!("{}\n", game);

    'game: loop {
        println!("Player A's turn\n");

        while game.player == Player::A {
            print!("Hole number: ");
            stdout().flush()?;
            let mut action = String::new();
            stdin().read_line(&mut action)?;
            let action = action.trim().parse()?;
            game = search.do_action(action)?;
            println!("{}\n", game);
            // let mut acts = Vec::new();
            // let instant = Instant::now();

            // while instant.elapsed().as_millis() < 5000 {
            //     acts = mcts_step(&mut search)
            // }
            // let hole = match acts.first() {
            //     Some(a) => *a,
            //     None => break,
            // };
            // println!("Hole number: {}", hole);
            // game = search.do_action(hole)?;
            // println!("{}\n", game);
            if let Some(score) = game.get_player_final_scores() {
                println!("{:#?}", score);
                break 'game;
            }
        }

        println!("Player B's turn\n");

        while game.player == Player::B {
            let mut acts = Vec::new();
            let instant = Instant::now();

            while instant.elapsed().as_millis() < 5000 {
                acts = mcts_step(&mut search)
            }
            let hole = match acts.first() {
                Some(a) => *a,
                None => break,
            };
            println!("Hole number: {}", hole);
            game = search.do_action(hole)?;
            println!("{}\n", game);
            if let Some(score) = game.get_player_final_scores() {
                println!("{:#?}", score);
                break 'game;
            }
        }
    }

    Ok(())
}
