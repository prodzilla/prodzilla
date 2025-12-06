# Icon Library

A centralized collection of SVG icons as React components.

## Usage

```tsx
import { XIcon } from '@/lib/icons';

// Basic usage
<XIcon />

// With custom className
<XIcon className="w-4 h-4 text-red-500" />

// With additional SVG props
<XIcon onClick={handleClick} aria-label="Close" />
```

## Available Icons

- **XIcon**: Close/X icon for buttons and modals

## Adding New Icons

1. Create a new component file in this directory (e.g., `ArrowIcon.tsx`)
2. Follow the existing pattern:
   ```tsx
   import type React from 'react';

   type ArrowIconProps = React.SVGProps<SVGSVGElement>;

   export default function ArrowIcon({ className = "w-5 h-5", ...props }: ArrowIconProps) {
     return (
       <svg
         className={className}
         fill="none"
         stroke="currentColor"
         viewBox="0 0 24 24"
         {...props}
       >
         {/* SVG paths here */}
       </svg>
     );
   }
   ```
3. Export it from `index.ts`
4. Update this README

## Design Guidelines

- Default size should be `w-5 h-5` (20px)
- Use `currentColor` for stroke/fill to inherit text color
- Accept all standard SVG props via spread operator
- Provide meaningful default className that can be overridden