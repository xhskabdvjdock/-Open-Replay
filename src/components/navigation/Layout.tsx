import React from "react";
import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";

export const Layout: React.FC = () => (
  <div className="app-layout">
    <Sidebar />
    <main className="app-content">
      <Outlet />
    </main>
  </div>
);
