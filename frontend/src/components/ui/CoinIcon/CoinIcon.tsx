import { FC } from "react";
import cn from "classnames";
import { getCoinDisplayProps } from "../../../utils/coinIcons";
import styles from "./CoinIcon.module.scss";

export interface CoinIconProps {
  symbol: string;
  size?: "small" | "medium" | "large";
  className?: string;
}

export const CoinIcon: FC<CoinIconProps> = ({
  symbol,
  size = "medium",
  className = "",
}) => {
  const displayProps = getCoinDisplayProps(symbol);

  return (
    <div className={cn(styles.coinIcon, styles[size], className)}>
      {displayProps.type === "icon" ? (
        <img
          src={displayProps.src}
          alt={displayProps.alt}
          className={styles.image}
        />
      ) : (
        <div className={styles.fallback}>{displayProps.letter}</div>
      )}
    </div>
  );
};
