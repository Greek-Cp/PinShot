import Cocoa
import SwiftUI
import PinShotCore

@MainActor
final class MenuBarController: NSObject {
    static let shared = MenuBarController()

    private var statusItem: NSStatusItem?

    func setupMenuBar() {
        guard statusItem == nil else { return }

        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = item.button {
            button.image = NSImage(systemSymbolName: "camera.viewfinder", accessibilityDescription: "PinShot")
            button.image?.isTemplate = true
        }

        let menu = NSMenu()

        // Capture Area
        let captureItem = NSMenuItem(
            title: "Capture Area",
            action: #selector(captureAreaAction),
            keyEquivalent: "a"
        )
        captureItem.keyEquivalentModifierMask = [.command, .shift]
        captureItem.target = self
        menu.addItem(captureItem)

        menu.addItem(NSMenuItem.separator())

        // Active Pins Submenu
        let pinsMenu = NSMenu()
        let pinsMenuItem = NSMenuItem(title: "Active Pins", action: nil, keyEquivalent: "")
        pinsMenuItem.submenu = pinsMenu
        menu.addItem(pinsMenuItem)

        // Close All Pins
        let closeAllItem = NSMenuItem(
            title: "Close All Pins",
            action: #selector(closeAllPinsAction),
            keyEquivalent: ""
        )
        closeAllItem.target = self
        menu.addItem(closeAllItem)

        menu.addItem(NSMenuItem.separator())

        // Settings / Preferences
        let settingsItem = NSMenuItem(
            title: "Preferences...",
            action: #selector(openSettingsAction),
            keyEquivalent: ","
        )
        settingsItem.keyEquivalentModifierMask = [.command]
        settingsItem.target = self
        menu.addItem(settingsItem)

        // Permission check
        let permissionItem = NSMenuItem(
            title: "Check Permissions...",
            action: #selector(openPermissionsAction),
            keyEquivalent: ""
        )
        permissionItem.target = self
        menu.addItem(permissionItem)

        menu.addItem(NSMenuItem.separator())

        // Quit
        let quitItem = NSMenuItem(
            title: "Quit PinShot",
            action: #selector(quitAction),
            keyEquivalent: "q"
        )
        quitItem.target = self
        menu.addItem(quitItem)

        item.menu = menu
        self.statusItem = item
    }

    @objc private func captureAreaAction() {
        OverlayWindowManager.shared.startCapture()
    }

    @objc private func closeAllPinsAction() {
        PinWindowManager.shared.closeAllPins()
    }

    @objc private func openSettingsAction() {
        SettingsWindowManager.shared.show()
    }

    @objc private func openPermissionsAction() {
        PermissionWindowManager.shared.show {
            // Permission granted
        }
    }

    @objc private func quitAction() {
        NSApplication.shared.terminate(nil)
    }
}
