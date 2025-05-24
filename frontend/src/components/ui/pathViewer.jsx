const { ArrowUp, HouseIcon, PenIcon } = require("lucide-react");
import { useRef } from "react";
import {
  Dialog,
  DialogClose,
  DialogFooter,
  DialogTitle,
  DialogHeader,
  DialogTrigger,
  DialogContent,
  DialogDescription,
} from "./dialog";
import { Input } from "./input";
import { Button } from "./button";
import { CopyIcon } from "lucide-react";
import { toast } from "./use-toast";
/**
 * @param {Object} param0
 * @param {string} param0.className - Additional class names
 * @param {string} param0.value - The value of the display
 * @param {string} param0.home - A default home value for the user to return to
 * @param {boolean} param0.hidden - Is the component visible
 * PathViewer shows the current directory path for file systems. It supports changing the path, viewing and copying it.*/
function PathViewer({
  className,
  onValueChange,
  value,
  home,
  hidden,
  children,
}) {
  var specificLocationInput = useRef();

  function goUp() {
    let path = value;
    let segments = path.split("/");
    segments.pop();
    segments.pop();
    let newPath = segments.join("/");
    onValueChange(newPath + "/");
  }

  function goHome() {
    onValueChange(home);
  }

  return (
    <span
      className={
        "flex items-center p-1 w-full whitespace-nowrap overflow-hidden max-w-full " +
        className +
        (hidden ? " hidden" : "")
      }
    >
      {children || <></>}
      <span title="Navigate to home directory">
        <HouseIcon
          className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-2"
          onClick={goHome}
        />
      </span>
      <span title="Copy current path">
        <CopyIcon
          className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-2"
          onClick={() => {
            navigator.clipboard.writeText(value);
            toast({
              title: "Copied path",
              description: "The current path was copied to your clipboard",
            });
          }}
        />
      </span>
      <Dialog>
        <DialogTrigger asChild>
          <span title="Navigate to specific location">
            <PenIcon className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1" />
          </span>
        </DialogTrigger>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Specific location</DialogTitle>
            <DialogDescription>Move to a specific location</DialogDescription>
          </DialogHeader>
          <p>
            <Input
              type="text"
              ref={specificLocationInput}
              defaultValue={value}
              className="w-full block"
            />
          </p>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Close</Button>
            </DialogClose>
            <DialogClose asChild>
              <Button
                onClick={() => {
                  let newPath = specificLocationInput.current.value;
                  if (newPath.endsWith("/")) {
                    onValueChange(newPath);
                  } else {
                    onValueChange(newPath + "/");
                  }
                }}
              >
                Confirm
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <span title="Traverse directory up">
        <ArrowUp
          className="w-5 h-5 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1"
          onClick={goUp}
        />
      </span>
      {value}{" "}
    </span>
  );
}

export default PathViewer;
