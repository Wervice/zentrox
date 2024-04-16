const chpr = require("child_process");

function deviceList() {
  var lsblkOutput = chpr.execSync("lsblk -o NAME,MOUNTPOINT --bytes --json");
  return JSON.parse(lsblkOutput)["blockdevices"];
}

function deviceInformation(deviceName) {
  const lsblkOutput = chpr.execSync("lsblk -O --bytes --json");
  const jsonOutput = JSON.parse(lsblkOutput)["blockdevices"];
  for (e of jsonOutput) {
    if (e["name"] == deviceName) {
      return e;
    }
    if (e["children"] != undefined) {
      for (child of e["children"]) {
        if (child["name"] == deviceName) {
          return child;
        }
      }
    }
  }
  return undefined;
}
