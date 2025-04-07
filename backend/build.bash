bash ./update_frontend.bash

echo "Target: "
read TARGET

mkdir ./dist_$TARGET
mkdir ./dist_$TARGET/assets
cross build --target $TARGET --release
cp ./target/release/zentrox ./dist_$TARGET/assets/zentrox
cp ./manifest.json ./dist_$TARGET/assets/manifest.json
cp ./music_default.svg ./dist_$TARGET/assets/music_default.svg
cp ./unknown_default.svg ./dist_$TARGET/assets/unknown_default.svg
cp ./video_default.svg ./dist_$TARGET/assets/video_default.svg
cp ./robots.txt ./dist_$TARGET/assets/robots.txt
cp -r ./static/ ./dist_$TARGET/assets/static/
cp ./install.bash ./dist_$TARGET/install.bash
