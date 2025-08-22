import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Settings, X } from "lucide-react";
import "./App.css";

function App() {
  const [noteContent, setNoteContent] = useState("");
  const [showSettings, setShowSettings] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Load existing note content on startup
    loadNote();
  }, []);

  useEffect(() => {
    // Auto-save note content after typing stops
    const timeoutId = setTimeout(() => {
      if (noteContent !== undefined) {
        saveNote();
      }
    }, 500);

    return () => clearTimeout(timeoutId);
  }, [noteContent]);

  const loadNote = async () => {
    try {
      const content = await invoke<string>("load_note");
      setNoteContent(content);
    } catch (error) {
      console.error("Failed to load note:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const saveNote = async () => {
    try {
      await invoke("save_note", { content: noteContent });
    } catch (error) {
      console.error("Failed to save note:", error);
    }
  };

  const handleDragStart = async () => {
    const window = getCurrentWindow();
    await window.startDragging();
  };

  const forceWindowOnTop = async () => {
    try {
      await invoke("force_window_on_top");
      console.log("Forced window on top");
    } catch (error) {
      console.error("Failed to force window on top:", error);
    }
  };

  const debugWindowInfo = async () => {
    try {
      const info = await invoke<string>("debug_window_info");
      console.log("Window Debug Info:\n", info);
      alert("Debug info logged to console:\n\n" + info);
    } catch (error) {
      console.error("Failed to get debug info:", error);
    }
  };

  if (isLoading) {
    return (
      <div className="app-container">
        <div className="glass-reflection"></div>
        <div className="drag-handle" data-tauri-drag-region onMouseDown={handleDragStart}>
          <span className="app-title">Notes</span>
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
        <span className="app-title">Notes</span>
        <button 
          className="settings-button"
          onClick={() => setShowSettings(true)}
          title="Settings"
        >
          <Settings size={14} />
        </button>
      </div>
      
      <div className="content-area">
        <textarea
          className="notes-textarea"
          value={noteContent}
          onChange={(e) => setNoteContent(e.target.value)}
          placeholder="Start typing your notes..."
          autoFocus
        />
      </div>

      {showSettings && (
        <div className="settings-panel">
          <div className="settings-header">
            <h2 className="settings-title">Settings</h2>
            <button 
              className="close-button"
              onClick={() => setShowSettings(false)}
              title="Close"
            >
              <X size={16} />
            </button>
          </div>
          
          <div className="settings-content">
            <div className="setting-group">
              <p>This is a minimal notes app that stays on top of all windows, including fullscreen apps.</p>
              <p>Drag the title bar to move the window around.</p>
              <p>Your notes are automatically saved.</p>
              <p><strong>Keyboard Shortcut:</strong> Press <kbd>⌘</kbd> + <kbd>⇧</kbd> + <kbd>`</kbd> once to hide, once more to show the app from anywhere.</p>
            </div>
            
            <div className="setting-group">
              <h3 style={{ fontSize: '14px', margin: '0 0 8px 0', fontWeight: '600' }}>Fullscreen Overlay</h3>
              <p style={{ fontSize: '12px', margin: '0 0 8px 0', color: 'rgba(255,255,255,0.7)' }}>
                If the app is not appearing over fullscreen applications, click the button below:
              </p>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button 
                  onClick={forceWindowOnTop}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#007AFF',
                    border: 'none',
                    borderRadius: '4px',
                    color: 'white',
                    fontSize: '12px',
                    fontWeight: '500',
                    cursor: 'pointer'
                  }}
                >
                  Force Window On Top
                </button>
                <button 
                  onClick={debugWindowInfo}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#6B7280',
                    border: 'none',
                    borderRadius: '4px',
                    color: 'white',
                    fontSize: '12px',
                    fontWeight: '500',
                    cursor: 'pointer'
                  }}
                >
                  Debug Info
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;