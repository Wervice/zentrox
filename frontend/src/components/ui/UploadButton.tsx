import { ReactNode, useRef, useState } from "react";
import { Button } from "./button";

type Props = {
  onFilesChange: (f: FileList) => any;
  files: FileList | null;
  onClick: () => any | undefined;
  children: ReactNode;
};

export default function UploadButton({
  onFilesChange,
  files,
  onClick,
  children,
  ...props
}: Props) {
  var uploadInput = useRef<HTMLInputElement>(null);

  function getFirstFilename(): string | null {
    if (files === null) return files;
    if (files.length === 0) {
      return null;
    } else {
      return files.item(0)?.name || null;
    }
  }

  function handleClick() {
    uploadInput.current?.click();
    if (typeof onClick === "undefined") return;
    onClick();
  }

  function handleChange() {
    const files = uploadInput.current?.files;
    if (typeof files === "undefined" || files === null) {
      console.warn("No file has been selected");
    } else {
      onFilesChange(files);
    }
  }

  return (
    <>
      <input type="file" ref={uploadInput} onChange={handleChange} hidden />
      <span className="flex items-center space-x-2">
        <Button onClick={handleClick} {...props}>
          {children}
        </Button>
        <span>{getFirstFilename()}</span>
      </span>
    </>
  );
}
