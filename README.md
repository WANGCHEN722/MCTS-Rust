# MCTS-Rust
This is a Monte-Carlo Tree Search alghorithm written in Rust. It includes some unit tests that use the algorithm to play tic-tac-toe. It should never lose since the game is simple enough to be completely solved.

I have tested this algorithm in playing Mancala, also known as Congklak. It has never lost to a human player. Feel free to compile and run.

Switch to the ``/mancala`` folder, then
```
cargo run --release
``
The board is a series of numbers. Each number represents one pit and corresponds to the number of stones in the pit.
The human player starts first.
Enter a number from 1 through 7 inclusive.
The number 1 corresponds to the player's store and counts up from the pit closest to the store.
After making your move, the computer makes a move.