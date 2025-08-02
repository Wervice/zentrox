export default function getParentPath(path) {
  const normalizedPath = path.replace(/\/+$/, "");
  const parts = normalizedPath.split("/");
  if (parts.length <= 1) {
    return "/";
  }
  const parentPath = parts.slice(0, -1).join("/");
  return parentPath;
}
