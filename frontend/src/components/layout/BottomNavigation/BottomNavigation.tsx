/**
 * Bottom Navigation Component
 * Mobile app navigation
 */

import { FC } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import "./BottomNavigation.scss";

const NAV_ITEMS = [
  {
    id: "wallet",
    path: "/",
    icon: "💳",
    label: "Wallet",
  },
  {
    id: "swap",
    path: "/swap",
    icon: "🔄",
    label: "Swap",
  },
  {
    id: "history",
    path: "/history",
    icon: "📋",
    label: "History",
  },
  {
    id: "settings",
    path: "/settings",
    icon: "⚙️",
    label: "Settings",
  },
] as const;

export const BottomNavigation: FC = () => {
  const location = useLocation();
  const navigate = useNavigate();

  const handleNavigate = (path: string) => {
    navigate(path);
  };

  return (
    <nav className="bottom-nav">
      <div className="bottom-nav__container">
        {NAV_ITEMS.map((item) => {
          const isActive = location.pathname === item.path;

          return (
            <button
              key={item.id}
              type="button"
              className={`bottom-nav__item ${
                isActive ? "bottom-nav__item--active" : ""
              }`}
              onClick={() => handleNavigate(item.path)}
            >
              <span className="bottom-nav__icon">{item.icon}</span>
              <span className="bottom-nav__label">{item.label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
};
