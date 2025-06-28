"use client";

import { scan } from "react-scan";
import { useEffect, useState, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input.jsx";
import { Slider } from "@/components/ui/slider.jsx";
import {
  Select,
  SelectTrigger,
  SelectItem,
  SelectContent,
} from "@/components/ui/select.jsx";
import {
  MusicIcon,
  VideoIcon,
  PlayIcon,
  PauseIcon,
  VolumeXIcon,
  VolumeIcon,
  Volume1Icon,
  Volume2Icon,
  FullscreenIcon,
  MinimizeIcon,
  XIcon,
  DownloadIcon,
  PenIcon,
  LibraryIcon,
  PopcornIcon,
} from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogTitle,
  DialogFooter,
  DialogHeader,
} from "@/components/ui/dialog";
import { SelectGroup } from "@radix-ui/react-select";
import { XAlign } from "@/components/ui/align";
import { fetchURLPrefix } from "@/lib/fetchPrefix.js";
import { Toaster } from "@/components/ui/toaster";
import { ToastAction } from "@/components/ui/toast";
import { toast } from "@/components/ui/use-toast";
import Label from "@/components/ui/ShortLabel";

const savePersistentVolume = (v) => {
  // Do not pass values greater 1
  if (v > 1 || v < 0) throw new Error("The volume has to be between 0 and 1.");
  localStorage.setItem("playerVolume", v * 100); // Stores a value in the player volume variable.
};
const getPersistentVolume = () => {
  let s = localStorage.getItem("playerVolume");
  return s === null ? 60 : Number(s); // The returned volume is either a value between 0 and 100 or a default fo 60
};

/// Pass the source path of the video file including an object of all other sources
const rememberMusic = (source) => {
  fetch(`${fetchURLPrefix}/api/rememberMusic/${encodeURIComponent(source)}`);
};

const rememberVideo = (source) => {
  fetch(`${fetchURLPrefix}/api/rememberVideo/${encodeURIComponent(source)}`);
};

scan({
  enabled: false,
  log: true, // logs render info to console (default: false)
  clearLog: false, // clears the console per group of renders (default: false)
});

function Title({ children }) {
  return (
    <span className="p-2 text-xl font-bold border-b border-zinc-900 w-full block">
      {children}
    </span>
  );
}

/** @param {Object} param0
 * @param {string} param0.name
 * @param {string} param0.cover
 * @param {string} param0.artist
 * @param {string} param0.genre
 * @param {string} param0.filename
 * @param {() => void} [param0.onClick=() => {}]
 */
function MediaCard({
  name,
  cover,
  artist,
  genre,
  filename,
  reload = () => {},
  onClick = () => {},
}) {
  const [metadataDialogOpen, setMetadataDialogOpen] = useState(false);
  var metadataNameInput = useRef();
  var metadataArtistInput = useRef();
  var metadataGenreInput = useRef();
  var metadataCoverInput = useRef();

  return (
    <>
      <Dialog open={metadataDialogOpen} onOpenChange={setMetadataDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Metadata for {name.length > 20 ? name.slice(0, 17) + "..." : name}
            </DialogTitle>
          </DialogHeader>

          <div>
            <Label>Name</Label>
            <Input
              defaultValue={name}
              ref={metadataNameInput}
              className="mb-2"
            />

            <Label>Artist</Label>
            <Input
              defaultValue={artist === "UNKNOWN_ARTIST" ? "" : artist}
              ref={metadataArtistInput}
              placeholder="e.g. Singer, Band, Composer,..."
              className="mb-2"
            />

            <Label>Genre</Label>
            <Input
              defaultValue={genre === "UNKNOWN_GENRE" ? "" : genre}
              ref={metadataGenreInput}
              placeholder="e.g. Rock, Pop, Techno,..."
              className="mb-2"
            />

            <Label>Cover</Label>
            <Input
              defaultValue={
                cover.includes("/api/cover/music")
                  ? ""
                  : decodeURIComponent(
                      new URL(cover).pathname.replace("/api/cover/", ""),
                    )
              }
              placeholder="Absolute path on the server"
              ref={metadataCoverInput}
              className="mb-2"
            />
          </div>
          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline">Cancle</Button>
            </DialogClose>
            <DialogClose asChild>
              <Button
                onClick={() => {
                  var newName = metadataNameInput.current.value;
                  var newGenre = metadataGenreInput.current.value;
                  var newCover = metadataCoverInput.current.value;
                  var newArtist = metadataArtistInput.current.value;
                  fetch(fetchURLPrefix + "/api/updateMetadata", {
                    method: "POST",
                    headers: {
                      "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                      name: newName,
                      genre: newGenre,
                      cover: newCover,
                      artist: newArtist,
                      filename,
                    }),
                  }).then(() => {
                    reload();
                  });
                }}
              >
                Apply
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <span className="inline-block w-40 align-top ml-2 mb-2 cursor-pointer">
        <span
          title={name}
          className="inline-block h-40 w-40 rounded bg-100% bg-no-repeat overflow-hidden transition-all duration-200 delay-150 focus-visible:outline focus-visible:outline-white focus-visible:outline-4 focus-visible:brightness-90"
          onClick={(e) => {
            setTimeout(() => onClick(), 200);
          }}
          style={{
            backgroundImage: `url('${cover}')`,
            backgroundColor: "#222",
          }}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              setMetadataDialogOpen(true);
            }}
            className="relative left-[85%] top-2 align-super pb-1 pr-1 pl-1 aspect-square rounded bg-transparent hover:bg-transparent opacity-55 hover:opacity-100 transition-all ease-in-out duration-100"
          >
            <PenIcon className="w-3 h-3 inline-block opacity-70 mt-[-7px]" />
          </button>{" "}
        </span>
        <span className="text-sm whitespace-nowrap overflow-ellipsis overflow-hidden max-w-40 inline-block">
          {name}
        </span>
      </span>
    </>
  );
}

function VideoPlayer({ src, name, closePlayer }) {
  var v = useRef();
  const [playing, setPlaying] = useState(false);
  const [controlsVisible, setControlsVisible] = useState(false);
  const [overlayVisible, setOverlayVisible] = useState(true);
  const [notYetStarted, setNotYetStarted] = useState(true);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [playerVisible, setPlayerVisible] = useState(true);
  const [playerFadingOut, setPlayerFadingOut] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const [currentTime, setCurrentTime] = useState(0);
  const [volume, setVolume] = useState(0);

  useEffect(() => {
    setTimeout(() => {
      if (
        (v.current || { paused: false }).paused ||
        (v.current || { ended: false }).ended ||
        !playing
      ) {
        setOverlayVisible(true);
      }
    }, 300);
  }, [overlayVisible, playing, controlsVisible]);

  useEffect(() => {
    v.current.volume = getPersistentVolume() / 100;
    setVolume(getPersistentVolume());

    const handleKeyDown = (e) => {
      if (document.activeElement.tagName === "INPUT") return;

      if (e.key === " " || e.key === "k") {
        e.preventDefault();
        if (v.current.paused) {
          // Check if video is at the start position
          if (notYetStarted) {
            v.current.currentTime = 0;
          }
          setPlaying(true);
          setOverlayVisible(false);
          setNotYetStarted(false);
          v.current.play();
        } else {
          setPlaying(false);
          setNotYetStarted(false);
          v.current.pause();
        }
      } else if (e.key === "f") {
        try {
          if (!window.screenTop && !window.screenY) {
            document.exitFullscreen();
            setIsFullscreen(false);
          } else {
            document.body.requestFullscreen();
            setIsFullscreen(true);
          }
        } catch {}
      } else if (e.key === "ArrowLeft") {
        v.current.currentTime = v.current.currentTime - 5;
      } else if (e.key === "ArrowRight") {
        v.current.currentTime = v.current.currentTime + 5;
      } else if (e.key === "q") {
        document.body.style.cursor = "default";
        exitPlayer();
      } else if (
        e.key === "ArrowUp" &&
        document.activeElement.tagName != "INPUT"
      ) {
        e.preventDefault();
        let volumePlusTen = v.current.volume + 0.1;
        let valueToBeApplied =
          volumePlusTen > 1 ? v.current.volume : volumePlusTen;
        v.current.volume = valueToBeApplied;
        savePersistentVolume(valueToBeApplied);
        setVolume(valueToBeApplied * 100);
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        let volumePlusTen = v.current.volume - 0.1;
        let valueToBeApplied =
          volumePlusTen < 0 ? v.current.volume : volumePlusTen;
        v.current.volume = valueToBeApplied;
        setVolume(valueToBeApplied * 100);
        savePersistentVolume(valueToBeApplied);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    // Cleanup listener on component unmount
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [notYetStarted, playing, v, isFullscreen]);

  useEffect(() => {
    v.current.volume = volume / 100;
  }, [volume]);

  useEffect(() => {
    var interval = 0;
    if (typeof v.current != "undefined") {
      interval = setInterval(() => {
        if (!notYetStarted) {
          setCurrentTime(v.current.currentTime);
        }
      }, 500);
      if (notYetStarted) {
        v.current.currentTime = Math.round((v.current.duration || 100) * 0.075);
      }
    }

    return () => clearInterval(interval);
  }, [notYetStarted, v]);

  function exitPlayer() {
    if (isFullscreen) {
      try {
        document.exitFullscreen();
      } catch {}
    }
    setPlayerFadingOut(true);
    setTimeout(() => closePlayer(), 100);
  }

  function formatSecondsToTime(seconds) {
    if (seconds === 0 || Number.isNaN(seconds)) {
      return "00:00";
    }

    seconds = Math.max(0, parseInt(Math.floor(seconds), 10));

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    const formattedHours = String(hours).padStart(2, "0");
    const formattedMinutes = String(minutes).padStart(2, "0");
    const formattedSeconds = String(remainingSeconds).padStart(2, "0");

    if (hours > 0) {
      return `${formattedHours}:${formattedMinutes}:${formattedSeconds}`;
    } else {
      return `${formattedMinutes}:${formattedSeconds}`;
    }
  }

  return (
    <span
      className={
        "fixed top-0 left-0 w-screen h-screen bg-black z-20 duration-100" +
        (playerFadingOut ? " animate-movedown" : " animate-moveup")
      }
    >
      {errorMessage == "" ? (
        <></>
      ) : (
        <span className="p-4 text-xl text-red-500">{errorMessage}</span>
      )}
      <video
        className="w-full h-full block"
        controls
        src={`${fetchURLPrefix}/api/getMedia/${encodeURIComponent(src)}`}
        ref={v}
        hidden={!playerVisible}
        autoPlay={false}
        preload="metadata"
        onError={() => {
          setPlayerVisible(false);
          setErrorMessage("Failed to load video.");
        }}
        onEnded={() => {
          setPlaying(false);
        }}
        onLoadedMetadata={() => {
          setControlsVisible(true);
        }}
      />
      <div
        className={
          "w-screen h-screen fixed top-0 left-0 z-30 transition-all duration-100 bg-black/80" +
          (overlayVisible && errorMessage === ""
            ? ""
            : " opacity-0 cursor-none")
        }
        onClick={() => {
          if (!overlayVisible) {
            setControlsVisible(!controlsVisible);
          }
        }}
      >
        <span className="font-bold fixed top-1/3 left-32">
          <h1
            className={
              "text-3xl mb-1" +
              (errorMessage !== "" || overlayVisible ? " block" : " hidden")
            }
          >
            {name}
          </h1>
          <Button
            className={
              "mt-5 bg-white/5 hover:bg-white/10" +
              (overlayVisible ? " block" : " hidden")
            }
            variant="outline"
            onClick={() => {
              exitPlayer();
            }}
          >
            <XIcon className="w-5 h-5 align-bottom inline-block" /> Close Player
          </Button>
        </span>
      </div>
      <div
        className={
          "fixed bottom-0 z-50 w-full max-w-[100vw] p-3 bg-zinc-900 border-t border-zinc-800 flex-nowrap text-nowrap transition-all ease-in-out duration-200 " +
          (controlsVisible ? "" : "opacity-0 bottom-[-1vh]")
        }
        onMouseLeave={() => {
          setControlsVisible(false);
        }}
        onMouseEnter={() => {
          setControlsVisible(true);
        }}
      >
        <XAlign className="w-full">
          {!playing ? (
            <PlayIcon
              onClick={() => {
                setPlaying(true);
                if (notYetStarted) {
                  setNotYetStarted(false);
                  v.current.currentTime = 1;
                }
                setOverlayVisible(false);
                v.current.play();
              }}
              className="w-4 h-4 inline-block mr-1"
            />
          ) : (
            <PauseIcon
              onClick={() => {
                setPlaying(false);
                v.current.pause();
              }}
              className="w-4 h-4 inline-block mr-1"
            />
          )}
          {!isFullscreen ? (
            <FullscreenIcon
              className="inline-block h-4 w-4 mr-1"
              onClick={() => {
                document.body.requestFullscreen();
                setIsFullscreen(true);
              }}
            />
          ) : (
            <MinimizeIcon
              onClick={() => {
                document.exitFullscreen();
                setIsFullscreen(false);
              }}
              className="inline-block h-4 w-4 mr-1"
            />
          )}
          <VolumeIcon className="w-4 h-4 inline-block mr-1" />
          <Slider
            value={[volume]}
            className="w-20 inline-flex mr-2"
            min={0}
            max={100}
            onValueChange={setVolume}
            step={1}
          />

          <span className="w-[75px] max-w-[100px] text-center">
            {formatSecondsToTime(currentTime)}
          </span>
          <Slider
            value={
              v.current
                ? [Math.round((currentTime / v.current.duration) * 1000) / 10]
                : [0]
            }
            style={{ width: "calc(100% - 290px)" }}
            className="mr-2 ml-2"
            onValueChange={(e) => {
              setCurrentTime((v.current.duration * e[0]) / 100);
              v.current.currentTime = (v.current.duration * e[0]) / 100;
            }}
            step={0.1}
            min={0}
            max={100}
          />

          <span className="w-[75px] max-w-[100px] text-center">
            {v.current && formatSecondsToTime(v.current.duration)}
          </span>
        </XAlign>
      </div>
    </span>
  );
}

function MusicPlayer({ src, cover, name, closePlayer = () => {} }) {
  const [fadingOut, setFadingOut] = useState(false);
  const [playing, setPlaying] = useState(true);
  const [time, setTime] = useState(0);
  const [volume, setVolume] = useState(getPersistentVolume()); // Value from 0 - 100 representing the value.
  var playerTag = useRef();
  var coverImg = useRef();
  var vr = useRef();

  function exitPlayer() {
    setFadingOut(true);
    document.body.onkeydown = () => {};
    setTimeout(() => {
      closePlayer();
    }, 200 - 20);
  }

  useEffect(() => {
    const interval = setInterval(() => {
      if (playerTag.current) {
        setTime(playerTag.current.currentTime);
      }
    }, 400);

    playerTag.current.volume = volume / 100; // Set the volume of the audio player to 1/100 of the volume state (which is set to the stored value by default).
    if (vr.current) {
      vr.current.value = Math.round(volume);
    }

    document.body.onkeydown = (e) => {
      if (document.activeElement.tagName === "INPUT") return;
      let key = e.key;
      if (key === " " || key === "k") {
        let ae = document.activeElement.tagName.toLowerCase();
        if (ae !== "input" && ae !== "textarea") {
          e.preventDefault();
        }
        if (playing) {
          setPlaying(false);
          playerTag.current.pause();
        } else {
          playerTag.current.play();
          setPlaying(true);
        }
      } else if (key === "q" && document.activeElement.tagName != "INPUT") {
        exitPlayer();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        let volumePlusTen = playerTag.current.volume + 0.1;
        let valueToBeApplied = volumePlusTen > 1 ? 1 : volumePlusTen;
        playerTag.current.volume = valueToBeApplied;
        savePersistentVolume(Math.round(valueToBeApplied * 100) / 100);
        setVolume(Math.round(valueToBeApplied * 100));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        let volumePlusTen = playerTag.current.volume - 0.1;
        let valueToBeApplied = volumePlusTen < 0 ? 0 : volumePlusTen;
        playerTag.current.volume = valueToBeApplied;
        savePersistentVolume(Math.round(valueToBeApplied * 100) / 100);
        setVolume(Math.round(valueToBeApplied * 100));
      } else if (e.key === "ArrowLeft") {
        let tMinusFive = playerTag.current.currentTime - 5;
        let valueToBeApplied = tMinusFive < 0 ? 0 : tMinusFive;
        playerTag.current.currentTime = valueToBeApplied;
      } else if (e.key === "ArrowRight") {
        let tPlusFive = playerTag.current.currentTime + 5;
        let valueToBeApplied =
          tPlusFive > playerTag.current.duration
            ? playerTag.current.duration
            : tPlusFive;
        playerTag.current.currentTime = valueToBeApplied;
      }
    };

    return () => clearInterval(interval);
  }, [playing, setPlaying]);

  function failPlayer(src) {
    let srcSegments = src.split(".");
    let extension = srcSegments[srcSegments.length - 1].toLowerCase();
    let badExtensions = ["opus", "ogg", "oga"]; // Some devices running iOS may have problems with these file types
    let ua = navigator.userAgent;
    let iosError =
      badExtensions.includes(extension) &&
      (ua.includes("iPhone") || ua.includes("Apple") || ua.includes("Safari"));

    toast({
      title: "Player Error",
      description: iosError
        ? "An error occured while trying to play the media file. Check your OS and make sure that your browser can properly play back opus and ogg files."
        : "An error occured while trying to play the media file.",
      duration: 10000,
    });
    setTimeout(() => exitPlayer, 10000);
  }

  const iconCn =
    "inline-block h-6 w-6 mr-1 align-middle transition-all duration-200";

  return (
    <>
      <Toaster />
      <audio
        autoPlay
        src={`${fetchURLPrefix}/api/getMedia/${encodeURIComponent(src)}`}
        ref={playerTag}
        onError={(e) => {
          console.error("Music Player Error", e);
          failPlayer(src);
        }}
        onEnded={exitPlayer}
        hidden
      />

      <div
        className={
          "fixed bottom-0 z-50 w-full max-w-[100vw] p-1 bg-zinc-900 border-t border-zinc-800 flex-nowrap text-nowrap " +
          (fadingOut ? "animate-movedown" : "animate-moveup")
        }
      >
        <img
          src={cover}
          ref={coverImg}
          className={
            "rounded mr-2 transition-all ease-in-out h-12 w-12 inline-block "
          }
        />
        <span className="font-semibold">{name}</span>
        <span className="inline-flex fixed bottom-2 right-2 w-fit p-2">
          <XIcon className={iconCn + " mr-2"} onClick={exitPlayer} />
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <DownloadIcon
                  className={iconCn + " mr-2"}
                  onClick={() => {
                    window.open(
                      `${fetchURLPrefix}/api/getMedia/${encodeURIComponent(src)}`,
                    );
                  }}
                />
              </TooltipTrigger>
              <TooltipContent>
                Download the media file to your computer
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
          {(() => {
            if (volume == 0) {
              return <VolumeXIcon className={iconCn} />;
            } else if (volume < 20) {
              return <VolumeIcon className={iconCn} />;
            } else if (volume < 40) {
              return <Volume1Icon className={iconCn} />;
            } else if (volume < 60) {
              return <Volume2Icon className={iconCn} />;
            } else if (volume < 100) {
              return <Volume2Icon className={iconCn} />;
            } else if (volume == 100) {
              return <Volume2Icon className={iconCn + " text-red-500"} />;
            } else {
              return <></>;
            }
          })()}{" "}
          <Slider
            className="w-32"
            value={[volume]}
            onValueChange={(value) => {
              setVolume(value);
              savePersistentVolume(value / 100);
              playerTag.current.volume = Math.round(value) / 100;
            }}
            min={0}
            max={100}
          />
        </span>

        <span className="text-center inline-flex fixed bottom-2 w-1/2 left-1/2 -translate-x-1/2 bg-zinc-900 p-2">
          {playing ? (
            <PauseIcon
              className={iconCn}
              onClick={() => {
                playerTag.current.pause();
                setPlaying(false);
              }}
            />
          ) : (
            <PlayIcon
              className={iconCn}
              onClick={() => {
                playerTag.current.play();
                setPlaying(true);
              }}
            />
          )}{" "}
          {(function () {
            if (Number.isNaN(time)) {
              return "00:00 ";
            }
            let minutes = Math.floor((time - (time % 60)) / 60);
            let seconds = Math.floor(time % 60);
            return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")} `;
          })()}
          <Slider
            min={0}
            max={100}
            value={[
              (time / (playerTag.current ? playerTag.current.duration : 1)) *
                100,
            ]}
            onValueChange={(value) => {
              let newTime = (value / 100) * playerTag.current.duration;
              setTime(newTime);
              playerTag.current.currentTime = Math.round(newTime || 0);
            }}
            className="mx-1"
          />
          {(function () {
            if (playerTag.current) {
              if (Number.isNaN(playerTag.current.duration)) {
                return "00:00";
              }
              let duration = Math.round(playerTag.current.duration);
              let minutes = (duration - (duration % 60)) / 60;
              let seconds = duration % 60;
              return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
            } else {
              return "00:00";
            }
          })()}
        </span>
      </div>
    </>
  );
}

function Desktop() {
  const [queryInputValue, setQueryInputValue] = useState(""); // What value is in the query input?
  const [musicPlayerSrc, setMusicPlayerSrc] = useState(""); // What source does the music-player get the music from?
  const [musicPlayerHidden, setMusicPlayerHidden] = useState(true); // Is the music-player hidden?
  const [musicPlayerName, setMusicPlayerName] = useState(""); // What name does the music-player show the user?
  const [musicPlayerCover, setMusicPlayerCover] = useState(""); // What is the cover the music-player shows the user
  const [videoPlayerSrc, setVideoPlayerSrc] = useState(""); // What source does the video-player get the video from?
  const [videoPlayerName, setVideoPlayerName] = useState(""); // What name does the video-player show the user?
  const [videoPlayerHidden, setVideoPlayerHidden] = useState(true); // Is the video-player hidden?
  const [selectedGenre, setSelectedGenre] = useState(""); // What is the selected genre?
  const [lastMediaFetch, setLastMediaFetch] = useState(false); // Should the frontend fetch media data again?

  const [videos, setVideos] = useState([]); // Array of objects that represent videos
  const [music, setMusic] = useState([]); // Array of objects that represent music

  const [recommendedVideos, setRecommendedVideos] = useState([]); // Array of objects that represent videos
  const [recommendedMusic, setRecommendedMusic] = useState([]); // Array of objects that represent music

  const [genres, setGenres] = useState([]); // Array of all genres

  var queryInput = useRef(); // Reference to the query input

  useEffect(() => {
    if (lastMediaFetch) {
      return;
    }

    setLastMediaFetch(true);

    fetch(`${fetchURLPrefix}/api/recommendedMusic`).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setRecommendedMusic(
            json.rec.filter((e) => {
              return Date.now() - e[1] < 2 * 24 * 60 * 60 * 1000;
            }),
          );
        });
      }
    });

    fetch(`${fetchURLPrefix}/api/recommendedVideos`).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          setRecommendedVideos(
            json.rec.filter((e) => {
              return Date.now() - e[1] < 2 * 24 * 60 * 60 * 100;
            }),
          );
        });
      }
    });
    fetchMediaList();
  }, [
    videos,
    music,
    genres,
    lastMediaFetch,
    recommendedVideos,
    recommendedMusic,
  ]);

  function fetchMediaList() {
    fetch(`${fetchURLPrefix}/api/getMediaList`).then((res) => {
      if (res.ok) {
        res.json().then((json) => {
          let med = json.media;
          let m = [];
          let v = [];
          let g = [];
          // Sort the array alphabetically to ensure a consistent display
          var e = Object.entries(med).toSorted(function (c, n) {
            // c: Current
            // n: Next
            let a = c[1][0]; // Get first value of info of current element
            let b = n[1][0]; // Get first value of info of next element
            if (a < b) {
              return -1;
            }
            if (a > b) {
              return 1;
            }
            return 0;
          });
          for (const [path, info] of e) {
            let pathSegments = path.split(".");
            let extension = pathSegments[pathSegments.length - 1].toLowerCase();
            if (!g.includes(info[2]) && info[2] !== "UNKNOWN_GENRE") {
              g.push(info[2]);
            }
            if (
              [
                "webm",
                "wav",
                "heic",
                "m4a",
                "flac",
                "ogg",
                "opus",
                "oga",
                "mp3",
              ].includes(extension)
            ) {
              m.push({
                cover:
                  info[1] !== "UNKNOWN_COVER"
                    ? fetchURLPrefix +
                      "/api/cover/" +
                      encodeURIComponent(info[1])
                    : fetchURLPrefix + "/api/cover/music",
                name: info[0],
                source: path,
                genre: info[2],
                artist: info[3],
              });
            } else if (
              [
                "mp4",
                "mov",
                "avi",
                "wmv",
                "ogv",
                "m4p",
                "m4v",
                "mpg",
                "mp2",
                "mpeg",
                "mpv",
                "mkv",
                "webv",
              ].includes(extension)
            ) {
              v.push({
                cover:
                  info[1] !== "UNKNOWN_COVER"
                    ? fetchURLPrefix +
                      "/api/cover/" +
                      encodeURIComponent(info[1])
                    : fetchURLPrefix + "/api/cover/video",
                name: info[0],
                source: path,
                genre: info[2],
                artist: info[3],
              });
            } else if (
              [
                "png",
                "jpeg",
                "jpg",
                "gif",
                "webp",
                "bmp",
                "svg",
                "avif",
                "tiff",
                "bash",
                "zsh",
                "fish",
                "pdf",
                "txt",
                "md",
                "kdenlive",
              ].includes(extension)
            ) {
              // Do nothing (ignored).
            } else {
              v.push({
                cover:
                  info[1] !== "UNKNOWN_COVER"
                    ? fetchURLPrefix +
                      "/api/cover/" +
                      encodeURIComponent(info[1])
                    : fetchURLPrefix + "/api/cover/badtype",
                name: info[0],
                source: path,
                genre: info[2],
                artist: info[3],
              });
            }
          }
          setMusic(m);
          setVideos(v);
          setGenres(g);
        });
      }
    });
  }

  function playVideo(src, name) {
    if (!musicPlayerHidden) {
      toast({
        title: "Music is playing",
        description:
          "You have an active music player. Please close it before opening a video file.",
        action: (
          <ToastAction
            altText="Stop Music"
            onClick={() => {
              setMusicPlayerHidden(true);
              setVideoPlayerHidden(false);
              setVideoPlayerSrc(src);
              setVideoPlayerName(name);
            }}
          >
            Stop Music
          </ToastAction>
        ),
      });
      return;
    }
    setVideoPlayerHidden(false);
    setVideoPlayerSrc(src);
    setVideoPlayerName(name);
  }

  function playMusic(src, name, cover) {
    setMusicPlayerHidden(false);
    setMusicPlayerSrc(src);
    setMusicPlayerName(name);
    setMusicPlayerCover(cover);
  }

  useEffect(() => {
    document.body.addEventListener("keypress", (e) => {
      if (["TEXTAREA", "INPUT"].includes(document.activeElement.tagName))
        return;
      if (e.key === "s" || e.key === "/") {
        queryInput.current.focus();
      }
    });
  }, []);

  return (
    <>
      <Toaster />
      {!videoPlayerHidden ? (
        <VideoPlayer
          src={videoPlayerSrc}
          name={videoPlayerName}
          closePlayer={() => {
            setVideoPlayerHidden(true);
            setVideoPlayerName("");
            setVideoPlayerSrc("");
          }}
        />
      ) : (
        <></>
      )}
      {!musicPlayerHidden ? (
        <MusicPlayer
          src={musicPlayerSrc}
          name={musicPlayerName}
          cover={musicPlayerCover}
          closePlayer={() => {
            setMusicPlayerHidden(true);
            setMusicPlayerName("");
            setMusicPlayerSrc("");
          }}
        />
      ) : (
        <></>
      )}

      <span className="w-full">
        <Title>Zentrox Media Center</Title>
        <span className="flex items-center space-x-1 m-2">
          <Select
            value={selectedGenre}
            onValueChange={(e) => setSelectedGenre(e)}
          >
            <SelectTrigger className="w-[180px] inline-flex bg-transparent">
              Genre
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {genres.map((e, i) => {
                  return (
                    <SelectItem key={i} value={e.toLowerCase()}>
                      {e}
                    </SelectItem>
                  );
                })}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Input
            type="text"
            placeholder="Search by name"
            className="inline-block mt-0"
            ref={queryInput}
            onKeyPress={(e) => {
              if (e.key === "Enter") {
                setQueryInputValue(queryInput.current.value);
              }
            }}
          />
          <Button
            variant="outline"
            className="focus-visible:outline-white focus-visible:outline-2 focus-visible:outline"
            onClick={() => {
              setSelectedGenre("");
              setQueryInputValue("");
              queryInput.current.value = "";
            }}
          >
            Clear filters
          </Button>
        </span>
        <h2 className="font-semibold flex items-center p-2">
          <VideoIcon className="h-6 w-6 mr-1" /> Videos in your library
        </h2>
        {videos.length === 0 ? (
          <span className="opacity-50 ml-2 flex items-center">
            <PopcornIcon className="mr-1" /> Nothing to watch
          </span>
        ) : (
          <></>
        )}
        {recommendedVideos.length > 0 && queryInputValue === "" ? (
          <strong className="pl-2 mt-1 mb-1 block">You may like</strong>
        ) : (
          <></>
        )}
        {recommendedVideos.map((v, i) => {
          if (videos.length === 0 || queryInputValue !== "") {
            return null;
          }

          let f = videos.find((e) => e.source === v[0]);
          if (typeof f == "undefined") {
            return null;
          }
          const lName = f.name;
          const lCover = f.cover;

          if (!lName) {
            return null;
          }

          return (
            <MediaCard
              src={lCover.length > 0 ? lCover : "askdjalsdklö"}
              name={lName}
              key={v[0] || i}
              onClick={() => {
                rememberVideo(v[0]);
                playVideo(v[0], lName);
              }}
            />
          );
        })}
        {recommendedVideos.length > 0 ? (
          <>
            <br />
            <br />
          </>
        ) : (
          <></>
        )}
        {videos
          .filter((e) => {
            if (selectedGenre !== "") {
              return selectedGenre.toLowerCase() == e.genre.toLowerCase();
            } else {
              return true;
            }
          })
          .filter((e) => {
            if ((queryInput.current || { value: "" }).value !== "") {
              return e.name
                .toLowerCase()
                .includes(queryInputValue.toLowerCase());
            } else {
              return true;
            }
          })
          .map((e, i) => {
            return (
              <MediaCard
                filename={e.source}
                cover={e.cover.length > 0 ? e.cover : "/api/cover/music"}
                name={e.name}
                artist={e.artist}
                genre={e.genre}
                key={i}
                reload={fetchMediaList}
                onClick={() => {
                  playVideo(e.source, e.name);
                }}
              />
            );
          })}
        <h2 className="font-semibold flex items-center p-2">
          <MusicIcon className="h-6 w-6 mr-1" /> Music in your library
        </h2>

        {music.length === 0 ? (
          <span className="opacity-50 ml-2 flex items-center">
            <LibraryIcon className="mr-1" /> Nothing to listen to
          </span>
        ) : (
          <></>
        )}
        {recommendedMusic.length > 0 && queryInputValue === "" ? (
          <strong className="pl-2 mt-1 mb-1 block">You may like</strong>
        ) : (
          <></>
        )}
        {recommendedMusic.map((v, i) => {
          if (music.length === 0 || queryInputValue !== "") {
            return null;
          }

          let f = music.find((e) => e.source === v[0]);
          if (typeof f == "undefined") {
            return null;
          }
          const lName = f.name;
          const lCover = f.cover;
          const lArtist = f.artist;
          const lGenre = f.genre;

          if (!lName) {
            return null;
          }

          return (
            <MediaCard
              filename={v[0]}
              cover={lCover.length > 0 ? lCover : "/api/cover/music"}
              name={lName}
              artist={lArtist}
              genre={lGenre}
              key={v[0] || i}
              reload={fetchMediaList}
              onClick={() => {
                rememberMusic(v[0]);
                playMusic(
                  v[0],
                  lName,
                  lCover.length > 0 ? lCover : "/api/cover/music",
                );
              }}
            />
          );
        })}
        {recommendedMusic.length > 0 && queryInputValue === "" ? (
          <>
            <br />
            <br />
          </>
        ) : (
          <></>
        )}
        {music
          .filter((e) => {
            if (selectedGenre !== "") {
              return selectedGenre.toLowerCase() == e.genre.toLowerCase();
            } else {
              return true;
            }
          })
          .filter((e) => {
            if ((queryInput.current || { value: "" }).value !== "") {
              return e.name
                .toLowerCase()
                .includes(queryInputValue.toLowerCase());
            } else {
              return true;
            }
          })
          .map((e, i) => {
            return (
              <MediaCard
                filename={e.source}
                cover={e.cover}
                name={e.name}
                artist={e.artist}
                genre={e.genre}
                key={e.source || i}
                reload={fetchMediaList}
                onClick={() => {
                  rememberMusic(e.source);
                  playMusic(
                    e.source,
                    e.name,
                    e.cover.length > 0 ? e.cover : "/api/cover/music",
                  );
                }}
              />
            );
          })}
      </span>
    </>
  );
}

function Mobile() {
  return <></>;
}

export default function Page() {
  const [deviceType, setDeviceType] = useState("desktop");

  useEffect(() => {
    if (
      /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        navigator.userAgent,
      )
    ) {
      // setDeviceType("mobile");
    }
  }, []);

  return deviceType == "mobile" ? <Mobile /> : <Desktop />;
}
