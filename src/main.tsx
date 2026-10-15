import "@fontsource/noto-sans-bengali/400.css";
import "@fontsource/noto-sans-bengali/500.css";
import "@fontsource/noto-sans-bengali/600.css";
import "@fontsource/noto-sans-bengali/700.css";
import "@fontsource/hind-siliguri/400.css";
import "@fontsource/hind-siliguri/600.css";
import "@fontsource/baloo-da-2/400.css";
import "@fontsource/baloo-da-2/600.css";
import "@fontsource/tiro-bangla/400.css";
import "@fontsource/anek-bangla/400.css";
import "@fontsource/anek-bangla/600.css";
import "@fontsource/galada/400.css";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/500.css";
import "./styles.css";

import React from "react";
import ReactDOM from "react-dom/client";
import { useApp } from "./lib/api";
import { useTheme } from "./lib/theme";
import { Converter } from "./screens/Converter";
import { Indicator } from "./screens/Indicator";
import { Keyboard } from "./screens/Keyboard";
import { Onboarding } from "./screens/Onboarding";
import { Preferences } from "./screens/Preferences";
import { Widget } from "./screens/Widget";

const route = window.location.hash.replace("#", "") || "main";
if (route === "widget" || route === "indicator") document.documentElement.classList.add("chromeless");

const SCREENS = { main: Preferences, keyboard: Keyboard, converter: Converter, onboarding: Onboarding, widget: Widget, indicator: Indicator };

function App() {
  const boot = useApp();
  useTheme(boot);
  if (!boot) return null;
  const Screen = SCREENS[route as keyof typeof SCREENS] ?? Preferences;
  return <Screen boot={boot} />;
}

// Our windows are app chrome, not web pages.
window.addEventListener("contextmenu", (e) => {
  if (!(e.target as HTMLElement).closest("textarea, input")) e.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
