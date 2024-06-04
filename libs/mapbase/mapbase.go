package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	var command = os.Args[1]
	var file = os.Args[2]
	var key = os.Args[3]
	var content, err = os.ReadFile(file);
	var content_string = string(content);
	var lines = strings.Split(content_string, "\n")
	if (command == "write" || command == "delete") {
		// var value = os.Args[4]
	}
	if err != nil {
		fmt.Printf("Failed to read database >> %s << (file: %s)", err, file)
		return
	}
	
	if (command == "read") {
		for _, line := range lines {
			if (strings.Split(line, " | ")[0] == key) {
				fmt.Printf("%s", strings.Split(line, " | ")[1])
				return
			}
		}
	}
}
