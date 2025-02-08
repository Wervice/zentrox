import { Button } from "@/components/ui/button.jsx";
import {
  RefreshCcw,
  HardDriveIcon,
  Package2,
  Loader2,
  CircleX,
  SearchIcon,
  TrashIcon,
  CircleCheck,
  BotIcon,
  CircleAlert,
  DownloadIcon,
  DatabaseIcon,
  RotateCcw,
  SparkleIcon,
  SparklesIcon,
} from "lucide-react";
import { useEffect, useState, useRef } from "react";
import "./table.css";
import "./scroll.css";
import StatCard from "@/components/ui/StatCard.jsx";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import Page from "@/components/ui/PageWrapper";
import { useToast } from "@/components/ui/use-toast";
import fetchURLPrefix from "@/lib/fetchPrefix";

function startTask(adress, options = {}, interval = 3000) {
  return new Promise((resolve, reject) => {
    fetch(adress, options).then((res) => {
      if (res.ok) {
        res.text().then((uuid) => {
			let ready_for_fetch = true;

          let ivd = setInterval(() => {
			  if (!ready_for_fetch) return;
			  ready_for_fetch = false
            fetch(fetchURLPrefix + "/api/fetchJobStatus/" + uuid).then(
              (checkRes) => {
				ready_for_fetch = true;
                if (checkRes.status === 200) {
                  clearInterval(ivd);
                  resolve(checkRes);
                } else if (checkRes.status === 422 || checkRes.status === 500) {
                  clearInterval(ivd);
                  reject(checkRes);
				}}
            );
          }, interval);
        });
      }
    });
  });
}

function Packages() {
  const { toast } = useToast();
  const [packagePopUpConfig, setPackagePopUp] = useState({
    visible: false,
    mode: "",
    packageName: "",
	errorFn: () => {},
	finishFn: () => {}
  });
  var databaseUpdateSudoPasswordInput = useRef();
  var packageSudoPasswordInput = useRef();
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
        });
      } else {
        toast({
          title: "Package Database Error",
          message: "Zentrox failed to retrieve a list of packages",
        });
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
        toast({
          title: "Package Database Error",
          message: "Zentrox failed to retrieve a list of packages",
        });
        setVisibility(false);
      }
    });
  }

function installPackage(packageName) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/installPackage", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            packageName: packageName,
            sudoPassword: packageSudoPasswordInput.current.value,
        }),
    }).then(() => {
        closePackagePopUp();
        setInstalledPackages([...installedPackages, packageName]);
        setOtherPackages(otherPackages.filter((e) => e !== packageName));
        
        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        toast({
            title: "Installed package",
            description: packageName + " has been successfully installed",
            duration: 60000,
        });
    }).catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });

        toast({
            title: "Failed to install package",
            description: "Zentrox failed to install " + packageName,
        });
    });
}

function removePackage(packageName) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/removePackage", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            packageName: packageName,
            sudoPassword: packageSudoPasswordInput.current.value,
        }),
    }).then(() => {
        closePackagePopUp();
        setInstalledPackages(installedPackages.filter((entry) => entry !== packageName));
        setOtherPackages([...otherPackages, packageName]);
        
        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        toast({
            title: "Removed package",
            description: packageName + " has been successfully removed",
            duration: 60000,
        });
    }).catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });

        toast({
            title: "Failed to remove package",
            description: "Zentrox failed to remove " + packageName,
        });
    });
}

function updatePackage(packageName) {
    let activeTasksCopy = { ...activeTasks, [packageName]: "working" };
    setActiveTasks(activeTasksCopy);

    startTask(fetchURLPrefix + "/api/updatePackage", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            packageName: packageName,
            sudoPassword: packageSudoPasswordInput.current.value,
        }),
    }).then(() => {
        closePackagePopUp();
        setUpdates(updates.filter((e) => e !== packageName));
        
        let updatedTasks = { ...activeTasks };
        delete updatedTasks[packageName];
        setActiveTasks(updatedTasks);

        toast({
            title: "Updated package",
            description: packageName + " has been successfully updated",
            duration: 60000,
        });
    }).catch(() => {
        closePackagePopUp();
        setActiveTasks({ ...activeTasks, [packageName]: "failed" });

        toast({
            title: "Failed to update package",
            description: "Zentrox failed to update " + packageName,
        });
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
			setButtonState("working")
		} else if (activeTasks[packageName] === "failed") {
			setButtonState("failed")
		}
	}, [activeTasks])

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
                  <CircleX className="h-4 w-4 inline-block" />{" "}
                  Failed
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

  function AutoRemoveButon() {
    const [clearAutoRemoveButtonState, setClearAutoRemoveButtonState] =
      useState("default");
    var sudoPasswordInput = useRef();
    if (clearAutoRemoveButtonState === "default") {
      return (
        <Dialog>
          <DialogTrigger asChild>
            <Button variant="destructive">
              <TrashIcon className="h-4 w-4 inline-block mr-2" /> Remove packages
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Remove packages</DialogTitle>
              <DialogDescription>
                Remove packages that may not be requried by the system according
                to your package manager.
                <br />
                Make sure you want to remove the packages before you continue.
                <br />
                Removing packages requires your sudo password.
              </DialogDescription>
            </DialogHeader>
            <Input
              type="password"
              placeholder="Sudo password"
              ref={sudoPasswordInput}
			  className="w-full"
            />
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="outline">Close</Button>
              </DialogClose>
              <DialogClose asChild>
                <Button
                  variant="destructive"
                  onClick={() => {
                    setClearAutoRemoveButtonState("working");
                    startTask(fetchURLPrefix + "/api/removeOrphanedPackages", {
                      method: "POST",
                      headers: {
                        "Content-Type": "application/json",
                      },
                      body: JSON.stringify({
                        sudoPassword: sudoPasswordInput.current.value,
                      }),
                    }).then(() => {
                        setAutoRemovePackages([]);
						setClearAutoRemoveButtonState("default")
						toast({
							title: "Removed orphaned packages",
							description: "Orphaned packages have successfully been removed"
						})
                    }).catch(() => {
						setClearAutoRemoveButtonState("default")
                        toast({
                          title: "Failed to remove packages",
                          description:
                            "Zentrox failed to remove orphaned packages from your system.",
                        });
					});
                  }}
                >
                  Proceed
                </Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      );
    } else if (clearAutoRemoveButtonState === "working") {
      return (
        <Button variant="destructive">
          <Loader2 className="h-4 w-4 inline-block animate-spin" /> Removing
          packages
        </Button>
      );
    }
  }

  function UpdateButton() {
    var sudoPasswordInput = useRef();
    const [buttonState, setButtonState] = useState("default");
    if (buttonState === "default") {
      return (
        <Dialog>
          <DialogTrigger asChild>
            <Button>
              <DownloadIcon className="h-4 w-4 inline-block mr-1" /> Update
              packages
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Update all packages</DialogTitle>
              <DialogDescription>
                To update all packages, please enter your sudo password.
              </DialogDescription>
            </DialogHeader>
            <Input
              type="password"
              placeholder="Sudo password"
              ref={sudoPasswordInput}
			  className="w-full"
            />
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="outline">Close</Button>
              </DialogClose>
              <DialogClose asChild>
                <Button
                  onClick={() => {
                    setButtonState("working");
                    startTask(fetchURLPrefix + "/api/updateAllPackages", {
                      method: "POST",
                      headers: {
                        "Content-Type": "application/json",
                      },
                      body: JSON.stringify({
                        sudoPassword: sudoPasswordInput.current.value,
                      }),
                    })
                      .then(() => {
                        fetch(fetchURLPrefix + "/api/packageDatabase/false", {
                          headers: {
                            "Content-Type": "application/json",
                          },
                        }).then((res) => {
                          if (res.ok) {
                            res.json().then((json) => {
                              setInstalledPackages(
                                Array.from(json["packages"]),
                              );
                              setOtherPackages(Array.from(json["others"]));
                              setCanProvideUpdates(json.canProvideUpdates);
                              setUpdates(json.updates);
                              setPackageManager(json.packageManager);
                              setVisibility(true);
                            });
                          } else {
                            toast({
                              title: "Package Database Error",
                              message:
                                "Zentrox failed to retrieve a list of packages",
                            });
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
                            toast({
                              title: "Package Database Error",
                              message:
                                "Zentrox failed to retrieve a list of packages",
                            });
                            setVisibility(false);
                          }
                        });
                      })
                      .catch(() => {
                        toast({
                          title: "Failed to update all packages",
						  description: "Zentrox was unable to update all installed packages."
                        });
                      });
                  }}
                >
                  Proceed
                </Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
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
              return (
                <PackageBox
                  packageName={pkg}
                  task="install"
                  key={i}
                ></PackageBox>
              );
            })}
        </>
      );
    } else {
      var PackageView = (
        <>
        </>
      );
    }
    return (
      <Page name="Packages">
        <Dialog
          open={packagePopUpConfig.visible}
          onOpenChange={closePackagePopUp}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {
                  {
                    install: "Install package",
                    remove: "Remove package",
                    update: "Update package",
                  }[packagePopUpConfig.mode]
                }
              </DialogTitle>
              <DialogDescription>
                {
                  {
                    install: `To install ${packagePopUpConfig.packageName}, please enter your sudo password.`,
                    remove: `Do you really want to remove ${packagePopUpConfig.packageName}? To remove ${packagePopUpConfig.packageName}, please enter your sudo password.`,
                    update: `To update ${packagePopUpConfig.packageName}, please enter your sudo password.`,
                  }[packagePopUpConfig.mode]
                }
              </DialogDescription>
            </DialogHeader>

            <p>
              <Input
                type="password"
                placeholder="Sudo password"
                ref={packageSudoPasswordInput}
				className="w-full"
              />
            </p>
            <DialogFooter>
              <DialogClose asChild>
                <Button variant="secondary" onClick={closePackagePopUp}>
                  Cancel
                </Button>
              </DialogClose>
              <DialogClose asChild>
                <Button
                  variant={
                    packagePopUpConfig.mode === "remove"
                      ? "destructive"
                      : "default"
                  }
                  onClick={() => {
                    if (packagePopUpConfig.mode === "install") {
                      installPackage(packagePopUpConfig.packageName);
                    } else if (packagePopUpConfig.mode === "remove") {
                      removePackage(packagePopUpConfig.packageName);
                    } else if (packagePopUpConfig.mode === "update") {
                      updatePackage(packagePopUpConfig.packageName);
                    }
                  }}
                >
                  Continue
                </Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>

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
        <div className="h-fit">
          <Input
            placeholder="Search for packages"
            onChange={(e) => {
              setPackageSearchValue(e.target.value);
            }}
            className="mt-2 w-[300px] mr-1 w-max-[75vw] inline-block"
          />
        </div>
        <br />
          <div className={packageSearchValue === "" ? "mb-2" : "hidden"}>
			<details>
			<summary className="mb-2 cursor-pointer">
            <strong><RefreshCcw className="w-6 h-6 inline-block" /> Updates</strong>
			</summary>
            {updates.length > 0 ? (
              <small className="mb-1 block">
                {updates.length} package{updates.length !== 1 ? "s" : ""} can be
                updated.
              </small>
            ) : (
              <></>
            )}

            <Dialog>
              <DialogTrigger asChild>
                <Button className="mr-2" variant="secondary">
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
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Update package database</DialogTitle>
                  <DialogDescription>
                    Update your package database to check for new packages and
                    updates. Please enter your sudo password to continue.
                  </DialogDescription>
                </DialogHeader>

                <p>
                  <Input
                    type="password"
                    placeholder="Sudo password"
                    ref={databaseUpdateSudoPasswordInput}
                  />
                </p>

                <DialogFooter>
                  <DialogClose asChild>
                    <Button variant="secondary">Cancel</Button>
                  </DialogClose>

                  <DialogClose asChild>
                    <Button
                      onClick={() => {
                        setDatabaseUpdateButton("working");
                        startTask(
                          fetchURLPrefix + "/api/updatePackageDatabase",
                          {
                            method: "POST",
                            headers: {
                              "Content-Type": "application/json",
                            },
                            body: JSON.stringify({
                              sudoPassword:
                                databaseUpdateSudoPasswordInput.current.value,
                            }),
                          },
                        ).then(() => {
						  toast({
							  title: "Updated database",
							  description: "The package database has been successfully updated"
						  })
                          setDatabaseUpdateButton("default");
                          fetch(fetchURLPrefix + "/api/packageDatabase/false", {
                            headers: {
                              "Content-Type": "application/json",
                            },
                          }).then((res) => {
                            if (res.ok) {
                              res.json().then((json) => {
                                setInstalledPackages(
                                  Array.from(json["packages"]),
                                );
                                setOtherPackages(Array.from(json["others"]));
                                setCanProvideUpdates(json.canProvideUpdates);
                                setUpdates(json.updates);
                                setPackageManager(json.packageManager);
                                setVisibility(true);
                              });
                            }
                          });
                        }).catch(() => {
							setDatabaseUpdateButton("default")
							toast({
								title: "Failed to update package database",
								description: "Zentrox was unable to update your package managers database"
							})
						})
                      }}
                    >
                      Continue
                    </Button>
                  </DialogClose>
                </DialogFooter>
              </DialogContent>
            </Dialog>

            {updates.length > 0 ? <UpdateButton /> : <></>}
            <br />
			{
				updates.length === 0 ? <div className="text-center align-middle opacity-75"><SparklesIcon className="w-8 h-8 inline-block" /> Everything is up-to-date</div> : <></>
			}
            {updates.map((pkg, i) => {
              return (
                <PackageBox
                  packageName={pkg}
                  task="update"
                  key={i}
                ></PackageBox>
              );
            })}
			</details>
          </div>
			<details className={autoRemovePackages.length > 0 && packageSearchValue === "" ? "" : "hidden"}>
			<summary className="mb-2 cursor-pointer">
            <strong><TrashIcon className="w-6 h-6 inline-block" /> Orphaned packages</strong>
			</summary>
            <small className="mb-1 block">
              {autoRemovePackages.length} package
              {autoRemovePackages.length !== 1
                ? "s may not be"
                : " may not be"}{" "}
              required by your system anymore.
            </small>
            <AutoRemoveButon />
            <br />
            {autoRemovePackages.map((pkg, i) => {
              return (
                <PackageBox
                  packageName={pkg}
                  task="remove"
                  key={i}
                ></PackageBox>
              );
            })}</details>
        {PackageView}
      </Page>
    );
  } else {
    return (
      <div className="p-auto pt-5">
        <Loader2 className="animate-spin m-auto w-20 h-20" />
      </div>
    );
  }
}

export default Packages;
