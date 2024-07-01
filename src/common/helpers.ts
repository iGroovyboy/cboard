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
