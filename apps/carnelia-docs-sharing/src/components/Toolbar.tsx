/**
 * Editor Toolbar Component
 * 
 * Formatting controls for the collaborative text editor.
 */

import { useCallback } from 'react';

interface ToolbarProps {
  onBold: () => void;
  onItalic: () => void;
  onUnderline: () => void;
  onStrikethrough: () => void;
  onLink: () => void;
  disabled?: boolean;
}

export function Toolbar({ 
  onBold, 
  onItalic, 
  onUnderline, 
  onStrikethrough,
  onLink,
  disabled = false 
}: ToolbarProps) {
  const buttonClass = `
    px-3 py-1.5 rounded font-medium text-sm
    transition-colors duration-150
    disabled:opacity-50 disabled:cursor-not-allowed
    hover:bg-gray-200 active:bg-gray-300
    border border-gray-300
  `;

  return (
    <div className="flex items-center gap-1 p-2 bg-gray-50 border-b border-gray-200 rounded-t-lg">
      <button
        onClick={onBold}
        disabled={disabled}
        className={buttonClass}
        title="Bold (Ctrl+B)"
      >
        <span className="font-bold">B</span>
      </button>
      
      <button
        onClick={onItalic}
        disabled={disabled}
        className={buttonClass}
        title="Italic (Ctrl+I)"
      >
        <span className="italic">I</span>
      </button>
      
      <button
        onClick={onUnderline}
        disabled={disabled}
        className={buttonClass}
        title="Underline (Ctrl+U)"
      >
        <span className="underline">U</span>
      </button>
      
      <button
        onClick={onStrikethrough}
        disabled={disabled}
        className={buttonClass}
        title="Strikethrough"
      >
        <span className="line-through">S</span>
      </button>
      
      <div className="w-px h-6 bg-gray-300 mx-1" />
      
      <button
        onClick={onLink}
        disabled={disabled}
        className={buttonClass}
        title="Insert Link (Ctrl+K)"
      >
        🔗
      </button>
    </div>
  );
}
