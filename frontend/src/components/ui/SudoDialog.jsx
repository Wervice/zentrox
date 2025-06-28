import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogClose,
  DialogDescription,
} from "@/components/ui/dialog";

import { Button } from "@/components/ui/button";
import { LockIcon, LoaderCircle } from "lucide-react";
import { useState, useRef } from "react";
import { Input } from "./input";
import { fetchURLPrefix } from "@/lib/fetchPrefix";

export default function SudoDialog({
  onFinish = (_) => {},
  modalOpen,
  onOpenChange = (_) => {},
  note = "",
}) {
  const [currentlyConfirming, setCurrentlyConfirming] = useState(false);
  const [wrongPassword, setWrongPassword] = useState(false);
  var input = useRef();

  function detectedWrongPassword() {
    input.current.value = "";
    setWrongPassword(true);
  }

  function tryPassword() {
    setCurrentlyConfirming(true);
    const pw = input.current.value;
    fetch(fetchURLPrefix + "/api/verifySudoPassword", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        sudoPassword: pw,
      }),
    }).then((res) => {
      setCurrentlyConfirming(false);
      if (res.ok) {
        onOpenChange(false);
        onFinish(pw);
      } else {
        detectedWrongPassword();
      }
    });
  }

  return (
    <Dialog open={modalOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Sudo password required</DialogTitle>
          <DialogDescription>
            Zentrox requires special permissions to perform this actions. Please
            enter the sudo password for the current active user.
            <br />
            {note}
          </DialogDescription>
        </DialogHeader>
        <p>
          <Input
            type="password"
            className="w-full"
            placeholder="Sudo password"
            onKeyPress={(e) => {
              if (e.key === "Enter") {
                tryPassword();
              }
            }}
            ref={input}
          />
          <span hidden={!wrongPassword} className="text-red-500 py-1">
            Wrong sudo password
          </span>
        </p>
        <DialogFooter>
          <DialogClose asChild onClick={() => onOpenChange(false)}>
            <Button variant="outline">Cancle</Button>
          </DialogClose>
          <Button onClick={tryPassword}>
            {currentlyConfirming ? (
              <LoaderCircle className="animate-spin duration-400 h-4 w-4 mr-1" />
            ) : (
              <LockIcon className="h-4 w-4 mr-1" />
            )}{" "}
            Confirm
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
