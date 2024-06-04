package main

import (
	"fmt"
	"os"
	"strings"
	"encoding/base64"
)

func EncodeBase64(input string) string {
	encoded := base64.StdEncoding.EncodeToString([]byte(input))
	return encoded
}

func DecodeBase64(encoded string) (string) {
	decodedBytes, err := base64.StdEncoding.DecodeString(encoded)
	if err != nil {
		return ""
	}
	return string(decodedBytes)
}

const (
	StdPadding rune = '=' // Standard padding character
	NoPadding  rune = -1  // No padding
)

func main() {
	var command = os.Args[1]
	var file = os.Args[2]
	var key = os.Args[3]
	var content, err = os.ReadFile(file);
	var content_string = string(content);
	var lines = strings.Split(content_string, "\n")
	if err != nil {
		fmt.Printf("Failed to read database >> %s << (file: %s)", err, file)
		return
	}
	
	if (command == "read") {
		for _, line := range lines {
			if (strings.Split(line, " | ")[0] == key) {
				fmt.Printf("%s", DecodeBase64(strings.Split(line, " | ")[1]))
				return
			}
		}
	} else if (command == "write") {
		var write_done = false
		if (len(os.Args) < 5) {
			fmt.Printf("Not enough arguments")
			return
		}
		var database_out string = ""
		for _, line := range lines {
			if (strings.Split(line, " | ")[0] == key) {
				database_out += key + " | " + EncodeBase64(os.Args[4]) + "\n"
				write_done = true
			} else {
				database_out += line+"\n"
			}
		}
		if (!write_done) {
			database_out += key + " | " + EncodeBase64(os.Args[4]) + "\n"
		}
		database_out = strings.ReplaceAll(database_out, "\n\n", "\n")
		os.WriteFile(file, []byte(database_out), 0667)
	} else if (command == "delete") {
		var database_out string = ""
		for _, line := range lines {
			if (strings.Split(line, " | ")[0] != key) {
				database_out += line+"\n"
			}
		}
		database_out = strings.ReplaceAll(database_out, "\n\n", "\n")
		os.WriteFile(file, []byte(database_out), 0667)
	} else {
		fmt.Print("Unknow command\n")
	}
}
