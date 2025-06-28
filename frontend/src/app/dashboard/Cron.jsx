import Page from "@/components/ui/PageWrapper";
import { ActionTd, ActionTh, Table, Td, Th, Tr } from "@/components/ui/table";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import useNotification from "@/lib/notificationState";
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogTitle,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogContent,
  AlertDialogAction,
} from "@/components/ui/alert-dialog";
import {
  HourglassIcon,
  Loader2,
  PlayIcon,
  PlusIcon,
  TrashIcon,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  Placeholder,
  PlaceholderIcon,
  PlaceholderSubtitle,
} from "@/components/ui/placeholder";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogDescription,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectValue,
  SelectTrigger,
} from "@/components/ui/select";
import Label from "@/components/ui/ShortLabel";
import { Input } from "@/components/ui/input";

function digitToVisual(digit) {
  function trailingPlural(v) {
    const parsed = Number.parseInt(v);
    if (!Number.isNaN(parsed)) {
      if (parsed == 1) {
        return "st";
      } else if (parsed == 2) {
        return "nd";
      } else {
        return "th";
      }
    } else {
      return "";
    }
  }

  if (digit == "Any") {
    return "Any";
  } else if (digit.Value !== undefined) {
    return digit.Value;
  } else if (digit.Range !== undefined) {
    return (
      <span>
        {digit.Range[0]} - {digit.Range[1]}
      </span>
    );
  } else if (digit.List !== undefined) {
    let string = "";
    for (var value of digit.List) {
      string += (string !== "" && ", ") + value;
    }
    return string;
  } else if (digit.Repeating !== undefined) {
    return (
      <>
        Starting {digit.Repeating[0]} every {digit.Repeating[1]}
        {trailingPlural(digit.Repeating[1])} iteration
        {digit.Repeating[2] > 1 && "s"}
      </>
    );
  } else if (digit.Composed !== undefined) {
    return digit.Composed;
  }
}

function dowToVisual(dow) {
  if (typeof dow == "string") {
    return dow.slice(0, 3);
  } else {
    return digitToVisual(dow.Digit);
  }
}

function monthToVisual(m) {
  if (typeof m == "string") {
    return m.slice(0, 3);
  } else {
    return digitToVisual(m.Digit);
  }
}

function timeOf(job) {
  const cn = "p-1 rounded bg-white/10";

  return (
    <span className="whitespace-nowrap block">
      <span title="minute" className={cn}>
        {digitToVisual(job.minute)}
      </span>{" "}
      <span title="hour" className={cn}>
        {digitToVisual(job.hour)}
      </span>{" "}
      <span title="day of the month" className={cn}>
        {digitToVisual(job.day_of_month)}
      </span>{" "}
      <span title="month" className={cn}>
        {monthToVisual(job.month)}
      </span>{" "}
      <span title="day of the week" className={cn}>
        {dowToVisual(job.day_of_week)}
      </span>
    </span>
  );
}

function CronjobRow({ job, index, onUpdate }) {
  const [running, setRunning] = useState(false);
  const [deleteJobModalOpen, setDeleteJobModalOpen] = useState(false);
  const RunIcon = !running ? PlayIcon : Loader2;
  const { deleteNotification, notify, notifications } = useNotification();

  function run() {
    fetch(fetchURLPrefix + "/api/runCronjobCommand", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        command: job.command,
      }),
    }).then((res) => {
      if (res.ok) {
        setRunning(true);
        res.text().then((uuid) => {
          const iv = setInterval(() => {
            fetch(fetchURLPrefix + "/api/fetchJobStatus/" + uuid).then(
              (res) => {
                if (res.status == 200) {
                  setRunning(false);
                  notify("Finished running " + job.command);
                  clearInterval(iv);
                } else if (res.status == 202) {
                  setRunning(true);
                } else {
                  notify("Failed to run " + job.command);
                  clearInterval(iv);
                }
              },
            );
          }, 500);
        });
      } else {
        notify("Failed to start cron job execution.");
      }
    });
  }

  function deleteJob() {
    setDeleteJobModalOpen(true);
  }

  function deleteJobFetch(_) {
    fetch(fetchURLPrefix + "/api/deleteCronjob", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        index: index,
        variant: job.interval ? "interval" : "specific",
      }),
    }).then((res) => {
      if (!res.ok) {
        notify("Failed to delete cronjob");
      } else {
        notify("Removed cronjob");
        onUpdate();
      }
    });
  }

  return (
    <>
      <AlertDialog
        open={deleteJobModalOpen}
        onOpenChange={setDeleteJobModalOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete cronjob?</AlertDialogTitle>
          </AlertDialogHeader>
          <p>Do you really want to delete this cronjob?</p>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={deleteJobFetch}>Yes</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Tr>
        <ActionTd className="space-x-1 whitespace-nowrap">
          <span
            title={!running ? "Run process" : "Waiting for process to finish"}
          >
            <RunIcon
              onClick={run}
              className={
                "w-4 h-4 opacity-75 hover:opacity-100 transition-all duration-200 inline-block cursor-pointer " +
                (running && "animate-spin")
              }
            />
          </span>
          <TrashIcon
            onClick={deleteJob}
            className="w-4 h-4 opacity-75 text-red-500 hover:opacity-100 transition-all duration-200 inline-block cursor-pointer"
          />
        </ActionTd>
        <Td className={"w-[300px]"}>
          {job.interval ? job.interval : timeOf(job)}
        </Td>
        <Td className={"w-max"}>
          <code>{job.command}</code>
        </Td>
      </Tr>
    </>
  );
}

function CreationForm({ onUpdate }) {
  const { deleteNotification, notify, notifications } = useNotification();

  const [cronjobVariant, setCronjobVariant] = useState("specific");
  const [selectedInterval, setSelectedInterval] = useState("");

  const [akaDay, setAkaDay] = useState(null);
  const [akaMonth, setAkaMonth] = useState(null);

  var specificDayOfWeekInput = useRef(null);
  var specificMinuteInput = useRef(null);
  var specificHourInput = useRef(null);
  var specificDayOfMonthInput = useRef(null);
  var specificMonthInput = useRef(null);
  var commandInput = useRef(null);

  function translateDay() {
    let value = specificDayOfWeekInput.current.value;
    let valueParsed = Number.parseInt(value);
    if (!Number.isNaN(valueParsed)) {
      setAkaDay(
        {
          0: "Sunday",
          1: "Monday",
          2: "Tuesday",
          3: "Wednesday",
          4: "Thursday",
          5: "Friday",
          6: "Saturday",
        }[valueParsed] || "Unknown day",
      );
    } else {
      setAkaDay(null);
    }
  }

  function translateMonth() {
    let value = specificMonthInput.current.value;
    let valueParsed = Number.parseInt(value);
    if (!Number.isNaN(valueParsed)) {
      setAkaMonth(
        {
          1: "January",
          2: "February",
          3: "March",
          4: "April",
          5: "May",
          6: "June",
          7: "July",
          8: "August",
          9: "September",
          10: "October",
          11: "November",
          12: "December",
        }[valueParsed] || "Unknown month",
      );
    } else {
      setAkaMonth(null);
    }
  }

  function createCronjob() {
    setAkaMonth(null);
    setAkaDay(null);

    let json;
    if (cronjobVariant === "specific") {
      json = {
        variant: "specific",
        command: commandInput.current.value,
        minute: specificMinuteInput.current.value,
        hour: specificHourInput.current.value,
        day_of_month: specificDayOfMonthInput.current.value,
        month: specificMonthInput.current.value,
        day_of_week: specificDayOfWeekInput.current.value,
        interval: null,
      };
    } else {
      json = {
        variant: "interval",
        command: commandInput.current.value,
        interval: selectedInterval,
        minute: null,
        hour: null,
        day_of_month: null,
        month: null,
        day_of_week: null,
      };
    }

    fetch(fetchURLPrefix + "/api/createCronjob", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(json),
    }).then((res) => {
      if (res.ok) {
        onUpdate();
      } else {
        notify("Failed to create a new cronjob.");
      }
    });
  }

  return (
    <Dialog>
      <DialogTrigger>
        <Button variant="secondary" className="flex items-center">
          <PlusIcon className="w-4 mr-1" />
          New cronjob
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Create cronjob</DialogTitle>
          <DialogDescription>
            Create a task that is run repeatedly at a fixed interval or at a
            specified time.
          </DialogDescription>
        </DialogHeader>
        <p>
          <Label>Cronjob variant</Label>
          <Select value={cronjobVariant} onValueChange={setCronjobVariant}>
            <SelectTrigger className="mb-2">
              <SelectValue placeholder="Choose a variant" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="specific">Specific</SelectItem>
              <SelectItem value="interval">Interval</SelectItem>
            </SelectContent>
          </Select>
          <span hidden={cronjobVariant === "specific"}>
            <Label>Interval</Label>
            <Select
              value={selectedInterval}
              onValueChange={setSelectedInterval}
            >
              <SelectTrigger className="mb-2">
                <SelectValue placeholder="Choose an interval" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="yearly">Yearly</SelectItem>
                <SelectItem value="monthly">Monthly</SelectItem>
                <SelectItem value="weekly">Weekly</SelectItem>
                <SelectItem value="daily">Daily</SelectItem>
                <SelectItem value="hourly">Hourly</SelectItem>
                <SelectItem value="reboot">At reboot</SelectItem>
              </SelectContent>
            </Select>
          </span>
          <span hidden={cronjobVariant === "interval"}>
            <Label>Specific time pattern</Label>
            <span className="flex items-center w-full text-neutral-300 mb-1">
              <Input
                ref={specificMinuteInput}
                placeholder="Minute"
                className="mr-2"
                type="text"
              />
            </span>

            <span className="flex items-center w-full text-neutral-300 mb-1">
              <Input
                ref={specificHourInput}
                placeholder="Hour"
                className="mr-2"
                type="text"
              />
            </span>

            <span className="flex items-center w-full text-neutral-300 mb-1">
              <Input
                ref={specificDayOfMonthInput}
                placeholder="Day of month"
                className="mr-2"
                type="text"
              />
            </span>

            <span className="flex items-center w-full text-neutral-300 mb-1">
              <Input
                onChange={translateMonth}
                ref={specificMonthInput}
                placeholder="Month"
                className="mr-2"
                type="text"
              />{" "}
              {akaMonth && "≈ "} {akaMonth || ""}
            </span>

            <span className="flex items-center w-full text-neutral-300 mb-1">
              <Input
                onChange={translateDay}
                ref={specificDayOfWeekInput}
                placeholder="Day"
                className="mr-2"
                type="text"
              />{" "}
              {akaDay && "≈ "} {akaDay || ""}
            </span>
          </span>

          <Label>Command</Label>
          <Input className="w-full" ref={commandInput} placeholder="Command" />
        </p>
        <DialogFooter>
          <DialogClose asChild>
            <Button
              onClick={() => {
                setAkaMonth(null);
                setAkaDay(null);
              }}
              variant="outline"
            >
              Cancel
            </Button>
          </DialogClose>
          <DialogClose asChild>
            <Button onClick={createCronjob}>Create</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default function Cron() {
  const [cronJobList, setCronjobList] = useState(null);
  const { _deleteNotification, notify, _notifications } = useNotification();

  function fetchCronjobList() {
    fetch(fetchURLPrefix + "/api/listCronjobs/current/None", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setCronjobList(json);
        });
      } else {
        notify("Failed to fetch list of installed cronjobs");
      }
    });
  }

  useEffect(fetchCronjobList, []);
  useEffect(() => {
    let i = setInterval(() => {
      fetchCronjobList();
    }, 2500);
    return () => clearInterval(i);
  }, []);

  return (
    <>
      {" "}
      <Page name="Cronjobs">
        {" "}
        <small className="block opacity-75">
          Only shows cronjobs of the user that is running Zentrox.
        </small>
        <CreationForm onUpdate={fetchCronjobList} />
        {cronJobList && cronJobList.crontab_exists ? (
          cronJobList.specific_jobs.length + cronJobList.interval_jobs.length >
          0 ? (
            <>
              <Table>
                <Tr>
                  <ActionTh />
                  <Th className="w-fit max-w-[500px]">Expression</Th>
                  <Th className="w-fit max-w-[500px]">Command</Th>
                </Tr>
                {cronJobList.specific_jobs.map((j, index) => {
                  return (
                    <CronjobRow
                      job={j}
                      index={index}
                      onUpdate={fetchCronjobList}
                    />
                  );
                })}
                {cronJobList.interval_jobs.map((j, index) => {
                  return (
                    <CronjobRow
                      job={j}
                      index={index}
                      onUpdate={fetchCronjobList}
                    />
                  );
                })}
              </Table>
            </>
          ) : (
            <>
              <Placeholder>
                <PlaceholderIcon icon={HourglassIcon} />
                <PlaceholderSubtitle>
                  No installed cronjobs for the current user
                </PlaceholderSubtitle>
              </Placeholder>
            </>
          )
        ) : (
          <>
            <>
              <Placeholder>
                <PlaceholderIcon icon={HourglassIcon} />
                <PlaceholderSubtitle>
                  No crontab file for this user
                </PlaceholderSubtitle>
              </Placeholder>
            </>
          </>
        )}
      </Page>
    </>
  );
}
