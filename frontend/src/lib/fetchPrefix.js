// const fetchURLPrefix = "https://192.168.105.70:3000"
const fetchURLPrefix = "https://localhost:8080";
module.exports = fetchURLPrefix;

/* This code is used to set a prefix before every fetch request.
   Using this, I can work on the code more easily.
   It is important to note, that the string above has to be set to "".
   You also have to set the origin in the cors policy to localhost in index.js with https.
   In addition to that Disable Auth may not be enabled during production in index.js
*/
