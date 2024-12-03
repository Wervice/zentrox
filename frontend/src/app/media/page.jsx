"use client";

import { scan } from "react-scan";
import { useEffect, useState, useRef } from "react";
import { Button } from "@/components/ui/button.jsx";
import { Input } from "@/components/ui/input.jsx";
import {
 Select,
 SelectTrigger,
 SelectItem,
 SelectContent,
 SelectValue,
} from "@/components/ui/select.jsx";
import "./slider.css";
import {
 MusicIcon,
 VideoIcon,
 PlayIcon,
 PauseIcon,
 VolumeIcon,
 FullscreenIcon,
 MinimizeIcon,
 XIcon,
 VolumeXIcon,
 Volume2Icon,
 Volume1Icon,
 CompassIcon,
 BoxesIcon,
 BrushIcon,
 DeleteIcon,
 HelpCircleIcon,
 LinkIcon,
} from "lucide-react";
import { SelectGroup } from "@radix-ui/react-select";

scan({
 enabled: false,
 log: true, // logs render info to console (default: false)
 clearLog: false, // clears the console per group of renders (default: false)
});

function Title({ children, openHelp = () => {} }) {
 return (
  <span className="p-2 text-xl font-bold border-b border-zinc-900 w-full block">
   {children}
   <HelpCircleIcon
    className="inline-block fixed top-3 right-3 h-4 w-4"
    onClick={() => openHelp()}
   />
  </span>
 );
}

function VideoCard({ name, src, onClick }) {
 const [active, setActive] = useState(false);

 return (
  <button
   title={name}
   className="inline-block ml-2 h-40 w-40 rounded bg-100% bg-no-repeat overflow-hidden transition-all duration-200 delay-150 focus-visible:outline focus-visible:outline-white focus-visible:outline-4 focus-visible:brightness-90"
   onClick={() => {
    setActive(true);
    setTimeout(() => onClick(), 200);
   }}
   onBlur={() => setActive(false)}
   style={{
    backgroundImage: `url('${src}')`,
   }}
  >
   <span
    className={
     "w-full inline-block relative font-bold p-4 bg-gradient-to-t from-black/100 to-transparent overflow-ellipsis overflow-x-hidden whitespace-nowrap transition-all duration-200 " +
     (active ? "opacity-0" : "")
    }
    style={{ bottom: "-41% " }}
   >
    <VideoIcon className="w-4 h-4 p-0 m-0 inline-block" /> {name}
   </span>
  </button>
 );
}

function MusicCard({ name, cover, onClick = () => {} }) {
 return (
  <button
   title={name}
   className="inline-block ml-2 h-40 w-40 rounded bg-100% bg-no-repeat overflow-hidden transition-all duration-200 delay-150 focus-visible:outline focus-visible:outline-white focus-visible:outline-4 focus-visible:brightness-90"
   onClick={() => {
    setTimeout(() => onClick(), 200);
   }}
   style={{
    backgroundImage: `url('${cover}')`,
    backgroundColor: "#222",
   }}
  >
   <span
    className={
     "w-full inline-block relative font-bold p-4 bg-gradient-to-t from-black/100 to-transparent overflow-ellipsis overflow-x-hidden whitespace-nowrap transition-all duration-200"
    }
    style={{ bottom: "-40% " }}
   >
    <MusicIcon className="w-4 h-4 p-0 m-0 inline-block" /> {name}
   </span>
  </button>
 );
}

function VideoPlayer({ src, name, closePlayer }) {
 var v = useRef();
 var t = useRef();
 var vr = useRef();
 const [playing, setPlaying] = useState(false);
 const [controlsVisible, setControlsVisible] = useState(true);
 const [overlayVisible, setOverlayVisible] = useState(true);
 const [notYetStarted, setNotYetStarted] = useState(true);
 const [isFullscreen, setIsFullscreen] = useState(false);
 const [isVideoLoaded, setVideoLoaded] = useState(false);
 const [playerVisible, setPlayerVisible] = useState(true);
 const [playerFadingOut, setPlayerFadingOut] = useState(false);
 const [errorMessage, setErrorMessage] = useState("");
 const [currentTime, setCurrentTime] = useState(0);

 useEffect(() => {
  if (!playing) {
   setTimeout(() => {
    if (
     (v.current || { paused: false }).paused ||
     (v.current || { ended: false }).ended
    ) {
     setOverlayVisible(true);
    }
   }, 300);
  }
 }, [playing, overlayVisible]);

 useEffect(() => {
  const handleKeyDown = (e) => {
   if (e.key === " " || e.key === "k") {
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
    exitPlayer();
   } else if (e.key === "ArrowUp") {
    let volumePlusTen = v.current.volume + 0.1;
    let valueToBeApplied = volumePlusTen > 1 ? v.current.volume : volumePlusTen;
    vr.current.value = valueToBeApplied * 100;
    v.current.volume = valueToBeApplied;
   } else if (e.key === "ArrowDown") {
    let volumePlusTen = v.current.volume - 0.1;
    let valueToBeApplied = volumePlusTen < 0 ? v.current.volume : volumePlusTen;
    vr.current.value = valueToBeApplied * 100;
    v.current.volume = valueToBeApplied;
   }
  };

  document.addEventListener("keydown", handleKeyDown);
  // Cleanup listener on component unmount
  return () => {
   document.removeEventListener("keydown", handleKeyDown);
  };
 }, [notYetStarted, playing, v, isFullscreen]);
 useEffect(() => {
  var interval = 0;
  if (typeof v.current != "undefined") {
   interval = setInterval(() => {
    if (!notYetStarted) {
     t.current.value = Math.round(v.current.currentTime);
     setCurrentTime(v.current.currentTime);
    }
   }, 500);
   if (notYetStarted) {
    v.current.currentTime = Math.round((v.current.duration || 100) * 0.075);
   }
  }

  return () => clearInterval(interval);
 }, [notYetStarted, vr, t, v]);

 function exitPlayer() {
  if (isFullscreen) {
   document.exitFullscreen();
  }
  setPlayerFadingOut(true);
  setTimeout(() => closePlayer(), 100);
 }

 function formatSecondsToTime(seconds) {
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
    src={`/api/getMedia/${encodeURIComponent(src)}`}
    ref={v}
    hidden={!playerVisible}
    autoPlay={false}
    preload="metadata"
    onLoadedMetadata={() => setVideoLoaded(true)}
    onError={() => {
     setPlayerVisible(false);
     setErrorMessage("Failed to load video.");
    }}
   />
   <div
    className={
     "w-screen h-screen fixed top-0 left-0 z-30 transition-all duration-100 bg-black/80" +
     (overlayVisible && errorMessage === "" ? "" : " opacity-0")
    }
   >
    <span className="font-bold fixed top-1/3 left-32">
     <h1 className={"text-3xl mb-1" + (overlayVisible ? " block" : " hidden")}>
      {name}
     </h1>
     <span className={overlayVisible ? "" : "hidden"}>
      {isVideoLoaded
       ? typeof v.current != "undefined"
         ? `${Math.round(v.current.duration / 60) || "Unknown"} minutes (${notYetStarted ? Math.round(v.current.duration / 60) : Math.round((v.current.duration - v.current.currentTime) / 60)} minutes remaining)`
         : "Some length"
       : ""}
     </span>
     <Button
      className={
       "mt-5 bg-white/5 hover:bg-white/10" +
       (overlayVisible ? " block" : " hidden")
      }
      variant="secondary"
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
     "fixed bottom-0 w-full bg-black p-2 z-40 transition-all whitespace-nowrap duration-100 " +
     (controlsVisible ? "" : "opacity-0 bottom-[-1vh]")
    }
    onMouseLeave={() => setControlsVisible(false)}
    onMouseEnter={() => setControlsVisible(true)}
   >
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
    <input
     type="range"
     ref={vr}
     className="w-20 inline-block mr-2"
     min={0}
     max={100}
     defaultValue={100}
     onChange={(e) => {
      v.current.volume = e.target.value / 100;
     }}
    />

    {formatSecondsToTime(currentTime)}
    <input
     type="range"
     ref={t}
     style={{ width: "calc(100% - 290px)" }}
     className="mr-2 ml-2"
     min={0}
     max={
      typeof v.current != "undefined"
       ? Number.isNaN(v.current.duration)
         ? 0
         : v.current.duration
       : 0
     }
     onChange={(e) => {
      v.current.currentTime = e.target.value;
     }}
    />

    {v.current ? formatSecondsToTime(v.current.duration) : "00:00"}
   </div>
  </span>
 );
}

function MusicPlayer({ src, cover, name, closePlayer = () => {} }) {
 const [fadingOut, setFadingOut] = useState(false);
 const [playing, setPlaying] = useState(true);
 const [time, setTime] = useState(0);
 const [playerBig, setPlayerBig] = useState(false);
 const [volume, setVolume] = useState(100);
 var playerTag = useRef();

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
    setTime(Math.floor(playerTag.current.currentTime));
   }
  }, 1000);

  document.body.onkeydown = (e) => {
   let key = e.key;
   if (key === " " || key === "k") {
    if (playing) {
     setPlaying(false);
     playerTag.current.pause();
    } else {
     playerTag.current.play();
     setPlaying(true);
    }
   } else if (key === "q") {
    exitPlayer();
   } else if (key === "f") {
    setPlayerBig(true);
   } else if (key === "Escape") {
    setPlayerBig(false);
   } else if (e.key === "ArrowUp") {
    let volumePlusTen = playerTag.current.volume + 0.1;
    let valueToBeApplied =
     volumePlusTen > 1 ? playerTag.current.volume : volumePlusTen;
    playerTag.current.volume = valueToBeApplied;
    setVolume(Math.floor(valueToBeApplied * 100));
   } else if (e.key === "ArrowDown") {
    let volumePlusTen = playerTag.current.volume - 0.1;
    let valueToBeApplied =
     volumePlusTen < 0 ? playerTag.current.volume : volumePlusTen;
    playerTag.current.volume = valueToBeApplied;
    setVolume(Math.floor(valueToBeApplied * 100));
   } else if (e.key === "ArrowLeft") {
    let tMinusFive = playerTag.current.currentTime - 5;
    let valueToBeApplied =
     tMinusFive < 0 ? playerTag.current.currentTime : tMinusFive;
    playerTag.current.currentTime = valueToBeApplied;
   } else if (e.key === "ArrowRight") {
    let tPlusFive = playerTag.current.currentTime + 5;
    let valueToBeApplied =
     tPlusFive < 0 ? playerTag.current.currentTime : tPlusFive;
    playerTag.current.currentTime = valueToBeApplied;
   }
  };

  return () => clearInterval(interval);
 }, [playing, setPlaying]);

 return (
  <>
   <audio
    src={src}
    autoPlay
    ref={playerTag}
    onError={() => exitPlayer()}
    hidden
   />
   <div
    className={
     "fixed bottom-8 right-8 min-w-72 max-w-72 duration-200 rounded bg-white text-black cursor-pointer shadow-black shadow-lg transition-all ease-in-out " +
     (fadingOut ? "animate-movedown" : "animate-moveup") +
     (playerBig ? " h-[20.5em] p-2" : " h-14 p-1")
    }
    onClick={(e) => {
     if (["img", "div", "span"].includes(e.target.tagName.toLowerCase())) {
      setPlayerBig(!playerBig);
     }
    }}
   >
    <img
     src={cover}
     className={
      "rounded mr-1 transition-all ease-in-out " +
      (playerBig ? "block w-72 h-72" : "inline-block w-12 h-12")
     }
    />
    <XIcon
     className="inline-block h-5 w-5 align-middle"
     onClick={() => exitPlayer()}
    />
    {playing ? (
     <PauseIcon
      className="inline-block h-5 w-5 align-middle"
      onClick={() => {
       playerTag.current.pause();
       setPlaying(false);
      }}
     />
    ) : (
     <PlayIcon
      className="inline-block h-5 w-5 align-middle"
      onClick={() => {
       playerTag.current.play();
       setPlaying(true);
      }}
     />
    )}{" "}
    <span className="align-middle">
     <span hidden={!playerBig}>
      {(function () {
       if (volume > 75) {
        return (
         <Volume2Icon
          className={
           "inline-block h-5 w-5 mt-[-3px] transition-all ease-in-out duration-200" +
           (volume === 100 ? " text-red-700" : "")
          }
         />
        );
       } else if (volume > 50) {
        return <Volume1Icon className="inline-block h-5 w-5 mt-[-3px]" />;
       } else if (volume > 0) {
        return <VolumeIcon className="inline-block h-5 w-5 mt-[-3px]" />;
       } else if (volume === 0) {
        return <VolumeXIcon className="inline-block h-5 w-5 mt-[-3px]" />;
       }
      })()}{" "}
     </span>
     {(function () {
      if (Number.isNaN(time)) {
       return "00:00";
      }
      let minutes = (time - (time % 60)) / 60;
      let seconds = time % 60;
      return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
     })()}{" "}
     <span hidden={!playerBig}>
      {" "}
      /{" "}
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
    </span>{" "}
    <span className="text-l font-bold align-middle">{name}</span>
   </div>
  </>
 );
}

function Key({ children }) {
 return (
  <span className="p-1 rounded bg-white/5 text-white border-b-2 border-bottom-neutral-500">
   {children}
  </span>
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
 const [helpHidden, setHelpHidden] = useState(true); // Is the help modal hidden?
 const [selectedGenre, setSelectedGenre] = useState(""); // What is the selected genre?
 const [helpFadingOut, setHelpFadingOut] = useState(false); // Is the help modal fading out?
 const [lastMediaFetch, setLastMediaFetch] = useState(false); // Should the frontend fetch media data again?
 const [lastGenresFetch, setLastGenresFetch] = useState(false); // Should the frontend fetch genre data again?

 const [videos, setVideos] = useState([]); // Array of objects that represent videos
 const [music, setMusic] = useState([]); // Array of objects that represent music

 const [genres, setGenres] = useState([]); // Array of all genres

 var queryInput = useRef(); // Reference to the query input

 useEffect(() => {
  if (lastMediaFetch) {
   return;
  }

  setLastMediaFetch(true);

  fetch("/api/getMediaList").then((res) => {
   if (res.ok) {
    res.json().then((json) => {
     let med = json.media;
      let m = [];
      let v = [];
	for (const [path, info] of Object.entries(med)) {
      console.log(path);
      let pathSegments = path.split(".");
      let extension = pathSegments[pathSegments.length - 1].toLowerCase();

      switch (extension) {
       case "wav":
       case "heic":
       case "m4a":
       case "flac":
       case "ogg":
       case "opus":
       case "oga":
       case "webm":
       case "mp3":
        m.push({
         cover: "/api/cover/" + encodeURIComponent(info[1]),
         name: info[0],
         source: path,
         genre: info[2],
        });
       default:
        v.push({
         cover: "/api/cover/" + encodeURIComponent(info[1]),
         name: info[0],
         source: path,
         genre: info[2],
        });
      }
     }
      setMusic(m);
      setVideos(v);
    });
   }
  });
 }, [videos, music, lastMediaFetch]);

 useEffect(() => {
  if (lastGenresFetch) {
   return;
  }
  setLastGenresFetch(true);

  fetch("/api/getGenreList").then((res) => {
   if (res.ok) {
    res.json((json) => {
     let g = json.genres;
     setGenres(g);
    });
   }
  });
 }, [genres, lastGenresFetch]);

 function playVideo(src, name) {
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
   if (e.key === "s" || e.key === "/") {
    queryInput.current.focus();
   } else if ((e.key === "Escape" || e.key === "q") && !helpHidden) {
    setHelpFadingOut(true);
    setTimeout(() => {
     setHelpHidden(true);
     setHelpFadingOut(false);
    }, 190);
   }
  });
 }, [helpHidden]);

 return (
  <>
   <span
    className={
     "fixed bg-black/20 backdrop-grayscale w-screen h-screen top-0 left-0 z-20 duration-200 ease-in-out " +
     (helpFadingOut ? "animate-movedown" : "animate-moveup")
    }
    hidden={helpHidden}
   >
    <span className="m-8 p-2 rounded w-[calc(100vw-4em)] h-[calc(100vh-4em)] block bg-zinc-950 border border-1 border-neutral-600 shadow-black/20 shadow-lg">
     <h1 className="text-2xl font-bold">Help & FAQ</h1>

     <h2 className="text-lg font-semibold mt-2">How to add media files?</h2>
     <p className="text-lg">
      In order to add video files to (Zentrox) Media Center, go to the{" "}
      <a href="/" className="underline text-blue-500 text-lg">
       {" "}
       <LinkIcon className="inline-block h-4 w-4 align-middle" /> Zentrox
       Dashboard
      </a>{" "}
      and select the Media tab.
      <br />
      From there, you can add resources by specifying a directory path that
      exists on your server and a directory alias. <br />
      Click apply so changes will take effect.
      <br />
      Depending on your servers performance, you will see the files included in
      the selected diretory appeare soon in Media Center.
     </p>

     <h2 className="text-lg font-semibold mt-2">Keybindings</h2>
     <p className="text-lg">
      Media Center has a few built in keybinds to make using the interface
      easier. <br />
      <table>
       <tr>
        <td className="p-1">Shortcut</td>
        <td className="p-1">Action</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>q</Key>
        </td>
        <td className="p-1">Quits and closes players and popovers</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>k</Key> <Key>Space</Key>
        </td>
        <td className="p-1">Pause or play a player</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>s</Key>
        </td>
        <td className="p-1">Focuses search input</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>f</Key> <Key>Escape</Key>
        </td>
        <td className="p-1">Enter / Exit fullscreen</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>Arrow Left</Key> <Key>Arrow Right</Key>
        </td>
        <td className="p-1">Skip forwards or backwards</td>
       </tr>
       <tr>
        <td className="p-1">
         <Key>Arrow Up</Key> <Key>Down</Key>
        </td>
        <td className="p-1">Increase or decrease volume</td>
       </tr>
      </table>
     </p>
    </span>
   </span>
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
    <Title
     openHelp={() => {
      setHelpHidden(false);
     }}
    >
     Zentrox Media Center
    </Title>
    <Select value={selectedGenre} onValueChange={(e) => setSelectedGenre(e)}>
     <SelectTrigger className="w-[180px] inline-flex bg-transparent ml-2">
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
    </Select>{" "}
    <Input
     type="text"
     placeholder="Search by name"
     className="ml-2 mt-2 inline-block"
     ref={queryInput}
     onKeyPress={(e) => {
      if (e.key === "Enter") {
       setQueryInputValue(queryInput.current.value);
      }
     }}
    />
    <Button
     variant="secondary"
     className="ml-2 focus-visible:outline-white focus-visible:outline-2 focus-visible:outline"
     onClick={() => {
      setSelectedGenre("");
      queryInput.current.value = "";
     }}
    >
     Clear Filters
    </Button>
    <br />
    <h2 className="font-semibold p-2">
     <VideoIcon className="inline-block h-6 w-6 align-middle" /> Videos in your
     library
    </h2>
    {videos.length === 0 ? (
     <span className="opacity-50 m-2">
      <BoxesIcon className="inline-block" /> No videos here
     </span>
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
       return e.name.includes(queryInputValue);
      } else {
       return true;
      }
     })
     .map((e, i) => {
      return (
       <VideoCard
        src={e.cover}
        name={e.name}
        key={i}
        onClick={() => playVideo(e.source, e.name)}
       />
      );
     })}
    <h2 className="font-semibold p-2">
     <MusicIcon className="inline-block h-6 w-6 align-middle" /> Music in your
     library
    </h2>
    {music.length === 0 ? (
     <span className="opacity-50 m-2">
      <BoxesIcon className="inline-block" /> No music here
     </span>
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
       return e.name.includes(queryInputValue);
      } else {
       return true;
      }
     })
     .map((e, i) => {
      return (
       <MusicCard
        name={e.name}
        cover={e.cover}
        key={i}
        onClick={() => playMusic(e.source, e.name, e.cover)}
       />
      );
     })}
    <h2 className="font-semibold p-2">
     <CompassIcon className="inline-block h-6 w-6 align-middle" /> Explore more
    </h2>
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
   setDeviceType("mobile");
  }
 }, []);

 return deviceType == "mobile" ? <Mobile /> : <Desktop />;
}
