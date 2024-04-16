cssCode = `@keyframes fly-in {
    0% {
        top: -100px;
        opacity: 0%;
    }
}

#modalMain {
    position: fixed;
    top: 50px;
    width: 50vw;
    left: calc(25vw - 40px);
    border-radius: 5px;
    padding: 20px;
    background-color: #232323;
    box-shadow: 0px 5px 15px #00000057;
    outline: rgb(64, 64, 64) solid 1px;
    color: white;
    font-family: "Work Sans", sans-serif;
    animation-name: fly-in;
    animation-duration: 0.25s;
    z-index: 300;
}

#modalMain.red {
    outline: rgba(224, 89, 89, 0.478) solid 1px;
}

#modalMain button {
    transition: ease-in-out 0.25s;
}

#modalMain button:focus {
    outline: none;
}

#modalMain button:hover {
    filter: brightness(1.1);
}

#modalMain button.cta {
    padding: 10px;
    border-radius: 5px;
    border-width: 0px;
    background-color: dodgerblue;
    color: white;
    font-family: "Work Sans", sans-serif;
}

#modalMain button.red {
    padding: 10px;
    border-radius: 5px;
    border-width: 0px;
    background-color: rgb(208, 14, 14);
    color: white;
    font-family: "Work Sans", sans-serif;
}

#modalMain button.grey {
    padding: 10px;
    border-radius: 5px;
    border-width: 0px;
    background-color: rgb(71, 71, 71);
    color: white;
    font-family: "Work Sans", sans-serif;
}

#modalMain #modalTitle {
    font-size: large;
    margin-bottom: 5px;
    font-weight: bold;
}

#modalMain input {
    padding: 5px;
    border-radius: 2.5px;
    background: #ffffff11;
    margin-bottom: 0px;
}

@keyframes fly-out {
    100% {
        top: -100vh;
        opacity: 0%;
    }
}

@keyframes fade-in {
    0% {
        opacity: 0%;
    }
}

@keyframes fade-out {
    100% {
        opacity: 0%;
    }
}

#failPopup {
    position: fixed;
    left: 20px;
    bottom: 20px;
    padding: 10px;
    border-radius: 5px;
    background-color: #222;
    color: white;
    border: solid 1px #777;
    animation-name: fade-in;
    animation-duration: 1s;
}

`;
code = `
        <div id='modalMain' hidden>
            <div id='modalTitle'></div>
            <div id='modalMessage'></div>
            <br>
            <button id='buttonConfirm' class='cta'>Ok</button> <button id='buttonConfirm' class='grey' onclick=killModalPopup()>Cancel</button>
        </div>
        <div id='failPopup' hidden>
        </div>
`; // * The HTML Code for a popup

popupDataIsThere = false;

function dataInit() {
  if (!popupDataIsThere) {
    this.document.head.innerHTML += "<style>" + cssCode + "</style>";
    this.document.body.innerHTML += code;
    popupDataIsThere = true;
  }
}

function killModalPopup() {
  document.getElementById("modalMain").classList.remove("red");
  setTimeout(function () {
    document.getElementById("modalMain").hidden = true;
  }, 510);
  flyOut("modalMain", 500);
}

function errorModal(title, message, command) {
  document.getElementById("modalMain").hidden = false;
  document.getElementById("modalMain").classList.add("red");
  document.getElementById("modalTitle").innerHTML = title;
  document.getElementById("modalMessage").innerHTML = message;
  document.getElementById("buttonConfirm").onclick = function () {
    command();
    killModalPopup();
  };
}

function confirmModal(title, message, command) {
  document.getElementById("modalMain").hidden = false;
  document.getElementById("modalTitle").innerHTML = title;
  document.getElementById("modalMessage").innerHTML = message;
  document.getElementById("buttonConfirm").onclick = function () {
    command();
    killModalPopup();
  };
}

function confirmModalWarning(title, message, command) {
  document.getElementById("modalMain").hidden = false;
  document.getElementById("modalTitle").innerHTML = title;
  document.getElementById("modalMessage").innerHTML = message;
  document.getElementById("buttonConfirm").onclick = function () {
    command();
    killModalPopup();
  };
  document.getElementById("buttonConfirm").classList.add("red");
}

function inputModal(title, message, inputName, type, command) {
  document.getElementById("modalMain").hidden = false;
  document.getElementById("modalTitle").innerHTML = title;
  document.getElementById("modalMessage").innerHTML =
    message + `<br><input type="${type}" id="${inputName}" class="inputModal">`;
  document.getElementById("buttonConfirm").onclick = function () {
    command();
    killModalPopup();
  };
}

function flyOut(id, duration) {
  animationName_before = document.getElementById(id).style.animationName;
  animationDuration_before =
    document.getElementById(id).style.animationDuration;
  document.getElementById(id).style.animationDuration = duration + "ms";
  document.getElementById(id).style.animationName = "fly-out";
  document.getElementById(id).classList.add("fly-out");
  setTimeout(function () {
    document.getElementById(id).hidden = true;
    document.getElementById(id).classList.remove("fly-out");
    document.getElementById(id).style.animationName = animationName_before;
    document.getElementById(id).style.animationDuration =
      animationDuration_before;
  }, duration - 10);
}

function fadeOut(id, duration) {
  animationName_before = document.getElementById(id).style.animationName;
  animationDuration_before =
    document.getElementById(id).style.animationDuration;
  document.getElementById(id).style.animationDuration = duration + "ms";
  document.getElementById(id).style.animationName = "fade-out";
  document.getElementById(id).classList.add("fade-out");
  setTimeout(function () {
    document.getElementById(id).hidden = true;
    document.getElementById(id).classList.remove("fade-out");
    document.getElementById(id).style.animationName = animationName_before;
    document.getElementById(id).style.animationDuration =
      animationDuration_before;
  }, duration - 10);
}

function failPopup(message) {
  document.getElementById("failPopup").hidden = false;
  document.getElementById("failPopup").innerHTML = message;
  setTimeout(function () {
    fadeOut("failPopup", 3000);
  }, 3000);
}
