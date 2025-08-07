"use client";

import { Button } from "@/components/ui/button";
import FileIcon from "@/components/ui/FileIcon";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";
import { toast } from "@/components/ui/use-toast";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { DownloadIcon, Loader2Icon, LoaderIcon } from "lucide-react";
import { useRef, useState } from "react";
import { saveAs } from "file-saver";

export default function Shared() {
  const [downloadButtonDisabled, setDownloadButtonDisabled] = useState(false);
  const [usePassword, setUsePassword] = useState(true);
  const [filename, setFilename] = useState("");
  const [metadataLoaded, setMetadataLoaded] = useState(false);
  const [invalidCode, setInvalidCode] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [filesize, setFilesize] = useState(0);
  const code =
    typeof window !== "undefined"
      ? window.location.search.substr(1).split("=")[1]
      : "";

  var passwordInput = useRef();

  function onPasswordInput() {
    setDownloadButtonDisabled(passwordInput.current.value === "");
  }

  function downloadFile() {
    setDownloading(true);
    fetch(fetchURLPrefix + "/shared/get", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code: code,
        password: usePassword ? passwordInput.current.value : null,
      }),
    }).then((res) => {
      setDownloading(false);
      if (res.ok) {
        res.blob().then((b) => {
          saveAs(b, filename);
        });
      } else {
        if (res.status === 403) {
          toast({
            title: "Bad password",
            description: "The provided password is invalid.",
          });
        } else if (res.status === 400) {
          toast({
            title: "Bad code",
            description: "The provided code is no longer active.",
          });
        } else if (res.status == 429) {
          toast({
            title: "Too many requests",
            description:
              "You have sent to many requests. Please wait one minute.",
          });
        } else {
          toast({
            title: "Unknown error",
          });
        }
      }
    });
  }

  useState(() => {
    if (metadataLoaded) return;
    fetch(fetchURLPrefix + "/shared/getMetadata", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        code,
      }),
    })
      .then((res) => {
        if (res.ok) {
          res.json().then((j) => {
            let fp = j.filepath;
            setFilename(fp.split("/")[fp.split("/").length - 1]);
            setFilesize(j.size);
            setUsePassword(j.use_password);
            setDownloadButtonDisabled(j.use_password);
            setMetadataLoaded(true);
          });
        } else {
          setInvalidCode(true);
        }
      })
      .catch((e) => {
        console.error(e);
        setInvalidCode(true);
      });
  }, []);

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
      <Toaster />
      <div className="flex items-center align-middle h-screen">
        <span className="m-3 text-center w-full">
          <span
            className={
              "inline-block p-5 rounded-xl bg-neutral-900 border-neutral-800 border " +
              (!metadataLoaded && "hidden")
            }
          >
            <h1 className="text-2xl font-semibold">Shared file</h1>
            <h2 className="font-light text-neutral-300 mb-2">
              <span className="inline-flex items-center">
                <FileIcon className="w-4 mr-1" filename={filename} /> {filename}{" "}
                ({prettyBytes(filesize)})
              </span>
            </h2>
            <Input
              type="password"
              className={"w-full mb-3 " + (!usePassword && "hidden")}
              placeholder="Password"
              onKeyUp={onPasswordInput}
              ref={passwordInput}
            />
            <Button
              className="w-[150px]"
              disabled={downloadButtonDisabled || downloading}
              onClick={downloadFile}
            >
              {downloading ? (
                <>
                  <Loader2Icon className="h-4 inline-block animate-spin duration-800" />{" "}
                  Downloading
                </>
              ) : (
                <>
                  <DownloadIcon className="h-4 inline-block" /> Download
                </>
              )}
            </Button>
          </span>
          <span
            className={
              "inline-block p-5 rounded-xl bg-neutral-900 border-neutral-800 border " +
              (!invalidCode && "hidden")
            }
          >
            <h1 className="text-2xl font-semibold">Invalid code</h1>
            No shared file was found with this link.
          </span>
        </span>
      </div>
    </>
  );
}
