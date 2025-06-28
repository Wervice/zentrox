export default function concatPath(a: string, b: string): string {
  if (a.endsWith("/")) {
    return a + b;
  } else {
    return a + "/" + b;
  }
}
