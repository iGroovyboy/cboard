import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import router from "./router";

createApp(App).use(router).mount("#app");

window.addEventListener("keydown", function (e) {
  if ((e.ctrlKey && e.code == "KeyF") || e.code == "F5" || e.code == "KeyR") {
    e.preventDefault();
  }
});