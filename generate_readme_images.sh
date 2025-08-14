#!/bin/bash

# Shell script to generate all images referenced in README.md
# and save them to the propaganda/ directory

set -e

echo "Generating README images..."

# Create propaganda directory if it doesn't exist
mkdir -p propaganda

# Sequential composition: (add neg copy)
echo "Generating sequential composition..."
cargo run -- '(add neg copy)' -qv -s signature.json > propaganda/sequential_composition.svg

# Parallel composition: {add copy}
echo "Generating parallel composition..."
cargo run -- '{add copy}' -qv -s signature.json > propaganda/parallel_composition.svg

# Identity with binding: [x y . x y]
echo "Generating identity with binding..."
cargo run -- '[x y . x y]' -qv > propaganda/identity_binding.svg

# Identity shorthand: [x y]
echo "Generating identity shorthand..."
cargo run -- '[x y]' -qv > propaganda/identity_shorthand.svg

# Joining wires: [x x . x]
echo "Generating joining wires..."
cargo run -- '[x x . x]' -qv > propaganda/joining_wires.svg

# Splitting wires: [x . x x]
echo "Generating splitting wires..."
cargo run -- '[x . x x]' -qv > propaganda/splitting_wires.svg

# Dispelling wires: [x.]
echo "Generating dispelling wires..."
cargo run -- '[x.]' -qv > propaganda/dispelling_wires.svg

# Summoning wires: [.x]
echo "Generating summoning wires..."
cargo run -- '[.x]' -qv > propaganda/summoning_wires.svg

# Complex imperative expression
echo "Generating complex imperative expression..."
cargo run -- '([a b.] { ([.a b] add [acc.]) ([.a acc] mul [result.]) })' -qv -s signature.json > propaganda/imperative_example.svg

echo "All images generated successfully in propaganda/ directory!"
