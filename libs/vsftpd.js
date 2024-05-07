const fs = require("fs");
const process = require("process");

if (process.argv[2] == "init") {
  console.log("Initial config");
  try {
    fs.appendFileSync(
      "/etc/vsftpd.conf",
      "userlist_enable=YES\nuserlist_file=/etc/vsftpd.userlist\nuserlist_deny=NO",
    );
    fs.writeFileSync("/etc/vsftpd.userlist", "ftp_zentrox");
  } catch (e) {
    process.exit(-1);
  }
} else if (process.argv[2] == "update_config") {
  let key = process.argv[3];
  let value = process.argv[4];
  let reassmbled_config_file_contents;
  let conf_file;
  let conf_file_value;
  let changed_value;

  conf_file = fs.readFileSync("/etc/vsftpd.conf").toString("ascii");
  conf_file_value = conf_file.split("\n").map((value) => {
    return value.split("=");
  });

  changed_value = 0;

  for (key_value_pair of conf_file_value) {
    if (key_value_pair[0] == key) {
      key_value_pair[1] = value;
      changed_value = 1;
    }
  }

  if (!changed_value) {
    conf_file_value.push([key, value]);
  }

  for (key_value_pair of conf_file_value) {
    if (key_value_pair[1] != null) {
      reassmbled_config_file_contents += `${key_value_pair[0]}=${key_value_pair[1]}\n`;
    } else {
      reassmbled_config_file_contents += `${key_value_pair[0]}\n`;
    }
  }

  fs.writeFileSync("/etc/vsftpd.conf", reassmbled_config_file_contents);
} else if (process.argv[2] == "update_username_list") {
  task = process.argv[3];
  userlist_file = fs.readFileSync("/etc/vsftpd.userlist").toString("ascii");
  usernames = userlist_file.split("\n");
  userlist_r = "";

  if (task == "replace") {
    old_username = process.argv[5];
    new_username = process.argv[6];
    new_usernames = usernames.map((x) => {
      if (x == old_username) {
        return new_username;
      } else {
        return x;
      }
    });
  } else if (task == "add") {
    new_username = process.argv[4];
    new_usernames = usernames;
    new_usernames.push(new_username);
  } else if (task == "remove") {
    old_username = process.argv[5];
    new_usernames = usernames.filter((x) => {
      return x != old_username;
    });
  }
  for (username of new_usernames) {
    if (username != undefined) {
      userlist_r += `${username}\n`;
    }
  }
  userlist_r = userlist_r;
  console.log(userlist_r);
} else {
  console.log("This command is not know to this program");
  process.exit(-1);
}
