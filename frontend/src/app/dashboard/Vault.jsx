import { Button } from "@/components/ui/button.jsx";
import { useEffect, useState, useRef } from "react";
import "./table.css";
import "./scroll.css";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import {
 KeyIcon,
 LockIcon,
 FolderIcon,
 CogIcon,
 PenLineIcon,
 UploadIcon,
 ArrowUp,
} from "lucide-react";
import {
 Dialog,
 DialogContent,
 DialogDescription,
 DialogHeader,
 DialogTitle,
 DialogTrigger,
 DialogFooter,
 DialogClose,
} from "@/components/ui/dialog";
import {
 AlertDialog,
 AlertDialogAction,
 AlertDialogCancel,
 AlertDialogContent,
 AlertDialogDescription,
 AlertDialogFooter,
 AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import "./scroll.css";
import { useToast } from "@/components/ui/use-toast";
import Page from "@/components/ui/PageWrapper";

// const fetchURLPrefix = "";
const fetchURLPrefix = require("@/lib/fetchPrefix");

function Vault() {
 var vaultEncryptionKey = useRef();
 var vaultKeyDecryptModal = useRef();
 var uploadInput = useRef();
 var newDirectoryInput = useRef();
 var renamingModalInput = useRef();

 const { toast } = useToast();
 const [decryptKeyModalVisible, setDecryptKeyModalVisibility] = useState(false);
 const [currentVaultPath, setCurrentVaultPath] = useState("");
 const [currentVaultContents, setCurrentVaultContents] = useState([]);
 const [vaultSessionKey, setVaultSessionKey] = useState("");
 const [uploadButton, setUploadButton] = useState("default");
 const [deletionModalVisible, setDeletionModalVisible] = useState(false);
 const [renamingModalVisible, setRenamingModalVisible] = useState(false);
 const [currentVaultFileRename, setCurrentVaultFileRename] = useState("");
 const [currentVaultFileDelete, setCurrentVaultFileDelete] = useState("");

 const [vaultConfigModalOpen, setVaultConfigModalOpen] = useState(false);
 const [vaultState, setVaultState] = useState("locked");

 useEffect(() => {
  if (vaultState == "locked") {
   fetch("/api/isVaultConfigured").then((r) => {
    if (r.ok) {
     r.text().then((t) => {
      console.log(t);
      if (t === "0") {
       setVaultState("unconfigured");
      }
     });
    } else {
     setVaultState("unconfigured");
    }
   });
  }
 }, [setVaultState]);

 function parentDir(path) {
  if (!path.endsWith("/")) path += "/";
  var parsedPath = path.split("/");
  parsedPath.pop();
  parsedPath.pop();
  var parentPath = parsedPath.join("/") + "/";
  if (parentPath === "/") parentPath = "";
  return parentPath;
 }

 function vaultTree(key = "") {
  var vaultKey;
  if (key === "") vaultKey = vaultKeyDecryptModal.current.value;
  else vaultKey = key;
  fetch(fetchURLPrefix + "/api/vaultTree", {
   method: "POST",
   headers: {
    "Content-Type": "application/json",
   },
   body: JSON.stringify({
    key: vaultKey,
   }),
  }).then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setCurrentVaultContents(json.fs);
     setVaultSessionKey(vaultKey);
     setVaultState("unlocked");
    });
   } else {
    if (res.status === 403) {
     toast({
      title: "Failed to authenticate",
      description: "Zentrox was unable to validate your key",
     });
    }
   }
  });
 }

 function requestRename() {
  setRenamingModalVisible(true);
 }

 function requestDeletion() {
  setDeletionModalVisible(true);
 }

 function isDirectChild(entry, currentVaultPath) {
  // Remove trailing `/` from entry if it exists
  entry = entry.endsWith("/") ? entry.slice(0, -1) : entry;

  console.log(currentVaultPath);

  // Check if the entry starts with the currentVaultPath
  if (!entry.startsWith(currentVaultPath)) return false;

  // Get the remaining part of the entry after currentVaultPath
  let remainingPath = entry.slice(currentVaultPath.length);

  console.log(remainingPath);

  // Check if the remaining path contains any `/`
  return !remainingPath.includes("/");
 }

 return (
  <Page name="Vault">
   <Toaster />
   <Dialog
    open={decryptKeyModalVisible}
    onOpenChange={setDecryptKeyModalVisibility}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Unlock Vault</DialogTitle>
      <DialogDescription>Please enter your vault key.</DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      placeholder="Current key"
      ref={vaultKeyDecryptModal}
     />
     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
       <Button
        onClick={() => {
         vaultTree();
        }}
       >
        <KeyIcon className="w-4 h-4 pr-1" /> Unlock
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <Dialog open={vaultConfigModalOpen} onOpenChange={setVaultConfigModalOpen}>
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Setup Vault</DialogTitle>
      <DialogDescription>
       Please enter a strong and secure password to configure vault. You need
       this password to view and upload files to vault.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      id="vaultEncryptionKey"
      ref={vaultEncryptionKey}
      placeholder="Key"
      className="inline"
     />

     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
       <Button
        variant="destructive"
        className="inline-block mb-1"
        onClick={() => {
         /** @type {string}*/
         var key = vaultEncryptionKey.current.value;
         if (key.length === 0) {
          toast({
           title: "Missing new key",
           description: "You need to input a new vault key",
          });
          return;
         }
         fetch(fetchURLPrefix + "/api/vaultConfigure", {
          method: "POST",
          headers: {
           "Content-Type": "application/json",
          },
          body: JSON.stringify({
           key: key,
          }),
         }).then((res) => {
          if (res.ok) {
           res.json().then((json) => {
            if (json.code === "no_decrypt_key") {
             noDecryptKeyModal();
            } else {
             toast({
              title: "Finished Vault configuration",
              description: "Vault has been configured and is ready for use",
             });
             setVaultState("locked");
            }
           });
          } else {
           if (res.status === 400) {
            toast({
             title: "Bad Request",
             description:
              "The data you provided was incorrect. The server responded with error 400.",
            });
           } else {
            toast({
             title: "Server Error " + res.status,
             description:
              "The server responded with an HTTP error of " + res.status * ".",
            });
           }
          }
         });
        }}
       >
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
   <div>
    <Button
     onClick={
      vaultState !== "unlocked"
       ? () => {
          location.reload();
         }
       : () => {
          setCurrentVaultContents([]);
          setCurrentVaultPath("");
          setVaultSessionKey("");
          setVaultState("locked");
         }
     }
     variant="destructive"
     className={
      vaultState !== "unlocked" ? "bg-blue-500 hover:bg-blue-600" : "mr-1"
     }
    >
     <LockIcon className="w-4 h-4 inline-block mr-1" />{" "}
     {vaultState !== "unlocked" ? "Reload" : "Exit"}
    </Button>
    <Dialog>
     <DialogTrigger asChild>
      <Button className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}>
       <FolderIcon className="w-4 h-4 inline-block mr-1" /> New Directory
      </Button>
     </DialogTrigger>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>New Directory</DialogTitle>
       <DialogDescription>Create a new directory.</DialogDescription>
      </DialogHeader>
      <Input type="text" ref={newDirectoryInput} placeholder="Name" />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Close</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          if (
           newDirectoryInput.current.value.includes("/") ||
           newDirectoryInput.current.value.includes(" ")
          ) {
           toast({
            title: "Illegal name",
            description: "A file name may not include slashes or spaces.",
           });
           return;
          }
          if (newDirectoryInput.current.value.length > 64) {
           toast({
            title: "Filename too long",
            description: "A filename can not be longer than 64 characters.",
           });
           return;
          }
          fetch(fetchURLPrefix + "/api/vaultNewFolder", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            key: vaultSessionKey,
            folder_name:
             currentVaultPath + "/" + newDirectoryInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) {
            vaultTree(vaultSessionKey);
           } else {
            toast({
             title: "Failed to create new directory",
             description: `Vault could not create a new directory ${newDirectoryInput.current.value} in ${currentVaultPath}`,
            });
           }
          });
         }}
        >
         <FolderIcon className="w-4 h-4 inline-block mr-1" /> Create
        </Button>
       </DialogClose>
      </DialogFooter>
     </DialogContent>
    </Dialog>
    <Dialog open={renamingModalVisible} onOpenChange={setRenamingModalVisible}>
     <DialogContent>
      <DialogHeader>
       <DialogTitle>Rename File</DialogTitle>
       <DialogDescription>Rename a file</DialogDescription>
      </DialogHeader>
      <Input type="text" ref={renamingModalInput} placeholder="New Name" />
      <DialogFooter>
       <DialogClose asChild>
        <Button variant="outline">Cancel</Button>
       </DialogClose>
       <DialogClose asChild>
        <Button
         onClick={() => {
          fetch(fetchURLPrefix + "/api/renameVaultFile", {
           method: "POST",
           headers: {
            "Content-Type": "application/json",
           },
           body: JSON.stringify({
            key: vaultSessionKey,
            path: currentVaultFileRename,
            newName: currentVaultPath + "/" + renamingModalInput.current.value,
           }),
          }).then((res) => {
           if (res.ok) vaultTree(vaultSessionKey);
           else
            toast({
             title: "Failed to rename file",
             description: "Zentrox failed to rename a file.",
            });
          });
         }}
        >
         <PenLineIcon className="w-4 h-4 inline-block mr-1" /> Rename
        </Button>
       </DialogClose>
      </DialogFooter>
     </DialogContent>
    </Dialog>
    <AlertDialog
     open={deletionModalVisible}
     onOpenChange={setDeletionModalVisible}
    >
     <AlertDialogContent>
      <AlertDialogTitle>Delete File</AlertDialogTitle>
      <AlertDialogDescription>
       Do you really want to delete{" "}
       {currentVaultFileDelete.length > 64
        ? currentVaultFileDelete.substring(0, 61) + "..."
        : currentVaultFileDelete}
       ?<br />
       This action can not be undone.
      </AlertDialogDescription>
      <AlertDialogFooter>
       <AlertDialogCancel>Cancel</AlertDialogCancel>
       <AlertDialogAction
        onClick={() => {
         fetch(fetchURLPrefix + "/api/deleteVaultFile", {
          method: "POST",
          headers: {
           "Content-Type": "application/json",
          },
          body: JSON.stringify({
           key: vaultSessionKey,
           deletePath: currentVaultFileDelete,
          }),
         }).then((res) => {
          if (res.ok) vaultTree(vaultSessionKey);
          else
           toast({
            title: "Failed to delete file",
            description: "Zentrox failed to delete a file.",
           });
         });
        }}
       >
        Delete
       </AlertDialogAction>
      </AlertDialogFooter>
     </AlertDialogContent>
    </AlertDialog>
    <Button
     className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}
     onClick={() => {
      uploadInput.current.click();
     }}
    >
     {uploadButton === "default" ? (
      <UploadIcon className="w-4 h-4 inline-block mr-1" />
     ) : (
      <LoaderIcon className="animate-spin h-4 w-4 inline mr-1" />
     )}{" "}
     Upload File
    </Button>
    <input
     type="file"
     ref={uploadInput}
     onInput={() => {
      if (event.target.files.length > 0) {
       setUploadButton("loading");
       var fileForSubmit = uploadInput.current.files[0];
       if (fileForSubmit.size >= 1024 * 1024 * 1024 * 10) {
        toast({
         title: "File to big",
         description: "The file you provided was larger than 10GB",
        });
       }
       var formData = new FormData();
       formData.append("file", fileForSubmit);
       formData.append("path", currentVaultPath);
       formData.append("key", vaultSessionKey);
       fetch(fetchURLPrefix + "/upload/vault", {
        method: "POST",
        body: formData,
       }).then((res) => {
        uploadInput.current.value = "";
        if (res.ok) {
         vaultTree(vaultSessionKey);
         setUploadButton("default");
        } else {
         setUploadButton("default");
         toast({
          title: "Failed to upload file",
          description: "Zentrox failed to upload the file you provided",
         });
        }
       });
      }
     }}
     hidden
    />
    <Button
     className={vaultState !== "unlocked" ? "invisible mr-1" : "mr-1"}
     onClick={() => {
      setCurrentVaultPath(parentDir(currentVaultPath));
     }}
    >
     <ArrowUp className="w-4 h-4 inline-block mr-1" /> Up
    </Button>
   </div>
   <div
    className={`no-scroll h-fit rounded-xl mt-2 overflow-hidden overflow-y-scroll no-scroll`}
    style={{
     minHeight: "fit-content",
     maxHeight: "calc(100vh - 220px)",
    }}
   >
    {vaultState == "locked" ? (
     <span className="h-fit">
      <div className="text-center text-2xl opacity-50">
       <LockIcon className="m-auto h-52 w-52" />
       Vault Is Locked
      </div>
      <Button
       className="m-auto block mt-4"
       onClick={() => {
        setDecryptKeyModalVisibility(true);
       }}
      >
       Unlock Vault
      </Button>
     </span>
    ) : vaultState == "unconfigured" ? (
     <span className="h-fit">
      <div className="text-center text-2xl opacity-50">
       <CogIcon className="m-auto h-52 w-52" />
       Vault Needs To Be Configured
      </div>
      <Button
       className="m-auto block mt-4"
       onClick={() => {
        setVaultConfigModalOpen(true);
       }}
      >
       Setup Vault
      </Button>
     </span>
    ) : (
     ""
    )}
    {
     /*
      * @param {string} entry*/
     currentVaultContents
      .filter((entry) => {
       return isDirectChild(entry, currentVaultPath);
      })
      .map((entry) => {
       var type = "";
       if (entry.endsWith("/")) {
        type = "folder";
       } else {
        type = "file";
       }
       return (
        <ContextMenu>
         <ContextMenuContent>
          <ContextMenuItem
           onClick={() => {
            setCurrentVaultFileDelete(entry);
            requestDeletion(entry);
           }}
          >
           <DeleteIcon className="w-4 h-4 inline-block mr-1" /> Delete
          </ContextMenuItem>
          <ContextMenuItem
           onClick={() => {
            setCurrentVaultFileRename(entry);
            requestRename(entry);
           }}
          >
           <PenLineIcon className="w-4 h-4 inline-block mr-1" /> Rename
          </ContextMenuItem>
         </ContextMenuContent>
         <ContextMenuTrigger>
          <span
           className="w-full p-4 bg-transparent block cursor-default select-none hover:bg-neutral-900 hover:transition-bg hover:duration-300 focus:outline-blue-500 focus:duration-50"
           onClick={
            type === "folder"
             ? () => {
                setCurrentVaultPath(entry);
               }
             : (e) => {
                e.target.classList.add("animate-pulse");
                e.target.classList.add("duration-300");

                e.target.classList.remove("duration-200");
                fetch(fetchURLPrefix + "/api/vaultFileDownload", {
                 method: "POST",
                 headers: {
                  "Content-Type": "application/json",
                 },
                 body: JSON.stringify({
                  key: vaultSessionKey,
                  path: entry,
                 }),
                }).then((res) => {
                 e.target.classList.remove("animate-pulse");
                 e.target.classList.remove("duration-300");

                 e.target.classList.add("duration-200");
                 if (res.ok) {
                  res.blob().then((blob) => {
                   var url = window.URL.createObjectURL(blob);
                   var a = document.createElement("a");
                   a.href = url;
                   a.download = entry;
                   document.body.appendChild(a); // we need to append the element to the dom -> otherwise it will not work in firefox
                   a.click();
                   a.remove();
                  });
                 } else {
                  toast({
                   title: "File download error",
                  });
                 }
                });
               }
           }
          >
           {type === "folder" ? (
            <FolderIcon className="w-6 h-6 inline-block mr-1" fill="white" />
           ) : (
            <FileIcon className="w-6 h-6 inline-block mr-1" />
           )}{" "}
           {type === "folder"
            ? entry.split("/").at(-2)
            : entry.split("/").at(-1)}
          </span>
         </ContextMenuTrigger>
        </ContextMenu>
       );
      })
    }
   </div>
  </Page>
 );
}

export default Vault;
