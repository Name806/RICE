#!/usr/bin/env fish

cd /home/parker/cs_projects/RICE/rice/target/debug
set rust_binary ./rice
set -x RUST_BACKTRACE 1
set depth 1
set fen "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
set moves ""

echo "Current Directory: (pwd)"
echo "Running binary from: $rust_binary"

cargo build
if test $status -ne 0
    echo "Build failed! Stopping..."
    exit 1
end

if test -n "$moves"
    $rust_binary $depth $fen $moves
else
    $rust_binary $depth $fen
end

