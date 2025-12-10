# UFW helper script exporting the status and rules of the UFW firewall in JSON parsable format.

import gettext
from typing import Any
import ufw.common
import ufw.frontend
from ufw.common import UFWRule
import json
import os
import sys

def to_dict(x: object) -> dict[str, Any]:
    return x.__dict__

class ConnectionLayer:
    def __init__(self):
        gettext.install(ufw.common.programName) # Required for _("") formatting to work as well as localization
        self.backend: ufw.frontend.UFWBackendIptables = ufw.frontend.UFWFrontend(False).backend

    def is_enabled(self) -> bool:
        return self.backend.is_enabled()

    def get_routes(self) -> list[UFWRule]:
        rules: list[UFWRule] = self.backend.get_rules();
        return rules

    def get_logging_state(self) -> int:
        return self.backend.get_loglevel()[0] // 100
 
    def get_defaults(self):
        return self.backend.defaults

class JsonExporter:
    def __init__(self):
        self.connection: ConnectionLayer = ConnectionLayer()

    def export(self) -> str:
        rules = self.connection.get_routes()
        rules_mapped = list(map(to_dict, rules))
        data = {
            "enabled": self.connection.is_enabled(),
            "logging": self.connection.get_logging_state(),
            "defaults": self.connection.get_defaults(),
            "rules": rules_mapped
        }
        return json.dumps({
            "success": True,
            "data": data
        })

def main():
    if os.getuid() != 0:
        res = json.dumps({
            "success": False,
            "error": "Insufficient permissions",
            "errorCode": 1
        })
        print(res, sys.stderr);
        exit()

    exporter = JsonExporter()
    json_res = exporter.export()
    print(json_res)

main()
