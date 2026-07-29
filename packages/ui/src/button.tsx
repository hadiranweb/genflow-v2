import React from "react";

type ButtonVariant = "primary" | "outline" | "danger";

interface GenButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  loading?: boolean;
  children: React.ReactNode;
}

const variantStyles: Record<ButtonVariant, string> = {
  primary:
    "bg-teal-500 text-white hover:bg-teal-600 focus:ring-teal-500 disabled:bg-teal-300",
  outline:
    "border-2 border-teal-500 text-teal-600 hover:bg-teal-50 focus:ring-teal-500 disabled:border-gray-300 disabled:text-gray-400",
  danger:
    "bg-red-500 text-white hover:bg-red-600 focus:ring-red-500 disabled:bg-red-300",
};

export function GenButton({
  variant = "primary",
  loading = false,
  children,
  className = "",
  disabled,
  ...props
}: GenButtonProps) {
  return (
    <button
      className={`inline-flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg font-medium text-sm transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:cursor-not-allowed ${variantStyles[variant]} ${className}`}
      disabled={disabled || loading}
      {...props}
    >
      {loading && (
        <svg
          className="animate-spin h-4 w-4"
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
          />
        </svg>
      )}
      {children}
    </button>
  );
}
