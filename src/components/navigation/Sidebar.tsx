import React from "react";
import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Home, Film, Settings, Circle } from "lucide-react";
import { useReplayStatus } from "../../hooks/useReplayStatus";

const NAV_ITEMS = [
  { path: "/", label: "nav.home", icon: Home },
  { path: "/library", label: "nav.library", icon: Film },
  { path: "/settings", label: "nav.settings", icon: Settings },
];

export const Sidebar: React.FC = () => {
  const { t } = useTranslation();
  const { status } = useReplayStatus(2000);

  return (
    <aside className="sidebar">
      <div className="sidebar-logo">
        <span className="sidebar-logo-icon">⏺</span>
        <span className="sidebar-logo-text">Open Replay</span>
      </div>

      <nav className="sidebar-nav">
        {NAV_ITEMS.map(({ path, label, icon: Icon }) => (
          <NavLink
            key={path}
            to={path}
            end={path === "/"}
            className={({ isActive }) => `sidebar-item${isActive ? " sidebar-item-active" : ""}`}
          >
            <Icon size={18} className="sidebar-item-icon" />
            <span className="sidebar-item-label">{t(label)}</span>
          </NavLink>
        ))}
      </nav>

      <div className="sidebar-footer">
        <div className="sidebar-status">
          <Circle
            size={8}
            fill={status.isRecording ? "var(--accent)" : "var(--text-tertiary)"}
            color={status.isRecording ? "var(--accent)" : "var(--text-tertiary)"}
            className={status.isRecording ? "pulse-dot" : ""}
          />
          <span className="sidebar-status-text">
            {status.isRecording ? t("home.replayActive") : t("home.replayInactive")}
          </span>
        </div>
        <span className="sidebar-version">v0.1.0</span>
      </div>
    </aside>
  );
};
