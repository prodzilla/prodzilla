import { useEffect } from 'react';

/**
 * Hook to handle escape key press for closing modals/sidebars
 * @param isOpen - Whether the modal/sidebar is currently open
 * @param onClose - Callback function to call when escape is pressed
 */
export function useEscapeKey(isOpen: boolean, onClose: () => void) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleKeyDown);
      return () => document.removeEventListener('keydown', handleKeyDown);
    }
  }, [isOpen, onClose]);
}
