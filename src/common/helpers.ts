export const formatDate = (timestamp: string) => {
  var date = new Date(parseInt(timestamp));
  var hours = date.getHours();
  var minutes = "0" + date.getMinutes();
  var seconds = "0" + date.getSeconds();

  var y = date.getFullYear();
  var m = "0" + (date.getMonth() + 1);
  var d = "0" + date.getDate();

  return (
    `${y}-${m.substr(-2)}-${d.substr(-2)} ` +
    hours +
    ":" +
    minutes.substr(-2) +
    ":" +
    seconds.substr(-2)
  );
};

export const getFileTypeByFilename = (filename: string) => {
  if (typeof filename !== "string") {
    return "";
  }

  const parts = filename.split(".");
  return parts && parts.length > 1 ? parts.pop()?.toLowerCase() : "";
};

export const truncateString = (
  str: string,
  maxLength = 250,
  suffix = "..."
) => {
  if (typeof str !== "string") {
    return str;
  }

  return str.length > maxLength ? str.substring(0, maxLength) + suffix : str;
};

export function debounce(func: Function, wait: number) {
  let timeout: number | undefined;
  return function (...args: any) {
    clearTimeout(timeout);
    // @ts-ignore
    timeout = setTimeout(() => func.apply(this, args), wait);
  };
}
