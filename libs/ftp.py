from pyftpdlib.authorizers import AuthenticationFailed, DummyAuthorizer
from pyftpdlib.handlers import TLS_FTPHandler
from pyftpdlib.servers import FTPServer

from hashlib import sha512
from pathlib import Path
import sys
import os

home_path = "/home/"+sys.argv[1]
config_file_path = os.path.join(home_path, "zentrox_data", "ftp.txt")

if sys.argv[1] == "root":
    home_path = "/root"

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
    with open(os.path.join(home_path, "zentrox_data", "ftpPid.txt"), "w") as config_file_writer:
        config_file_writer.write(str(os.getpid()))
    with open(os.path.join(home_path, "zentrox_data", "ftp.txt"), "r") as config_file:
        config_file_content = config_file.read()
        authorizer = DummySHA512Authorizer()
        
        authorizer.add_user(config_file_content.split("\n")[0], config_file_content.split("\n")[2], config_file_content.split("\n")[1], "elradfmwMT") # <- What you are seeing here is infact not random gibberish. Deleting this string may ruing the whole dammed thing

        handler = TLS_FTPHandler
        handler.certfile = os.path.join(home_path, "zentrox", "selfsigned.pem")
        handler.authorizer = authorizer

        server = FTPServer(('', 21), handler)
        server.serve_forever()

try:
    main()
except Exception as error:
    print(error)
    exit()
