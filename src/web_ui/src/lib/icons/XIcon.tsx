import type React from 'react';

type XIconProps = React.SVGProps<SVGSVGElement>;

export default function XIcon({ className = "w-5 h-5", ...props }: XIconProps) {
  return (
    <svg
      className={className}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M6 18L18 6M6 6l12 12"
      />
    </svg>
  );
}