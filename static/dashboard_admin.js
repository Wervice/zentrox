currFPath = "/";

updatingFTPstatus = false;
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
	addSkeletons();
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
		var FTPlocalRoot = null;
		var ftpUserUsername = null;
		var ftpUserPassword = null;
		updatingFTPstatus = true;

		attachLoader("ftp_running");

		// Not yet reading the sudo password
		document.getElementById("ftpSettingsApply").innerText = "Updating";
		fetch("/api", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				r: "updateFTPconfig",
				enableFTP: document.getElementById("ftp_running").checked,
				ftpLocalRoot: FTPlocalRoot,
				ftpUserUsername: ftpUserUsername,
				ftpUserPassword: ftpUserPassword,
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
						document.getElementById("ftp_running").checked =
							!document.getElementById("ftp_running").checked;
						throw new Error("Failed to update FTP configuration");
					});
				} else {
					document.getElementById("ftpError").innerHTML = "";
				}
				updatingFTPstatus = false;
				return res.json(); // ! The JSON is empty => Fix on server side!!!!
			})
			.then(() => {
				updatingFTPstatus = false;
				fetchFTPconnectionInformation();
				document.getElementById("ftpSettingsApply").innerText = "Apply";
				removeLoader("ftp_running");
			});
	});
	document
		.getElementById("submit_vault_config")
		.addEventListener("click", function () {
			this.innerHTML = "Updating";
			attachLoader("submit_vault_config", "white");
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "vault_configure",
					key: document.getElementById("vault_key_config").value,
				}),
			})
				.then((res) => {
					return res.json();
				})
				.then((json) => {
					if (json["code"] == "no_decrypt_key") {
						if (document.getElementById("vault_key_config").value.length != 0) {
							confirmModal(
								"Change vault key",
								"Do you want to change the encryption key of Zentrox Vault?",
								() => {
									ask_for_vault_dec_key();
								},
								() => {},
							);
						}
					}
					this.innerHTML = "Apply";
					removeLoader("submit_vault_config");
				});
		});
};
// Intervals

setInterval(function () {
	setCPUBar();
	setRAMBar();
	setDiskBar();
	getDriveList();
	getDeviceInformation();
}, 1000);

// Functions

function ask_for_vault_dec_key() {
	inputModal(
		"Vault key",
		"Please enter the <b>current vault key</b>.",
		"vault_key_old",
		"password",
		() => {
			document;

			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "vault_configure",
					old_key: document.getElementById("vault_key_old").value,
					new_key: document.getElementById("vault_key_config").value,
				}),
			})
				.then((res) => {
					return res.json();
				})
				.then((json) => {
					if (json["message"] == "auth_failed") {
						failPopup("Failed to change Vault key");
					} else {
						confirmModal(
							"Changed vault key",
							"The vault key was sucessfully changed",
							() => {},
							() => {},
							false,
						);
					}
					removeLoader("submit_vault_config");
				});
		},
	);
}

function addSkeletons() {
	const skeleton = `
		<span class="skeleton"></span>
	`;
	document.getElementById("operating_system_name").innerHTML = skeleton;
	document.getElementById("power_supply").innerHTML = skeleton;
	document.getElementById("zentrox_pid").innerHTML = skeleton;
	document.getElementById("process_number").innerHTML = skeleton;
	document.getElementById("hostname").innerHTML = skeleton;
	document.getElementById("hostname_subtitle").innerHTML = skeleton;
	document.getElementById("uptime").innerHTML = skeleton;
	document.getElementById("small_uptime").innerHTML = skeleton;
	document.getElementById("temperature").innerHTML = skeleton;
}

function open_vault() {
	document.getElementById("vault_config").hidden = true;
	document.getElementById("vault_files").hidden = true;
	inputModal(
		"Unlock Vault",
		"Enter the vault key to unlock the vault",
		"vault_key_unlock",
		"password",
		() => {
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "vault_tree",
					key: document.getElementById("vault_key_unlock").value,
				}),
			})
				.then((res) => {
					if (!res.ok) {
						failPopup("Vault connection failed");
					}
					return res.json();
				})
				.then((data) => {
					if (data["message"] == "auth_failed") {
						failPopup("Permission error");
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else if (data["message"] == "vault_not_configured") {
						failPopup("Vault not configured");
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else {
						document.getElementById("vault_view").hidden = false;
						document.getElementById("vault_files").hidden = false;
						draw_vault_file_structure("/", data["fs"]);
						sessionStorage.setItem(
							"vault_key",
							btoa(document.getElementById("vault_key_unlock").value),
						);
					}
				});
		},
		() => {
			document.getElementById("vault_view").hidden = true;
			document.getElementById("vault_configure").hidden = false;
		},
	);
}

function path_join(parts, sep) {
	var separator = sep || "/";
	var replace = new RegExp(separator + "{1,}", "g");
	return parts.join(separator).replace(replace, separator);
}

function draw_vault_file_structure(path, data) {
	var files = [];
	var files_code = "";
	if (path == "/") {
		for (file of data) {
			files.push(file);
		}
	}
	for (file of files) {
		files_code += `<button class="fileButtons" onclick="open_vault_file('${file}', '${path}', this)">${file}</button>`;
	}
	document.getElementById("vault_files").innerHTML = files_code;
}

function open_vault_file(filename, path, button = null) {
	if (button != null) {
		button.innerHTML += `<img src="small_loading_white.svg" class="loader">`;
	}
	fetch(`/api`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "vault_file_download",
			path: path_join([path, filename]),
			key: atob(sessionStorage.getItem("vault_key")),
		}),
	})
		.then((res) => {
			if (button != null) {
				button.querySelector("img").remove();
			}
			return res.blob();
		})
		.then((blob) => {
			var url = window.URL.createObjectURL(blob);
			var a = document.createElement("a");
			a.href = url;
			a.download = filename;
			document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
			a.click();
			a.remove();
		});
}

function close_vault_files() {
	document.getElementById("vault_view").hidden = true;
	document.getElementById("vault_config").hidden = false;
	sessionStorage.removeItem("vault_key");
}

function vault_file_upload() {
	inputModal(
		"Upload file",
		"Pick a file to upload",
		"vault_file_upload_input",
		"file",
		() => {
			var file_for_upload = document.getElementById("vault_file_upload_input")
				.files[0];
			if (file_for_upload) {
				var form_data = new FormData();
				form_data.append("file", file_for_upload);
				form_data.append("key", atob(sessionStorage.getItem("vault_key")));
				fetch("/upload/vault", {
					method: "POST",
					headers: {},
					body: form_data,
				})
					.then((res) => {
						return res.json();
					})
					.then((json) => {
						if (json["message"]) {
							confirmModalWarning("Upload error", "Upload failed");
						}
						else {
							confirmModal("Upload success", "The file was uploaded to vault", () => {}, () => {}, false)
						fetch("/api", {
							method: "POST",
							headers: {
								"Content-Type": "application/json",
							},
							body: JSON.stringify({
								r: "vault_tree",
								key: atob(sessionStorage.getItem("vault_key")),
							}),
						})
							.then((res) => {
								if (!res.ok) {
									failPopup("Vault connection failed");
									document.getElementById("vault_view").hidden = true;
									document.getElementById("vault_config").hidden = false;
								}
								return res.json();
							})
							.then((data) => {
								if (data["message"] == "auth_failed") {
									failPopup("Permission error");
									document.getElementById("vault_view").hidden = true;
									document.getElementById("vault_config").hidden = false;
								} else if (data["message"] == "vault_not_configured") {
									failPopup("Vault not configured");
									document.getElementById("vault_view").hidden = true;
									document.getElementById("vault_config").hidden = false;
								} else {
									draw_vault_file_structure("/", data["fs"]);
								}
							});}
					});
			}
		},
		() => {},
	);
}

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
			if (data["message"] == "no_permissions") {
				failPopup("Missing permission to open this file");
				goFUp();
			} else {
				document.getElementById("filesContainer").innerHTML = data["content"];
			}
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
			document.getElementById("packageSearch").hidden = false;
			console.log(guiApps);
			var htmlCode = "";
			for (e of Array.from(guiApps)) {
				if (e != undefined) {
					var htmlCode =
						htmlCode +
						"<div class='package'>" +
						e[0].split(".")[0].replace("-", " ") +
						"<button class='remove_package' onclick='removePackage(\"" +
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
	var packageName = document
		.getElementById("packageSearch")
		.value.toLowerCase();
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
		}

		if (packageName.length > 1) {
			for (e of [...new Set(Array.prototype.concat(allApps, anyApps))]) {
				if (e.includes(packageName)) {
					var htmlCode =
						htmlCode +
						`<div class=package_small>${e.split(".")[0]} <button class=install_package onclick=\"installPackage('${e}', this)\">Install</button></div>`;
				}
			}
		}

		document.getElementById("packageSearchResults").innerHTML = htmlCode;
		if (htmlCode == "") {
			document.getElementById("packageSearchResults").innerHTML =
				`<div style="text-align:center">No results</div>`;
		}
	} else {
		document.getElementById("packageSearchResults").hidden = true;
		document.getElementById("installedPackagesDetails").hidden = false;
		document.getElementById("installedAppsDetails").hidden = false;
	}
}

function removePackage(packageName, button) {
	confirmModal(
		"Remove package",
		"<input type='password' placeholder='Root password' id='sudoPasswordInput'>",
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
					button.innerHTML = "Install";
					button.classList.remove("remove_package");
					button.classList.add("install_package");
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
					button.innerHTML = "Remove";
					button.classList.remove("install_package");
					button.classList.add("remove_package");
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

	rootInputModal(
		"Elevated privileges",
		"Please enter your root password to change these settings",
		"sudoPasswordFTP",
		"password",
		function () {
			attachLoader("ftpSettingsApply", "white");
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
					removeLoader("ftpSettingsApply");
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
			if (data["enabled"] == true) {
				document.getElementById("ftpUserUsername").disabled = true;
				document.getElementById("ftpUserPassword").disabled = true;
				document.getElementById("ftpLocalRoot").disabled = true;
			} else {
				document.getElementById("ftpUserUsername").disabled = false;
				document.getElementById("ftpUserPassword").disabled = false;
				document.getElementById("ftpLocalRoot").disabled = false;
			}
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
			document.getElementById("process_number").innerText =
				data["process_number"];
			document.getElementById("hostname").innerText = data["hostname"];
			document.getElementById("hostname_subtitle").innerText = data["hostname"];
			document.getElementById("uptime").innerText = data["uptime"];
			document.getElementById("small_uptime").innerText = data["uptime"]
				.split(", ")[0]
				.replaceAll("\n", "");
			document.getElementById("temperature").innerText =
				data["temperature"] != null ? data[temperature] : `No temerpature`;
		});
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
				failPopup("Can not fetch device information");
				throw new Error("Failed to fetch device information");
			}
			return res.json();
		})
		.then((data) => {
			if (!updatingFTPstatus) {
				document.getElementById("ftp_running").checked = data["enabled"];
			}
		});
}

function poweroffSystem() {
	confirmModal(
		"Power off system",
		"Do you want to power off the system",
		() => {
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "power_off",
				}),
			})
				.then((res) => {
					if (!res.ok) {
						failPopup("Can not power off");
						throw new Error("Failed to power off system");
					}
					return res.json();
				})
				.then((data) => {
					document.documentElement.innerHTML = "System Power Off";
				});
		},
		() => {},
	);
}

function openDetails(id) {
	for (detail_tag of document.querySelectorAll("div")) {
		if (detail_tag.id.includes("storage_")) {
			detail_tag.hidden = true;
		}
	}
	document.getElementById(id).hidden = false;
}

function attachLoader(id, color = "black") {
	var object = document.getElementById(id);
	if (object.tagName.toLowerCase() == "button") {
		object.innerHTML += `<img src="small_loading_${color}.svg" class="loader">`;
	} else if (object.tagName.toLowerCase() == "input") {
		object.style.backgroundImage = `url("small_loading_${color}.svg")`;
	}
}

function removeLoader(id) {
	var object = document.getElementById(id);
	if (object.tagName.toLowerCase() == "input") {
		object.style.backgroundImage = `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1em' height='1em' viewBox='0 0 24 24'%3E%3Cpath fill='black' d='m9.55 18l-5.7-5.7l1.425-1.425L9.55 15.15l9.175-9.175L20.15 7.4z'/%3E%3C/svg%3E")`;
	}
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
	top: calc((100vh - 40vh) / 2 - 40px);
	width: 40vh;
    left: calc((100vw - 40vh) / 2 - 40px);
    border-radius: 5px;
    padding: 20px;
	padding-bottom: 5vh;
	padding-top: 5vh;
	background-color: #232323;
    outline: rgb(64, 64, 64) solid 1px;
    color: white;
    font-family: "Work Sans", sans-serif;
    animation-name: pop_open;
    animation-duration: 0.25s;
    z-index: 300;

	input {
		padding: 8px;
		font-size: 15px;
		margin-left: 0px;
		margin-top: 8px;
		margin-bottom: 8px;
	}
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
    font-size: 25px;
    margin-bottom: 10px;
    font-weight: bold;
}

#modalMain #modalMessage {
    font-size: 16px;

	input {
		padding: 8px;
		font-size: 15px;
		margin-left: 0px;
		margin-top: 8px;
		margin-bottom: 8px;
	}
}

#modalMain input {
    padding: 5px;
    border-radius: 2.5px;
    background: #ffffff11;
    margin-bottom: 0px;
	max-width: 100%;
}

@keyframes fly-out {
    100% {
        top: -100vh;
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

#failPopup {
    position: fixed;
    left: 20px;
    bottom: 20px;
    padding: 10px;
    border-radius: 5px;
    background-color: #333;
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
            <button id='buttonConfirm' class='cta'>Ok</button> <button id='buttonCancle' class='grey' onclick=killModalPopup()>Cancel</button>
        </div>
        <div id='failPopup' hidden>
        </div>
		<div id='statusPopup' hidden></div>
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

function errorModal(title, message, command, cancled = () => {}) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalMain").classList.add("red");
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML = message;
	document.getElementById("buttonConfirm").onclick = function () {
		killModalPopup();
		command();
	};
}

function confirmModal(
	title,
	message,
	command,
	cancled = () => {},
	show_cancle = true,
) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML = message;
	document.getElementById("buttonConfirm").onclick = function () {
		killModalPopup();
		setTimeout(() => {
			command();
		}, 600);
	};
	document.getElementById("buttonCancle").hidden = !show_cancle;
}

function confirmModalWarning(title, message, command, cancled = () => {}) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML = message;
	document.getElementById("buttonConfirm").onclick = function () {
		killModalPopup();
		setTimeout(() => {
			command();
		document.getElementById("buttonConfirm").classList.remove("red");
		}, 600);
	};
	document.getElementById("buttonConfirm").classList.add("red");
}

function inputModal(
	title,
	message,
	inputName,
	type,
	command,
	cancled = () => {},
) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML =
		message + `<br><input type="${type}" id="${inputName}" class="inputModal">`;
	document.getElementById("buttonConfirm").onclick = function () {
		killModalPopup();
		setTimeout(() => {
			command();
		}, 600);
	};
	document.getElementById("buttonCancle").onclick = function () {
		killModalPopup();
		cancled();
	};
}

function rootInputModal(
	title,
	message,
	inputName,
	type,
	command,
	cancled = () => {},
) {
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

function statusPopup(message) {
	document.getElementById("statusPopup").hidden = false;
	document.getElementById("statusPopup").innerHTML = "✅ " + message;
	setTimeout(function () {
		fadeOut("failPopup", 5000);
	}, 5000);
}
