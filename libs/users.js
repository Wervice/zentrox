const fs = require("fs");
const process = require("process");
const os = require("os");

const shadow_file = "/home/constantin/shadow.txt";
const passwd_file = "/home/constantin/passwd.txt";
const group_file = "/home/constantin/group.txt";

if (os.userInfo().username != "root") { 
  console.log("Error: This tool can only be used by root"); 
  process.exit(-2);
}

function change_username() {
  const old_username = process.argv[4];
  const new_username = process.argv[5];

  const shadow_file_content = fs.readFileSync(shadow_file).toString("ascii");
  const passwd_file_content = fs.readFileSync(passwd_file).toString("ascii");
  const group_file_content = fs.readFileSync(group_file).toString("ascii");

  let shadow_file_content_mod = "";
  let passwd_file_content_mod = "";
  let group_file_content_mod = "";
  let line_new = "";
  let username, password, lastchanged, min, max, warn, inact, expire, uid, gid, gecos, homedir, shell, group_name, group_list;

  // Process shadow file
  for (line of shadow_file_content.split("\n")) {
    if (line.split(":")[0] == old_username) {
      [username, password, lastchanged, min, max, warn, inact, expire] = line.split(":");
      line_new = `${new_username}:${password}:${lastchanged}:${min}:${max}:${warn}:${inact}:${expire}:\n`;
      shadow_file_content_mod += line_new;
    } else {
      shadow_file_content_mod += `${line}\n`;
    }
  }
  
  // Process passwd file
  for (line of passwd_file_content.split("\n")) {
    if (line.split(":")[0] == old_username) {
      [username, password, uid, gid, gecos, homedir, shell] = line.split(":");
      line_new = `${new_username}:${password}:${uid}:${gid}:${gecos}:${homedir.replaceAll(old_username, new_username)}:${shell}\n`;
      passwd_file_content_mod += line_new;
    } else {
      passwd_file_content_mod += `${line}\n`;
    }
  }
 
  // Process group file
  for (line of group_file_content.split("\n")) {
    if (line.length < 1) { continue; }
    if (line.split(":")[3].split(",").includes(old_username)) {
      [group_name, password, gid, group_list] = line.split(":");
      line_new = `${group_name}:${password}:${gid}:${group_list.replaceAll(old_username, new_username)}\n`;
      group_file_content_mod += line_new;
    } else {
      group_file_content_mod += `${line}\n`;
    }
  }

  // Remove empty lines
  shadow_file_content_mod = shadow_file_content_mod.replaceAll("\n\n", "");
  passwd_file_content_mod = passwd_file_content_mod.replaceAll("\n\n", "");
  group_file_content_mod = group_file_content_mod.replaceAll("\n\n", "");
  
  console.log(shadow_file, group_file_content_mod);
}

if (process.argv[2] == "updateUser") {
  if (process.argv[3] == "username") {
    change_username();
  } else if (process.argv[4] == "password") {
    change_password();
  }
} else {
  console.log("Error: Unknow command");
  process.exit(-1);
}
