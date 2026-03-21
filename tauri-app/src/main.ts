import { createApp } from "vue";
import App from "./App.vue";
import { create } from "naive-ui";
import DynamicForm from "./components/DynamicForm.vue";
import AuthMethodEditor from "./components/AuthMethodEditor.vue";

const naive = create();

const app = createApp(App);
app.use(naive);
app.component("DynamicForm", DynamicForm);
app.component("AuthMethodEditor", AuthMethodEditor);
app.mount("#app");
