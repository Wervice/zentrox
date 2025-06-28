import { Button } from "@/components/ui/button";
import {
  Clock,
  SkipBackIcon,
  SkipForward,
  StepBackIcon,
  StepForward,
} from "lucide-react";
import { useRef, useState } from "react";
import "./no-clock-icon.css";

export default function CalendarButton({
  placeholder,
  onValueChange = (_) => {},
  className,
  confirmMode = false,
  variant,
}) {
  var calendarModalRef = useRef();
  var calendarButtonRef = useRef();
  const [calendarModalStyle, setCalendarModalStyle] = useState({
    display: "none",
  });
  const [calendarModalVisible, setCalendarModalVisible] = useState(false);
  const [currentSelectedTime, setCurrentSelectedTime] = useState(Date.now());

  const Calendar = () => {
    function human(time) {
      const now = new Date();
      now.setTime(time);
      const options = {
        month: "long", // "Oct" for October
        year: "numeric", // "2024" for the year
      };

      return now.toLocaleDateString("en-GB", options);
    }

    function z(v) {
      if (v >= 10) {
        return v;
      } else {
        return "0" + v;
      }
    }

    function grid() {
      var date = new Date();
      date.setTime(currentSelectedTime); // Assuming currentSelectedTime is a valid timestamp
      date.setDate(1); // Set the date to the first day of the month
      const i =
        new Date(date.getUTCFullYear(), date.getMonth() + 1, 0).getUTCDate() +
        1;

      var result = [];

      for (let num = 1; num <= i; num += 7) {
        // Create a sub-array from num to num + 6 (inclusive)
        const subArray = [];
        for (let j = num; j < num + 7 && j <= i; j++) {
          subArray.push(j);
        }
        result.push(subArray);
      }

      return result;
    }

    const skipIconClass =
      "inline-block w-4 h-4 text-neutral-500 hover:text-white";

    return (
      <>
        <span
          className="fixed z-10 top-0 left-0 w-screen h-screen"
          onClick={() => toggleCalendar()}
          hidden={!calendarModalVisible}
        ></span>
        <div
          ref={calendarModalRef}
          className="absolute z-20 h-30 rounded bg-neutral-950 border-neutral-700 p-2 border text-center shadow-lg"
          style={calendarModalStyle}
        >
          <span className="relative left-1 w-1/2 inline-block text-left">
            <SkipBackIcon
              onClick={() => {
                var date = new Date(currentSelectedTime);
                if (date.getFullYear() > 0) {
                  date.setFullYear(date.getFullYear() - 1);
                }
                setCurrentSelectedTime(date.getTime());
              }}
              className={skipIconClass}
            />
            <StepBackIcon
              onClick={() => {
                var date = new Date(currentSelectedTime);
                let currentMonth = date.getMonth() + 1;
                let currentYear = date.getFullYear();
                let resultMonth;
                let resultYear;
                if (currentMonth !== 0) {
                  resultMonth = currentMonth - 2;
                  resultYear = currentYear;
                } else {
                  resultYear = currentYear - 1;
                  resultMonth = 11;
                }
                date.setFullYear(resultYear);
                date.setMonth(resultMonth);
                setCurrentSelectedTime(date.getTime());
              }}
              className={skipIconClass}
            />
          </span>

          <span className="relative right-1 w-1/2 inline-block text-right">
            <StepForward
              onClick={() => {
                var date = new Date(currentSelectedTime);
                let currentMonth = date.getMonth() + 1;
                let currentYear = date.getFullYear();
                let resultMonth;
                let resultYear;
                if (currentMonth !== 12) {
                  resultMonth = currentMonth;
                  resultYear = currentYear;
                } else {
                  resultYear = currentYear + 1;
                  resultMonth = 0;
                }
                date.setFullYear(resultYear);
                date.setMonth(resultMonth);
                setCurrentSelectedTime(date.getTime());
              }}
              className={skipIconClass}
            />

            <SkipForward
              onClick={() => {
                var date = new Date(currentSelectedTime);
                date.setFullYear(date.getFullYear() + 1);
                setCurrentSelectedTime(date.getTime());
              }}
              className={skipIconClass}
            />
          </span>
          <span className="text-center font-medium text-sm mb-1 block w-full align-middle">
            <span class="text-neutral-500 block pb-1">
              {human(currentSelectedTime)}
            </span>
            <input
              type="time"
              className="border-none bg-transparent focus:outline-none shadow-black"
              onBlur={(e) => {
                var date = new Date(currentSelectedTime);
                var time = e.target.value.split(":");
                date.setHours(time[0], time[1]);
                if (!confirmMode) {
                  onValueChange(date.getTime());
                }
                setCurrentSelectedTime(date.getTime());
              }}
              onKeyDown={(e) => {
                if (e.key == "Enter") {
                  e.target.blur();
                }
              }}
              defaultValue={(function () {
                var date = new Date(currentSelectedTime);
                return `${z(date.getHours())}:${z(date.getMinutes())}`;
              })()}
            />
            <br />
            <div className="text-left m-auto w-fit">
              {grid().map((r, i) => {
                return (
                  <span key={i}>
                    {r.map((e, i) => {
                      return (
                        <span
                          key={i}
                          onClick={() => {
                            let date = new Date();
                            date.setTime(currentSelectedTime);
                            date.setDate(e);
                            if (!confirmMode) {
                              onValueChange(date.getTime());
                            }
                            setCurrentSelectedTime(date.getTime());
                          }}
                          className={
                            "w-9 h-9 text-center inline-block rounded align-middle cursor-pointer duration-150 " +
                            (function () {
                              if (
                                new Date(currentSelectedTime).getDate() == e
                              ) {
                                return "text-black bg-white";
                              } else {
                                return "";
                              }
                            })()
                          }
                          style={{
                            lineHeight: "34px",
                          }}
                        >
                          {e}
                        </span>
                      );
                    })}
                    <br />
                  </span>
                );
              })}
            </div>
          </span>
          {confirmMode ? (
            <>
              <Button
                className="mb-2"
                onClick={() => {
                  onValueChange(currentSelectedTime);
                  toggleCalendar();
                }}
              >
                Confirm
              </Button>
            </>
          ) : (
            <></>
          )}
        </div>
      </>
    );
  };

  const toggleCalendar = () => {
    var button = calendarButtonRef.current;

    // Get the button's position and size
    var rect = button.getBoundingClientRect();

    // Position the floating div just below the button
    if (!calendarModalVisible) {
      setCalendarModalStyle({
        top: rect.bottom + window.scrollY + 10 + "px",
        left: rect.left - 35 + "px",
        display: "block",
        width: "fit-content",
        animation: "fade-in ease-in 100ms",
      });
      setTimeout(() => {
        setCalendarModalStyle({
          top: rect.bottom + window.scrollY + 10 + "px",
          left: rect.left - 35 + "px",
          display: "block",
          width: "fit-content",
          opacity: "1",
          visibility: "visible",
        });
      }, 80);
      setCalendarModalVisible(true);
    } else {
      setCalendarModalStyle({
        top: rect.bottom + window.scrollY + 10 + "px",
        left: rect.left - 35 + "px",
        display: "block",
        width: "fit-content",
        animationName: "fade-out",
        animationDuration: "100ms",
      });
      setTimeout(() => {
        setCalendarModalStyle({
          top: rect.bottom + window.scrollY + "px",
          display: "none",
        });
        setCalendarModalVisible(false);
      }, 100);
    }
  };

  return (
    <>
      <Calendar />
      <Button
        className={"relative border-neutral-600 align-middle " + className}
        variant={variant}
        ref={calendarButtonRef}
        onClick={toggleCalendar}
      >
        <Clock className="w-4 h-4 mr-1" />{" "}
        {placeholder == undefined ? "Select date and time" : placeholder}
      </Button>
    </>
  );
}

export { CalendarButton };
