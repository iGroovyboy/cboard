const save = (key, data) => {
  localStorage.setItem(key, JSON.stringify(data));
};

const get = (key) => {
  var data = localStorage.getItem(key);
  return data ? JSON.parse(data) : null;
};

const remove = (key) => {
  localStorage.removeItem(key);
};

const has = (key) => {
  return localStorage.getItem(key) !== null;
};

export default {
  save,
  get,
  remove,
  has,
};
