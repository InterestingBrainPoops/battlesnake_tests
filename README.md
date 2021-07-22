# battlesnake_tests
A collection of battlesnake JSON states along with their expected move(s).  Feel free to use this to test any of your battlesnakes.

## Format
Each test will be in a file of its number.  
Test 01 can be found at `/tests/01.json`  
Each json will be like the following: 
```
{
state: board_state,
expected: ["up", "down"]
}
```  
`state` is the board state that gets sent to the snake.  
`expected` is an array of maximum size 4 that holds the correct moves.  

## Difficulty
These tests might be very hard, and that is the intention.  There might be some easier tests in there, but most of them are intended to strain your evaluation function.

## CLI

The CLI is written in Rust and available at the root of this repo.

Assuming Rust is installed, you can use the CLI via `cargo run`.
To provide CLI arguments use a double dash `--` after run, and then you can provide any arguments
Ex: `cargo run -- --url http://localhost:8000'

## Thanks
This was inspired by the [PoorFish](https://github.com/mcostalba/PoorFish) testset for chess engines.  
Smallsco for the format suggestion  
TheApX aka Butterfly :butterfly: Tamer for the suggestion of having multiple possible correct moves.
