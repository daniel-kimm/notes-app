use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use tauri_nspanel::{WebviewWindowExt, cocoa::appkit::NSWindowCollectionBehavior};

// macOS-specific window configuration for fullscreen overlay
#[cfg(target_os = "macos")]
#[tauri::command]
async fn ensure_window_top_level(window: WebviewWindow) -> Result<(), String> {
    window.with_webview(|webview| {
        #[cfg(target_os = "macos")]
        unsafe {
            let ns_window = webview.ns_window() as id;
            
            // CRITICAL: Try MAXIMUM window levels for fullscreen overlay
            #[allow(non_upper_case_globals)]
            const NSScreenSaverWindowLevel: i32 = 1000;    // Screen saver level
            #[allow(non_upper_case_globals)]
            const NSModalPanelWindowLevel: i32 = 8;        // Modal panel level
            #[allow(non_upper_case_globals)]
            const NSMainMenuWindowLevel: i32 = 24;         // Main menu level
            #[allow(non_upper_case_globals)]
            const NSStatusWindowLevel: i32 = 25;           // Status window level
            #[allow(non_upper_case_globals)]
            const NSPopUpMenuWindowLevel: i32 = 101;       // Pop-up menu level
            #[allow(non_upper_case_globals)]
            const NSFloatWindowLevel: i32 = 4;             // Float level
            
            // Try the ABSOLUTE HIGHEST level - even higher than screen saver
            let level = 2147483647; // INT_MAX - maximum possible window level
            let _: () = msg_send![ns_window, setLevel: level];
            
            // CRITICAL: Set window collection behavior for fullscreen support
            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary 
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle;
            let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
            
            // CRITICAL: DON'T use nonactivating panel mask on NSWindow - it doesn't support it!
            // Instead, use these settings to prevent activation without breaking the window
            
            // CRITICAL: Only use absolutely SAFE API calls
            let _: () = msg_send![ns_window, setIgnoresMouseEvents: false];
            
            println!("Applied macOS fullscreen overlay configuration with MAXIMUM LEVEL ({})", level);
        }
    }).map_err(|e| e.to_string())?;
    
    // Apply standard settings as well
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.set_visible_on_all_workspaces(true).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Non-macOS fallback
#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn ensure_window_top_level(window: WebviewWindow) -> Result<(), String> {
    // For non-macOS platforms, use standard settings
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.set_visible_on_all_workspaces(true).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn save_note(app_handle: AppHandle, content: String) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;
    
    let notes_file = app_data_dir.join("notes.txt");
    std::fs::write(notes_file, content)
        .map_err(|e| format!("Failed to save note: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn load_note(app_handle: AppHandle) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    
    let notes_file = app_data_dir.join("notes.txt");
    
    if notes_file.exists() {
        std::fs::read_to_string(notes_file)
            .map_err(|e| format!("Failed to load note: {}", e))
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
async fn toggle_window(window: WebviewWindow) -> Result<(), String> {
    let is_visible = window.is_visible().map_err(|e| e.to_string())?;
    
    if is_visible {
        // Hide the window
        window.hide().map_err(|e| e.to_string())?;
        println!("Window hidden via toggle shortcut");
    } else {
        // Show the window with full setup
        window.show().map_err(|e| e.to_string())?;
        
        // Small delay to ensure window is properly shown before applying settings
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Apply all window properties to ensure it's properly positioned and configured
        // Note: Removed set_focus() call to preserve non-activating behavior for fullscreen overlay
        window.set_always_on_top(true).map_err(|e| e.to_string())?;
        window.set_visible_on_all_workspaces(true).map_err(|e| e.to_string())?;
        
        // CRITICAL: Re-apply the top-level window settings aggressively when showing
        for i in 0..3 {
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            if let Err(e) = ensure_window_top_level(window.clone()).await {
                eprintln!("Failed to re-apply window settings on show (attempt {}): {}", i + 1, e);
            }
        }
        
        println!("Window shown via toggle shortcut");
    }
    Ok(())
}

#[tauri::command]
async fn force_window_on_top(window: WebviewWindow) -> Result<(), String> {
    // Force the window to be on top - can be called from frontend
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.set_visible_on_all_workspaces(true).map_err(|e| e.to_string())?;
    
    // CRITICAL: Also force NSPanel settings if on macOS
    #[cfg(target_os = "macos")]
    {
        if let Ok(panel) = window.to_panel() {
            panel.set_style_mask(128); // NSNonactivatingPanelMask
            panel.set_level(2147483647); // MAXIMUM level
            panel.set_collection_behaviour(
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary |
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces |
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary |
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle
            );
            
            // AGGRESSIVE FOCUS PREVENTION: Add more settings to prevent activation
            panel.set_accepts_mouse_moved_events(true);
            panel.set_becomes_key_only_if_needed(false);
            
            panel.set_ignore_mouse_events(false);
            println!("FORCED NSPANEL ON TOP WITH AGGRESSIVE FOCUS PREVENTION!");
        }
    }
    
    // Also apply the regular fullscreen overlay settings
    let _ = ensure_window_top_level(window.clone()).await;
    
    Ok(())
}

#[tauri::command]
async fn debug_window_info(window: WebviewWindow) -> Result<String, String> {
    // Simple debug without complex macOS introspection for now
    let is_visible = window.is_visible().map_err(|e| e.to_string())?;
    let is_always_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    
    let debug_info = format!(
        "Window visible: {}\nAlways on top: {}\nPlatform: macOS\nNote: Advanced debug disabled to avoid compilation issues\n",
        is_visible, is_always_on_top
    );
    
    println!("DEBUG Window Info:\n{}", debug_info);
    Ok(debug_info)
}

#[tauri::command]
async fn position_window_top_right(window: WebviewWindow) -> Result<(), String> {
    use tauri::PhysicalPosition;
    
    // Get the primary monitor size
    let monitor = window.primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("No primary monitor found")?;
    
    let monitor_size = monitor.size();
    let window_size = window.outer_size().map_err(|e| e.to_string())?;
    
    // Position in top-right corner with some padding
    let x = (monitor_size.width as i32) - (window_size.width as i32) - 20;
    let y = 40;
    
    window.set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_nspanel::init())
        .invoke_handler(tauri::generate_handler![
            save_note,
            load_note,
            toggle_window,
            position_window_top_right,
            ensure_window_top_level,
            force_window_on_top,
            debug_window_info
        ])
        .setup(|app| {
            // Set activation policy to Accessory to prevent dock icon (macOS only)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            let window = app.get_webview_window("main").unwrap();
            
            // CRITICAL: Convert to NSPanel for proper fullscreen overlay support
            #[cfg(target_os = "macos")]
            {
                let panel = window.to_panel().unwrap();
                
                // Set NSPanel with the correct style mask value
                panel.set_style_mask(128); // NSNonactivatingPanelMask = 1 << 7 = 128
                panel.set_level(2147483647); // MAXIMUM level
                panel.set_collection_behaviour( // British spelling!
                    NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary |
                    NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces |
                    NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary |
                    NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle
                );
                
                // AGGRESSIVE FOCUS PREVENTION: Add more settings to prevent activation
                panel.set_accepts_mouse_moved_events(true);
                panel.set_becomes_key_only_if_needed(false);
                
                // Make panel not ignore mouse events
                panel.set_ignore_mouse_events(false);
                
                println!("CONVERTED WINDOW TO NSPANEL WITH AGGRESSIVE FOCUS PREVENTION!");
            }
            
            let window_clone = window.clone();
            
            // Set window to be visible on all workspaces and always on top
            let _ = window.set_visible_on_all_workspaces(true);
            let _ = window.set_always_on_top(true);
            
            // NSPanel is already configured above, no need for repeated settings
            
            // Create a debouncer to prevent multiple rapid shortcut triggers
            let shortcut_debouncer = Arc::new(AtomicBool::new(false));
            let debouncer_clone = shortcut_debouncer.clone();
            
            // Register global shortcut for Cmd+Shift+` with debouncing
            app.global_shortcut().on_shortcut("Cmd+Shift+`", move |_, _, _| {
                // Check if we're already processing a shortcut
                if debouncer_clone.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    let window = window_clone.clone();
                    let debouncer = debouncer_clone.clone();
                    
                    tauri::async_runtime::spawn(async move {
                        // Process the toggle
                        let _ = toggle_window(window).await;
                        
                        // Add a small delay to prevent rapid repeated triggers
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        
                        // Reset the debouncer
                        debouncer.store(false, Ordering::SeqCst);
                    });
                }
            }).expect("Failed to register global shortcut");
            
            // Position window in top-right corner
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                let _ = position_window_top_right(window).await;
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}