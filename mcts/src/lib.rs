use ordered_float::OrderedFloat;
use rand::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug)]
pub struct InvalidAction;

impl Display for InvalidAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Action")
    }
}

impl Error for InvalidAction {}

pub trait Game: Clone + Eq + Hash {
    type Player: Eq + Hash;
    type Action: Eq + Hash + Copy;

    fn list_actions(&self) -> Vec<Self::Action>;
    fn get_current_player(&self) -> Self::Player;
    fn get_player_final_scores(&self) -> Option<HashMap<Self::Player, f64>>;
    fn do_action(self, action: Self::Action) -> Result<Self, InvalidAction>;
}

pub struct SearchInfo<G: Game<Action = A, Player = P>, A, P> {
    root: Rc<RefCell<Node<G, A, P>>>,
    transpositions: HashMap<G, Rc<RefCell<Node<G, A, P>>>>,
}

impl<G: Game<Action = A, Player = P>, A: Eq + Hash + Clone, P> SearchInfo<G, A, P> {
    pub fn new(game: G) -> Self {
        let mut transp = HashMap::new();
        let refer = Rc::new(RefCell::new(Node::new(game.clone())));
        transp.insert(game, Rc::clone(&refer));

        SearchInfo {
            root: refer,
            transpositions: transp,
        }
    }

    pub fn do_action(&mut self, action: A) -> Result<G, InvalidAction> {
        let next_state = self.root.borrow().state.clone().do_action(action)?;
        self.root = Rc::clone(
            self.transpositions
                .entry(next_state.clone())
                .or_insert(Rc::new(RefCell::new(Node::new(next_state.clone())))),
        );

        let mut prev_len = 0;
        while self.transpositions.len() != prev_len {
            prev_len = self.transpositions.len();
            self.transpositions.retain(|_, v| Rc::strong_count(v) > 1);
        }

        Ok(next_state)
    }

    pub fn get_game(&self) -> G {
        self.root.borrow().state.clone()
    }
}

struct Node<G: Game<Action = A, Player = P>, A, P> {
    number: f64,
    rewards: HashMap<P, f64>,
    state: G,
    next_states: HashMap<A, Rc<RefCell<Node<G, A, P>>>>,
}

impl<G: Game<Action = A, Player = P>, A, P> Node<G, A, P> {
    fn new(state: G) -> Node<G, A, P> {
        Node {
            number: 0.,
            rewards: HashMap::new(),
            state: state,
            next_states: HashMap::new(),
        }
    }
}

pub fn mcts_step<G: Game<Action = A, Player = P>, A: Eq + Hash + Copy, P: Eq + Hash>(
    search: &mut SearchInfo<G, A, P>,
) -> Vec<A> {
    let mut terminal = false;
    let mut reward = HashMap::new();

    //selection
    let mut node = &search.root;
    let mut action = None;
    let mut path = Vec::new();
    path.push(Rc::clone(&search.root));

    loop {
        {
            let n = node.borrow();

            if let Some(scores) = n.state.get_player_final_scores() {
                terminal = true;
                reward = scores;
                break;
            }

            action = n.state.list_actions().into_iter().max_by_key(|act| {
                OrderedFloat(match n.next_states.get(act) {
                    Some(state) => {
                        let mut s = state.borrow_mut();
                        *s.rewards.entry(n.state.get_current_player()).or_insert(0.) / s.number
                            + (2. * n.number.ln() / s.number).sqrt()
                    }
                    None => f64::INFINITY,
                })
            });
        }

        if let Some(state) = node
            .clone()
            .borrow()
            .next_states
            .get(&action.expect("There should be some valid actions in a non-terminal state"))
        {
            path.push(Rc::clone(state));
            node = &path.last().unwrap();
        } else {
            break;
        }
    }

    if !terminal {
        //expansion
        let next_state;
        {
            let action =
                action.expect("There should be some valid actions in a non-terminal state");
            let mut n = node.borrow_mut();
            let state = n
                .state
                .clone()
                .do_action(action)
                .expect("Action should be valid");

            next_state = search
                .transpositions
                .entry(state.clone())
                .or_insert(Rc::new(RefCell::new(Node::new(state))));

            n.next_states.insert(action, Rc::clone(next_state));
        }

        //rollout
        let mut rng = thread_rng();
        let mut game = next_state.borrow().state.clone();
        let mut actions = game.list_actions();

        while let None = game.get_player_final_scores() {
            game = game
                .do_action(
                    *actions
                        .choose(&mut rng)
                        .expect("There should be some valid actions in a non-terminal state"),
                )
                .expect("All generated actions should be valid");
            actions = game.list_actions();
        }

        reward = game
            .get_player_final_scores()
            .expect("Game in terminal state should have a score");

        path.push(Rc::clone(next_state));
    }

    //backprop
    {
        let mut it = path.into_iter();
        let root = it.next().unwrap();
        let mut r = root.borrow_mut();
        r.number += 1.;
        let mut prev_player = r.state.get_current_player();

        for node in it {
            let mut n = node.borrow_mut();
            n.number += 1.;
            *n.rewards.entry(prev_player).or_insert(0.) += reward[&prev_player];
            prev_player = n.state.get_current_player();
        }
    }

    let s = &search.root.borrow();
    let root_player = s.state.get_current_player();
    let mut list_actions: Vec<A> = s.next_states.keys().copied().collect();
    list_actions.sort_unstable_by_key(|x| {
        let n = s
            .next_states
            .get(x)
            .expect("Key should be in HashMap")
            .borrow();
        OrderedFloat(-n.rewards[&root_player] / n.number)
    });
    list_actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::{Eq, PartialEq};
    use std::iter;

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct TicTacToe {
        player: Square,
        board: Vec<Vec<Square>>,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    enum Square {
        Empty,
        Circle,
        Cross,
    }

    impl Square {
        fn other(&self) -> Self {
            match self {
                Square::Circle => Square::Cross,
                Square::Cross => Square::Circle,
                Square::Empty => panic!("Empty has no opponent"),
            }
        }
    }

    impl TicTacToe {
        fn new(grid_size: usize) -> Self {
            TicTacToe {
                board: iter::repeat(iter::repeat(Square::Empty).take(grid_size).collect())
                    .take(grid_size)
                    .collect(),
                player: Square::Circle,
            }
        }
    }

    impl Game for TicTacToe {
        type Player = Square;
        type Action = (usize, usize);

        fn get_current_player(&self) -> Self::Player {
            self.player
        }
        fn get_player_final_scores(&self) -> Option<HashMap<Self::Player, f64>> {
            let check_for = self.player.other();
            let size = self.board.len();
            let mut ret = HashMap::new();
            if self.board.iter().any(|x| x.iter().all(|y| y == &check_for))
                || (0..size).any(|x| self.board.iter().all(|y| y[x] == check_for))
                || (0..size).all(|x| self.board[x][x] == check_for)
                || (0..size).all(|x| self.board[x][size - x - 1] == check_for)
            {
                ret.insert(check_for, 1.);
                ret.insert(check_for.other(), 0.);
                Some(ret)
            } else if self
                .board
                .iter()
                .all(|x| x.iter().all(|y| y != &Square::Empty))
            {
                ret.insert(check_for, 0.5);
                ret.insert(check_for.other(), 0.5);
                Some(ret)
            } else {
                None
            }
        }

        fn list_actions(&self) -> Vec<Self::Action> {
            let mut actions = Vec::new();
            let size = self.board.len();
            for row in 0..size {
                for col in 0..size {
                    if self.board[row][col] == Square::Empty {
                        actions.push((row, col))
                    }
                }
            }

            actions
        }

        fn do_action(mut self, action: Self::Action) -> Result<Self, InvalidAction> {
            if self.board[action.0][action.1] == Square::Empty {
                self.board[action.0][action.1] = self.player;
                self.player = self.player.other();
                Ok(self)
            } else {
                Err(InvalidAction)
            }
        }
    }

    #[test]
    fn test_player_init() {
        assert_eq!(TicTacToe::new(3).player, Square::Circle)
    }

    #[test]
    fn test_board_state_init() {
        let game = TicTacToe::new(5);
        assert_eq!(
            game.board,
            vec![
                vec![
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty
                ],
                vec![
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty
                ],
                vec![
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty
                ],
                vec![
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty
                ],
                vec![
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty,
                    Square::Empty
                ]
            ]
        )
    }

    #[test]
    fn check_winners() {
        let mut game = TicTacToe::new(10);
        assert_eq!(game.get_player_final_scores(), None);
        for col in 0..10 {
            game.board[0][col] = Square::Cross;
        }
        println!("{:#?}", game.board);
        let mut ret = HashMap::new();
        ret.insert(Square::Cross, 1.);
        ret.insert(Square::Circle, 0.);
        assert_eq!(game.get_player_final_scores(), Some(ret))
    }

    #[test]
    fn check_list() {
        let game = TicTacToe::new(2);
        assert_eq!(game.list_actions(), vec![(0, 0), (0, 1), (1, 0), (1, 1)])
    }

    #[test]
    fn few_steps() {
        let game = TicTacToe::new(3);
        let mut search = SearchInfo::new(game);
        let mut x = Vec::new();

        for _ in 0..10000 {
            x = mcts_step(&mut search)
        }

        assert_eq!(*x.first().expect(""), (1, 1))
    }
}
