import { ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import { useEffect, useState } from "react";

function Details({
  children,
  title,
  open,
  className,
  rememberState = false,
  name,
}) {
  var defaultOpenValue = open;
  var localStorageValue =
    typeof window !== "undefined"
      ? localStorage.getItem("detailsOpenStates") || "{}"
      : "{}";
  var localStorageValueParsed = JSON.parse(localStorageValue);
  if (localStorageValueParsed[name] !== undefined) {
    defaultOpenValue = localStorageValueParsed[name];
  }

  const [innerOpen, setInnerOpen] = useState(defaultOpenValue);
  const I = innerOpen ? ChevronUpIcon : ChevronDownIcon;

  function remember(newState) {
    if (rememberState) {
      var currentValue =
        typeof window !== "undefined"
          ? localStorage.getItem("detailsOpenStates")
          : "{}";
      var currentValueParsed;
      if (currentValue === null) {
        currentValueParsed = {};
      } else {
        currentValueParsed = JSON.parse(currentValue);
      }
      currentValueParsed[name] = newState;
      if (typeof window !== "undefined") {
        localStorage.setItem(
          "detailsOpenStates",
          JSON.stringify(currentValueParsed),
        );
      }
    }
  }

  return (
    <>
      <div
        className={
          "flex items-center mb-1 font-semibold w-full cursor-pointer select-none "
        }
        onClick={() => {
          remember(!innerOpen);
          setInnerOpen(!innerOpen);
        }}
      >
        <I className="h-4 opacity-50 inline-block mr-1" /> {title}
      </div>
      <span hidden={!innerOpen} className={className || ""}>
        {children}
      </span>
    </>
  );
}

export { Details };
