const save = (key: string, data: any) => {
  localStorage.setItem(key, JSON.stringify(data));
};

const get = (key: string) => {
  var data = localStorage.getItem(key);
  return data ? JSON.parse(data) : null;
};

const remove = (key: string) => {
  localStorage.removeItem(key);
};

const has = (key: string) => {
  return localStorage.getItem(key) !== null;
};

export default {
  save,
  get,
  remove,
  has,
};
