import Cocoa
import CoreGraphics
import ScreenCaptureKit

@MainActor
final class PermissionService: ObservableObject {
    static let shared = PermissionService()

    @Published var hasScreenRecordingPermission: Bool = false
    private var timer: Timer?

    init() {
        checkPermission()
    }

    /// Checks if screen recording permission is currently granted.
    @discardableResult
    func checkPermission() -> Bool {
        let granted = CGPreflightScreenCaptureAccess()
        self.hasScreenRecordingPermission = granted
        return granted
    }

    /// Prompts the macOS system permission dialog and registers the app in System Settings (TCC).
    func requestPermission() {
        // 1. CGRequestScreenCaptureAccess initiates the system prompt
        CGRequestScreenCaptureAccess()

        // 2. Trigger ScreenCaptureKit query to force TCC to register the bundle
        Task {
            do {
                _ = try await SCShareableContent.current
            } catch {
                // Ignore error; the call itself triggers TCC registration
            }
        }

        // 3. Attempt sample display image to register
        _ = CGDisplayCreateImage(CGMainDisplayID())

        startPolling()
    }

    /// Opens macOS System Settings directly to Privacy & Security -> Screen Recording.
    func openSystemSettings() {
        requestPermission()

        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture") {
            NSWorkspace.shared.open(url)
        }
        startPolling()
    }

    /// Starts polling in the background so when the user toggles permission in Settings,
    /// the app notices immediately without requiring a full manual relaunch.
    func startPolling() {
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                guard let self = self else { return }
                if self.checkPermission() {
                    self.stopPolling()
                }
            }
        }
    }

    func stopPolling() {
        timer?.invalidate()
        timer = nil
    }
}
