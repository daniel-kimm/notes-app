import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Info, X, Type, Bold, Italic, Underline, CheckSquare, List } from "lucide-react";
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import TaskList from '@tiptap/extension-task-list';
import TaskItem from '@tiptap/extension-task-item';
import Placeholder from '@tiptap/extension-placeholder';
import "./App.css";

function App() {
  const [showInfo, setShowInfo] = useState(false);
  const [showToolbar, setShowToolbar] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize TipTap editor
  const editor = useEditor({
    extensions: [
      StarterKit,
      TaskList,
      TaskItem.configure({
        nested: true,
      }),
      Placeholder.configure({
        placeholder: 'Start typing your notes...',
      }),
    ],
    content: '',
    onUpdate: ({ editor }) => {
      const text = editor.getText();
      // Auto-save after typing stops
      debouncedSave(text);
    },
  });

  // Debounced save function
  const debouncedSave = debounce((content: string) => {
    saveNote(content);
  }, 500);

  useEffect(() => {
    loadNote();
  }, []);

  const loadNote = async () => {
    try {
      const content = await invoke<string>("load_note");
      
      // Set content in TipTap editor
      if (editor && content) {
        // Convert plain text to basic HTML
        const htmlContent = content.replace(/\n/g, '<br>');
        editor.commands.setContent(htmlContent);
      }
      
      // Focus editor
      setTimeout(() => {
        editor?.commands.focus();
      }, 100);
    } catch (error) {
      console.error("Failed to load note:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const saveNote = async (content: string) => {
    try {
      await invoke("save_note", { content });
    } catch (error) {
      console.error("Failed to save note:", error);
    }
  };

  const handleDragStart = async () => {
    const window = getCurrentWindow();
    await window.startDragging();
  };

  // TipTap formatting functions
  const formatBold = () => editor?.chain().focus().toggleBold().run();
  const formatItalic = () => editor?.chain().focus().toggleItalic().run();
  const formatUnderline = () => editor?.chain().focus().toggleUnderline().run();
  const formatBulletPoint = () => editor?.chain().focus().toggleBulletList().run();
  const formatChecklist = () => editor?.chain().focus().toggleTaskList().run();

  // Debounce utility function
  function debounce<T extends (...args: any[]) => any>(func: T, delay: number): T {
    let timeoutId: number | undefined;
    return ((...args: any[]) => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => func.apply(null, args), delay);
    }) as T;
  }

  if (isLoading) {
    return (
      <div className="app-container">
        <div className="glass-reflection"></div>
        <div className="drag-handle" data-tauri-drag-region onMouseDown={handleDragStart}>
          <span className="app-title">Float</span>
        </div>
        <div className="content-area">
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
            Loading...
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <div className="glass-reflection"></div>
      <div className="drag-handle" data-tauri-drag-region onMouseDown={handleDragStart}>
        <span className="app-title">Float</span>
        <div className="toolbar-buttons">
          <button 
            className="toolbar-button"
            onClick={() => setShowToolbar(!showToolbar)}
            title="Formatting Toolbar"
          >
            <Type size={14} />
          </button>
          <button 
            className="info-button"
            onClick={() => setShowInfo(true)}
            title="Info"
          >
            <Info size={14} />
          </button>
        </div>
      </div>
      
      {showToolbar && (
        <div className="formatting-toolbar">
          <button 
            className="format-button"
            onClick={formatBold}
            title="Bold"
          >
            <Bold size={14} />
          </button>
          <button 
            className="format-button"
            onClick={formatItalic}
            title="Italic"
          >
            <Italic size={14} />
          </button>
          <button 
            className="format-button"
            onClick={formatUnderline}
            title="Underline"
          >
            <Underline size={14} />
          </button>
          <button 
            className="format-button"
            onClick={formatChecklist}
            title="Checklist"
          >
            <CheckSquare size={14} />
          </button>
          <button 
            className="format-button"
            onClick={formatBulletPoint}
            title="Bullet Point"
          >
            <List size={14} />
          </button>
        </div>
      )}
      
      <div className="content-area">
        <div 
          onFocus={(e) => e.stopPropagation()}
          onBlur={(e) => e.stopPropagation()}
          onClick={(e) => e.stopPropagation()}
        >
          <EditorContent 
            editor={editor} 
            className="notes-editor"
            onFocus={(e) => e.stopPropagation()}
            onBlur={(e) => e.stopPropagation()}
          />
        </div>
      </div>

      {showInfo && (
        <div className="info-panel">
          <div className="info-header">
            <h2 className="info-title">Info</h2>
            <button 
              className="close-button"
              onClick={() => setShowInfo(false)}
              title="Close"
            >
              <X size={14} />
            </button>
          </div>
          
          <div className="info-content">
            <div className="info-group">
              <p>This is a minimal notes app that stays on top of all windows, including fullscreen apps.</p>
              <p>Drag the title bar to move the window around.</p>
              <p>Your notes are automatically saved.</p>
              <p><strong>Keyboard Shortcut:</strong> Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>`</kbd> (Windows) or <kbd>⌘</kbd> + <kbd>⇧</kbd> + <kbd>`</kbd> (macOS) once to hide, once more to show the app from anywhere.</p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;