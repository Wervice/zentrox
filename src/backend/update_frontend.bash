cd ../frontend/
pnpm run build
rm -rf ../backend/static
cp -r ../frontend/out ../backend/static 
