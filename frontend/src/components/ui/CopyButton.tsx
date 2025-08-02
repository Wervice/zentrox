import { useState } from "react";
import { Button } from "./button";
import { CheckIcon, ClipboardIcon } from "lucide-react";

export default function CopyButton({ link, ...props }: { link: string }) {
  const [copied, setCopied] = useState(false);

  return (
    <Button
      onClick={() => {
        setCopied(true);
        setTimeout(() => {
          setCopied(false);
        }, 3000);
        navigator.clipboard.writeText(link);
      }}
      {...props}
    >
      <span className="flex items-center">
        {copied ? (
          <>
            <CheckIcon className="w-4 mr-1" /> Copied
          </>
        ) : (
          <>
            <ClipboardIcon className="w-4 mr-1" /> Copy
          </>
        )}
      </span>
    </Button>
  );
}
