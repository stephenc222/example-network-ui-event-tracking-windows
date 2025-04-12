# Windows Application Event & Network Tracker — Tutorial Project

This repository demonstrates how to monitor user interactions and network activity of a Windows desktop application in real time.

The system consists of two components:

- A C# WinForms application that triggers outbound HTTP requests.
- A Rust-based observer tool that logs UI events and correlates them with TCP connections at the process level.

This project is intended as an educational resource for learning how to use low-level Windows APIs (UI Automation, Win32 hooks, IP Helper APIs) to build practical tooling for system monitoring.

[Read the full blog post](https://stephencollins.tech/posts/windows-ui-network-monitoring-rust-csharp) about this project.

The macos counterpart tutorial project can be found [here](https://github.com/stephenc222/example-network-ui-event-tracking-macos)

---

## 📁 Repository Structure

```
.
├── ExampleWindowsApp/        # C# application for simulating UI + network activity
├── windows-watcher/          # Rust monitoring tool with hooks and TCP inspection
├── .gitignore
```

---

## 🧱 Project Breakdown

### `ExampleWindowsApp/` (C# WinForms)

A simple .NET 8.0 desktop application with three buttons (`A`, `B`, and `C`). Each button sends an HTTP request to a mock API (`jsonplaceholder.typicode.com`) and prints the result.

**Learning goals:**

- Create a minimal UI for testing
- Simulate real-world network activity on user interaction
- Understand how to programmatically trigger and inspect frontend actions

---

### `windows-watcher/` (Rust)

A background service that runs alongside any desktop application and performs:

- **Mouse + keyboard monitoring** via global Win32 hooks
- **UI element inspection** using the Windows UI Automation (UIA) API
- **Network connection tracking** by inspecting the system TCP table and mapping connections to PIDs

**Learning goals:**

- Understand how to hook into user input events system-wide
- Extract metadata from UI elements (e.g., element names, automation IDs, process ownership)
- Track live TCP connections using the Windows IP Helper API
- Correlate UI interactions with network activity at runtime

---

## 🚀 Running the Tutorial

### Prerequisites

- Windows 10 or later
- [.NET 8.0 SDK](https://dotnet.microsoft.com/)
- [Rust toolchain](https://rustup.rs/)
- Visual Studio (to build the C# project)

---

### 1. Launch the Example UI Application

```bash
cd ExampleWindowsApp
start ExampleWindowsApp.sln
```

Build and run the application via Visual Studio.

---

### 2. Start the Rust Monitoring Tool

```bash
cd windows-watcher
cargo run --release
```

The watcher will immediately begin logging:

- UI clicks (with element name and automation ID)
- Active TCP connections, including remote addresses, ports, and process IDs

> Press `ESC` or `Ctrl+C` to exit the watcher.

---

## 📝 Log Output

Logs are stored at:

```
%LOCALAPPDATA%\WindowsWatcher\windows_watcher.log
```

Example entry:
```
[2025-04-10 15:13:07.775] Element: App='ExampleWindowsApp.exe', Name='Button A', AutomationID='ButtonA'
[2025-04-10 15:13:07.882] TCP: 10.0.0.5:56832 → 104.21.64.1:47872, PID=10792, STATE=ESTABLISHED
```

> See [`example_output.txt`](windows-watcher/example_output.txt) for a full sample log.

---

## 📚 Key Concepts Explored

| Concept                        | Technique / API                                |
|-------------------------------|--------------------------------------------------|
| UI event logging              | Win32 Hooks: `SetWindowsHookExW`                |
| Element inspection            | `IUIAutomation::ElementFromPoint`               |
| Network monitoring            | `GetExtendedTcpTable` (IP Helper API)           |
| Process identification        | `GetForegroundWindow`, `GetWindowThreadProcessId` |
| Logging to disk               | `dirs`, `chrono`, file I/O in Rust              |
| Graceful shutdown             | Keyboard hook (`ESC` / `Ctrl+C`) detection      |

---

## 🧠 Why This Matters

Being able to correlate user actions with process-level behavior is essential for:

- Debugging complex UI applications
- Teaching system programming concepts
- Building monitoring tools or security software
- Understanding behavioral baselines for applications

---

## ⚠️ Disclaimer

This project installs global system hooks and inspects process-level behavior. It should only be used in secure, local environments for learning and experimentation.

---

## License

This tutorial project is open source under the MIT License. You are free to learn from and extend it.