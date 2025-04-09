import {
  BracesIcon,
  DeleteIcon,
  Flame,
  FileIcon,
  FileText,
  FolderIcon,
  Music,
  PenLineIcon,
  ShieldIcon,
  VideoIcon,
  DownloadIcon,
  Clock2,
  HouseIcon,
  PlugIcon,
  UploadIcon,
  FileTextIcon,
  TextIcon,
  BracketsIcon,
} from "lucide-react";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";

import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";

import { useEffect, useState, useRef } from "react";
import PathViewer from "@/components/ui/pathViewer";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "./use-toast";
import { Toaster } from "@/components/ui/toaster";
import {
  AlertDialog,
  AlertDialogHeader,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogTitle,
  AlertDialogFooter,
} from "@/components/ui/alert-dialog";
import useNotification from "@/lib/notificationState";
import { DialogTrigger } from "@radix-ui/react-dialog";

const fetchURLPrefix = require("@/lib/fetchPrefix");

export default function FileView({ className = "" }) {
  const [currentPath, setCurrentPath] = useState("/");
  const [files, setFiles] = useState([]);
  const [deletionPopupVisible, setDeletionPopupVisible] = useState(false);
  const [deletionFile, setDeletionFile] = useState("");
  const [renamePopupVisible, setRenamePopupVisible] = useState(false);
  const [renameFile, setRenameFile] = useState("");
  const [burnPopupVisible, setBurnPopupVisible] = useState(false);
  const [burnFile, setBurnFile] = useState("");
  const [uploadFileName, setUploadFileName] = useState(null);
  const { deleteNotification, notify, notifications } = useNotification();
  var renameFileInput = useRef();
  var uploadFileInput = useRef();

  /**
   * Fetch entries for directory
   * @param {string} [path=currentPath]
   */
  function fetchFiles(path = currentPath) {
    fetch(fetchURLPrefix + "/api/filesList/" + encodeURIComponent(path)).then(
      (res) => {
        if (res.ok) {
          res.json().then((json) => {
            if (typeof json["content"] !== "undefined") {
              setFiles(json["content"]);
              setCurrentPath(path);
            } else {
              setFiles([]);
              notify(`The path ${path} could not be accessed.`);
              toast({
                title: "Path error",
                description:
                  "The provided path does not exist or can not be accessed",
              });
            }
          });
        } else {
          setFiles([]);
          notify(`The path ${path} could not be accessed.`);
          toast({
            title: "Path error",
            description:
              "The provided path does not exist or can not be accessed",
          });
        }
      },
    );
  }

  useEffect(() => {
    fetchFiles();
  }, []);

  var entries = [];
  var entries = entries.concat(
    files
      .filter((entry) => {
        if (entry[1] === "d") return false;
        return true;
      })
      .sort((e, eB) => {
        if (e[0].split(".").pop() > eB[0].split(".").pop()) return -1;
        return 1;
      }),
  );
  var entries = entries.concat(
    files
      .filter((entry) => {
        if (entry[1] === "d") return true;
        return false;
      })
      .sort((a, b) => {
        if (a > b) return 1;
        if (b < a) return -1;
        return 0;
      }),
  );

  var viewClassName =
    "w-full p-4 bg-transparent border border-neutral-800 border-x-transparent block cursor-default select-none hover:bg-neutral-800 hover:transition-bg hover:duration-400 focus:bg-neutral-800 focus:duration-50";
  var iconViewClassName = "inline-block h-6 w-6 pr-1";

  function requestDeletion(name) {
    setDeletionPopupVisible(true);
    setDeletionFile(currentPath + name);
  }

  function requestRename(name) {
    setRenamePopupVisible(true);
    setRenameFile(currentPath + name);
  }

  function requestBurn(name) {
    setBurnPopupVisible(true);
    setBurnFile(currentPath + name);
  }

  return (
    <div
      className={`${className} no-scroll overflow-scroll`}
      onContextMenu={(e) => e.preventDefault()}
    >
      <AlertDialog
        open={deletionPopupVisible}
        onOpenChange={setDeletionPopupVisible}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete file</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to delete {deletionFile}?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const forDeletion = deletionFile;
                fetch(
                  fetchURLPrefix +
                    "/api/deleteFile/" +
                    encodeURIComponent(forDeletion),
                ).then((res) => {
                  if (res.ok) {
                    notify("Deleted " + forDeletion);
                    fetchFiles(currentPath);
                  } else {
                    notify("Failed to delete " + forDeletion);
                    toast({
                      title: "Deletion failed",
                      description: `Zentrox failed to delete ${forDeletion}`,
                    });
                  }
                });
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>{" "}
      </AlertDialog>

      <AlertDialog open={burnPopupVisible} onOpenChange={setBurnPopupVisible}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete file</AlertDialogTitle>
            <AlertDialogDescription>
              Do you really want to burn {deletionFile}?
              <br />
              This will overwrite the file with random data and then delete it.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const forBurn = burnFile;
                fetch(
                  fetchURLPrefix +
                    "/api/burnFile/" +
                    encodeURIComponent(forBurn),
                ).then((res) => {
                  setBurnFile("");
                  if (res.ok) {
                    notify("Burned " + forBurn);
                    fetchFiles(currentPath);
                  } else {
                    notify("Failed to burn " + forBurn);
                    toast({
                      title: "Burn failed",
                      description: `Zentrox failed to burn ${forBurn}`,
                    });
                  }
                });
              }}
            >
              Burn file
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>{" "}
      </AlertDialog>
      <Dialog open={renamePopupVisible} onOpenChange={setRenamePopupVisible}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename file</DialogTitle>
            <DialogDescription>
              Enter a new path to rename the file.
            </DialogDescription>
          </DialogHeader>
          <Input
            id="renameFileInput"
            ref={renameFileInput}
            defaultValue={renameFile}
            className="w-full"
            placeholder="New path"
          />
          <DialogFooter>
            <DialogClose>
              <Button variant="outline">Cancel</Button>
            </DialogClose>
            <DialogClose>
              <Button
                onClick={() => {
                  var newPath = renameFileInput.current.value;
                  if (!newPath.includes("/")) {
                    newPath =
                      currentPath +
                      (!currentPath.endsWith("/") ? "/" : "") +
                      newPath;
                  }
                  const forRename = renameFile;
                  fetch(
                    fetchURLPrefix +
                      "/api/renameFile/" +
                      encodeURIComponent(forRename) +
                      "/" +
                      encodeURIComponent(newPath),
                  ).then((res) => {
                    setRenameFile("");
                    if (res.ok) {
                      notify(`Renamed ${forRename} to ${newPath}`);
                      fetchFiles(currentPath);
                    } else {
                      notify(`Failed to rename ${forRename} to ${newPath}`);
                      toast({
                        title: "Renaming failed",
                        description: `Zentrox failed at renaming ${forRename} to ${newPath}.`,
                      });
                    }
                  });
                }}
              >
                Rename
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Toaster />
      <PathViewer
        onValueChange={(e) => {
          setCurrentPath(e);
          fetchFiles(e);
        }}
        home="/home/"
        value={currentPath}
      >
        <span title="Upload new file">
          <Dialog>
            <DialogTrigger>
              <UploadIcon className="w-4 h-4 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-2" />
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Upload new file</DialogTitle>
                <DialogDescription>
                  Upload a file with up to 8GB file size to your device.
                </DialogDescription>
              </DialogHeader>
              <p>
                <input
                  className="hidden"
                  type="file"
                  ref={uploadFileInput}
                  onChange={(e) => {
                    let filename = e.target.files[0].name;
                    setUploadFileName(filename);
                  }}
                />
                <Button onClick={() => uploadFileInput.current.click()}>
                  Select file
                </Button>{" "}
                {uploadFileName || "No selected file"}
              </p>
              <DialogFooter>
                <DialogClose asChild>
                  <Button variant="outline">Cancel</Button>
                </DialogClose>
                <DialogClose asChild>
                  <Button
                    onClick={() => {
                      if (uploadFileName == null) return;
                      var fileForSubmit = uploadFileInput.current.files[0];
                      uploadFileInput.current.value = null;
                      var formData = new FormData();
                      formData.append("file", fileForSubmit);
                      formData.append("path", currentPath);
                      notify(`Started upload of ${fileForSubmit.name}`);
                      fetch(fetchURLPrefix + "/upload/file", {
                        method: "POST",
                        body: formData,
                      }).then((e) => {
                        if (e.ok) {
                          notify(`Finished upload file ${fileForSubmit.name}`);
                          fetchFiles(currentPath);
                        } else {
                          e.text().then((t) => {
                            notify(
                              `Failed to upload file ${fileForSubmit.name}: ${t}`,
                            );
                            fetchFiles(currentPath);
                          });
                        }
                        setUploadFileName(null);
                      });
                    }}
                  >
                    Upload
                  </Button>
                </DialogClose>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </span>
      </PathViewer>
      <div
        className="rounded-xl m-2 overflow-hidden overflow-y-scroll border-2 border-neutral-800"
        style={{
          maxHeight: "calc(100vh - 200px)",
        }}
      >
        {entries.map((entry, index) => {
          if (entry[1] === "d") {
            if (entry[0] === "home" && currentPath === "/") {
              var icon = (
                <HouseIcon className={iconViewClassName + " text-red-500"} />
              );
            } else if (entry[0] === "dev" && currentPath === "/") {
              var icon = (
                <PlugIcon className={iconViewClassName + " text-red-500"} />
              );
            } else {
              var icon = (
                <FolderIcon
                  className={iconViewClassName + " text-yellow-500"}
                  fill="#eab308"
                />
              );
            }
          } else if (entry[1] === "f") {
            switch (entry[0].split(".").slice(-1)[0]) {
              case "odt":
              case "docx":
              case "doc":
              case "html":
              case "txt":
                var icon = (
                  <TextIcon className={iconViewClassName + " text-blue-500"} />
                );
                break;
              case "pdf":
                var icon = (
                  <TextIcon className={iconViewClassName + " text-blue-500"} />
                );
                break;
              case "wav":
              case "mp3":
              case "m4a":
              case "aiv":
              case "flac":
              case "opus":
              case "webm":
                var icon = (
                  <Music className={iconViewClassName + " text-pink-500"} />
                );
                break;
              case "mp4":
              case "avi":
              case "mkv":
              case "webv":
                var icon = (
                  <VideoIcon className={iconViewClassName + " text-red-500"} />
                );
                break;
              case "css":
              case "js":
              case "ts":
              case "jsx":
              case "tsx":
              case "py":
              case "cpp":
              case "c":
              case "h":
              case "php":
              case "rs":
              case "go":
              case "json":
              case "bash":
              case "sh":
              case "Makefile":
              case "lua":
              case "exe":
              case "elf":
              case "dll":
              case "ini":
              case "cnf":
                var icon = (
                  <BracesIcon
                    className={iconViewClassName + " text-green-500"}
                  />
                );
                break;
              case "old":
              case "bak":
                var icon = (
                  <Clock2 className={iconViewClassName + " text-green-500"} />
                );
                break;
              default:
                if (entry[0].split(".")[0] == "") {
                  var icon = (
                    <BracketsIcon
                      className={iconViewClassName + " text-neutral-100"}
                    />
                  );
                } else {
                  var icon = (
                    <FileIcon
                      className={iconViewClassName + " text-neutral-100"}
                    />
                  );
                }
            }
          } else if (entry[1] === "a") {
            var icon = (
              <ShieldIcon className={iconViewClassName + " text-neutral-100"} />
            );
          }

          const contextMenuIcon = "inline-block pr-1 h-5 w-5 mb-[-2px]";

          if (entry[1] === "f") {
            return (
              <ContextMenu key={"file" + index} modal={false}>
                <ContextMenuTrigger>
                  <span
                    id={index}
                    className={viewClassName}
                    onClick={() =>
                      window.open(
                        fetchURLPrefix +
                          "/api/callFile/" +
                          encodeURIComponent(currentPath + entry[0]),
                      )
                    }
                  >
                    {icon} {entry[0]}
                  </span>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem
                    onClick={(event) => {
                      requestDeletion(entry[0]);
                    }}
                  >
                    <DeleteIcon className={contextMenuIcon} /> Delete
                  </ContextMenuItem>
                  <ContextMenuItem
                    onClick={(event) => {
                      requestRename(entry[0]);
                    }}
                  >
                    <PenLineIcon className={contextMenuIcon} /> Rename
                  </ContextMenuItem>
                  <ContextMenuItem
                    onClick={(event) => {
                      requestBurn(entry[0]);
                    }}
                  >
                    <Flame className={contextMenuIcon} /> Burn
                  </ContextMenuItem>
                  <ContextMenuItem
                    onClick={() =>
                      window.open(
                        fetchURLPrefix +
                          "/api/callFile/" +
                          encodeURIComponent(currentPath + entry[0]),
                      )
                    }
                  >
                    <DownloadIcon className={contextMenuIcon} /> Download
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            );
          } else {
            return (
              <ContextMenu key={"folder" + index} modal={false}>
                <ContextMenuTrigger>
                  <span
                    id={index}
                    className={viewClassName}
                    onClick={() => fetchFiles(`${currentPath}${entry[0]}/`)}
                  >
                    {icon} {entry[0]}
                  </span>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem
                    onClick={(event) => {
                      requestDeletion(entry[0]);
                      event.preventDefault();
                    }}
                  >
                    <DeleteIcon className={contextMenuIcon} /> Delete
                  </ContextMenuItem>
                  <ContextMenuItem
                    onClick={(event) => {
                      requestRename(entry[0]);
                    }}
                  >
                    <PenLineIcon className={contextMenuIcon} /> Rename
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            );
          }
        })}
      </div>
      {entries.length === 0 ? (
        <>
          <span className="block w-full text-center">
            No files or folders in <br /> {currentPath}
          </span>
        </>
      ) : (
        <></>
      )}
    </div>
  );
}
