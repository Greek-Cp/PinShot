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

    /// Actively triggers the macOS system dialog & registers PinShot in System Settings (TCC).
    func requestPermission() {
        // 1. CGRequestScreenCaptureAccess (macOS legacy / standard TCC prompt)
        CGRequestScreenCaptureAccess()

        // 2. Modern ScreenCaptureKit attempt - this forces macOS 14/15/16 to show
        // "PinShot would like to record this computer's screen" and adds it to the list in Settings.
        Task {
            do {
                let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
                if let display = content.displays.first {
                    let filter = SCContentFilter(display: display, excludingWindows: [])
                    let config = SCStreamConfiguration()
                    config.width = 64
                    config.height = 64
                    _ = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: config)
                }
            } catch {
                // Throws if permission is denied/not yet granted, but the attempt itself registers the app in TCC.
            }
        }

        // 3. Fallback display capture to touch CG display
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
