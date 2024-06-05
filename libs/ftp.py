from pyftpdlib.authorizers import AuthenticationFailed, DummyAuthorizer
from pyftpdlib.handlers import TLS_FTPHandler
from pyftpdlib.servers import FTPServer

from hashlib import sha512
from pathlib import Path
import sys
import os
import subprocess

home_path = "/home/"+sys.argv[1]
config_file_path = os.path.join(home_path, "zentrox_data", "ftp.txt")

if sys.argv[1] == "root":
    home_path = "/root"

def readDatabase(key):
    return subprocess.check_output(["./libs/mapbase/mapbase", "read", os.path.join(home_path, "zentrox_data/config.db"), key]).decode()
def writeDatabase(key, value):
    value = str(value)
    return subprocess.check_output(["./libs/mapbase/mapbase", "write", os.path.join(home_path, "zentrox_data/config.db"), key, value]).decode()

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
    writeDatabase("ftp_pid", os.getpid())
    authorizer = DummySHA512Authorizer()    
    authorizer.add_user(readDatabase("ftp_username"), readDatabase("ftp_password"), readDatabase("ftp_root"), "elradfmwMT") 
    handler = TLS_FTPHandler
    handler.certfile = os.path.join(home_path, "zentrox", "selfsigned.pem")
    handler.authorizer = authorizer
    server = FTPServer(('', 21), handler)
    server.serve_forever()

main()
