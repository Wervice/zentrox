from pyftpdlib.authorizers import AuthenticationFailed, DummyAuthorizer
from pyftpdlib.handlers import TLS_FTPHandler
from pyftpdlib.servers import FTPServer

from hashlib import sha512
import sys
import os

home_path = "/home/"+sys.argv[1]
data_path = os.path.join(home_path, ".local", "share", "zentrox")
config_file_path = os.path.join(data_path, "zentrox_store.toml")

if sys.argv[1] == "root":
    home_path = "/root"


# Parsing Toml file to a HashMap (or dict)
def parse_toml_to_dict(string: str) -> dict:
    table = {}
    for line in string.split("\n"):
        if len(line) > 0:
            split_line = line.split(" = ")
            table[split_line[0]] = split_line[1].replace("\"", "")
    return table


# Parsing a toml dict to a string
def parse_table_to_string(table: dict) -> str:
    string = ""
    table_keys = table.keys()
    for key in table_keys:
        try:
            value = int(table[key])
        except ValueError:
            value = "\"" + str(table[key]) + "\""
        string += key + " = " + str(value) + "\n"
    return string


# Optain a value from the config file
def read_config_file(key) -> str:
    with open(config_file_path, "r") as toml_file:
        toml_file_contents = toml_file.read()
        parsed_toml = parse_toml_to_dict(toml_file_contents)
        if key in parsed_toml.keys():
            return parsed_toml[key]
        else:
            return ""


# Change a value in the config file
def write_config_file(key, value) -> bool:
    with open(config_file_path, "r") as toml_file:
        toml_file_contents = toml_file.read()
        parsed_toml_file = parse_toml_to_dict(toml_file_contents)
        parsed_toml_file[key] = value
    with open(config_file_path, "w") as toml_file:
        toml_file.write(parse_table_to_string(parsed_toml_file))
    return True


class DummySHA512Authorizer(DummyAuthorizer):
    def validate_authentication(self, username, password, handler):
        if sys.version_info >= (3, 0):
            password = sha512(password.encode('latin1'))

        hash = password.hexdigest()

        try:
            if self.user_table[username]['pwd'] != hash:
                raise KeyError
        except KeyError:
            raise AuthenticationFailed


def main():
    write_config_file("ftp_pid", os.getpid())
    write_config_file("ftp_running", "1")
    authorizer = DummySHA512Authorizer()

    username = read_config_file("ftp_username")
    password = read_config_file("ftp_password")
    local_root = read_config_file("ftp_local_root")

    authorizer.add_user(username, password, local_root, "elradfmwMT")
    handler = TLS_FTPHandler
    handler.certfile = os.path.join(data_path, "certificates", read_config_file("tls_cert"))
    handler.authorizer = authorizer

    server = FTPServer(('::0.0.0.0', 21), handler)
    server.serve_forever()


try:
    main()
except OSError as error:
    print("❌ 🐍 FTP: OS Error: "+str(error))
    print("❌ 🐍 FTP: Most likely due to FTP port being blocked")
    write_config_file("ftp_running", "0")
    write_config_file("ftp_pid", "0")
    exit()
except Exception as error:
    print("❌ 🐍 FTP: General Error")
    print("❌ 🐍 FTP: ", error)
    write_config_file("ftp_running", "0")
    write_config_file("ftp_pid", "")
    exit()
