const fetchURLPrefix = ""
// const fetchURLPrefix = "https://localhost:8080";
export { fetchURLPrefix };

/* This code is used to set a prefix before every fetch request.
   Using this, I can work on the code more easily.
   It is important to note, that the string above has to be set to "" during production.
   You also have to set the origin in the cors policy in main.rs. 
*/
