import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import "./styles/main.css";
import { markStartup, markStartupStep } from "./utils/startupPerformance";

markStartup("startup:app-start");

const app = createApp(App);
app.use(router);
router.isReady().then(() => {
  markStartupStep("startup:router-ready", "路由就绪");
});

app.mount("#app");
markStartupStep("startup:app-mounted", "应用挂载完成");
