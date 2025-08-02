import { FileSelection } from "@/components/ui/FileSelection";
import Page from "@/components/ui/PageWrapper";
import PathViewer from "@/components/ui/pathViewer";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { useEffect, useRef, useState } from "react";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";
import { toast } from "@/components/ui/use-toast";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import InformationHighlight from "@/components/ui/InformationHighlight";
import {
  BadgePlusIcon,
  EarthIcon,
  HardDriveIcon,
  KeyIcon,
  PenIcon,
  PinIcon,
  Share2Icon,
  ShieldIcon,
  UploadIcon,
  UserIcon,
  Users2,
} from "lucide-react";
import secondsToFormat from "@/lib/dates";
import { DialogClose, DialogDescription } from "@radix-ui/react-dialog";
import { Button } from "@/components/ui/button";
import useNotification from "@/lib/notificationState";
import { v4 } from "uuid";
import { Input } from "@/components/ui/input";
import Label from "@/components/ui/ShortLabel";
import UploadButton from "@/components/ui/UploadButton";
import { Switch } from "@/components/ui/switch";
import CopyButton from "@/components/ui/CopyButton";

function SharingPopup({ open = false, onOpenChange = () => {} }) {
  const { deleteNotification, notify, notifications } = useNotification();

  const [sharedFilesList, setSharedFilesList] = useState([]);
  function getSharedFiles() {
    fetch(fetchURLPrefix + "/api/getSharedFilesList").then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setSharedFilesList(json.files);
        });
      } else {
        toast({
          title: "Failed to fetch shared files",
          description:
            "A list of shared files could not be fetched from the server",
        });
      }
    });
  }

  useEffect(() => {
    getSharedFiles();
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      getSharedFiles();
    }, 1000);

    return () => clearInterval(interval);
  }, []);
  function copySharingLink(code) {
    navigator.clipboard.writeText(`${location.origin}/shared?code=${code}`);
    toast({
      title: "Copied code to clipboard",
    });
  }

  function unshareFile(code, path) {
    fetch(fetchURLPrefix + "/api/unshareFile/" + code).then((res) => {
      if (res.ok) {
        getSharedFiles();
      } else {
        toast({
          title: "Failed to unshare file",
          description:
            "Zentrox was unable to remove the share for the file at " +
            path +
            ".",
        });
        notify("Zentrox failed to unshare the file at " + path + ".");
      }
    });
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Shared files</DialogTitle>
          <DialogDescription>Manage your shared files</DialogDescription>
        </DialogHeader>

        <span className="block max-h-[300px] overflow-y-scroll">
          {sharedFilesList.length === 0 ? (
            <small className="text-white/80">
              You have not shared any files
            </small>
          ) : (
            sharedFilesList.map((e) => {
              let file_name = e.file_path.split("/").slice(-1);

              return (
                <InformationHighlight
                  title={file_name}
                  Icon={e.use_password ? KeyIcon : EarthIcon}
                >
                  <span className="text-base mb-1">
                    Path: {e.file_path} <br />
                    Code:{" "}
                    <span
                      className="underline cursor-pointer items-center"
                      onClick={() => copySharingLink(e.code)}
                    >
                      Copy sharing link
                    </span>{" "}
                    <br />
                    Password protected: {e.use_password ? "Yes" : "No"} <br />
                    Shared since:{" "}
                    {secondsToFormat(
                      e.shared_since,
                      localStorage.getItem("dateFormat") || "8601",
                    )}{" "}
                    <br />
                  </span>
                  <span
                    className="underline text-red-500 cursor-pointer"
                    onClick={() => unshareFile(e.code, e.file_path)}
                  >
                    Unshare
                  </span>
                </InformationHighlight>
              );
            })
          )}{" "}
        </span>

        <DialogFooter>
          <DialogClose>
            <Button onClick={() => onOpenChange(false)}>Close</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function Files() {
  const [path, setPath] = useState("/");
  const [filterPattern, setFilterPattern] = useState("");
  const [metadataPopupOpen, setMetadataPopupOpen] = useState(false);
  const [metadataPopupData, setMetadataPopupData] = useState({});
  const [burnFilePopupOpen, setBurnFilePopupOpen] = useState(false);
  const [burnFilePopupPath, setBurnFilePopupPath] = useState("");
  const [movePopupOpen, setMovePopupOpen] = useState(false);
  const [movePopupPath, setMovePopupPath] = useState("");
  const [unlinkPopupOpen, setUnlinkPopupOpen] = useState(false);
  const [unlinkPopupPath, setUnlinkPopupPath] = useState("");
  const [uploadPopupOpen, setUploadPopupOpen] = useState(false);
  const [sharedFilesPopupOpen, setSharedFilesPopupOpen] = useState(false);
  const [shareFilePopupOpen, setShareFilePopupOpen] = useState(false);
  const [shareFilePopupPath, setShareFilePopupPath] = useState("");
  const [shareEnablePassword, setShareEnablePassword] = useState(false);
  const [burningFile, setBurningFile] = useState(false);
  const [selectedFiles, setSelectedFiles] = useState(null);

  const [updateKey, _setUpdateKey] = useState(v4());
  const { deleteNotification, notify, notifications } = useNotification();

  var fileSharePasswordInput = useRef();
  var movePopupInput = useRef();

  function traverseInto(path) {
    setPath(path);
  }

  function getPathFilename(path) {
    if (path === "") return "";
    if (typeof path !== "string")
      throw new TypeError("Function can only handle strings");
    const segments = path.split("/");
    if (path.endsWith("/")) {
      return segments.splice(-2)[0];
    } else {
      return segments.splice(-1)[0];
    }
  }

  function downloadFile(path) {
    const url = fetchURLPrefix + "/api/callFile/" + encodeURIComponent(path);
    var link = document.createElement("a");
    link.setAttribute("download", getPathFilename(path));
    link.setAttribute("hidden", "true");
    link.setAttribute("href", url);
    link.setAttribute("target", "_blank");
    link.click();
  }

  function copyPath(path) {
    if (typeof navigator !== "undefined" && typeof window !== "undefined") {
      navigator.clipboard.writeText(path);
      toast({
        title: "Copied path",
      });
    }
  }

  function viewMetadata(path) {
    fetch(fetchURLPrefix + "/api/getMetadata/" + encodeURIComponent(path))
      .then((res) => {
        res.json().then((json) => {
          setMetadataPopupData(json);
        });
        setMetadataPopupOpen(true);
      })
      .catch(() => {
        toast({
          title: "Failed to fetch metadata",
          description: "No metadata could be provided for this file.",
        });
        notify("No metadata could be provided for " + path);
      });
  }

  function confirmBurnFile(path) {
    setBurnFilePopupOpen(true);
    setBurnFilePopupPath(path);
  }

  function confirmMove(path) {
    setMovePopupOpen(true);
    setMovePopupPath(path);
  }

  function confirmUnlink(path) {
    setUnlinkPopupOpen(true);
    setUnlinkPopupPath(path);
  }

  function shareFile(path) {
    setShareFilePopupOpen(true);
    setShareFilePopupPath(path);
  }

  function FileManagerDirectoryContextMenu({ path }) {
    return (
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => traverseInto(path)}>
          Open directory
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => confirmMove(path)}>
          Move recursively
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => viewMetadata(path)}>
          View metadata
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => copyPath(path)}>
          Copy path
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={() => confirmUnlink(path)}
          className="text-red-500 focus:text-red-500 focus:bg-red-500/20"
        >
          Remove recursively
        </ContextMenuItem>
      </ContextMenuContent>
    );
  }

  function FileManagerFileContextMenu({ path }) {
    return (
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => downloadFile(path)}>
          View file
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem onSelect={() => confirmMove(path)}>
          Move file
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => confirmBurnFile(path)}>
          Burn file
        </ContextMenuItem>
        <ContextMenuItem onClick={() => shareFile(path)}>
          Share file
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => viewMetadata(path)}>
          View metadata
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => copyPath(path)}>
          Copy path
        </ContextMenuItem>
        <ContextMenuItem
          onSelect={() => confirmUnlink(path)}
          className="text-red-500 focus:text-red-500 focus:bg-red-500/20"
        >
          Unlink file
        </ContextMenuItem>
      </ContextMenuContent>
    );
  }

  function prettyBytes(bytes) {
    if (bytes === 0) return "0 B";

    const units = ["b", "KB", "MB", "GB", "TB"];
    const k = 1000;
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const value = bytes / Math.pow(k, i);

    return `${value.toFixed(2)} ${units[i]}`;
  }

  function requestFileBurn(burnPath) {
    fetch(
      fetchURLPrefix + "/api/burnFile/" + encodeURIComponent(burnPath),
    ).then((res) => {
      if (res.ok) {
        setBurnFilePopupOpen(false);
        setBurnFilePopupPath("");
        setBurningFile(false);
        toast({
          title: "File has been burned",
        });
        notify("The file " + burnPath + " has been burned.");
      } else {
        setBurnFilePopupOpen(false);
        setBurnFilePopupPath("");
        setBurningFile(false);
        toast({
          title: "Failed to burn file",
        });
        notify("The file " + burnPath + " has not been burned.");
      }
    });
  }

  function requestMove(origin, destination) {
    fetch(
      fetchURLPrefix +
        "/api/movePath/" +
        encodeURIComponent(origin) +
        "/" +
        encodeURIComponent(destination),
    )
      .then((res) => {
        if (res.ok) {
          setMovePopupOpen(false);
          setMovePopupPath("");
        } else {
          toast({
            title: "Failed to move directory",
            description:
              "This directory could not be moved. This could be due to a lack of permissions.",
          });
        }
      })
      .catch(() => {
        toast({
          title: "Failed to move directory",
          description:
            "This directory could not be moved. This could be due to a lack of permissions.",
        });
      });
  }
  function requestUnlink(unlinkPath) {
    fetch(
      fetchURLPrefix + "/api/deleteFile/" + encodeURIComponent(unlinkPath),
    ).then((res) => {
      if (res.ok) {
        setUnlinkPopupOpen(false);
        setUnlinkPopupPath("");
        toast({
          title: "Finished removing",
        });
        notify("The path " + unlinkPath + " has been removed.");
      } else {
        setUnlinkPopupOpen(false);
        setUnlinkPopupPath("");
        toast({
          title: "Failed to remove",
        });
        notify("The path " + unlinkPath + " has not been burned.");
      }
    });
  }

  function requestUpload() {
    const file = selectedFiles[0];
    setSelectedFiles(null);
    setUploadPopupOpen(false);
    var formData = new FormData();
    formData.append("file", file);
    formData.append("path", path);
    notify(`Started upload of ${file.name}`);
    fetch(fetchURLPrefix + "/upload/file", {
      method: "POST",
      body: formData,
    }).then((res) => {
      if (res.ok) {
        notify(`Finished upload of ${file.name}`);
      } else {
        notify(`Failed upload of ${file.name}`);
        res.text().then(() => {
          toast({
            title: "Failed to upload file",
            description: `Zentrox failed to upload the file you provided`,
          });
        });
      }
    });
  }

  function requestFileShare() {
    fetch(fetchURLPrefix + "/api/shareFile", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        file_path: shareFilePopupPath,
        password: shareEnablePassword
          ? fileSharePasswordInput.current.value
          : null,
      }),
    }).then((res) => {
      if (res.ok) {
        setShareFilePopupOpen(false);
        setShareFilePopupPath("");
        res.text().then((t) => {
          toast({
            title: "Shared file",
            description:
              "The desired file share was completed. Click to copy the sharing link code.",
            action: (
              <CopyButton
                className="mt-2 border !border-black/20"
                link={`${location.origin}/shared?code=${t}`}
              />
            ),
          });
        });
      }
    });
  }

  return (
    <>
      <Page name="Files">
        <SharingPopup
          open={sharedFilesPopupOpen}
          onOpenChange={setSharedFilesPopupOpen}
        />

        <Dialog
          onOpenChange={(v) => {
            setShareFilePopupOpen(v);
            setShareFilePopupPath(null);
          }}
          open={shareFilePopupOpen}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Share file</DialogTitle>
              <DialogDescription>
                Please configure your file share for <br />
                <code>{shareFilePopupPath}</code>
              </DialogDescription>
            </DialogHeader>
            <p>
              <span className="items-center flex mb-1.5">
                <Switch
                  className="mr-1"
                  checked={shareEnablePassword}
                  onCheckedChange={setShareEnablePassword}
                />{" "}
                Enable password protection
              </span>
              {shareEnablePassword && (
                <Input
                  type="password"
                  placeholder="Password"
                  ref={fileSharePasswordInput}
                />
              )}
            </p>
            <DialogFooter>
              <DialogClose>
                <Button
                  variant="outline"
                  onClick={() => {
                    setShareFilePopupPath("");
                    setShareEnablePassword(false);
                  }}
                >
                  Cancel
                </Button>
              </DialogClose>
              <Button onClick={() => requestFileShare()}>Share</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog
          onOpenChange={(v) => {
            setUploadPopupOpen(v);

            setSelectedFiles(null);
          }}
          open={uploadPopupOpen}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Upload a new file</DialogTitle>
              <DialogDescription>
                Select a file for upload to:
                <br />
                <code>{path}</code>
                <br />
              </DialogDescription>
              <p>
                <UploadButton
                  variant="outline"
                  files={selectedFiles}
                  onFilesChange={setSelectedFiles}
                >
                  Select file
                </UploadButton>
              </p>
            </DialogHeader>
            <DialogFooter>
              <DialogClose>
                <Button
                  variant="outline"
                  onClick={() => {
                    setUploadPopupOpen(false);
                  }}
                >
                  Cancel
                </Button>
              </DialogClose>
              <Button onClick={() => requestUpload()}>Upload</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog onOpenChange={setUnlinkPopupOpen} open={unlinkPopupOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {unlinkPopupPath.endsWith("/")
                  ? "Remove directory"
                  : "Unlink file"}
              </DialogTitle>
              <DialogDescription>
                Do you want to remove this{" "}
                {unlinkPopupPath.endsWith("/") ? "directory" : "file"}?<br />
                <code>{unlinkPopupPath}</code>
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose>
                <Button
                  variant="outline"
                  onClick={() => {
                    setUnlinkPopupOpen(false);
                    setUnlinkPopupPath("");
                  }}
                >
                  Cancel
                </Button>
              </DialogClose>
              <Button
                onClick={() => requestUnlink(unlinkPopupPath)}
                variant="destructive"
              >
                {movePopupPath.endsWith("/")
                  ? "Remove directory"
                  : "Unlink file"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog onOpenChange={setMovePopupOpen} open={movePopupOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                Move {movePopupPath.endsWith("/") ? "directory" : "file"}
              </DialogTitle>
              <DialogDescription>
                Where do you want to move this{" "}
                {movePopupPath.endsWith("/") ? "directory" : "file"}?<br />
                <code>{movePopupPath}</code>
              </DialogDescription>
            </DialogHeader>
            <p>
              <Label>Move to</Label>
              <Input
                type="text"
                placeholder="Path"
                ref={movePopupInput}
                defaultValue={movePopupPath}
                className="w-full"
              />
            </p>
            <DialogFooter>
              <DialogClose>
                <Button
                  variant="outline"
                  onClick={() => {
                    setMovePopupOpen(false);
                    setMovePopupPath("");
                  }}
                >
                  Cancel
                </Button>
              </DialogClose>
              <Button
                onClick={() =>
                  requestMove(movePopupPath, movePopupInput.current.value)
                }
              >
                Move {movePopupPath.endsWith("/") ? "directory" : "file"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>

        <Dialog onOpenChange={setBurnFilePopupOpen} open={burnFilePopupOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Burn file?</DialogTitle>
              <DialogDescription>
                Are you sure you want to <strong>permanently</strong> overwrite
                and delete this file?
                <br />
                <code>{burnFilePopupPath}</code>
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose>
                <Button
                  variant="outline"
                  onClick={() => {
                    setBurnFilePopupOpen(false);
                    setBurnFilePopupPath("");
                    setBurningFile(false);
                  }}
                >
                  Cancel
                </Button>
              </DialogClose>
              <Button
                variant="destructive"
                onClick={() => requestFileBurn(burnFilePopupPath)}
              >
                {burningFile && (
                  <Loader2 className="h-4 w-4 inline-block animate-spin" />
                )}{" "}
                Burn file
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
        <Dialog onOpenChange={setMetadataPopupOpen} open={metadataPopupOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                Metadata for {metadataPopupData.filename}
                {metadataPopupData.entry_type == "Directory" && "/"}
              </DialogTitle>
            </DialogHeader>
            <div>
              <InformationHighlight title={"Permissions"} Icon={ShieldIcon}>
                {metadataPopupData.permissions}
              </InformationHighlight>
              <InformationHighlight title={"Owner"} Icon={UserIcon}>
                {metadataPopupData.owner_username} (
                {metadataPopupData.owner_uid})
              </InformationHighlight>
              <InformationHighlight title={"Owner GID"} Icon={Users2}>
                {metadataPopupData.owner_gid}
              </InformationHighlight>
              <InformationHighlight title={"Created"} Icon={BadgePlusIcon}>
                {secondsToFormat(
                  metadataPopupData.created,
                  localStorage.getItem("dateFormat") || "8601",
                )}
              </InformationHighlight>
              <InformationHighlight title={"Modified"} Icon={PenIcon}>
                {secondsToFormat(
                  metadataPopupData.modified,
                  localStorage.getItem("dateFormat") || "8601",
                )}
              </InformationHighlight>
              <InformationHighlight title={"Size"} Icon={HardDriveIcon}>
                {prettyBytes(metadataPopupData.size)}
              </InformationHighlight>
              <InformationHighlight title={"Absolute path"} Icon={PinIcon}>
                {metadataPopupData.absolute_path}
              </InformationHighlight>
            </div>
            <DialogFooter>
              <DialogClose>
                <Button>Close</Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
        <div className="h-[calc(100%-100px)] max-h-[calc(100%-100px)]">
          <PathViewer
            value={path}
            home="/home"
            onValueChange={setPath}
            onFilter={setFilterPattern}
          >
            <UploadIcon
              onClick={() => setUploadPopupOpen(true)}
              className="w-4 h-5 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1.5"
            />
            <Share2Icon
              onClick={() => setSharedFilesPopupOpen(true)}
              className="w-4 h-5 transition-all cursor-pointer opacity-75 hover:opacity-100 inline-block align-middle mr-1.5"
            />
          </PathViewer>

          <FileSelection
            className="mt-2"
            selectionBoxes={false}
            path={path}
            key={updateKey}
            onDirectoryClick={(name) => setPath(name)}
            patternMatching={filterPattern}
            onFileSelect={downloadFile}
            DirectoryContextMenuContent={FileManagerDirectoryContextMenu}
            FileContextMenuContent={FileManagerFileContextMenu}
          />
        </div>
      </Page>
    </>
  );
}

export default Files;
