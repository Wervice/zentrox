import { Button } from "@/components/ui/button";
import {
  RefreshCcw,
  HardDriveIcon,
  Package2,
  Loader2,
  CircleX,
  TrashIcon,
  CircleCheck,
  BotIcon,
  CircleAlert,
  DownloadIcon,
  DatabaseIcon,
  SparklesIcon,
  PackageIcon,
} from "lucide-react";
import { useEffect, useState, useRef } from "react";
import "./table.css";
import "./scroll.css";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
import Page from "@/components/ui/PageWrapper";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import useNotification from "@/lib/notificationState";
import secondsToFormat from "@/lib/dates";
import { Details } from "@/components/ui/Details";
import {
  Placeholder,
  PlaceholderIcon,
  PlaceholderSubtitle,
} from "@/components/ui/placeholder";
import SudoDialog from "@/components/ui/SudoDialog";

function startTask(adress, options = {}, interval = 500) {
  return new Promise((resolve, reject) => {
    fetch(adress, options)
      .then((res) => {
        if (res.ok) {
          res.text().then((uuid) => {
            let ivd = setInterval(() => {
              fetch(fetchURLPrefix + "/api/fetchJobStatus/" + uuid).then(
                (checkRes) => {
                  if (checkRes.status === 200) {
                    clearInterval(ivd);
                    resolve(checkRes);
                  } else if (
                    checkRes.status === 422 ||
                    checkRes.status === 500
                  ) {
                    clearInterval(ivd);
                    reject(checkRes);
                  }
                },
              );
            }, interval);
          });
        }
      })
      .catch((err) => {
        console.error(err);
      });
  });
}

function Packages() {
  const { deleteNotification, notify, notifications } = useNotification();
  const [packagePopUpConfig, setPackagePopUp] = useState({
    visible: false,
    mode: "",
    packageName: "",
  });
  var databaseUpdateSudoPasswordInput = useRef();
  const [installedPackages, setInstalledPackages] = useState([]);
  const [otherPackages, setOtherPackages] = useState([]);
  const [canProvideUpdates, setCanProvideUpdates] = useState(false);
  const [updates, setUpdates] = useState([]);
  const [packageManager, setPackageManager] = useState("");
  const [autoRemovePackages, setAutoRemovePackages] = useState([]);
  const [visible, setVisibility] = useState(false);
  const [packageSearchValue, setPackageSearchValue] = useState("");
  const [databaseUpdateButton, setDatabaseUpdateButton] = useState("default");
  const [activeTasks, setActiveTasks] = useState({});
  const [lastDatabaseUpdate, setLastDatabaseUpdate] = useState(0);
  const [updateDatabaseModalOpen, setUpdateDatabaseModalOpen] = useState(false);

  useEffect(() => fetchPackageList(), []);

  function closePackagePopUp() {
    setPackagePopUp({
      visible: false,
      mode: "",
      packageName: "",
    });
  }

  function fetchPackageList() {
    if (installedPackages.length + otherPackages.length !== 0) return;
    fetch(fetchURLPrefix + "/api/packageDatabase/false", {
      headers: {
        "Content-Type": "application/json",
      },
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setInstalledPackages(Array.from(json["packages"]));
          setOtherPackages(Array.from(json["others"]));
          setCanProvideUpdates(json.canProvideUpdates);
          setUpdates(json.updates);
          setPackageManager(json.packageManager);
          setVisibility(true);
          setLastDatabaseUpdate(json.lastDatabaseUpdate);
        });
      } else {
        notify("Failed to retrieve list of packages");
        setVisibility(false);
      }
    });

    fetch(fetchURLPrefix + "/api/listOrphanedPackages", {
      headers: {
        "Content-Type": "application/json",
      },
    }).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setAutoRemovePackages(json["packages"]);
        });
      } else {
        notify("Failed to retrieve list of packages");
        setVisibility(false);
      }
    });
  }

  function installPackage(packageName, password) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/installPackage", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        packageName: packageName,
        sudoPassword: password,
      }),
    })
      .then(() => {
        closePackagePopUp();
        setInstalledPackages([...installedPackages, packageName]);
        setOtherPackages(otherPackages.filter((e) => e !== packageName));
        fetchPackageList();

        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        notify("Installed " + packageName + " successfully");
      })
      .catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });
        notify("Failed to install " + packageName);
      });
  }

  function removePackage(packageName, password) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/removePackage", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        packageName: packageName,
        sudoPassword: password,
      }),
    })
      .then(() => {
        closePackagePopUp();
        setInstalledPackages(
          installedPackages.filter((entry) => entry !== packageName),
        );
        setAutoRemovePackages(
          autoRemovePackages.filter((entry) => entry !== packageName),
        );
        setOtherPackages([...otherPackages, packageName]);
        fetchPackageList();

        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        notify("Removed " + packageName + " successfully");
      })
      .catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });

        notify("Failed to remove " + packageName);
      });
  }

  function updatePackage(packageName, password) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/updatePackage", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        packageName: packageName,
        sudoPassword: password,
      }),
    })
      .then(() => {
        closePackagePopUp();
        setUpdates(updates.filter((e) => e !== packageName));
        fetchPackageList();

        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        notify("Updated package " + packageName);
      })
      .catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });

        notify("Failed to update " + packageName);
      });
  }

  /**
   * @param {string} mode
   * @param {function} stateFn
   * @param {string} packageName */
  function packageActionPopUp(packageName, mode) {
    setPackagePopUp({
      visible: true,
      mode: mode,
      packageName,
    });
  }

  function PackageBox({ packageName, task }) {
    const [buttonState, setButtonState] = useState("default");

    useEffect(() => {
      if (activeTasks[packageName] === "working") {
        setButtonState("working");
      } else if (activeTasks[packageName] === "failed") {
        setButtonState("failed");
      }
    }, [activeTasks]);

    return (
      <div
        className="inline-block p-4 mr-2 mt-2 w-80 h-24 border border-zinc-900 md:w-52 text-white rounded-xl align-top relative"
        title={packageName}
      >
        <span className="block mb-1">
          {packageName.length > 20
            ? packageName.substring(0, 17) + "..."
            : packageName}
        </span>

        <Button
          className={
            "block right-2 bottom-2 absolute " +
            {
              remove:
                "bg-red-500 hover:bg-red-500 hover:brightness-95 text-white",
              install: "",
              update:
                "bg-blue-500 hover:bg-blue-500 hover:brightness-95 text-white",
            }[task]
          }
          onClick={() => {
            packageActionPopUp(packageName, task);
          }}
        >
          {(function () {
            if (task === "remove" && buttonState === "default") {
              return "Remove";
            } else if (task === "install" && buttonState === "default") {
              return "Install";
            } else if (task === "update" && buttonState === "default") {
              return "Install update";
            } else if (buttonState === "working") {
              return (
                <>
                  <Loader2 className="h-4 w-4 inline-block animate-spin" />{" "}
                  Working
                </>
              );
            } else if (buttonState === "failed") {
              return (
                <>
                  <CircleX className="h-4 w-4 inline-block" /> Failed
                </>
              );
            } else if (buttonState === "done") {
              return (
                <>
                  <CircleCheck className="w-4 h-4 inline-block text-green-800" />{" "}
                  Done
                </>
              );
            }
          })()}
        </Button>
      </div>
    );
  }

  function AutoRemoveButton() {
    const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] =
      useState("default");
    const [removeOrphanedPackagesDialog, setRemoveOrphanedPackagesDialog] =
      useState(false);
    if (clearAutoRemoveButtonState === "default") {
      return (
        <>
          <Button
            variant="destructive"
            onClick={() => {
              setRemoveOrphanedPackagesDialog(true);
            }}
          >
            <TrashIcon className="mr-1 h-4 w-4 inline-block" /> Remove all
            orphaned packages
          </Button>
          <SudoDialog
            modalOpen={removeOrphanedPackagesDialog}
            onOpenChange={setRemoveOrphanedPackagesDialog}
            note={
              "Removing orphaned packages may also remove packages that are relevant for your system."
            }
            onFinish={(password) => {
              setClearAutoRemoveButtonState("working");
              startTask(fetchURLPrefix + "/api/removeOrphanedPackages", {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  sudoPassword: password,
                }),
              })
                .then(() => {
                  setAutoRemovePackages([]);
                  setClearAutoRemoveButtonState("default");
                  notify("Removed orphaned packages");
                })
                .catch(() => {
                  setClearAutoRemoveButtonState("default");
                  notify("Failed to remove orphaned packages");
                });
            }}
          />
        </>
      );
    } else if (clearAutoRemoveButtonState === "working") {
      return (
        <Button variant="destructive">
          <Loader2 className="h-4 w-4 mr-1 inline-block animate-spin" />{" "}
          Removing packages
        </Button>
      );
    }
  }

  function UpdateButton() {
    const [buttonState, setButtonState] = useState("default");
    const [sudoPasswordDialogOpen, setSudoPasswordDialogOpen] = useState(false);

    if (buttonState === "default") {
      return (
        <>
          <Button onClick={() => setSudoPasswordDialogOpen(true)}>
            <DownloadIcon className="h-4 w-4 mr-1 inline-block" /> Update all
            packages
          </Button>
          <SudoDialog
            modalOpen={sudoPasswordDialogOpen}
            onOpenChange={setSudoPasswordDialogOpen}
            onFinish={(password) => {
              notify("Started full system update");
              setButtonState("working");
              startTask(fetchURLPrefix + "/api/updateAllPackages", {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  sudoPassword: password,
                }),
              })
                .then(() => {
                  notify("Full package update succesful");
                  fetch(fetchURLPrefix + "/api/packageDatabase/false", {
                    headers: {
                      "Content-Type": "application/json",
                    },
                  }).then((res) => {
                    if (res.ok) {
                      notify(
                        "Updated package statistics after full package update",
                      );
                      res.json().then((json) => {
                        setInstalledPackages(Array.from(json["packages"]));
                        setOtherPackages(Array.from(json["others"]));
                        setCanProvideUpdates(json.canProvideUpdates);
                        setUpdates(json.updates);
                        setPackageManager(json.packageManager);
                        setVisibility(true);
                      });
                    } else {
                      notify("Failed to fetch database");
                      setVisibility(false);
                    }
                  });

                  fetch(fetchURLPrefix + "/api/listOrphanedPackages", {
                    headers: {
                      "Content-Type": "application/json",
                    },
                  }).then((res) => {
                    if (res.ok) {
                      res.json().then((json) => {
                        setAutoRemovePackages(json["packages"]);
                      });
                    } else {
                      notify("Failed to retrieve list of packages");
                      setVisibility(false);
                    }
                  });
                })
                .catch(() => {
                  notify("Failed to retrieve list of packages");
                });
            }}
          />
        </>
      );
    } else {
      return (
        <Button>
          <Loader2 className="h-4 w-4 inline-block animate-spin mr-1" />{" "}
          Updating
        </Button>
      );
    }
  }

  var installedCount = 0;
  var otherCount = 0;

  if (visible) {
    if (packageSearchValue.length > 1) {
      var PackageView = (
        <>
          {installedPackages
            .filter((pkg) => {
              return pkg.includes(packageSearchValue);
            })
            .sort((pkg) => {
              if (pkg == packageSearchValue) return -1;
              return +1;
            })
            .map((pkg, i) => {
              installedCount++;
              return (
                <PackageBox
                  packageName={pkg}
                  task="remove"
                  key={i}
                ></PackageBox>
              );
            })}
          {otherPackages
            .filter((pkg) => {
              return pkg.includes(packageSearchValue);
            })
            .sort((pkg) => {
              if (pkg == packageSearchValue) return -1;
              return +1;
            })
            .map((pkg, i) => {
              otherCount++;
              return (
                <PackageBox
                  packageName={pkg}
                  task="install"
                  key={i}
                ></PackageBox>
              );
            })}
          {installedCount === 0 && otherCount === 0 && (
            <Placeholder>
              <PlaceholderIcon icon={PackageIcon} />
              <PlaceholderSubtitle>No packages found</PlaceholderSubtitle>
            </Placeholder>
          )}
        </>
      );
    } else {
      var PackageView = <></>;
    }
    return (
      <Page className="overflow-y-scroll" name="Packages">
        <SudoDialog
          modalOpen={packagePopUpConfig.visible}
          onOpenChange={closePackagePopUp}
          note={`You are ${packagePopUpConfig.mode} ${packagePopUpConfig.packageName}.`}
          onFinish={(password) => {
            if (packagePopUpConfig.mode === "install") {
              installPackage(packagePopUpConfig.packageName, password);
            } else if (packagePopUpConfig.mode === "remove") {
              removePackage(packagePopUpConfig.packageName, password);
            } else if (packagePopUpConfig.mode === "update") {
              updatePackage(packagePopUpConfig.packageName, password);
            }
          }}
        />
        <StatCard
          name="Installed packages"
          Icon={<HardDriveIcon className="h-5 w-5 inline-block" />}
          Info="Packages that are installed on your system. This includes apps."
        >
          {installedPackages.length}
        </StatCard>

        {canProvideUpdates ? (
          <StatCard
            important={updates.length > 0}
            name="Available updates"
            Icon={<RefreshCcw className="h-5 w-5 inline-block" />}
            Info="Packages that can be updated"
          >
            {updates.length}
          </StatCard>
        ) : (
          <></>
        )}
        <StatCard
          name="Available packages"
          Icon={<Package2 className="h-5 w-5 inline-block" />}
          Info="Packages including apps, that are not installed on your system but listed in your package manager."
        >
          {otherPackages.length}
        </StatCard>
        <StatCard
          name="Orphaned packages"
          Icon={<TrashIcon className="h-5 w-5 inline-block" />}
          Info="Packages that are no other packages depend on. They may still be relevant to you."
        >
          {autoRemovePackages.length}
        </StatCard>

        {packageManager != "" ? (
          <StatCard
            name="Package manager"
            Icon={<BotIcon className="h-5 w-5 inline-block" />}
            Info="The package manager used by the system to install, remove and update packages."
          >
            {packageManager}
          </StatCard>
        ) : (
          <></>
        )}

        <br />
        <div className="flex w-full items-center space-x-2 mb-2">
          <Input
            placeholder="Search for packages"
            onChange={(e) => {
              setPackageSearchValue(e.target.value);
            }}
            autoFocus
            className="w-[300px] mr-1 w-max-[75vw] inline-block"
          />
        </div>
        <div className={packageSearchValue === "" ? "mb-2" : "hidden"}>
          <Details rememberState name={"packagesUpdate"} title="Updates">
            {updates.length > 0 ? (
              <small className="mb-1 block">
                {updates.length} package{updates.length !== 1 ? "s" : ""} can be
                updated.
              </small>
            ) : (
              <></>
            )}

            <span className="block mb-2">
              Last successful database update:{" "}
              {secondsToFormat(
                lastDatabaseUpdate,
                localStorage.getItem("dateFormat") || "8601",
              )}
            </span>
            <Button
              onClick={() => setUpdateDatabaseModalOpen(true)}
              className="mr-2"
              variant="secondary"
            >
              {
                {
                  default: (
                    <>
                      <DatabaseIcon className="w-4 h-4 inline-block mr-1" />{" "}
                      Update database
                    </>
                  ),
                  working: (
                    <>
                      <Loader2 className="w-4 h-4 inline-block animate-spin mr-1" />{" "}
                      Updating database
                    </>
                  ),
                  failed: (
                    <>
                      <CircleAlert className="w-4 h-4 inline-block mr-1" />{" "}
                      Failed to update database
                    </>
                  ),
                }[databaseUpdateButton]
              }
            </Button>

            <SudoDialog
              modalOpen={updateDatabaseModalOpen}
              onOpenChange={setUpdateDatabaseModalOpen}
              onFinish={(password) => {
                setDatabaseUpdateButton("working");
                notify("Package database update started");
                startTask(fetchURLPrefix + "/api/updatePackageDatabase", {
                  method: "POST",
                  headers: {
                    "Content-Type": "application/json",
                  },
                  body: JSON.stringify({
                    sudoPassword: password,
                  }),
                })
                  .then(() => {
                    notify("Package database update successful");
                    setDatabaseUpdateButton("default");
                    fetch(fetchURLPrefix + "/api/packageDatabase/false", {
                      headers: {
                        "Content-Type": "application/json",
                      },
                    }).then((res) => {
                      if (res.ok) {
                        res.json().then((json) => {
                          setInstalledPackages(Array.from(json["packages"]));
                          setOtherPackages(Array.from(json["others"]));
                          setCanProvideUpdates(json.canProvideUpdates);
                          setUpdates(json.updates);
                          setPackageManager(json.packageManager);
                          setVisibility(true);
                          setLastDatabaseUpdate(json.lastDatabaseUpdate);
                        });
                      }
                    });
                  })
                  .catch(() => {
                    setDatabaseUpdateButton("failed");
                    notify("Failed to update package database");
                  });
              }}
            />

            {updates.length > 0 ? <UpdateButton /> : <></>}
            <br />
            {updates.length === 0 ? (
              <div className="text-center align-middle opacity-75">
                <SparklesIcon className="w-full h-8 block mb-2" /> Everything is
                up-to-date
              </div>
            ) : (
              <></>
            )}
            {updates.map((pkg, i) => {
              return (
                <PackageBox
                  packageName={pkg}
                  task="update"
                  key={i}
                ></PackageBox>
              );
            })}
          </Details>
        </div>
        <Details
          rememberState
          name={"packagesOrphaned"}
          className={
            autoRemovePackages.length > 0 && packageSearchValue === ""
              ? ""
              : "hidden"
          }
          title="Orphaned packages"
        >
          <small className="mb-1 block">
            {autoRemovePackages.length} package
            {autoRemovePackages.length !== 1
              ? "s may not be"
              : " may not be"}{" "}
            required by your system anymore.
          </small>
          <AutoRemoveButton />
          <br />
          {autoRemovePackages.map((pkg, i) => {
            return (
              <PackageBox packageName={pkg} task="remove" key={i}></PackageBox>
            );
          })}
        </Details>
        {PackageView}
      </Page>
    );
  } else {
    return (
      <div className="h-screen w-screen flex items-center justify-center">
        <Loader2 className="animate-spin m-auto w-32 h-32 opacity-75" />
      </div>
    );
  }
}

export default Packages;
