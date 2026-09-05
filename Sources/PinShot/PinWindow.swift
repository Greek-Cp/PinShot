import Cocoa
import SwiftUI
import PinShotCore

@MainActor
final class PinWindowManager: ObservableObject {
    static let shared = PinWindowManager()

    @Published private(set) var activePins: [PinWindow] = []

    func createPin(image: NSImage, at origin: CGPoint? = nil) {
        let pin = PinWindow(image: image, initialOrigin: origin)
        activePins.append(pin)
        pin.show()
        SoundService.shared.playPinSound()
    }

    func closePin(_ pin: PinWindow) {
        pin.orderOut(nil)
        activePins.removeAll(where: { $0.pinID == pin.pinID })
    }

    func closeAllPins() {
        for pin in activePins {
            pin.orderOut(nil)
        }
        activePins.removeAll()
    }
}

final class PinWindow: NSPanel {
    let pinID = UUID()
    let baseImage: NSImage

    init(image: NSImage, initialOrigin: CGPoint? = nil) {
        self.baseImage = image
        let imageSize = image.size
        let maxInitialWidth: CGFloat = 600
        let scale = (imageSize.width > 0 && imageSize.width > maxInitialWidth) ? maxInitialWidth / imageSize.width : 1.0
        let windowWidth = max(imageSize.width * scale, 60)
        let windowHeight = max(imageSize.height * scale, 40)
        let windowSize = CGSize(width: windowWidth, height: windowHeight)

        let initialRect: CGRect
        if let origin = initialOrigin {
            initialRect = CGRect(origin: origin, size: windowSize)
        } else {
            let screenRect = NSScreen.main?.visibleFrame ?? CGRect(x: 100, y: 100, width: 800, height: 600)
            let x = screenRect.midX - (windowSize.width / 2)
            let y = screenRect.midY - (windowSize.height / 2)
            initialRect = CGRect(x: x, y: y, width: windowSize.width, height: windowSize.height)
        }

        super.init(
            contentRect: initialRect,
            styleMask: [.borderless, .resizable, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        self.isReleasedWhenClosed = false
        self.level = .floating
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.isOpaque = false
        self.backgroundColor = .clear
        self.hasShadow = true
        self.isMovableByWindowBackground = true
        if imageSize.width > 0 && imageSize.height > 0 {
            self.aspectRatio = imageSize
            self.minSize = CGSize(width: 80, height: max(80 * (imageSize.height / imageSize.width), 40))
        }

        setupContentView()
    }

    private func setupContentView() {
        let hostingView = NSHostingView(
            rootView: PinContentView(
                image: baseImage,
                onClose: { [weak self] in
                    guard let self = self else { return }
                    PinWindowManager.shared.closePin(self)
                },
                onCopy: { [weak self] in
                    guard let self = self else { return }
                    PasteboardService.shared.copyImage(self.baseImage)
                    SoundService.shared.playCaptureSound()
                },
                onSave: { [weak self] in
                    guard let self = self else { return }
                    self.saveImage()
                }
            )
        )
        self.contentView = hostingView
    }

    func show() {
        self.makeKeyAndOrderFront(nil)
    }

    private func saveImage() {
        let savePanel = NSSavePanel()
        savePanel.allowedContentTypes = [.png]
        savePanel.nameFieldStringValue = "PinShot_\(Int(Date().timeIntervalSince1970)).png"

        savePanel.begin { [weak self] response in
            guard response == .OK, let url = savePanel.url, let self = self else { return }
            if let tiff = self.baseImage.tiffRepresentation,
               let bitmap = NSBitmapImageRep(data: tiff),
               let pngData = bitmap.representation(using: .png, properties: [:]) {
                try? pngData.write(to: url)
            }
        }
    }
}

struct PinContentView: View {
    let image: NSImage
    let onClose: () -> Void
    let onCopy: () -> Void
    let onSave: () -> Void

    @State private var isHovering: Bool = false
    @State private var opacity: Double = 1.0

    var body: some View {
        ZStack(alignment: .topTrailing) {
            // Image Content
            Image(nsImage: image)
                .resizable()
                .scaledToFit()
                .opacity(opacity)
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .overlay(
                    RoundedRectangle(cornerRadius: 10)
                        .stroke(isHovering ? Color.blue.opacity(0.6) : Color.white.opacity(0.2), lineWidth: 1.5)
                )

            // Hover Action Overlay
            if isHovering {
                HStack(spacing: 6) {
                    // Quick Action Buttons
                    Button(action: onCopy) {
                        Image(systemName: "doc.on.doc.fill")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.white)
                            .padding(5)
                            .background(Color.black.opacity(0.6))
                            .clipShape(Circle())
                    }
                    .buttonStyle(.plain)
                    .help("Copy Image (⌘C)")

                    Button(action: onSave) {
                        Image(systemName: "arrow.down.circle.fill")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.white)
                            .padding(5)
                            .background(Color.black.opacity(0.6))
                            .clipShape(Circle())
                    }
                    .buttonStyle(.plain)
                    .help("Save Image (⌘S)")

                    // Close Button (✕)
                    Button(action: onClose) {
                        Image(systemName: "xmark")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.white)
                            .padding(5)
                            .background(Color.red.opacity(0.85))
                            .clipShape(Circle())
                    }
                    .buttonStyle(.plain)
                    .help("Close Pin (Esc)")
                }
                .padding(8)
                .transition(.opacity.combined(with: .scale(scale: 0.95)))
            }
        }
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.18)) {
                self.isHovering = hovering
            }
        }
        .contextMenu {
            Button("Copy Image (⌘C)", action: onCopy)
            Button("Save Image As... (⌘S)", action: onSave)
            Divider()
            Menu("Opacity") {
                Button("100%") { opacity = 1.0 }
                Button("75%") { opacity = 0.75 }
                Button("50%") { opacity = 0.50 }
                Button("25%") { opacity = 0.25 }
            }
            Divider()
            Button("Close Pin", action: onClose)
        }
    }
}
