import { FC, ReactNode } from "react";
import cn from "classnames";
import { ErrorBoundary } from "../../ui/ErrorBoundary/ErrorBoundary";
import { TopBar } from "../TopBar/TopBar";
import styles from "./MobileLayout.module.scss";

interface MobileLayoutProps {
  children: ReactNode;
  className?: string;
}

export const MobileLayout: FC<MobileLayoutProps> = ({
  children,
  className = "",
}) => {
  return (
    <ErrorBoundary>
      <div className={cn(styles.mobileLayout, className?.trim())}>
        <TopBar />

        <main className={styles.main}>{children}</main>
      </div>
    </ErrorBoundary>
  );
};
