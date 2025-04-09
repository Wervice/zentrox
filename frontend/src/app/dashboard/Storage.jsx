import { Button } from "@/components/ui/button.jsx";
import {
  HardDriveIcon,
  RepeatIcon,
  LayoutPanelTopIcon,
  TagIcon,
  MapPinIcon,
  WeightIcon,
  UserIcon,
  MountainIcon,
  PieChartIcon,
} from "lucide-react";
import { useEffect, useState } from "react";
import "./table.css";
import "./scroll.css";
import { Toaster } from "@/components/ui/toaster";
import { useToast } from "@/components/ui/use-toast";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import "./scroll.css";
import Page from "@/components/ui/PageWrapper";

// const fetchURLPrefix = "";
const fetchURLPrefix = require("@/lib/fetchPrefix");

/**
 * @param {string} value to check
 * @description Returns the string or "N/A" when the string is not defined*/
function na(value) {
  if (typeof value === "undefined" || value === null) return "N/A";
  return value;
}

function Storage() {
  const { toast } = useToast();
  const [drivesList, setDrivesList] = useState([]);
  const [driveInformation, setDriveInformation] = useState({
    drives: {
      model: "N/A",
      path: "N/A",
      owner: "N/A",
      mountpoint: "",
      size: 0,
    },
    ussage: [],
  });
  const [currentDrive, setCurrentDrive] = useState([]);
  const [driveInformationDialogOpen, setDriveInformationDialogOpen] =
    useState(false);

  useEffect(() => {
    fetchDrivesList();
  }, []);

  function fetchDrivesList() {
    fetch(fetchURLPrefix + "/api/driveList").then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setDrivesList(json["drives"]);
        });
      } else {
        toast({
          title: "Failed to fetch drives list",
          description:
            "Zentrox failed to fetch a list of all connected storage mediums.",
        });
      }
    });
  }

  function showDriveDetails(driveName) {
    fetch(
      fetchURLPrefix + "/api/driveInformation/" + encodeURIComponent(driveName),
    )
      .then((res) => {
        if (res.ok) {
          res.json().then((json) => {
            setDriveInformation(json);
            setDriveInformationDialogOpen(true);
            setCurrentDrive(driveName);
          });
        } else {
          toast({
            title: "Failed to fetch drive information",
            description: "Zentrox failed to fetch drive details",
          });
        }
      })
      .catch(() => {
        toast({
          title: "Failed to fetch drive information",
          description: "Zentrox failed to fetch drive details",
        });
      });
  }

  function DriveEntry({ entry, inset = 0 }) {
    var children = <></>;
    if (entry.children != null) {
      children = entry.children.map((entry) => {
        return <DriveEntry entry={entry} inset={inset + 1} />;
      });
    }

    return (
      <span>
        <span
          className="w-full p-4 bg-transparent border border-neutral-800 border-x-transparent block cursor-default select-none hover:bg-neutral-800 hover:transition-bg hover:duration-400 duration-200 focus:bg-neutral-800 focus:duration-50"
          style={{
            paddingLeft: 16 + inset * 10,
          }}
          onClick={() => {
            showDriveDetails(entry.name);
          }}
        >
          {(function (entry) {
            if (entry.name.startsWith("loop")) {
              return (
                <RepeatIcon className="inline-block h-6 w-6 pr-1 text-neutral-700" />
              );
            } else if (inset != 0) {
              return (
                <LayoutPanelTopIcon className="inline-block h-6 w-6 pr-1" />
              );
            } else {
              return <HardDriveIcon className="inline-block h-6 w-6 pr-1" />;
            }
          })(entry)}{" "}
          {entry.name}
        </span>
        {children}
      </span>
    );
  }

  /**
   * @param {number} bytes
   * @description Converts the unit Bytes into a higher unit and add a unit symbol*
   * @returns {string} */
  function bytesUnitToOther(bytes) {
    if (bytes >= 1024 * 1024 * 1024) {
      return Math.round(bytes / (1024 * 1024 * 1024)) + " GB";
    } else if (bytes >= 1024 * 1024) {
      return Math.round(bytes / (1024 * 1024)) + " MB";
    } else if (bytes >= 1024) {
      return Math.round(bytes / 1024) + " KB";
    } else {
      return bytes + " B";
    }
  }

  var driveCapacity = "N/A";
  var drive;
  for (drive of driveInformation.ussage) {
    if (
      drive[0] === driveInformation.drives.mountpoint ||
      drive[0] == driveInformation.drives.path
    ) {
      driveCapacity = drive[4] + "%";
    }
  }

  return (
    <>
      <Toaster />
      <Dialog
        open={driveInformationDialogOpen}
        onOpenChange={setDriveInformationDialogOpen}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{currentDrive}</DialogTitle>
          </DialogHeader>
          <DialogDescription className="text-white">
            <b className="block mb-1">
              <TagIcon className="w-4 h-4 inline" /> Model
            </b>
            {na(driveInformation.drives.model)}
            <br />
            <b className="block mb-1">
              <MapPinIcon className="w-4 h-4 inline" /> Path
            </b>
            {na(driveInformation.drives.path)} <br />
            <b className="block mb-1">
              <WeightIcon className="w-4 h-4 inline" /> Size
            </b>
            {na(bytesUnitToOther(driveInformation.drives.size))} <br />
            <b className="block mb-1">
              <UserIcon className="w-4 h-4 inline" /> Owner
            </b>
            {na(driveInformation.drives.owner)} <br />
            <b className="block mb-1">
              <MountainIcon className="w-4 h-4 inline" /> Mountpoint
            </b>
            {na(driveInformation.drives.mountpoint)}
            <b className="block mb-1">
              <PieChartIcon className="w-4 h-4 inline" /> Usage (Capacity)
            </b>
            {na(driveCapacity)}
          </DialogDescription>
          <DialogFooter>
            <DialogClose asChild>
              <Button>Close</Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Page name="Storage">
        <div
          className="rounded-xl m-2 overflow-hidden overflow-y-scroll border-2 border-neutral-800"
          style={{ maxHeight: "calc(100vh - 180px)" }}
        >
          {drivesList
            .sort((a) => {
              if (a.name.includes("loop")) return 1;
              return -1;
            })
            .map((entry, i) => {
              return <DriveEntry entry={entry} key={i} />;
            })}
        </div>
      </Page>
    </>
  );
}

export default Storage;
