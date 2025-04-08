/**
 * @returns {String}
 * @param {number} seconds
 * @param {string} format
 * Convert a number of seconds since UNIX epoch to a date-time string
 * Supported formats are:
 * - 8601 (YYYY-MM-DDThh:mm:ss)
 * - European (DD.MM.YYYY hh:mm:ss)
 * - Short (DD/MM/YY hh:mm:ss AM/PM)
 * - UNIX (Seconds since UNIX epoch)
 */
export default function secondsToFormat(inSeconds, format) {
  let date_obj = new Date(inSeconds * 1000); // Convert seconds to milliseconds
  let year = date_obj.getFullYear();
  let month = date_obj.getMonth() + 1;
  let day = date_obj.getDate();
  let hour = date_obj.getHours();
  let minutes = date_obj.getMinutes();
  let seconds = date_obj.getSeconds();
  let ampm = hour >= 12 ? "PM" : "AM";

  let hour12 = hour % 12 || 12;

  switch (format) {
    case "8601":
      return `${year}-${("0" + month).slice(-2)}-${("0" + day).slice(-2)}T${("0" + hour).slice(-2)}:${("0" + minutes).slice(-2)}:${("0" + seconds).slice(-2)}`;
    case "European":
      return `${("0" + day).slice(-2)}.${("0" + month).slice(-2)}.${year} ${("0" + hour).slice(-2)}:${("0" + minutes).slice(-2)}`;
    case "Short":
      return `${("0" + day).slice(-2)}/${("0" + month).slice(-2)}/${year.toString().slice(-2)} ${("0" + hour12).slice(-2)}:${("0" + minutes).slice(-2)} ${ampm}`;
    case "UNIX":
      return inSeconds.toString();
    default:
      throw new Error("Unsupported format");
  }
}
