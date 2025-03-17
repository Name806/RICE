# RICE
RICE is the Rust Intelligent Chess Engine. The goal of this project is to create a fast chess engine by using efficient data structures and search algorithms, while being readable and maintainable. Currently, RICE is in its early stages of development, and is limited to generating valid moves and choosing a move without thinking. This was done using Bitboards and Move encoding to optimize memory usage.

## Installation
 * clone the project ``` git clone https://github.com/Name806/RICE <file_name> ```
 * source code can be found in rice/src and precalculator/src. The executable files can be foudn in rice/target/release and rice/target/debug.

### Perftree:
If you want to test the ai using perftree (this is a program that makes sure the moves generated are accurate) follow these steps
 * install the fish shell
 * enter the source file ``` cd <file_name>/rice/src ```
 * edit the file perftree_script.fish and change the line ``` /home/parker/projects/rice/target/debug/rice "perftree" $argv ``` so that the section ```/home/parker/projects ``` instead leads to where the file is on your system. To get that path, use the command ``` pwd ``` while in the directory you cloned the projects to.
 * Install perftree; directions here: [perftree](https://github.com/agausmann/perftree)
 * Make sure to install Stockfish 13, as later versions have a different response to perftree's inputs and perftree won't work, and earlier versions haven't been tested by me, so they may not work.
 * Make sure ```perftree``` and ```stockfish``` are part of your path and can be run from the command line.

## Usage
Currently, RICE's ai functionality is a work in progress, so the chess moves it picks, while guaranteed to be valid moves, are just the first moves in its list of moves that are legal.
### Initailization
 * To initailize the ai, either run the file called rice in the folder ```<file_name>/rice/target/release/```. To initailize the debug version, replace "release" with "debug".
 * Alternatively if you have cargo installed, run the command ```cargo run```
 * Running the ai will allow you to send commands by inputting them into the terminal.
### UCI Commands
RICE has limited compatability with the UCI (Universal Chess Interface) protocol. This is how you can communicate with it:
 * To get the ai's uci info: ```uci```
 * To check the ai's readiness: ```isready```
 * To set up a custom position, enter the command ```position fen <fen>```; as an example, here is the fen ([Forsyth-Edwards Notation](https://www.chess.com/terms/fen-chess)) for the standard starting position in chess: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
 * To set up the starting position without entering its fen: ```position startpos```
 * To get a move: ```go infinite``` (make sure to set up a position first)
### Testing with Perftree
 * Check the installation instructions under "Perftree" to make sure you install and set up perftree correctly.
 * The ./move_perft file included in rice/src is intended to run perftree in such a way that it knows how to call RICE's initailization file so that it takes perftree's inputs and outputs data in the format perftree expects.
 * Instructions for using perftree can be found on [its github page](https://github.com/agausmann/perftree).
 * Simply run ```./move_perft``` from rice/src and from there you can enter perftree commands.
