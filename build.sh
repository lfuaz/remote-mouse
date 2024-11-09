# Build the webapp

cd client && npm run build

# Remove existing dist folder if it exists
rm -rf ../server/dist

# Move the dist folder inside the server folder
mv dist ../server/

# Build the Rust project
cd ../server && cargo build --release

echo "Build complete" 
echo "Your binary / exefile is located at server/target/release/server"