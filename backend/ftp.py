from pyftpdlib.authorizers import AuthenticationFailed, DummyAuthorizer
from pyftpdlib.handlers import TLS_FTPHandler
from pyftpdlib.servers import FTPServer

import sqlite3
from hashlib import sha512
import sys
import os

if len(sys.argv) < 2:
    print("FTP: Missing username argument.")
    exit()

home_path = "/home/" + sys.argv[1]
data_path = os.path.join(home_path, ".local", "share", "zentrox")
database_fp = os.path.join(data_path, "database.db")

if sys.argv[1] == "root":
    home_path = "/root"


class SHA512Authorizer(DummyAuthorizer):
    def validate_authentication(self, username: str, password: str, handler):
        if sys.version_info >= (3, 0):
            hash = sha512(password.encode("utf8"))

        hex_hash: str = hash.hexdigest()

        try:
            if self.user_table[username]["pwd"] != hex_hash:
                raise KeyError
        except KeyError:
            raise AuthenticationFailed


def main():
    connection = sqlite3.connect(database_fp)
    cursor = connection.cursor()

    _x0 = cursor.execute(
        "UPDATE Ftp SET pid = {} WHERE key = 0".format(os.getpid()), ()
    )
    _x1 = cursor.execute("UPDATE Ftp SET running = 1 WHERE key = 0", ())
    connection.commit()

    authorizer = SHA512Authorizer()

    username: str = cursor.execute("SELECT username FROM Ftp WHERE key = 0").fetchall()[
        0
    ][0]
    local_root: str = cursor.execute(
        "SELECT local_root FROM Ftp WHERE key = 0"
    ).fetchall()[0][0]
    password: str = cursor.execute(
        "SELECT value FROM Secrets WHERE name = 'ftp_password'"
    ).fetchall()[0][0]
    certificate_name: str = cursor.execute("SELECT value FROM Settings WHERE name = 'tls_cert'").fetchall()[0][0]

    _ = authorizer.add_user(username, password, local_root, "elradfmwMT")
    # msg_login="Welcome to Zentrox FTP Share powered by pyftpdlib."
    handler = TLS_FTPHandler
    handler.passive_ports = list(range(60000, 60500))
    handler.certfile = os.path.join(
        data_path,
        "certificates",
        certificate_name,
    )
    handler.authorizer = authorizer

    server = FTPServer(("::0.0.0.0", 21), handler)
    server.serve_forever()


main()
