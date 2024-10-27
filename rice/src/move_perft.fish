#!/bin/fish
set output (cargo build 2>&1)
set build_status $status
if test $build_status -ne 0
	echo "build failed with errors."
	echo "$output"
	exit 1
end

echo "$output" | grep -q "warning"
if test $status -eq 0
	echo "warnings detected, perftree will not work"
	exit 1
end

echo "build successful, running perftree"
perftree ./perftree_script.fish
