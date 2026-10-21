import React, { useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { useTranslation } from "react-i18next";
import "./i18n";
import "./styles/globals.css";
import "./styles/components.css";

import { ToastProvider } from "./components/common/Toast";
import { Layout } from "./components/navigation/Layout";
import { HomePage } from "./features/replay/HomePage";
import { LibraryPage } from "./features/library/LibraryPage";
import { EditorPage } from "./features/editor/EditorPage";
import { SettingsPage } from "./features/settings/SettingsPage";

const AppInner: React.FC = () => {
  const { i18n } = useTranslation();

  useEffect(() => {
    const lang = i18n.language;
    document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
    document.documentElement.lang = lang;
  }, [i18n.language]);

  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<HomePage />} />
          <Route path="/library" element={<LibraryPage />} />
          <Route path="/editor/:clipId" element={<EditorPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
};

const App: React.FC = () => (
  <ToastProvider>
    <AppInner />
  </ToastProvider>
);

export default App;
