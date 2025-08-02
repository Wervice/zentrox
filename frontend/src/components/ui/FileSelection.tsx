import { FolderDotIcon, FolderIcon, TelescopeIcon } from "lucide-react";
import { ElementType, useEffect, useState } from "react";
import FileIcon from "./FileIcon";
import {
  Placeholder,
  PlaceholderIcon,
  PlaceholderSubtitle,
} from "./placeholder";
import { ContextMenuTrigger, ContextMenu } from "./context-menu";
import { Checkbox } from "./checkbox";
import { cn } from "@/lib/utils";
import { Url } from "next/dist/shared/lib/router/router";
import concatPath from "@/lib/concatPath";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { toast } from "./use-toast";
import getParentPath from "@/lib/paths";

enum DirectoryListingEntry {
  File,
  Directory,
  InsufficientPermissions,
}

type SerializedFile = [string, string];

type DirectoryListingSerialized = {
  content: SerializedFile[];
};

type DeserializedFile = {
  filename: string;
  type: DirectoryListingEntry;
};

function serializeFileList(e: SerializedFile): DeserializedFile {
  var entryType: DirectoryListingEntry;
  var name: string = e[0];
  switch (e[1]) {
    case "File":
      entryType = DirectoryListingEntry.File;
      break;
    case "Directory":
      entryType = DirectoryListingEntry.Directory;
      name = e[0].endsWith("/") ? e[0] : e[0] + "/";
      break;
    case "InsufficientPermissions":
      entryType = DirectoryListingEntry.InsufficientPermissions;
      break;
    default:
      entryType = DirectoryListingEntry.File;
  }
  return {
    filename: name,
    type: entryType,
  };
}

function EntryIcon({
  filename,
  type,
}: {
  filename: string;
  type: DirectoryListingEntry;
}) {
  if (type === DirectoryListingEntry.Directory) {
    if (filename.startsWith(".")) {
      return <FolderDotIcon color="#ffffff66" />;
    } else {
      return <FolderIcon fill="#fff" />;
    }
  } else {
    return <FileIcon filename={filename} />;
  }
}

type Props = {
  path: string;
  onFileSelect: (e: string) => any;
  onDirectorySelect: (e: string) => any;
  onDirectoryClick: (e: string) => any;
  onInsufficientPermissionPerformed: (e: string) => any;
  patternMatching: string | ((e: string) => boolean);
  sortingFunction:
    | undefined
    | ((a: DeserializedFile, b: DeserializedFile) => number);
  selection: string[];
  selectionBoxes: boolean;
  className: string | undefined;
  DirectoryContextMenuContent: ElementType;
  FileContextMenuContent: ElementType;
};

function FileSelection({
  path,
  onFileSelect,
  onDirectorySelect = (_) => {},
  onDirectoryClick = (_) => {},
  onInsufficientPermissionPerformed = (_) => {},
  patternMatching,
  sortingFunction,
  selection = [],
  selectionBoxes = true,
  className = "",
  FileContextMenuContent,
  DirectoryContextMenuContent,
}: Props) {
  const [currentFiles, setCurrentFiles] = useState<DeserializedFile[]>([]);
  const [currentFilesSorted, setCurrentFilesSorted] = useState<
    DeserializedFile[]
  >([]);

  // Fetch a list of files for the current directory from the server
  async function fetchFiles(requestedPath: string) {
    const url: Url =
      fetchURLPrefix + "/api/filesList/" + encodeURIComponent(requestedPath);

    const oldPath = path;

    const res: Response = await fetch(url);
    if (!res.ok) throw new Error("Failed to get directory details");
    const json: DirectoryListingSerialized = await res.json();
    if (typeof json.content === "undefined") return;

    const parsedEntries = json.content.map(serializeFileList);
    if (oldPath === path) {
      setCurrentFiles(parsedEntries);
    }
  }

  function entryClick(name: string, variant: DirectoryListingEntry) {
    if (variant == DirectoryListingEntry.File) {
      onFileSelect(concatPath(path, name));
    } else if (variant == DirectoryListingEntry.Directory) {
      onDirectoryClick(concatPath(path, name));
    } else if (variant == DirectoryListingEntry.InsufficientPermissions) {
      onInsufficientPermissionPerformed(concatPath(path, name));
    }
  }
  useEffect(() => {
    const interval = setInterval(() => {
      fetchFiles(path).catch((e) => {
        toast({
          title: "Failed to get directory contents",
          description:
            "This could be due to a lack of permissions or because the directory does not exist.",
        });
        console.error(e);
      });
    }, 750);

    return () => clearInterval(interval);
  }, [currentFiles, currentFilesSorted]);

  useEffect(() => {
    fetchFiles(path).catch((e) => {
      toast({
        title: "Failed to get directory contents",
        description:
          "This could be due to a lack of permissions or because the directory does not exist.",
      });
      onDirectoryClick(getParentPath(path));
      console.error(e);
    });
  }, [path]);

  useEffect(() => {
    const sortedArray = currentFiles
      .filter((e) => {
        if (typeof patternMatching === "string") {
          return e.filename.includes(patternMatching);
        } else {
          return patternMatching(e.filename);
        }
      })
      .toSorted((a, b) => {
        if (typeof sortingFunction === "function") {
          return sortingFunction(a, b);
        } else {
          if (
            a.type === DirectoryListingEntry.Directory ||
            a.type === DirectoryListingEntry.InsufficientPermissions
          ) {
            return a.filename.startsWith(".") ? 0 : -1;
          } else {
            return 1;
          }
        }
      });
    setCurrentFilesSorted(sortedArray);
  }, [currentFiles]);

  return (
    <>
      <Placeholder hidden={currentFiles.length !== 0}>
        <PlaceholderIcon icon={TelescopeIcon} />
        <PlaceholderSubtitle>
          There are no files in this directory
        </PlaceholderSubtitle>
      </Placeholder>

      {currentFiles.length > 0 && (
        <span
          className={cn(
            "rounded-xl block border-2 border-neutral-800 h-full max-h-full overflow-scroll",
            className,
          )}
        >
          {currentFilesSorted.map((e: DeserializedFile, i) => {
            const directoryCheckboxHandler = () => {
              onDirectorySelect(concatPath(path, e.filename));
            };
            const fileCheckboxHandler = () => {
              onFileSelect(concatPath(path, e.filename));
            };

            function checkboxHandler(ev: React.MouseEvent) {
              ev.stopPropagation();
              if (e.type === DirectoryListingEntry.Directory) {
                directoryCheckboxHandler();
              } else {
                fileCheckboxHandler();
              }
            }

            return (
              <ContextMenu
                key={"fileSelectionContextMenu_" + concatPath(path, e.filename)}
              >
                <ContextMenuTrigger>
                  <span
                    onClick={() => entryClick(e.filename, e.type)}
                    className={
                      "flex items-center w-full p-4 border-neutral-800 cursor-pointer hover:bg-neutral-900 transition-colors duration-200 " +
                      (i !== currentFiles.length - 1 && "border-b-2")
                    }
                  >
                    {selectionBoxes && (
                      <Checkbox
                        className="mr-2"
                        onClick={checkboxHandler}
                        checked={selection.includes(
                          concatPath(path, e.filename),
                        )}
                      />
                    )}
                    <EntryIcon filename={e.filename} type={e.type} />
                    <span className="ml-2">{e.filename}</span>
                  </span>
                </ContextMenuTrigger>
                {e.type == DirectoryListingEntry.Directory ? (
                  <DirectoryContextMenuContent
                    path={concatPath(path, e.filename)}
                  />
                ) : (
                  <FileContextMenuContent path={concatPath(path, e.filename)} />
                )}
              </ContextMenu>
            );
          })}
        </span>
      )}
    </>
  );
}

export { FileSelection, DirectoryListingEntry };
