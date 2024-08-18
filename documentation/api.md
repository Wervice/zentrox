# API Documentaiton

## METHOD | URL [PARAMETERS]
Ussage

``` json
Return value
```

## Get | / **Obsolete**
This page is automatically servered from frontend/out
like a static asset
``` html
<!-- HTML Code -->
```

## Post | /login
This post request checks if a user is in the database and can
be authenticated.
1. Get caller IP
2. Store IP in a public hashmap and increment a counter for this IP by 1 everytime the user calls the API. <- Brute force protection
3. Add a time for the last call to the API.
If IP calls > 10 and (current time - last time) < 20000 ms, return;
Else if (current time - last call time) > 20000 ms, reset counter to 0
4. Increment call counter
5. If no password or no username return;
6. If password.len > 1024 return
7. If username.len < 512 return
8. Look in [database](database.md) for key useOtp === 1, then retrieve
    otpSecret key.
9. If otpSecret !== user req.json return
10. Auth username and password
    a) Check if user exists in zentrox_path/users.txt
        base64(username): base64(password)
11. If the auth passes set
    isAdmin: true
    zentroxPassword (used for sys admin auth): decryptAes(database.get(zentrox_user_password), password)
12. Send HTTP 200 "{}" or for failed: HTTP 403 {message: "Wrong password or username"}

## Post | /login/otpSecret
If the user has not yet viewed their otp secret key and useOtp is === 1, the user is provided with the key.
{
    secret: database.get(otpSecret) 
}

## Get | /login/useOtp
if useOtp === 1
return: 
{
    used: true
}
else {
    used: false
}

## Get | /logout
session values:
    signedIn false
    isAdmoin false
    zentroxPassword ""
redirect /

## Get | /dashboard
renders dashboard.html from frontend/out

## The Actual API
### Get | /api/cpuPercent
returns current cpu ussage in percent as a u32
{
    p: i32
}

### Get | /api/ramPercent
returns the current memory ussage in percent as i32
{
    p: i32 (os.totalmem - os.freemem) / os.totalmem
}


### Get | /api/diskPercent
returns the current percentage of free space on the default fs storage medium as an i32
Calculation: Block Size * Block Number - Block Size * Free Block Number  /  Block Size * Block Number
{
    p: i32
}

### Get | /api/driveList
Returns an array of every drive (storage medium) on the device.
1. `lsblk -o NAME,MOUNTPOINT --bytes --json`
2. Parse to json
Looks like:
{
    array {name: "name", mountpoint: "mountpoint"},
    array {name: "name2", mountpoint: "mountpoint", array children}
}

### Get | /api/callFile/ [:file]
file is encoded as **URI component**
returns the binary contents of a file.
> Headers: attachment; filename=filename
If the file is not found, it returns 500

### Get | /api/deleteUser/ [:username]
Deletes user
Therefore it removes the line for the user from users.txt
base64(username): password...

### Get | /api/userList
#### Old
Constructs HTML code from the users.txt file
{
    text: userTable
}
#### New
Sends users.txt as a parsed JSON

### Get | /api/fileList/ [:path]
path is encoded as **URI component**
returns an array fo all entries in the given path and a flag.
[
    [file_name, f for file, d for directory, a for admin]
]

### Get | /api/deleteFile/ [:path]
path is encoded as **URI component**
deletes file from path
returns 200
or 500

### Get | /api/renameFile [:oldPath, newName]
oldPath: Path to the file
newName: new *Name* of the file
returns 500 on fail

### Get | /burnFile/ [:burnFile]
**URI Component**
Overwrites file with random data
Sends 500 on fail

### Get | /api/packageDatavase
Returns JSON of every package that:
    a) Apps: Has a .desktop file
    b) Packages: Is installed on the system as a package
    c) Others: Every not installed backage
{
    apps: ..., packages: ..., others: ...
}
returns 500 on fail

### Get | /api/packageDatabaseAutoremove
Lists every package that can be removed from the system.

### Get | /api/clearAutoRemove
Runs Autoremove as admin
returns 400 on fail **Should be 500**

returns
{
    packages: packagesForAutoRemove
}

### Get | /api/removePackage [:packageName]
packageName is **URI encoded**
Removes the package from the system

### Get | /api/installPackage [:packageName]
packageName is **URI encoded**
Installs the package on the system

### Post | /api/updateFTPConfig
if !enableFTP
    if ftp_may_be_killed === 1
        var killDelay = 0ms
    else
        var killDely = 3000ms

    remove ftp_pid from database
    sudo kill ftpServerPid

    set ftp_running 0 in database
else
Start server
    ```
const ftpProcess = new Shell(
				"zentrox",
				"sh",
				zentroxUserPassword,
				(data) => {
					writeDatabase(
						path.join(zentroxInstallationPath, "config.db"),
						"ftp_running",
						"0",
					);
					writeDatabase(
						path.join(zentroxInstallationPath, "config.db"),
						"ftp_pid",
						"",
					);
					zlog(`FTP server exited with return of: \n${data}`);
				},
			);
			ftpProcess.write(`python3 ./libs/ftp.py ${os.userInfo().username} \n`);

			writeDatabase(
				path.join(zentroxInstallationPath, "config.db"),
				"ftp_running",
				"1",
			);
		}
    ```
    write database ftp_running 1

if ! enableDisable
    ftp_password to ftpUserPassword
    ftp_username to ftpUserUsername
    ftp_root to ftpLocalRoot
    ftp_running depending on enableFTP

return 200

### Get | /api/fetchFTPconfig
return
{
    enabled: ftp_running === 1
    ftpUserUsername: ftp_username
    ftpLocalRoot: ftp_root
}

### Get | /api/driveInformation [:driveName]
**URI Component**
1. Call `df -P`, trim, split \n, slice 1 (header)
2. map 1 to values of 1 to [filesystem, size, used, available, capacity, mounted] by splitting every line using /\s+/ regex syntax
send
{
    drives: deviceInformation about driveName (see drives.js)
    ussage: 2
}

### Get | /api/deviceInformation
send json with
    os_name: lsb_release -d, Remove "Description\t" and new line in the end
    power_supply: /sys/class/power_supply/BAT0/status and BAT0/capacity
    zentrox_pid: process pid
    process_number: ps -e | wc -l
    hostname: "hostname" remove new line in the end
    uptime: uptime -p replace "up " with ""
    temperature: /sys/class/thermal/thermal_zone0/temp / 1000 (Celcius)

### Get | /api/powerOff
Shutdown system
