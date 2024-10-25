#!/usr/bin/env fish

cd /home/parker/cs_projects/RICE/rice/target/debug
set rust_binary ./rice
set -x RUST_BACKTRACE 1
set depth 3
set fen "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
set moves ""

if test -n "$moves"
    $rust_binary $depth $fen $moves
else
    $rust_binary $depth $fen
end

