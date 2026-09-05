import SwiftUI
import AppKit

struct PermissionOnboardingView: View {
    @ObservedObject var permissionService = PermissionService.shared
    var onPermissionGranted: () -> Void

    var body: some View {
        VStack(spacing: 24) {
            // App Icon / Logo
            ZStack {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [Color.blue, Color.purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 80, height: 80)
                    .shadow(color: .blue.opacity(0.3), radius: 10, x: 0, y: 5)

                Image(systemName: "camera.viewfinder")
                    .font(.system(size: 38, weight: .medium))
                    .foregroundColor(.white)
            }
            .padding(.top, 12)

            // Header Title
            VStack(spacing: 8) {
                Text("Screen Recording Permission")
                    .font(.system(size: 20, weight: .bold))
                
                Text("PinShot needs Screen Recording permission to capture screenshots across your displays and create floating pinned images.")
                    .font(.system(size: 13))
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                    .lineSpacing(2)
                    .frame(maxWidth: 360)
            }

            // Steps Guide
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 12) {
                    Image(systemName: "1.circle.fill")
                        .foregroundColor(.blue)
                        .font(.system(size: 16))
                    Text("Click the button below to open System Settings.")
                        .font(.system(size: 12))
                }
                HStack(spacing: 12) {
                    Image(systemName: "2.circle.fill")
                        .foregroundColor(.blue)
                        .font(.system(size: 16))
                    Text("Find PinShot and enable Screen Recording access.")
                        .font(.system(size: 12))
                }
                HStack(spacing: 12) {
                    Image(systemName: "3.circle.fill")
                        .foregroundColor(.blue)
                        .font(.system(size: 16))
                    Text("PinShot will automatically activate once granted.")
                        .font(.system(size: 12))
                }
            }
            .padding(16)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.6))
            .cornerRadius(10)
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 0.5)
            )

            // Action Buttons
            HStack(spacing: 12) {
                Button(action: {
                    permissionService.openSystemSettings()
                }) {
                    HStack(spacing: 6) {
                        Image(systemName: "gearshape.fill")
                        Text("Open System Settings")
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 6)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)

                Button(action: {
                    if permissionService.checkPermission() {
                        onPermissionGranted()
                    } else {
                        permissionService.requestPermission()
                    }
                }) {
                    Text("Check Again")
                        .padding(.vertical, 6)
                }
                .buttonStyle(.bordered)
                .controlSize(.large)
            }
            .frame(maxWidth: 360)
            .padding(.bottom, 12)
        }
        .padding(28)
        .frame(width: 440)
        .onReceive(permissionService.$hasScreenRecordingPermission) { granted in
            if granted {
                onPermissionGranted()
            }
        }
    }
}

@MainActor
final class PermissionWindowManager {
    static let shared = PermissionWindowManager()
    private var window: NSWindow?

    func show(onGranted: @escaping () -> Void) {
        if window == nil {
            let view = PermissionOnboardingView(onPermissionGranted: { [weak self] in
                self?.close()
                onGranted()
            })
            let hostingController = NSHostingController(rootView: view)
            let win = NSWindow(contentViewController: hostingController)
            win.title = "PinShot Permissions"
            win.styleMask = [.titled, .closable]
            win.titlebarAppearsTransparent = true
            win.center()
            win.isReleasedWhenClosed = false
            self.window = win
        }
        window?.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
    }

    func close() {
        window?.orderOut(nil)
        window = nil
    }
}
