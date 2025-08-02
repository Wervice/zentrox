const { ArrowUp, HouseIcon, PenIcon } = require("lucide-react");
import { useRef, useState } from "react";
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
import { CopyIcon, XIcon } from "lucide-react";
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
  onFilter,
  value,
  home,
  hidden,
  children,
}) {
  var specificLocationInput = useRef();
  var fileSearchInput = useRef();
  const [pathChangeOpen, setPathChangeOpen] = useState(false);

  function goUp() {
    let path = value;
    let segments = path.split("/");
    segments.pop();
    segments.pop();
    let newPath = segments.join("/");
    onFilter("");
    onValueChange(newPath + "/");
  }

  function goHome() {
    onValueChange(home);
  }

  const changePath = () => {
    setPathChangeOpen(false);
    let newPath = specificLocationInput.current.value;
    if (newPath.endsWith("/")) {
      onValueChange(newPath);
    } else {
      onValueChange(newPath + "/");
    }
  };

  return (
    <span
      className={
        "flex items-end p-1 w-full whitespace-nowrap overflow-hidden max-w-full " +
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
      <Dialog onOpenChange={setPathChangeOpen} open={pathChangeOpen}>
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
              placeholder="Absolute target path"
              onKeyPress={(ev) => {
                if (ev.key === "Enter") {
                  changePath();
                }
              }}
            />
          </p>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Close</Button>
            </DialogClose>
            <DialogClose asChild>
              <Button onClick={changePath}>Confirm</Button>
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
      {value}
      <span className="flex items-center ml-auto">
        <Input
          className="mr-2 mt-0 h-8"
          placeholder="Search for file or folder"
          ref={fileSearchInput}
          onKeyUp={() => {
            onFilter(fileSearchInput.current.value);
          }}
        />{" "}
        <XIcon
          className="mr-2 w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle"
          onClick={() => {
            onFilter("");
            fileSearchInput.current.value = "";
          }}
        />
      </span>
    </span>
  );
}

export default PathViewer;
