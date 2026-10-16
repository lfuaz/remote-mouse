# Build the webapp (outputs to server/dist, see client/vite.config.js)
cd client && npm run build

# Build the Rust project
cd ../server && cargo build --release

echo "Build complete" 
echo "Your binary / exefile is located at server/target/release/webpad"
