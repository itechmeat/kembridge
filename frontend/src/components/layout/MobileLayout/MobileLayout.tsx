/**
 * Mobile App Layout
 * Mobile-first layout with bottom navigation
 */

import { FC, ReactNode } from "react";
import { ErrorBoundary } from "../../ui/ErrorBoundary/ErrorBoundary";
import { BottomNavigation } from "../BottomNavigation/BottomNavigation";
import { TopBar } from "../TopBar/TopBar";
import "./MobileLayout.scss";

interface MobileLayoutProps {
  children: ReactNode;
  showBottomNav?: boolean;
  className?: string;
}

export const MobileLayout: FC<MobileLayoutProps> = ({
  children,
  showBottomNav = true,
  className = "",
}) => {
  return (
    <ErrorBoundary>
      <div className={`mobile-layout ${className}`}>
        <TopBar />

        <main className="mobile-layout__main">{children}</main>

        {showBottomNav && <BottomNavigation />}
      </div>
    </ErrorBoundary>
  );
};
