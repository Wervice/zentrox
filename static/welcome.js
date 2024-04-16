// For index.html in templates

function requestSetupStart() {
  document.getElementById("startButton").innerText = "Loading...";
  fetch("/api?r=startsetup")
    .then((res) => res.json())
    .then((data) => {
      if (data["status"] == "s") {
        document.getElementById("startButton").innerText = "Forwarding...";
        location.href = "/setup";
      } else {
        document.getElementById("startButton").innerText = "Failed";
      }
    });
}
