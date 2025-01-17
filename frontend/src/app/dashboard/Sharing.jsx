import { useState, useRef, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import InfoButton from "@/components/ui/InfoButton";
import { Button } from "@/components/ui/button.jsx";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
 Dialog,
 DialogContent,
 DialogDescription,
 DialogHeader,
 DialogTitle,
 DialogFooter,
 DialogClose,
} from "@/components/ui/dialog";
import { KeyIcon } from "lucide-react";

const fetchURLPrefix = require("@/lib/fetchPrefix");

function Sharing() {
 var ftpUserNameInput = useRef();
 var ftpPassWordInput = useRef();
 var ftpRootInput = useRef();
 var ftpApplySudoPasswordInput = useRef();
 var tlsCertFileInput = useRef();

 const [ftpConfig, setFtpConfig] = useState({
  enabled: false,
  ftpUserName: "",
  ftpLocalRoot: "",
 });
 const [ftpCheckBoxChecked, setFtpCheckBoxChecked] = useState(false);
 const [certNames, setCertName] = useState({
  tls: "TLS Certificate",
 });
 const [sudoPasswordDialogOpen, setSudoPasswordDialogOpen] = useState(false);

 const fetchData = () => {
  fetch(fetchURLPrefix + "/api/fetchFTPconfig").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     setFtpConfig(json);
     setFtpCheckBoxChecked(json.enabled);
     ftpUserNameInput.current.value = json.ftpUserUsername;
     ftpRootInput.current.value = json.ftpLocalRoot;
    });
   } else {
    toast({
     title: "Failed to fetch FTP configuration",
     description: "Zentrox failed to fetch the current FTP configuration",
    });
   }
  });

  fetch(fetchURLPrefix + "/api/certNames").then((res) => {
   res.json().then((j) => {
    setCertName(j);
   });
  });
 };

 useEffect(fetchData, []);

 return (
  <Page name="Sharing">
   <h1 className="text-xl">
    Certificates{" "}
    <InfoButton
     title={"Certificates"}
     info={
      <>
       Zentrox automatically generates self signed certificates to provide an
       encrypted connection.
       <br />
       This connection is not protected from Man-In-The-Middle attacks, which is
       why it is recommended to use a SSL certificate by a Certificate
       Authority.
      </>
     }
    />
   </h1>
   <h2 className="font-semibold">TLS</h2>
   <input
    type="file"
    ref={tlsCertFileInput}
    hidden={true}
    onChange={() => {
     var fileForSubmit = tlsCertFileInput.current.files[0];
     if (fileForSubmit.size >= 1024 * 1024 * 1024 * 1) {
      toast({
       title: "File to big",
       description: "The file you provided was larger than 1GB",
      });
     }

     var fileName = tlsCertFileInput.current.files[0].name;

     if (fileName.split(".").reverse()[0].toLowerCase() != "pem") {
      toast({
       title: "Not a pem file",
       description: "Zentrox can only use pem certificates.",
      });
     }

     var formData = new FormData();
     formData.append("file", fileForSubmit);
     fetch(fetchURLPrefix + "/upload/tls", {
      method: "POST",
      body: formData,
     }).then((res) => {
      if (res.ok) {
       setCertName({
        tls: fileName,
       });
       tlsCertFileInput.current.value = "";
       toast({
        title: "Upload finished",
        description:
         "Zentrox successfully uploaded the new certificate. You need to manually restart Zentrox to start using the new certificate.",
        duration: 200000,
       });
      } else {
       toast({
        title: "Failed to upload TLS certificate",
        description:
         "Zentrox failed to upload the TLS certificate you provided",
       });
      }
     });
    }}
   />
   <Button
    onClick={() => {
     tlsCertFileInput.current.click();
    }}
    className="mr-1"
   >
    Upload
   </Button>{" "}
   <span className="text-neutral-600">{certNames.tls}</span>
   <h1 className="text-xl pt-3">
    FTP{" "}
    <InfoButton
     title={"File Transfer Protocol"}
     info={
      <>
       The FTP Protocol is used to transfer files. <br />
       Zentrox automatically encrypts the trafic using the provided
       certificates.
       <br />
      </>
     }
    />
   </h1>
   <div className="flex items-center">
    <Checkbox
     onCheckedChange={() => {
      setFtpCheckBoxChecked(!ftpCheckBoxChecked);
     }}
     id="ftpUserEnabledCheckBox"
     className="ml-1 mr-1 mb-1"
     checked={ftpCheckBoxChecked}
    />
    <label htmlFor="ftpUserEnabledCheckBox" className="mb-1">
     Enabled
    </label>
   </div>
   <Label className="block p-1">Username</Label>
   <Input
    type="text"
    ref={ftpUserNameInput}
    placeholder="Username"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Username"}
    info={
     <>
      The FTP username is the username that is used to connect to your FTP
      server.
     </>
    }
   />
   <br />
   <Label className="block p-1">Password</Label>
   <Input
    type="password"
    ref={ftpPassWordInput}
    placeholder="Password"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Password"}
    info={
     <>
      The FTP password is the password that is used to connect to your FTP
      server. <br />
      You should change it to prevent getting hacked.
      <br />
      By Default it is: <code>CHANGE_ME</code>
     </>
    }
   />
   <br />
   <Label className="block p-1">Root Directory</Label>
   <Input
    type="text"
    ref={ftpRootInput}
    placeholder="FTP Root Directory"
    className="inline-block mb-1"
    disabled={ftpCheckBoxChecked}
   />{" "}
   <InfoButton
    title={"FTP Root"}
    info={
     <>
      The FTP root directory is the directory that a connected FTP user can
      access. By Default it is: <code>$HOME</code> or <code>/home</code>
     </>
    }
   />
   <br />
   <Button
    className="mt-1"
    onClick={() => {
     if (ftpCheckBoxChecked !== ftpConfig.enabled) {
      setSudoPasswordDialogOpen(true);
     } else {
      fetch(fetchURLPrefix + "/api/updateFTPConfig", {
       method: "POST",
       headers: {
        "Content-Type": "application/json",
       },
       body: JSON.stringify({
        enableDisable: false,
        enableFTP: ftpCheckBoxChecked,
        ftpUserUsername: ftpUserNameInput.current.value,
        ftpLocalRoot: ftpRootInput.current.value,
        ftpUserPassword: ftpPassWordInput.current.value,
       }),
      }).then((res) => {
       if (res.ok) {
        toast({
         title: "FTP server updated",
         description: "Zentrox updated your FTP server configuration",
        });
       } else {
        toast({
         title: "FTP server error",
         description: "Failed to update FTP server configuration",
        });
       }
      });
     }
    }}
   >
    Apply
   </Button>
   <Dialog
    open={sudoPasswordDialogOpen}
    onOpenChange={setSudoPasswordDialogOpen}
   >
    <DialogContent>
     <DialogHeader>
      <DialogTitle>Elevated privileges</DialogTitle>
      <DialogDescription>
       Zentrox requires your sudo password for this action.
      </DialogDescription>
     </DialogHeader>
     <Input
      type="password"
      ref={ftpApplySudoPasswordInput}
      placeholder="Password"
     />
     <DialogFooter>
      <DialogClose asChild>
       <Button variant="outline">Close</Button>
      </DialogClose>
      <DialogClose
       onClick={() => {
        fetch(fetchURLPrefix + "/api/updateFTPConfig", {
         method: "POST",
         headers: {
          "Content-Type": "application/json",
         },
         body: JSON.stringify({
          enableDisable: false,
          enableFTP: ftpCheckBoxChecked,
          ftpUserUsername: ftpUserNameInput.current.value,
          ftpLocalRoot: ftpRootInput.current.value,
          ftpUserPassword: ftpPassWordInput.current.value,
          sudoPassword: ftpApplySudoPasswordInput.current.value,
         }),
        }).then((res) => {
         if (res.ok) {
          toast({
           title: "FTP server updated",
           description: "Zentrox updated your FTP server configuration",
          });
         } else {
          toast({
           title: "FTP server error",
           description: "Failed to update FTP server configuration",
          });
         }
        });
       }}
       asChild
      >
       <Button>
        <KeyIcon className="w-4 h-4 inline-block mr-1" />
        Proceed
       </Button>
      </DialogClose>
     </DialogFooter>
    </DialogContent>
   </Dialog>
  </Page>
 );
}

export default Sharing;
