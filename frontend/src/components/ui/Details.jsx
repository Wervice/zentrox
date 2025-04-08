import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import { useEffect, useState } from "react";

function Details({ children, title, open, className }) {
  const [innerOpen, setInnerOpen] = useState(open);
  const I = innerOpen ? ChevronUpIcon : ChevronDownIcon;

  return (
    <>
      <div
        className={
          "flex items-center mb-1 font-bold w-full cursor-pointer " +
            className || ""
        }
        onClick={() => {
          setInnerOpen(!innerOpen);
        }}
      >
        <I className="h-4 opacity-50 inline-block mr-1" /> {title}
      </div>
      <span hidden={!innerOpen}>{children}</span>
    </>
  );
}

export { Details };
