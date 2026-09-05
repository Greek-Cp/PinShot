import Cocoa
import PinShotCore

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Run as menu bar accessory app (no Dock icon)
        NSApp.setActivationPolicy(.accessory)

        // Setup Menu Bar Item
        MenuBarController.shared.setupMenuBar()

        // Check Screen Recording Permission on launch
        if PermissionService.shared.checkPermission() {
            setupServices()
        } else {
            // Show Onboarding / Permission Guide window
            PermissionWindowManager.shared.show { [weak self] in
                self?.setupServices()
            }
        }
    }

    private func setupServices() {
        // Register Global HotKey (Command + Shift + A)
        HotKeyService.shared.registerDefaultHotKey {
            OverlayWindowManager.shared.startCapture()
        }
    }
}
