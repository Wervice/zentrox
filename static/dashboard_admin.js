currFPath = "/";
updatingFTPstatus = false;
// Windows events

window.onload = function () {
	dataInit();
	setTimeout(
		() => {
			location.href = "/logout";
		},
		12 * 60 * 60 * 1000,
	);
	setCPUBar();
	setRAMBar();
	getDriveList();
	getUserList();
	setDiskBar();
	renderFiles(currFPath);
	getDeviceInformation();
	getFireWallInformation();
	addSkeletons();
	window.onclick = function () {
		document.getElementById("contextmenu").hidden = true;
		document.getElementById("vault_context_menu").hidden = true;
	};

	window.addEventListener("mousemove", function (e) {
		mouseX = e.pageX;
		mouseY = e.pageY;
	});

	window.onkeyup = (event) => {
		if (!document.getElementById("modalMain").hidden) {
			if (event.key == "Enter") {
				document.getElementById("buttonConfirm").click();
			}
		}
	};
	document
		.querySelector("#contextmenu #deleteButton")
		.addEventListener("click", function () {
			confirm_modal_warning(
				"Delete",
				"Do you want to proceed",
				function () {
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
				},
				() => {},
				true,
			);
		});

	document
		.querySelector("#contextmenu #renameButton")
		.addEventListener("click", function () {
			confirm_modal(
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

		attach_loader("ftp_running");

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
						fail_popup("Failed to update FTP configuration");
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
				remove_loader("ftp_running");
			});
	});
	document.getElementById("enableUFW").addEventListener("change", () => {
		fetch("/api", {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify({
				r: "switchUFW",
				enableUFW: document.getElementById("enableUFW").checked,
			}),
		})
			.then((res) => {
				if (!res.ok) {
					fail_popup("Failed to change UFW status");
					throw new Error("Failed to change UFW status");
				}
				return res.json();
			})
			.then((json) => {
				console.log(json);
				document.getElementById("fireWallRuleOverview").innerHTML = "";
			});
	});
	document
		.getElementById("submit_vault_config")
		.addEventListener("click", function () {
			this.innerHTML = "Updating";
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
							confirm_modal(
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
				});
		});
};
// Intervals

setInterval(function () {
	setCPUBar();
	setRAMBar();
}, 4000);

setInterval(() => {
	getDeviceInformation();
	getFireWallInformation();
	getDriveList();
}, 6000);

// Functions

function changeTheme(theme = "light") {
	if (theme === "light") {
		for (image of document.querySelectorAll("img")) {
			image.style.filter = "invert()";
		}
		document.documentElement.style.setProperty("--background-dark", "#fff");
		document.documentElement.style.setProperty(
			"--background-semi-dark",
			"#9cf",
		);
		document.documentElement.style.setProperty(
			"--background-semi-light",
			"#ccccff22",
		);
		document.documentElement.style.setProperty("--foreground", "#000");
	}
}

function ask_for_vault_dec_key() {
	input_modal(
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
					oldKey: document.getElementById("vault_key_old").value,
					newKey: document.getElementById("vault_key_config").value,
				}),
			})
				.then((res) => {
					return res.json();
				})
				.then((json) => {
					if (json["message"] == "auth_failed") {
						fail_popup("Failed to change Vault key");
					} else {
						confirm_modal(
							"Changed vault key",
							"The vault key was sucessfully changed",
							() => {},
							() => {},
							false,
						);
					}
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

function backup_vault() {
	confirm_modal(
		"Backup",
		`Please select a backup location<br><select id="vault_backup_location"><option value="download">Download</option></select>`,
		() => {
			const backup_location = document.getElementById(
				"vault_backup_location",
			).value;
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "vault_backup",
					key: atob(sessionStorage.getItem("vault_key")),
				}),
			})
				.then((res) => {
					if (!res.ok) {
						fail_popup("Vault download failed");
					}
					return res.blob();
				})
				.then((blob) => {
					if (backup_location == "download") {
						var url = window.URL.createObjectURL(blob);
						var a = document.createElement("a");
						a.href = url;
						a.download = "vault.tar";
						document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
						a.click();
						a.remove();
					}
				});
		},
		() => {},
		true,
	);
}

function open_vault(button) {
	document.getElementById("vault_config").hidden = true;
	document.getElementById("vault_files").hidden = true;
	input_modal(
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
						fail_popup("Vault connection failed");
					}
					return res.json();
				})
				.then((data) => {
					if (data["message"] == "auth_failed") {
						fail_popup(
							"Permission error <button onclick='location.reload()'>Reload</button>",
						);
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else if (data["message"] == "vault_not_configured") {
						fail_popup("Vault not configured");
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else {
						document.getElementById("vault_view").hidden = false;
						document.getElementById("vault_files").hidden = false;
						vault_path = "/";
						draw_vault_file_structure(vault_path, data["fs"]);
						vault_file_system = data["fs"];
						sessionStorage.setItem(
							"vault_key",
							btoa(document.getElementById("vault_key_unlock").value),
						);
					}
				});
		},
		() => {
			document.getElementById("vault_view").hidden = true;
			document.getElementById("vault_config").hidden = false;
		},
	);
}

function path_join(parts, sep) {
	var separator = sep || "/";
	var replace = new RegExp(separator + "{1,}", "g");
	return parts.join(separator).replace(replace, separator);
}

function get_current_folder_paths(paths, location) {
	// If starts with path
	// and has nothing after the last /
	var paths = paths.map((path) => {
		return "/" + path;
	});
	var paths = paths.filter((path) => {
		if (!path.startsWith(location)) return false;
		if (path == location) return false;
		if (path.replace(location, "").split("/").length < 3) {
			try {
				if (path.replace(location, "").split("/")[1].length === 0) return true;
			} catch {
				return true;
			}
			return false;
		}
	});
	return paths;
}

function draw_vault_file_structure(path, data) {
	console.log(data);
	console.log(path);
	var files = get_current_folder_paths(data, path);
	var files_code = "";

	for (file of files) {
		file = "/" + file;
		if (file.endsWith("/")) {
			file = file.split("/")[file.split("/").length - 2] + "/";
		} else {
			file = file.split("/")[file.split("/").length - 1];
		}
		files_code += `<button class="fileButtons" onclick="open_vault_file('${file}', '${path}', this)" oncontextmenu="open_vault_context('${file}', this)">${file}</button>`;
	}
	document.getElementById("vault_files").innerHTML = files_code;
}

function reload_file_structure(loader, button = null) {
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
				fail_popup("Vault connection failed");
				document.getElementById("vault_view").hidden = true;
				document.getElementById("vault_config").hidden = false;
			}
			return res.json();
		})
		.then((data) => {
			if (loader) {
				remove_loader(button.id);
			}
			if (data["message"] == "auth_failed") {
				fail_popup(
					"Permission error <button onclick='location.reload()'>Reload</button>",
				);
				document.getElementById("vault_view").hidden = true;
				document.getElementById("vault_config").hidden = false;
			} else if (data["message"] == "vault_not_configured") {
				fail_popup("Vault not configured");
				document.getElementById("vault_view").hidden = true;
				document.getElementById("vault_config").hidden = false;
			} else {
				draw_vault_file_structure(vault_path, data["fs"]);
				vault_file_system = data["fs"];
			}
		});
}

function open_vault_file(filename, path, button = null) {
	if (button != null) {
		button.innerHTML += `<img src="small_loading_white.svg" class="loader">`;
	}
	if (filename.slice(-1) === "/") {
		draw_vault_file_structure(path_join([path, filename]), vault_file_system);
		vault_path = path_join([path, filename]);
	} else {
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
}

function open_vault_context(filename, button = null) {
	document.getElementById("vault_context_menu").hidden = false;
	document.getElementById("vault_context_menu").style.left = mouseX + 20 + "px";
	document.getElementById("vault_context_menu").style.top = mouseY + 10 + "px";
	vault_context_file = path_join([vault_path, filename]);
	vault_context_button = button;
}

function delete_vault_file() {
	const file_for_delete = vault_context_file;
	vault_context_button.style.boxShadow = "0px 0px 0px 2px red inset";
	confirm_modal_warning(
		"Delete",
		"Do you want to proceed?",
		() => {
			vault_context_button.remove();
			vault_context_button = null;
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "delete_vault_file",
					key: atob(sessionStorage.getItem("vault_key")),
					deletePath: file_for_delete,
				}),
			})
				.then((res) => {
					if (!res.ok) {
						fail_popup("Vault connection failed");
					}
					return res.json();
				})
				.then((data) => {
					if (data["message"] == "auth_failed") {
						fail_popup(
							"Permission error <button onclick='location.reload()'>Reload</button>",
						);
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else if (data["message"] == "vault_not_configured") {
						fail_popup("Vault not configured");
						document.getElementById("vault_view").hidden = true;
						document.getElementById("vault_config").hidden = false;
					} else {
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
								return res.json();
							})
							.then((data) => {
								draw_vault_file_structure(vault_path, data["fs"]);
							});
					}
				});
		},
		() => {
			vault_context_button.style.boxShadow = "";
		},
		true,
	);
}

function rename_vault_file() {}

function close_vault_files() {
	document.getElementById("vault_view").hidden = true;
	document.getElementById("vault_config").hidden = false;
	sessionStorage.removeItem("vault_key");
	vault_path = "/";
}

function vault_file_upload(button) {
	input_modal(
		"Upload file",
		"Pick a file to upload",
		"vault_file_upload_input",
		"file",
		() => {
			var file_for_upload = document.getElementById("vault_file_upload_input")
				.files[0];
			if (file_for_upload.size >= 1024 * 1024 * 2024 * 2) {
				confirm_modal_warning(
					"File to large",
					"The file you are trying to upload is larger than 2GB. This can not be uploaded.",
					() => {},
					() => {},
					false,
				);
				return;
			}
			var file_name = document.getElementById("vault_file_upload_input").value;
			if (file_for_upload) {
				var form_data = new FormData();
				form_data.append("file", file_for_upload);
				form_data.append("path", vault_path);
				form_data.append("key", atob(sessionStorage.getItem("vault_key")));
				console.log(button);
				attach_loader(button.id, "white");
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
							confirm_modal_warning("Upload error", "Upload failed");
							remove_loader(button.id);
						} else {
							confirm_modal(
								"Upload success",
								"The file was uploaded to vault",
								() => {},
								() => {},
								false,
							);
							document.getElementById("vault_files").innerHTML +=
								"<button class='fileButtons' style='opacity: 0.5;'>" +
								file_name.split("\\")[file_name.split("\\").length - 1] +
								"</button>";
							reload_file_structure(true, button);
						}
					});
			}
		},
		() => {},
	);
}

function vault_new_folder() {
	input_modal(
		"New folder",
		"Name the folder",
		"folder_name",
		"text",
		() => {
			fetch("/api", {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
				},
				body: JSON.stringify({
					r: "vault_new_folder",
					key: atob(sessionStorage.getItem("vault_key")),
					folder_name: path_join([
						vault_path,
						document.getElementById("folder_name").value,
					]),
				}),
			})
				.then((res) => {
					return res.json();
				})
				.then((json) => {
					reload_file_structure(false);
				});
		},
		() => {},
		true,
	);
}

function vault_walk_up() {
	vault_path = vault_path.replace(/\/$/, ""); // Remove trailing slash if present
	var lastIndex = vault_path.lastIndexOf("/");
	if (lastIndex === -1) {
		// Handle case when there's no parent (e.g., root path)
		vault_path = "/";
	} else {
		vault_path = vault_path.substring(0, lastIndex + 1); // Include the trailing slash
	}
	draw_vault_file_structure(vault_path, vault_file_system);
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
				fail_popup("Failed to fetch CPU status");
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
				fail_popup("Failed to fetch RAM status");
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
				fail_popup("Failed to fetch Disk status");
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
				fail_popup("Failed to fetch disk list");
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
				fail_popup("Failed to fetch list of users");
				throw new Error("Failed to fetch list of users");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("usersTable").innerHTML = data["text"];
		});
}

function getFireWallInformation() {
	fetch("/api", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({
			r: "fireWallInformation",
		}),
	})
		.then((res) => {
			if (!res.ok) {
				fail_popup("Failed to get firewall information");
				throw new Error("Failed to fetch list of users");
			}
			return res.json();
		})
		.then((data) => {
			document.getElementById("enableUFW").checked = data["enabled"];

			var rules = data["rules"];
			var rulesTable =
				"<table><tr><td>To</td><td>Action</td><td>From</td></tr>";
			rules = rules.sort((rule, rule2) => {
				if (Number(rule["to"]) == "NaN") {
					return -1;
				}
				if (Number(rule["to"]) > Number(rule2["to"])) {
					return +1;
				}
			});
			for (rule of rules) {
				rulesTable += `<tr>
					<td id="fireWallRule${rule["index"]}"
					onmouseover="showFireWallRuleOptions('fireWallRule${rule["index"]}', ${rule["index"]})"
					onmouseleave="hideFireWallRuleOptions('fireWallRule${rule["index"]}')"
					>${rule["to"].replaceAll("(v6)", `<span class="fireWallTag v6">v6</span>`)}
					</td>
					<td>${rule["action"].replaceAll("ALLOW", `<span class="fireWallTag allow">Allow</span>`).replaceAll("DENY", `<span class="fireWallTag deny">Deny</span>`)}</td>
					<td>${rule["from"].replaceAll("(v6)", `<span class="fireWallTag v6">v6</span>`)}</td></tr>`;
			}
			rulesTable += "</table>";

			document.getElementById("fireWallRuleOverview").innerHTML = rulesTable;
		});
}

function showFireWallRuleOptions(ruleId, index) {
	const ruleTd = document.getElementById(ruleId);
	if (!ruleTd.querySelector("button")) {
		ruleTd.innerHTML = ruleTd.innerHTML +
		`
		<button onclick="deleteFireWallRule(${index})" class="fireWallRuleButton"><img src="delete.png"></button>
		`
	}
}

function hideFireWallRuleOptions(ruleId, index) {
	const ruleTd = document.getElementById(ruleId);
	ruleTd.querySelectorAll("button").forEach((button) => {
		button.remove()
	})
}

function deleteFireWallRule(index) {
	confirm_modal_warning("Delete rule", `You are deleting firewall rule ${index}.`, () => {}, () => {}, true)
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
					fail_popup("Failed to delete user");
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
				fail_popup("Failed to submit new user");
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
		button.style.backgroundColor = "#00000000";
	}

	document.activeElement.style.backgroundColor = "var(--background-semi-light)";
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
				fail_popup("Failed to fetch list of files");
				throw new Error("Failed to fetch list of files");
			}
			return res.json();
		})
		.then((data) => {
			if (data["message"] == "no_permissions") {
				fail_popup("Missing permission to open this file");
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

function uploadFileSystem() {
	input_modal(
		"Upload file",
		"Select a file to upload",
		"fileSystemUpload",
		"file",
		() => {
			var fileForUpload = document.getElementById("fileSystemUpload").files[0];
			var formData = new FormData();
			formData.append("file", fileForUpload);
			formData.set("filePath", currFPath);
			fetch("/upload/fs", {
				method: "POST",
				headers: {},
				body: formData,
			}).then(() => {
				renderFiles(currFPath);
			});
		},
		() => {},
		true,
	);
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
				fail_popup("Failed to fetch drive information");
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
				fail_popup("Failed to fetch package list");
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
					if (e[1].length > 18) {
						var appExec = e[1].substring(0, 9) + "...";
					} else {
						var appExec = e[1];
					}
					if (e[0].length > 14) {
						var appName =
							e[0].split(".")[0].replace("-", " ").substring(0, 9) + "...";
					} else {
						var appName = e[0].split(".")[0].replace("-", " ");
					}
					var htmlCode =
						htmlCode +
						"<div class='package'>" +
						appName +
						` (${appExec})` +
						"<button class='remove_package' onclick='removePackage(\"" +
						e[1] +
						"\", this)'>Remove</button></div>";
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
	confirm_modal(
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
						fail_popup("Failed to remove package");
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
	confirm_modal(
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
						fail_popup("Failed to install package");
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

	root_input_modal(
		"Elevated privileges",
		"Please enter your root password to change these settings",
		"sudoPasswordFTP",
		"password",
		function () {
			attach_loader("ftpSettingsApply", "white");
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
							fail_popup("Failed to update FTP configuration");
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
					remove_loader("ftpSettingsApply");
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
				fail_popup("Can not fetch FTP configuration information");
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
				fail_popup("Can not fetch device information");
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
				data["temperature"] != null ? data["temperature"] : `No temperature`;
			document.title = "Zentrox (" + data["hostname"].split(".")[0] + ")";
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
				fail_popup("Can not fetch device information");
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
	confirm_modal(
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
						fail_popup("Can not power off");
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

function attach_loader(id, color = "black") {
	var object = document.getElementById(id);
	if (object.tagName.toLowerCase() == "button") {
		object.innerHTML += `<img src="small_loading_${color}.svg" class="loader">`;
	} else if (object.tagName.toLowerCase() == "input") {
		object.style.backgroundImage = `url("small_loading_${color}.svg")`;
	}
}

function remove_loader(id) {
	var object = document.getElementById(id);
	if (object.tagName.toLowerCase() == "input") {
		object.style.backgroundImage = `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1em' height='1em' viewBox='0 0 24 24'%3E%3Cpath fill='black' d='m9.55 18l-5.7-5.7l1.425-1.425L9.55 15.15l9.175-9.175L20.15 7.4z'/%3E%3C/svg%3E")`;
	} else if (object.tagName.toLowerCase() == "button") {
		if (object.querySelector("img:nth-child(2)")) {
			object.querySelector("img:nth-child(2)").remove();
			return;
		} else {
			object.querySelector("img:nth-child(1)").remove();
		}
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
    background-color: #22222F; /* Brighter semi-dark background with a blue tint */
    outline: #333340 solid 1px; /* Brighter semi-light background with a blue tint */
    color: #ffffff; /* Keeping white for maximum contrast */
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
    outline: rgba(255, 58, 58, 0.478) solid 1px; /* Using --red color */
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
    background-color: #1f9af7; /* Using --accent-color */
    color: #ffffff; /* Keeping white for maximum contrast */
    font-family: "Work Sans", sans-serif;
}

#modalMain button.red {
    padding: 10px;
    border-radius: 5px;
    border-width: 0px;
    background-color: #ff3a3a; /* Using --red color */
    color: #ffffff; /* Keeping white for maximum contrast */
    font-family: "Work Sans", sans-serif;
}

#modalMain button.grey {
    padding: 10px;
    border-radius: 5px;
    border-width: 0px;
    background-color: #6b7077; /* Brighter semi-light foreground with a blue tint */
    color: #ffffff; /* Keeping white for maximum contrast */
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

#fail_popup {
    position: fixed;
    left: 20px;
    bottom: 20px;
    padding: 10px;
    border-radius: 5px;
    background-color: #333340; /* Brighter semi-light background with a blue tint */
    color: #ffffff; /* Keeping white for maximum contrast */
    border: solid 1px #7d828a; /* Brighter light foreground with a blue tint */
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
        <div id='fail_popup' hidden>
        </div>
		<div id='status_popup' hidden></div>
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
	document.getElementById("buttonConfirm").classList.remove("red");
	setTimeout(function () {
		document.getElementById("modalMain").hidden = true;
	}, 510);
	flyOut("modalMain", 500);
}

function confirm_modal(title, message, cb, cancle, show_cancle = true) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML = message;
	document.getElementById("buttonConfirm").onclick = () => {
		killModalPopup();
		setTimeout(cb, 600);
		document.getElementById("buttonCancle").hidden = false;
	};
	document.getElementById("buttonCancle").hidden = !show_cancle;
	document.getElementById("buttonCancle").onclick = () => {
		killModalPopup();
		cancle();
		document.getElementById("buttonCancle").hidden = false;
	};
}

function confirm_modal_warning(title, message, cb, cancle, show_cancle = true) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML = message;
	document.getElementById("buttonConfirm").classList.add("red");
	document.getElementById("buttonConfirm").onclick = () => {
		killModalPopup();
		setTimeout(cb, 600);
		document.getElementById("buttonCancle").hidden = false;
		document.getElementById("buttonConfirm").classList.remove("red");
	};
	document.getElementById("buttonCancle").hidden = !show_cancle;
	document.getElementById("buttonCancle").onclick = () => {
		killModalPopup();
		cancle();
		document.getElementById("buttonCancle").hidden = false;
		document.getElementById("buttonConfirm").classList.remove("red");
	};
}

function input_modal(
	title,
	message,
	input_id,
	input_type,
	cb,
	cancle,
	show_cancle = true,
) {
	document.getElementById("modalMain").hidden = false;
	document.getElementById("modalTitle").innerHTML = title;
	document.getElementById("modalMessage").innerHTML =
		message + `<br><input type=${input_type} id="${input_id}">`;
	document.getElementById("buttonConfirm").onclick = () => {
		killModalPopup();
		setTimeout(cb, 600);
		document.getElementById("buttonCancle").hidden = false;
	};
	document.getElementById("buttonCancle").hidden = !show_cancle;
	document.getElementById("buttonCancle").onclick = () => {
		killModalPopup();
		cancle();
		document.getElementById("buttonCancle").hidden = false;
	};
	document.getElementById(input_id).focus();
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

function fail_popup(message) {
	document.getElementById("fail_popup").hidden = false;
	document.getElementById("fail_popup").innerHTML = message;
	setTimeout(function () {
		fadeOut("fail_popup", 3000);
	}, 3000);
}

function status_popup(message) {
	document.getElementById("status_popup").hidden = false;
	document.getElementById("status_popup").innerHTML = "✅ " + message;
	setTimeout(function () {
		fadeOut("fail_popup", 5000);
	}, 5000);
}
