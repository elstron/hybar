# MUELLE - A Hyprland Taskbar

**Muelle** is a **custom status bar for Wayland/Hyprland** written in Rust. It is a graphical interface project that provides a top bar with widgets and system information. It is an alternative inspired by Waybar but designed for the Hyprland compositor.

### **Features:**

- Monitoring of active and urgent workspaces and changes
- AutoHide in full-screen mode
- Three bar sections: left, center, and right
- Configuration system using JSON (`config.json`)
- Custom CSS styles
- Hot reload system (work in progress...)
- **AutoHide** mode.

### **Architecture**

- **Language**: Rust (with GTK4 for the graphical interface)
- **Platform**: Linux with Wayland/Hyprland compositor
- Uses `gtk4-layer-shell` to integrate as an overlay layer in Wayland
- Modular architecture with workspace (several packages: `muelle-core`, widgets, apps)
