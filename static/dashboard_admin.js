// Global variables
// TODO Send proper http status codes !!!!!
currFPath = "/";

// Windows events

window.onclick = function () {
	document.getElementById("contextmenu").hidden = true;
};

window.addEventListener("mousemove", function (e) {
	mouseX = e.pageX;
	mouseY = e.pageY;
});

window.onload = function () {
	dataInit();
	setCPUBar();
	setRAMBar();
	setDiskBar();
	getDriveList();
	getUserList();
	renderFiles(currFPath);
	getDeviceInformation();
	document
		.querySelector("#contextmenu #deleteButton")
		.addEventListener("click", function () {
			confirmModal("Delete", "Do you want to proceed", function () {
				fetch("/api", {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify({
						path: contextFMenuFile,
						r: "deleteFile",
					}),
				})
					.then((res) => res.json())
					.then((data) => {
						if (data["status"] == "s") {
							renderFiles(currFPath);
						} else {
							alert("Can not delete this file");
						}
					});
				renderFiles(currFPath);
			});
		});

	document
		.querySelector("#contextmenu #renameButton")
		.addEventListener("click", function () {
			confirmModal(
				"Rename",
				"Filename<br><br><input type='text' id='renameNameInput'>",
				function () {
					var newFileName = document.getElementById("renameNameInput").value;
					fetch("/api", {
						method: "POST",
						headers: {
							"Content-Type": "application/json",
						},
						body: JSON.stringify({
							path: contextFMenuFile,
							newName: currFPath + newFileName,
							r: "renameFile",
						}),
					})
						.then((res) => res.json())
						.then((data) => {
							if (data["status"] == "s") {
								renderFiles(currFPath);
							} else {
								alert("Can not rename this file");
							}
						});
					renderFiles(currFPath);
				},
			);
		});

	document.getElementById("ftp_running").addEventListener("change", () => {
		var enableFTP = this.checked;
		var FTPlocalRoot = null;
		var ftpUserUsername = null;
		var ftpUserPassword = null;

		inputModal(
			"Sudo password",
			"Please enter your sudo password to change these settings",
			"sudoPasswordFTP",
			"password",
			function () {
				// TODO Not yet reading the sudo password
				document.getElementById("ftpSettingsApply").innerText = "Updating";
				fetch("/api", {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify({
						r: "updateFTPconfig",
						enableFTP: enableFTP,
						ftpLocalRoot: FTPlocalRoot,
						ftpUserUsername: ftpUserUsername,
						ftpUserPassword: ftpUserPassword,
						sudo: document.getElementById("sudoPasswordFTP").value,
						enableDisable: true,
					}),
				})
					.then((res) => {
						if (!res.ok) {
							res.json().then(function (jsonResponse) {
								document.getElementById("ftpSettingsApply").innerHTML =
									"Failed (retry)";
								document.getElementById("ftpError").innerHTML =
									jsonResponse["details"];
								failPopup("Failed to update FTP configuration");
								throw new Error("Failed to update FTP configuration");
							});
						} else {
							document.getElementById("ftpError").innerHTML = "";
						}
						return res.json(); // ! The JSON is empty => Fix on server side!!!!
					})
					.then(() => {
						fetchFTPconnectionInformation();
						document.getElementById("ftpSettingsApply").innerText = "Apply";
					});
			},
		);
	});
};

// Intervals

setInterval(function () {
	setCPUBar();
	setRAMBar();
	setDiskBar();
	getDriveList();
}, 10000);

// Functions

// Status bars (Dashboard)

function setCPUBar() {
	fetch("/api?r=cpuPercent", {
		method: "GET",
		headers: {
			"Content-Type": "application/json",
		},
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch CPU status");
				throw new Error("Failed to fetch CPU status");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("cpu_bar").style.width =
				Math.floor(Number(data["p"] * 2)) + "px";
			document.getElementById("cpu_bar").title =
				"CPU " + Math.round(data["p"]) + "%";
		});
}

function setRAMBar() {
	fetch("/api?r=ramPercent", {
		method: "GET",
		headers: {
			"Content-Type": "application/json",
		},
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch RAM status");
				throw new Error("Failed to fetch RAM status");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("ram_bar").style.width =
				Math.floor(Number(data["p"]) * 2) + "px";
			document.getElementById("ram_bar").title =
				"RAM " + Math.round(data["p"]) + "%";
		});
}

function setDiskBar() {
	fetch("/api?r=diskPercent", {
		method: "GET",
		headers: {
			"Content-Type": "application/json",
		},
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch Disk status");
				throw new Error("Failed to fetch Disk status");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("disk_bar").style.width =
				Math.floor(Number(data["p"]) * 2) + "px";
			document.getElementById("disk_bar").title =
				"Disk " + Math.round(data["p"]) + "%";
		});
}

function getDriveList() {
	fetch("/api?r=driveList", {
		method: "GET",
		headers: {
			"Content-Type": "application/json",
		},
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch disk list");
				throw new Error("Failed to fetch disk list");
			}
			return res.json();
		})
		.then((data) => {
			var htmlCode = "";
			for (drive of Array.from(data["drives"])) {
				var childrenHtmlCode = "";
				if (drive["children"] != null) {
					for (child of drive["children"]) {
						var childrenHtmlCode =
							childrenHtmlCode +
							`<button class="drive" onclick="driveInformationModal('${child["name"]}')">${child["name"]}</button>`;
					}
					var htmlCode =
						htmlCode +
						`<button class="drive" onclick="driveInformationModal('${drive["name"]}')">${drive["name"]}</button>${childrenHtmlCode}`;
				} else {
					var htmlCode =
						htmlCode +
						`<button class="drive" onclick="driveInformationModal('${drive["name"]}')">${drive["name"]}</button>`;
				}
			}
			document.getElementById("disks").innerHTML = htmlCode;
		});
}

function getUserList() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "userList",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch list of users");
				throw new Error("Failed to fetch list of users");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("usersTable").innerHTML = data["text"];
		});
}

// User management

function deleteUser(username) {
	if (confirm(`Do you want to delete ${username}?`)) {
		fetch("/api", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				username: username,
				r: "deleteUser",
			}),
		})
			.then((res) => {
				if (!res.ok) {
					failPopup("Failed to delete user");
					throw new Error("Failed to delete user");
				}
				return res.json();
			})
			.then((data) => {
				getUserList();
			});
	}
}

function addNewUser() {
	document.getElementById("newUserModal").hidden = false;
}

function submitNewUser() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "newUser",
			username: "",
			password: "",
			userChoosesPassword: false,
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to submit new user");
				throw new Error("Failed to submit new user");
			}
			return res.json();
		})
		.then((data) => {
			// TODO Missing
		});
}

// Interface

function changePage(pageName) {
	for (page of document.querySelectorAll("#pages > div")) {
		console.log(page);
		if (page.id != pageName) {
			page.hidden = true;
		} else {
			page.hidden = false;
		}
	}

	for (button of document.querySelectorAll("#sideBar > button")) {
		button.style.backgroundColor = "#242424";
	}

	document.activeElement.style.backgroundColor = "#333333";
	document.activeElement.blur();

	if (pageName == "applications" && typeof allApps == "undefined") {
		renderApplicationManagerList();
	}
	if (pageName == "connections") {
		fetchFTPconnectionInformation();
	}
	if (pageName == "users") {
		getUserList();
	}
}

// Files / Stroage

function renderFiles(path) {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			path: path,
			showHiddenFiles: document.getElementById("showHiddenFiles").checked,
			r: "filesRender",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch list of files");
				throw new Error("Failed to fetch list of files");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("filesContainer").innerHTML = data["content"];
		});
}

function navigateFolder(file) {
	currFPath = currFPath + file + "/";
	renderFiles(currFPath);
}

function downloadFile(file) {
	window.open("/api?r=callfile&file=" + btoa(currFPath + file));
}

function goFUp() {
	if (currFPath != "/") {
		currFPathReps = currFPath.split("/")[currFPath.split("/").length - 2] + "/";
		currFPath = currFPath.replace(currFPathReps, "");
		renderFiles(currFPath);
	}
}

function contextMenuF(filename) {
	document.getElementById("contextmenu").hidden = false;
	document.getElementById("contextmenu").style.top = mouseY + "px";
	document.getElementById("contextmenu").style.left = mouseX + "px";
	contextFMenuFile = currFPath + filename;
}

function driveInformationModal(driveName) {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "driveInformation",
			driveName: driveName,
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch drive information");
				throw new Error("Failed to fetch drive information");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("driveName").innerText = data["drives"]["name"];
			document.getElementById("driveModel").innerText =
				data["drives"]["model"] == null ? "N/A" : data["drives"]["model"];
			document.getElementById("driveSize").innerText =
				data["drives"]["size"] == null
					? "N/A"
					: Math.floor(Number(data["drives"]["size"]) / 1073741824) + " GB" ==
						  "0 GB"
						? data["drives"]["size"] + " B"
						: Math.floor(Number(data["drives"]["size"]) / 1073741824) + " GB";
			document.getElementById("driveMountpoint").innerText =
				data["drives"]["mountpoint"] == null
					? "N/A"
					: data["drives"]["mountpoint"];
			document.getElementById("drivePath").innerText =
				data["drives"]["path"] == null ? "N/A" : data["drives"]["path"];
			document.getElementById("driveMounted").innerHTML = driveName.includes(
				"sda",
			)
				? "True"
				: data["drives"]["mountpoint"] != null
					? "True"
					: "False";
			document.getElementById("driveUssage").innerHTML = "N/A";
			for (drive of data["ussage"]) {
				if (drive["mounted"] == data["drives"]["mountpoint"]) {
					document.getElementById("driveUssage").innerHTML = drive["capacity"];
				}
			}

			document.getElementById("driveModal").hidden = false;
		});
}

// Packages

function renderApplicationManagerList() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "packageDatabase",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Failed to fetch package list");
				throw new Error("Failed to fetch package list");
			}
			return res.json();
		})
		.then((data) => {
			var responseJSON = JSON.parse(data["content"]);
			guiApps = responseJSON["gui"]; // ? Installed & has GUI
			anyApps = responseJSON["any"]; // ? Installed and can have GUI
			allApps = responseJSON["all"]; // ? All packages in the DB
			document.getElementById("loadingApplications").hidden = true;
			document.getElementById("packageSearchResults").hidden = true;
			document.getElementById("installedPackagesDetails").hidden = false;
			document.getElementById("installedAppsDetails").hidden = false;
			console.log(guiApps);
			var htmlCode = "";
			for (e of Array.from(guiApps)) {
				if (e != undefined) {
					var htmlCode =
						htmlCode +
						"<div class='package'><img src='" +
						e[1] +
						"'><br>" +
						e[0].split(".")[0].replace("-", " ") +
						"<br><button class='remove_package' onclick='removePackage(\"" +
						e[2] +
						"\", this)'>Remove</button></div>";
					console.log(e[1]);
				}
			}
			document.getElementById("installedApps").innerHTML = htmlCode;

			var htmlCode = "";
			for (e of Array.from(anyApps)) {
				if (e.length != 0) {
					if (e != undefined) {
						var htmlCode =
							htmlCode +
							"<div class='package_small'>" +
							e.split(".")[0].replace("-", " ") +
							"<button class='remove_package' onclick='removePackage(\"" +
							e +
							"\", this)'>Remove</button></div>";
						console.log(e[1]);
					}
				}
			}
			document.getElementById("installedPackages").innerHTML = htmlCode;
		});
}

function lookForPackage() {
	var packageName = document.getElementById("packageSearch").value;
	if (packageName != "" && packageName != null) {
		document.getElementById("packageSearchResults").hidden = false;
		document.getElementById("installedPackagesDetails").hidden = true;
		document.getElementById("installedAppsDetails").hidden = true;
		var htmlCode = "";
		if (packageName.length > 1) {
			for (e of anyApps) {
				if (e.includes(packageName)) {
					var htmlCode =
						htmlCode +
						`<div class=package_small>${e.split(".")[0]} <button class=remove_package onclick=\"removePackage('${e}', this)\">Remove</button></div>`;
				}
			}

			for (e of allApps) {
				if (e.includes(packageName)) {
					var htmlCode =
						htmlCode +
						`<div class=package_small>${e.split(".")[0]} <button class=install_package onclick=\"installPackage('${e}', this)\">Install</button></div>`;
				}
			}
		}
		document.getElementById("packageSearchResults").innerHTML = htmlCode;
	} else {
		document.getElementById("packageSearchResults").hidden = true;
		document.getElementById("installedPackagesDetails").hidden = false;
		document.getElementById("installedAppsDetails").hidden = false;
	}
}

function removePackage(packageName, button) {
	confirmModal(
		"Remove app",
		"<input type='password' placeholder='SUDO Password' id='sudoPasswordInput'>",
		function () {
			button.innerHTML = "In work";
			button.disabled = true;
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "removePackage",
					packageName: packageName,
					sudoPassword: document.getElementById("sudoPasswordInput").value,
				}),
			})
				.then((res) => {
					if (!res.ok) {
						failPopup("Failed to remove package");
						button.innerHTML = "Failed";
						button.disabled = false;
						button.style.color = "rgb(255, 75, 75);";
						throw new Error("Failed to remove package");
					}
					return res.json();
				})
				.then((data) => {
					if (document.getElementById("packageSearch").value.length > 0) {
						lookForPackage();
					} else {
						renderApplicationManagerList();
					}
				});
		},
	);
}

function installPackage(packageName, button) {
	confirmModal(
		"Install package",
		"<input type='password' placeholder='SUDO Password' id='sudoPasswordInput'>",
		function () {
			button.innerHTML = "In work";
			button.disabled = true;
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "installPackage",
					packageName: packageName,
					sudoPassword: document.getElementById("sudoPasswordInput").value,
				}),
			})
				.then((res) => {
					if (!res.ok) {
						failPopup("Failed to install package");
						button.innerHTML = "Failed";
						button.disabled = false;
						button.style.color = "rgb(255, 75, 75);";
						throw new Error("Failed to install package");
					}
					return res.json();
				})
				.then((data) => {
					renderApplicationManagerList();
				});
		},
	);
}

// Network

// FTP

function updateFTPConnectionSettings() {
	var enableFTP = document.getElementById("enableFTP").checked;
	var FTPlocalRoot = document.getElementById("ftpLocalRoot").value;
	var ftpUserUsername = document.getElementById("ftpUserUsername").value;
	var ftpUserPassword = document.getElementById("ftpUserPassword").value;

	if (ftpUserPassword.length == 0) {
		document.getElementById("ftpUserPassword").focus();
		return;
	}

	inputModal(
		"Sudo password",
		"Please enter your sudo password to change these settings",
		"sudoPasswordFTP",
		"password",
		function () {
			// TODO Not yet reading the sudo password
			document.getElementById("ftpSettingsApply").innerText = "Updating";
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "updateFTPconfig",
					enableFTP: enableFTP,
					ftpLocalRoot: FTPlocalRoot,
					ftpUserUsername: ftpUserUsername,
					ftpUserPassword: ftpUserPassword,
					sudo: document.getElementById("sudoPasswordFTP").value,
				}),
			})
				.then((res) => {
					if (!res.ok) {
						res.json().then(function (jsonResponse) {
							document.getElementById("ftpSettingsApply").innerHTML =
								"Failed (retry)";
							document.getElementById("ftpError").innerHTML =
								jsonResponse["details"];
							failPopup("Failed to update FTP configuration");
							throw new Error("Failed to update FTP configuration");
						});
					} else {
						document.getElementById("ftpError").innerHTML = "";
					}
					return res.json(); // ! The JSON is empty => Fix on server side!!!!
				})
				.then(() => {
					fetchFTPconnectionInformation();
					document.getElementById("ftpSettingsApply").innerText = "Apply";
				});
		},
	);
}

function fetchFTPconnectionInformation() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "fetchFTPconfig",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Can not fetch FTP configuration information");
				throw new Error("Failed to fetch FTP configuration information");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("enableFTP").checked = data["enabled"];
			document.getElementById("ftp_running").checked = data["enabled"];
			document.getElementById("ftpUserUsername").value =
				data["ftpUserUsername"];
			document.getElementById("ftpLocalRoot").value = data["ftpLocalRoot"];
		});
}

function otherConnectionsTab(pageName) {
	for (page of document.querySelectorAll("#connectionTabsPages > div")) {
		console.log(page);
		if (page.id != pageName) {
			page.hidden = true;
		} else {
			page.hidden = false;
		}
	}

	for (page of document.querySelectorAll("#conectionsTabs > button")) {
		console.log(page);
		if (page.id != pageName + "Button") {
			page.classList.remove("active");
		} else {
			page.classList.add("active");
		}
	}
}

function getDeviceInformation() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "deviceInformation",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				failPopup("Can not fetch device information");
				throw new Error("Failed to fetch device information");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("operating_system_name").innerText =
				data["os_name"];
			document.getElementById("power_supply").innerText = data["power_supply"];
			document.getElementById("zentrox_pid").innerText = data["zentrox_pid"];
		});
}

// Modal.js

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
