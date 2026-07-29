import React from "react";

interface GenInputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

export function GenInput({ label, className = "", id, ...props }: GenInputProps) {
  const inputId = id || label?.replace(/\s+/g, "-").toLowerCase();

  return (
    <div className="space-y-1.5">
      {label && (
        <label htmlFor={inputId} className="block text-sm font-medium text-gray-700">
          {label}
        </label>
      )}
      <input
        id={inputId}
        className={`w-full border border-gray-300 rounded-lg px-4 py-2.5 text-sm focus:ring-2 focus:ring-teal-500 focus:border-teal-500 transition disabled:bg-gray-100 disabled:cursor-not-allowed ${className}`}
        {...props}
      />
    </div>
  );
}
