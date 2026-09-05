import SwiftUI
import AppKit

struct PermissionOnboardingView: View {
    @ObservedObject var permissionService = PermissionService.shared
    var onPermissionGranted: () -> Void

    var body: some View {
        VStack(spacing: 20) {
            // App Icon / Header
            ZStack {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [Color.blue, Color.purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 72, height: 72)
                    .shadow(color: .blue.opacity(0.3), radius: 8, x: 0, y: 4)

                Image(systemName: "camera.viewfinder")
                    .font(.system(size: 34, weight: .medium))
                    .foregroundColor(.white)
            }
            .padding(.top, 8)

            // Header Title
            VStack(spacing: 6) {
                Text("Screen Recording Permission")
                    .font(.system(size: 19, weight: .bold))
                
                Text("PinShot membutuhkan izin Screen Recording untuk mengambil screenshot multi-monitor dan membuat pinned image.")
                    .font(.system(size: 12))
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                    .lineSpacing(2)
                    .frame(maxWidth: 380)
            }

            // Steps Guide
            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: "1.circle.fill")
                        .foregroundColor(.blue)
                        .font(.system(size: 15))
                    Text("Klik **Request Permission** atau **Open System Settings** di bawah.")
                        .font(.system(size: 12))
                }
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: "2.circle.fill")
                        .foregroundColor(.blue)
                        .font(.system(size: 15))
                    Text("Aktifkan toggle untuk **PinShot** di daftar Screen Recording.")
                        .font(.system(size: 12))
                }
                HStack(alignment: .top, spacing: 10) {
                    Image(systemName: "info.circle.fill")
                        .foregroundColor(.orange)
                        .font(.system(size: 15))
                    Text("Jika PinShot belum muncul di daftar, klik tombol **+** di System Settings dan pilih PinShot.app (klik tombol 'Show in Finder' di bawah).")
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                }
            }
            .padding(14)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.6))
            .cornerRadius(10)
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color(NSColor.separatorColor), lineWidth: 0.5)
            )

            // Action Buttons
            VStack(spacing: 8) {
                HStack(spacing: 10) {
                    Button(action: {
                        permissionService.openSystemSettings()
                    }) {
                        HStack(spacing: 6) {
                            Image(systemName: "gearshape.fill")
                            Text("Open System Settings")
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 5)
                    }
                    .buttonStyle(.borderedProminent)
                    .controlSize(.large)

                    Button(action: {
                        permissionService.requestPermission()
                    }) {
                        HStack(spacing: 4) {
                            Image(systemName: "hand.raised.fill")
                            Text("Request Prompt")
                        }
                        .padding(.vertical, 5)
                    }
                    .buttonStyle(.bordered)
                    .controlSize(.large)
                }

                HStack(spacing: 12) {
                    Button(action: {
                        let appUrl = Bundle.main.bundleURL
                        NSWorkspace.shared.activateFileViewerSelecting([appUrl])
                    }) {
                        Label("Show PinShot.app in Finder", systemImage: "folder")
                            .font(.system(size: 11))
                    }
                    .buttonStyle(.link)

                    Spacer()

                    Button(action: {
                        if permissionService.checkPermission() {
                            onPermissionGranted()
                        } else {
                            permissionService.requestPermission()
                        }
                    }) {
                        Text("Check Again")
                            .font(.system(size: 11, weight: .semibold))
                    }
                    .buttonStyle(.plain)
                    .foregroundColor(.blue)
                }
                .padding(.horizontal, 4)
            }
            .frame(maxWidth: 380)
            .padding(.bottom, 6)
        }
        .padding(24)
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
