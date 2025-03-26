# RICE
RICE is the Rust Intelligent Chess Engine. The goal of this project is to create a fast chess engine by using efficient data structures and search algorithms, while being readable and maintainable.  RICE now has the ability to search to certain depths and suggest the move it thinks is best. RICE's move selection isn't thouroughly tested, so the quality of its moves are not garuenteed."
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
### Initailization
 * To initailize the ai, either run the file called rice in the folder ```<file_name>/rice/target/release/```. To initailize the debug version, replace "release" with "debug".
 * Alternatively if you have cargo installed, run the command ```cargo run```
 * Running the ai will allow you to send commands by inputting them into the terminal.
### UCI
RICE has limited compatability with the UCI (Universal Chess Interface) protocol. 
## Cutechess & other chess gui's
I use cutechess to test the ai and play against it. Other programs may work, but I have not tested them and I don't know how to get the ai working on them.
Here is how you can use cutechess to download and play against rice:
 * install cutechess: ```yay -S cutechess```
 * run cutechess: ```cutechess```
 * go to tools/settings; This should open up a new window.
 * under the "engines" tab, click the plus buttom at the bottom
 * under commands, enter: ```./rice```
 * under working directory, input the path to ```/rice/target/release/```
 * click "ok"
You now have the engine set up. Cutechess has what it needs to call the engine and get its responses. To set up a game: 
 * go to "Game/new", and set up the game. 
 * For time controls, select "infinite".
note: The engine responds to "go infinite" by searching to a depth of 6 and returning the best move it found. This isn't technically correct, but it is a way to make the ai able to play on cutechess without much effort. UCI compatability will need to be expanded in the future to allow for more complex options to be chosen.
## Commands
The following is a comprehensive list of uci commands the ai has responses to.
 * To get the ai's uci info: ```uci```
 * To check the ai's readiness: ```isready```
 * To set up a custom position, enter the command ```position fen <fen>```; as an example, here is the fen ([Forsyth-Edwards Notation](https://www.chess.com/terms/fen-chess)) for the standard starting position in chess: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
 * To set up the starting position without entering its fen: ```position startpos```
 * To get a move: ```go depth <depth>``` (make sure to set up a position first, otherwise it panics with the message "King not found!") If you choose too high a depth, the ai will think for a while before responding. How high a depth you can use before it becomes too slow depends on the system the ai is run on and how optimized the ai is, but I was able to get it to output on a depth of 5 in a few seconds.
 * to search to a set depth: ```go infinite```

### Testing with Perftree
 * Check the installation instructions under "Perftree" to make sure you install and set up perftree correctly.
 * The ./move_perft file included in rice/src is intended to run perftree in such a way that it knows how to call RICE's initailization file so that it takes perftree's inputs and outputs data in the format perftree expects.
 * Instructions for using perftree can be found on [its github page](https://github.com/agausmann/perftree).
 * Simply run ```./move_perft``` from rice/src and from there you can enter perftree commands.
