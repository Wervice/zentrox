import {
  AppWindowIcon,
  FileTextIcon,
  PresentationIcon,
  TableIcon,
} from "lucide-react";
import { ArchiveIcon } from "lucide-react";
import { VideotapeIcon } from "lucide-react";
import { FileCodeIcon } from "lucide-react";
import { MusicIcon } from "lucide-react";
import { ImageIcon } from "lucide-react";
import { File } from "lucide-react";

/**
 * @param {Object} param0
 * @param {String} param0.filename
 */
export default function FileIcon({ filename, className = "" }) {
  let filenameSplit = filename.split(".");
  let extension = filenameSplit[filenameSplit.length - 1].toLowerCase();
  switch (extension) {
    case "png":
    case "jpeg":
    case "jpg":
    case "gif":
    case "webm":
    case "tiff":
    case "svg":
    case "webp":
    case "xcf":
      return <ImageIcon className={className} />;
    case "mp3":
    case "wav":
    case "heic":
    case "m4a":
    case "opus":
      return <MusicIcon className={className} />;
    case "pdf":
    case "docx":
    case "doc":
    case "odt":
    case "txt":
    case "md":
    case "dot":
    case "docm":
      return <FileTextIcon className={className} />;
    case "csv":
    case "xlsx":
    case "ods":
      return <TableIcon className={className} />;
    case "ppt":
    case "pptx":
    case "odp":
      return <PresentationIcon className={className} />;
    case "py":
    case "js":
    case "c":
    case "cpp":
    case "bash":
    case "sh":
    case "rs":
    case "java":
    case "kt":
    case "jsx":
    case "ts":
    case "tsx":
      return <FileCodeIcon className={className} />;
    case "mp4":
    case "webv":
    case "mpeg":
    case "m4v":
    case "wmv":
    case "ogg":
      return <VideotapeIcon className={className} />;
    case "zip":
    case "gz":
    case "xz":
    case "7z":
    case "iso":
    case "apk":
    case "tar":
      return <ArchiveIcon className={className} />;
    case "exe":
    case "dll":
    case "elf":
    case "app":
    case "scr":
    case "msi":
      return <AppWindowIcon className={className} />;
    default:
      return <File className={className} />;
  }
}
