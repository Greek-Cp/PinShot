import Cocoa
import SwiftUI
import PinShotCore

@MainActor
final class PinWindowManager: ObservableObject {
    static let shared = PinWindowManager()

    @Published private(set) var activePins: [PinWindow] = []
    private var pinKeyMonitor: Any?

    private init() {
        setupKeyMonitor()
    }

    private func setupKeyMonitor() {
        pinKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self = self, !self.activePins.isEmpty else { return event }
            
            // Check if key event is intended for a focused PinWindow
            if let keyWindow = NSApp.keyWindow as? PinWindow {
                let isCmd = event.modifierFlags.contains(.command)
                // ⌘W (key 13) or Esc (key 53)
                if (isCmd && event.keyCode == 13) || event.keyCode == 53 {
                    self.closePin(keyWindow)
                    return nil
                }
                // ⌘C (key 8)
                if isCmd && event.keyCode == 8 {
                    PasteboardService.shared.copyImage(keyWindow.baseImage)
                    SoundService.shared.playCaptureSound()
                    return nil
                }
            }
            return event
        }
    }

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

final class PinHostingView<Content: View>: NSHostingView<Content> {
    weak var pinWindow: PinWindow?
    private var initialFrameOnPinch: CGRect = .zero

    @MainActor required init(rootView: Content) {
        super.init(rootView: rootView)
        self.wantsLayer = true
        self.layer?.masksToBounds = false
        self.allowedTouchTypes = [.direct, .indirect]

        // Native AppKit 2-finger pinch gesture recognizer
        let pinchRecognizer = NSMagnificationGestureRecognizer(target: self, action: #selector(handlePinchGesture(_:)))
        self.addGestureRecognizer(pinchRecognizer)
    }

    @MainActor required dynamic init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func handlePinchGesture(_ recognizer: NSMagnificationGestureRecognizer) {
        guard let win = pinWindow else { return }
        switch recognizer.state {
        case .began:
            initialFrameOnPinch = win.frame
        case .changed:
            let scale = max(0.15, 1.0 + recognizer.magnification)
            let baseW = initialFrameOnPinch.width * scale
            let newWidth = max(min(baseW, 3500), win.minSize.width)
            let newHeight = newWidth * (initialFrameOnPinch.height / initialFrameOnPinch.width)

            let deltaW = newWidth - initialFrameOnPinch.width
            let deltaH = newHeight - initialFrameOnPinch.height
            let newX = initialFrameOnPinch.origin.x - (deltaW / 2)
            let newY = initialFrameOnPinch.origin.y - (deltaH / 2)

            win.setFrame(CGRect(x: newX, y: newY, width: newWidth, height: newHeight), display: true, animate: false)
        default:
            break
        }
    }

    override func magnify(with event: NSEvent) {
        pinWindow?.handleMagnifyEvent(event)
    }

    override func smartMagnify(with event: NSEvent) {
        pinWindow?.handleSmartMagnify(event)
    }
}

final class PinWindow: NSPanel {
    let pinID = UUID()
    let baseImage: NSImage
    public static let glowPadding: CGFloat = 16

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
    override var acceptsFirstResponder: Bool { true }

    init(image: NSImage, initialOrigin: CGPoint? = nil) {
        self.baseImage = image
        let imageSize = image.size
        let maxInitialWidth: CGFloat = 600
        let scale = (imageSize.width > 0 && imageSize.width > maxInitialWidth) ? maxInitialWidth / imageSize.width : 1.0
        let baseW = max(imageSize.width * scale, 60)
        let baseH = max(imageSize.height * scale, 40)
        let totalW = baseW + (Self.glowPadding * 2)
        let totalH = baseH + (Self.glowPadding * 2)
        let windowSize = CGSize(width: totalW, height: totalH)

        let initialRect: CGRect
        if let origin = initialOrigin {
            // Inset so the screenshot content aligns with original captured position
            let adjX = origin.x - Self.glowPadding
            let adjY = origin.y - Self.glowPadding
            initialRect = CGRect(origin: CGPoint(x: adjX, y: adjY), size: windowSize)
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

        // Ensure window never gets hidden when user switches to other apps
        self.isReleasedWhenClosed = false
        self.hidesOnDeactivate = false
        self.isFloatingPanel = true
        self.level = .floating
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .stationary, .ignoresCycle]
        self.isOpaque = false
        self.backgroundColor = .clear
        self.hasShadow = false // Shadow/Aura handled natively by PinAuraGlowView
        self.isMovableByWindowBackground = true
        if totalW > 0 && totalH > 0 {
            self.aspectRatio = windowSize
            self.minSize = CGSize(width: 120, height: max(120 * (totalH / totalW), 70))
        }

        setupContentView()
    }

    private func setupContentView() {
        let contentView = PinContentView(
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
        let hostingView = PinHostingView(rootView: contentView)
        hostingView.pinWindow = self
        self.contentView = hostingView
    }

    func show() {
        self.orderFrontRegardless()
        self.makeKeyAndOrderFront(nil)
    }

    // MARK: - 2-Finger Pinch-to-Resize Event Fallback
    func handleMagnifyEvent(_ event: NSEvent) {
        let factor = 1.0 + event.magnification
        var currentFrame = self.frame
        let oldSize = currentFrame.size
        
        let newWidth = max(min(oldSize.width * factor, 3500), self.minSize.width)
        let newHeight = newWidth * (oldSize.height / oldSize.width)
        
        let deltaW = newWidth - oldSize.width
        let deltaH = newHeight - oldSize.height
        currentFrame.origin.x -= deltaW / 2
        currentFrame.origin.y -= deltaH / 2
        currentFrame.size = CGSize(width: newWidth, height: newHeight)
        
        self.setFrame(currentFrame, display: true, animate: false)
    }

    func handleSmartMagnify(_ event: NSEvent) {
        let imageSize = baseImage.size
        let baseW = imageSize.width + (Self.glowPadding * 2)
        let baseH = imageSize.height + (Self.glowPadding * 2)
        
        var currentFrame = self.frame
        let deltaW = baseW - currentFrame.width
        let deltaH = baseH - currentFrame.height
        currentFrame.origin.x -= deltaW / 2
        currentFrame.origin.y -= deltaH / 2
        currentFrame.size = CGSize(width: baseW, height: baseH)
        
        self.setFrame(currentFrame, display: true, animate: true)
    }

    override func mouseDown(with event: NSEvent) {
        self.orderFrontRegardless()
        super.mouseDown(with: event)
    }

    override func keyDown(with event: NSEvent) {
        let isCmd = event.modifierFlags.contains(.command)
        if (isCmd && event.keyCode == 13) || event.keyCode == 53 { // ⌘W or Esc
            PinWindowManager.shared.closePin(self)
            return
        }
        if isCmd && event.keyCode == 8 { // ⌘C
            PasteboardService.shared.copyImage(self.baseImage)
            SoundService.shared.playCaptureSound()
            return
        }
        if isCmd && event.keyCode == 1 { // ⌘S
            self.saveImage()
            return
        }
        super.keyDown(with: event)
    }

    override func performKeyEquivalent(with event: NSEvent) -> Bool {
        let isCmd = event.modifierFlags.contains(.command)
        if isCmd && event.keyCode == 13 { // ⌘W
            PinWindowManager.shared.closePin(self)
            return true
        }
        if isCmd && event.keyCode == 8 { // ⌘C
            PasteboardService.shared.copyImage(self.baseImage)
            SoundService.shared.playCaptureSound()
            return true
        }
        if isCmd && event.keyCode == 1 { // ⌘S
            self.saveImage()
            return true
        }
        return super.performKeyEquivalent(with: event)
    }

    func saveImage() {
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

    @ObservedObject private var settingsManager = SettingsManager.shared
    @State private var isHovering: Bool = false
    @State private var opacity: Double = 1.0

    var body: some View {
        ZStack(alignment: .topTrailing) {
            // Image Content with matched Background Glow Aura
            Image(nsImage: image)
                .resizable()
                .scaledToFit()
                .opacity(opacity)
                .clipShape(RoundedRectangle(cornerRadius: 10))
                .background(
                    PinAuraGlowView(
                        cornerRadius: 10,
                        style: settingsManager.settings.pinShadowStyle,
                        isHovering: isHovering
                    )
                )
                // Double tap / double click to close pin automatically
                .onTapGesture(count: 2) {
                    onClose()
                }

            // Hover Action Overlay
            if isHovering {
                HStack(spacing: 6) {
                    // Quick Action Buttons
                    Button(action: onCopy) {
                        Image(systemName: "doc.on.doc.fill")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.white)
                            .padding(5)
                            .background(Color.black.opacity(0.65))
                            .clipShape(Circle())
                    }
                    .buttonStyle(.plain)
                    .help("Copy Image (⌘C)")

                    Button(action: onSave) {
                        Image(systemName: "arrow.down.circle.fill")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.white)
                            .padding(5)
                            .background(Color.black.opacity(0.65))
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
                    .help("Close Pin (⌘W / Esc / Double Click)")
                }
                .padding(8)
                .transition(.opacity.combined(with: .scale(scale: 0.95)))
            }
        }
        .padding(PinWindow.glowPadding) // Generous margin ensuring glow fades to transparent without clipping
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.18)) {
                self.isHovering = hovering
            }
        }
        .contextMenu {
            Button("Copy Image (⌘C)", action: onCopy)
            Button("Save Image As... (⌘S)", action: onSave)
            Divider()
            Menu("Shadow / Aura Effect") {
                ForEach(PinShadowStyle.allCases, id: \.self) { style in
                    Button(style.rawValue) {
                        settingsManager.settings.pinShadowStyle = style
                    }
                }
            }
            Menu("Opacity") {
                Button("100%") { opacity = 1.0 }
                Button("75%") { opacity = 0.75 }
                Button("50%") { opacity = 0.50 }
                Button("25%") { opacity = 0.25 }
            }
            Divider()
            Button("Close Pin (⌘W / Esc)", action: onClose)
        }
    }
}
