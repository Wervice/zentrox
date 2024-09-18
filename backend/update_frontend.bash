cd ../frontend/
npm run build
rm -rf ../backend/static
cp -r ../frontend/out ../backend/static 
