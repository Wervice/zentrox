import { FileSelection } from "@/components/ui/FileSelection";
import Page from "@/components/ui/PageWrapper";
import PathViewer from "@/components/ui/pathViewer";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { useState } from "react";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
} from "@/components/ui/context-menu";
import { toast } from "@/components/ui/use-toast";
import {
  Dialog,
  DialogContent,
  DialogTitle,
} from "@/components/ui/dialog";
import InformationHighlight from "@/components/ui/InformationHighlight";
import {
  BadgePlusIcon,
  HardDriveIcon,
  PenIcon,
  PinIcon,
  ShieldIcon,
  UserIcon,
  Users2,
} from "lucide-react";
import secondsToFormat from "@/lib/dates";

// TODO: Finish file context menu
// TODO: Add filesharing

function Files() {
  const [path, setPath] = useState("/");
  const [filterPattern, setFilterPattern] = useState("");
  const [metadataPopupOpen, setMetadataPopupOpen] = useState(false);
  const [metadataPopupData, setMetadataPopupData] = useState({});

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
    fetch(fetchURLPrefix + "/api/getMetadata/" + encodeURIComponent(path)).then(
      (res) => {
        res.json().then((json) => {
          setMetadataPopupData(json);
        });
        setMetadataPopupOpen(true);
      },
    );
  }

  function FileManagerDirectoryContextMenu({ path }) {
    return (
      <ContextMenuContent>
        <ContextMenuItem onSelect={() => traverseInto(path)}>
          Open directory
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem>Remove recursively</ContextMenuItem>
        <ContextMenuItem>Move recursively</ContextMenuItem>
        <ContextMenuItem>Share directory</ContextMenuItem>
        <ContextMenuItem onSelect={() => viewMetadata(path)}>
          View metadata
        </ContextMenuItem>
        <ContextMenuItem onSelect={() => copyPath(path)}>
          Copy path
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
        <ContextMenuItem>Remove file</ContextMenuItem>
        <ContextMenuItem>Move file</ContextMenuItem>
        <ContextMenuItem>Burn file</ContextMenuItem>
        <ContextMenuItem>Share file</ContextMenuItem>
        <ContextMenuItem onSelect={() => viewMetadata(path)}>View metadata</ContextMenuItem>
        <ContextMenuItem onSelect={() => copyPath(path)}>
          Copy path
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
  return (
    <>
      <Page name="Files">
        <Dialog onOpenChange={setMetadataPopupOpen} open={metadataPopupOpen}>
          <DialogContent>
            <DialogTitle>
              Metadata for {metadataPopupData.filename}
              {metadataPopupData.entry_type == "Directory" && "/"}
            </DialogTitle>
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
                {secondsToFormat(metadataPopupData.created, localStorage.getItem("dateFormat") || "8601")}
              </InformationHighlight>
              <InformationHighlight title={"Modified"} Icon={PenIcon}>
                {secondsToFormat(metadataPopupData.modified, localStorage.getItem("dateFormat") || "8601")}
              </InformationHighlight>
              <InformationHighlight title={"Size"} Icon={HardDriveIcon}>
                {prettyBytes(metadataPopupData.size)}
              </InformationHighlight>
                <InformationHighlight title={"Absolute path"} Icon={PinIcon}>
                {metadataPopupData.absolute_path}
              </InformationHighlight>
            </div>
          </DialogContent>
        </Dialog>
        <div className="h-[calc(100%-100px)] max-h-[calc(100%-100px)]">
          <PathViewer
            value={path}
            home="/home"
            onValueChange={setPath}
            onFilter={setFilterPattern}
          />

          <FileSelection
            className="mt-2"
            selectionBoxes={false}
            path={path}
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
