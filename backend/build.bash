bash ./update_frontend.bash

echo "Target: "
read TARGET

mkdir ./dist_$TARGET
mkdir ./dist_$TARGET/assets
cross build --target $TARGET --release
cp ./target/release/zentrox ./dist_$TARGET/assets/zentrox
cp -r ./assets/ ./dist_$TARGET/
cp ./install.bash ./dist_$TARGET/
cp -r ./static/ ./dist_$TARGET/assets/static/
