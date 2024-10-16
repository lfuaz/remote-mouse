# change directory to mouse-remote-app run npm run build
cd $(pwd)/mouse-remote-app
npm ci
npm run build

# deplace le dossier dist dans le dossier mouse-server
mv $(pwd)/mouse-remote-app/dist $(pwd)/mouse-server

#build app 
cd $(pwd)/mouse-server
cargo build --release

