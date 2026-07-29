import React from "react";

type CardVariant = "default" | "elevated";

interface GenCardProps {
  variant?: CardVariant;
  children: React.ReactNode;
  className?: string;
}

export function GenCard({ variant = "default", children, className = "" }: GenCardProps) {
  const base = "bg-white rounded-xl border border-gray-200";
  const styles = variant === "elevated" ? "shadow-lg" : "shadow-sm";

  return <div className={`${base} ${styles} ${className}`}>{children}</div>;
}
