import SwiftUI
import AppKit
import PinShotCore

struct SettingsView: View {
    @ObservedObject var settingsManager = SettingsManager.shared
    @State private var selectedTab: SettingsTab = .shortcuts

    enum SettingsTab: String, CaseIterable, Identifiable {
        case shortcuts = "Shortcuts"
        case general = "General"
        case output = "Output"

        var id: String { rawValue }
        var icon: String {
            switch self {
            case .shortcuts: return "keyboard"
            case .general: return "gearshape"
            case .output: return "folder"
            }
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Tab Selector
            Picker("", selection: $selectedTab) {
                ForEach(SettingsTab.allCases) { tab in
                    Label(tab.rawValue, systemImage: tab.icon).tag(tab)
                }
            }
            .pickerStyle(.segmented)
            .padding(.horizontal, 24)
            .padding(.top, 16)
            .padding(.bottom, 12)

            Divider()

            // Tab Contents
            Group {
                switch selectedTab {
                case .shortcuts:
                    shortcutsTab
                case .general:
                    generalTab
                case .output:
                    outputTab
                }
            }
            .padding(24)
            .frame(width: 480, height: 320, alignment: .topLeading)
        }
    }

    // MARK: - Shortcuts Tab
    private var shortcutsTab: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("Keyboard Shortcuts")
                .font(.headline)

            VStack(spacing: 12) {
                // Global Capture Area Shortcut
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Capture Area (Global)")
                            .font(.system(size: 13, weight: .medium))
                        Text("Activates fullscreen capture across all monitors")
                            .font(.system(size: 11))
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Picker("", selection: $settingsManager.settings.globalShortcutName) {
                        Text("⌘⇧A (Command + Shift + A)").tag("⌘⇧A (Command + Shift + A)")
                        Text("⌥A (Option + A)").tag("⌥A (Option + A)")
                        Text("⌘⇧1 (Command + Shift + 1)").tag("⌘⇧1 (Command + Shift + 1)")
                        Text("F1").tag("F1")
                    }
                    .frame(width: 200)
                    .onChange(of: settingsManager.settings.globalShortcutName) { _, newName in
                        updateGlobalKeyCode(from: newName)
                    }
                }

                Divider()

                // In-Overlay Pin Shortcut
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Pin Selected Area")
                            .font(.system(size: 13, weight: .medium))
                        Text("Turns selection into a floating pinned window")
                            .font(.system(size: 11))
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Picker("", selection: $settingsManager.settings.pinShortcutName) {
                        Text("⌘P or P").tag("⌘P / P")
                        Text("⌘P only").tag("⌘P")
                        Text("P only").tag("P")
                    }
                    .frame(width: 140)
                }

                Divider()

                // In-Overlay Copy & Save Shortcuts
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Copy to Clipboard")
                            .font(.system(size: 13, weight: .medium))
                    }
                    Spacer()
                    Text("⌘C / Enter / C")
                        .font(.system(size: 12, weight: .semibold, design: .monospaced))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(6)
                }

                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Save to File")
                            .font(.system(size: 13, weight: .medium))
                    }
                    Spacer()
                    Text("⌘S / S")
                        .font(.system(size: 12, weight: .semibold, design: .monospaced))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(6)
                }
            }
            .padding(14)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
            .cornerRadius(10)
        }
    }

    private func updateGlobalKeyCode(from name: String) {
        if name.contains("⌥A") {
            settingsManager.settings.globalKeyCode = 0 // 'A'
            settingsManager.settings.globalModifiers = 0x0800 // optionKey
        } else if name.contains("⌘⇧1") {
            settingsManager.settings.globalKeyCode = 18 // '1'
            settingsManager.settings.globalModifiers = 0x0100 | 0x0200 // cmdKey | shiftKey
        } else if name.contains("F1") {
            settingsManager.settings.globalKeyCode = 122 // F1
            settingsManager.settings.globalModifiers = 0
        } else {
            // Default ⌘⇧A
            settingsManager.settings.globalKeyCode = 0
            settingsManager.settings.globalModifiers = 0x0100 | 0x0200
        }
        HotKeyService.shared.reloadHotKey()
    }

    // MARK: - General Tab
    private var generalTab: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("General Preferences")
                .font(.headline)

            VStack(alignment: .leading, spacing: 14) {
                Toggle("Play camera sound effect on capture & pin", isOn: $settingsManager.settings.playSoundEffects)
                Toggle("Show Loupe magnifier & color picker under cursor", isOn: $settingsManager.settings.showMagnifierLoupe)
                Toggle("Auto-copy to clipboard immediately upon selection", isOn: $settingsManager.settings.autoCopyToClipboardOnCapture)
            }
            .padding(14)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
            .cornerRadius(10)
        }
    }

    // MARK: - Output Tab
    private var outputTab: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("Output & Storage")
                .font(.headline)

            VStack(alignment: .leading, spacing: 14) {
                HStack {
                    Text("Default Format:")
                    Spacer()
                    Picker("", selection: $settingsManager.settings.imageFormat) {
                        ForEach(ImageOutputFormat.allCases, id: \.self) { format in
                            Text(format.rawValue).tag(format)
                        }
                    }
                    .frame(width: 120)
                }

                Divider()

                VStack(alignment: .leading, spacing: 6) {
                    Text("Save Directory:")
                    HStack {
                        Text(settingsManager.settings.saveDirectoryPath)
                            .font(.system(size: 11, design: .monospaced))
                            .lineLimit(1)
                            .truncationMode(.middle)
                            .padding(6)
                            .background(Color(NSColor.textBackgroundColor))
                            .cornerRadius(6)

                        Button("Choose Folder...") {
                            chooseDirectory()
                        }
                    }
                }
            }
            .padding(14)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.5))
            .cornerRadius(10)
        }
    }

    private func chooseDirectory() {
        let openPanel = NSOpenPanel()
        openPanel.canChooseFiles = false
        openPanel.canChooseDirectories = true
        openPanel.canCreateDirectories = true
        openPanel.allowsMultipleSelection = false
        openPanel.begin { response in
            if response == .OK, let url = openPanel.url {
                settingsManager.settings.saveDirectoryPath = url.path
            }
        }
    }
}

@MainActor
final class SettingsWindowManager {
    static let shared = SettingsWindowManager()
    private var window: NSWindow?

    func show() {
        if window == nil {
            let hosting = NSHostingController(rootView: SettingsView())
            let win = NSWindow(contentViewController: hosting)
            win.title = "PinShot Preferences"
            win.styleMask = [.titled, .closable]
            win.center()
            win.isReleasedWhenClosed = false
            self.window = win
        }
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }
}
