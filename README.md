# MCTS-Rust
This is a Monte-Carlo Tree Search alghorithm written in Rust. It includes some unit tests that use the algorithm to play tic-tac-toe. It should never lose since the game is simple enough to be completely solved.

I have tested this algorithm in playing Mancala, also known as Congklak. It has never lost to a human player. Feel free to compile and run if you know how to play Mancala.

Switch to the ``/mancala`` folder, then
```
cargo run --release
```
The board is a series of numbers. Each number represents one pit and corresponds to the number of stones in the pit. The top store corresponds to Player B, and the bottom store to Player A.

The human player is Player A and starts first.

Enter a number from 0 through 5 inclusive to make a move.

The number 0 corresponds to the pit furthest from the player's store and counts up, with 5 corresponding to the pit closest to the player's store.

After making your move, the computer makes a move after thinking for 5s.

## Example game:

The board is initialised as
```
 0 
4 4
4 4
4 4
4 4
4 4
4 4
 0
```
I make move 2.
```
 0 
4 4
4 4
0 4
5 4
5 4
5 4
 1
```
The stones are taken from the pit third furtherst from my own store and sowed in order. According to the rules, I get a free move since the last stone landed in my store.

I make move 1.
```
 0 
4 4
0 4
1 4
6 4
6 4
6 4
 1
```
Again, the stones are removed from the pit second furthest from my own store.

It's the computer's turn now. It makes move 2, earning a free move.
```
 1 
4 5
0 6
1 6
6 1
6 5
6 0
 1
```
Then, it makes move 0.
```
 1 
4 5
0 6
1 6
6 1
6 5
6 0
 1
```
As the game continues, the computer manages to capture 10 of my stones at once causing me to lose 22:26. Can you do better?
