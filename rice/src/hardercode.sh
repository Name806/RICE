# Create a script named exact_example.sh with the exact same content from the README
cat <<EOF > exact_example.sh
#!/bin/bash
echo "a2a3 380"
echo "b2b3 420"
echo "c2c3 420"
echo "d2d3 539"
echo "e2e3 599"
echo "f2f3 380"
echo "g2g3 420"
echo "h2h3 380"
echo "a2a4 420"
echo "b2b4 421"
echo "c2c4 441"
echo "d2d4 560"
echo "e2e4 600"
echo "f2f4 401"
echo "g2g4 421"
echo "h2h4 420"
echo "b1c3 440"
echo "g1h3 400"
echo "b1a3 400"
echo "g1f3 440"
echo
echo "8902"
EOF

# Make it executable
chmod +x exact_example.sh

# Run this exact script with perftree
perftree ./exact_example.sh
