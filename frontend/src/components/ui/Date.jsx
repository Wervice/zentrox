"use client";

import { useEffect } from "react";
import secondsToFormat from "../../lib/dates";
import { useState } from "react";

/**
 * @param {Object} param0
 * @param {boolean} param0.updating
 * @param {number} param0.interval
 * @param {number} param0.seconds
 * @param {string} param0.className
 * DateWrapper is a component that wraps around the secondsToFormat function.
 * It automatically chooses and sets a date format prerefence for the user.
 * By clicking the component, that format is changed.*/

function DateWrapper({ updating, interval, seconds, className }) {
  if (typeof window === "undefined") return;
  const userPreference = localStorage.getItem("dateFormat") || "8601"; // Get user preference and default to ISO-8601
  const options = ["8601", "European", "Short", "UNIX"];
  const [userSelection, setUserSelection] = useState(userPreference);
  const [time, setTime] = useState(
    secondsToFormat(seconds || 0, userPreference),
  );
  function updateTime(pref) {
    if (updating) {
      let dateObj = new Date();
      let c = Math.floor(dateObj.getTime() / 1000);
      setTime(secondsToFormat(c, pref));
    } else {
      setTime(secondsToFormat(seconds, pref));
    }
  }

  useEffect(() => {
    const iv = setInterval(() => {
      updateTime(userPreference);
    }, interval);
    return () => clearInterval(iv);
  }, [userPreference]);

  return (
    <span
      className={"inline-block cursor-pointer " + className}
      onClick={() => {
        let val =
          options[
            (options.findIndex((e) => userSelection == e) + 1) % options.length
          ];
        setUserSelection(val);
        updateTime(val);
        localStorage.setItem("dateFormat", val);
      }}
    >
      {time}
    </span>
  );
}

export default DateWrapper;
