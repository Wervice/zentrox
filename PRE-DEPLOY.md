# Pre Deploy

## A list of things to do before deploying / releasing

1. Change the URL prefix in the frontend to ""
2. npm run build, so that the user can npm run start in the frontend
3. Use port 3000 for backend and 443 for frontend

Change prefix, disable Dev Dis Auth

1. The individual pages in use a variable as a prefix to fetch() requests
2. index.js may use devDisAuth. This has to be false before release
3. Now, you can run npm run build
4. After the build is done, the express is all we need
