import Cocoa
import SwiftUI
import PinShotCore

final class OverlayWindowPanel: NSPanel {
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }
    override var acceptsFirstResponder: Bool { true }
}

@MainActor
final class OverlayWindowManager: NSObject, FloatingToolbarDelegate {
    static let shared = OverlayWindowManager()

    private var overlayWindows: [NSWindow] = []
    private var toolbarPanel: FloatingToolbarPanel?
    private var capturedFrames: [CapturedScreenFrame] = []
    private var localKeyMonitor: Any?
    
    var currentSelection: SelectionRect?
    var annotationDoc = AnnotationDocument()
    var history = HistoryStack<AnnotationDocument>(initialState: AnnotationDocument())
    
    var activeTool: AnnotationToolType = .select
    var activeColor: ColorRepresentation = .red
    var activeStrokeWidth: CGFloat = 3.0
    var isAnnotating: Bool = false

    func startCapture() {
        // Ensure screen recording permission
        guard PermissionService.shared.checkPermission() else {
            PermissionWindowManager.shared.show { [weak self] in
                self?.startCapture()
            }
            return
        }

        closeOverlay()

        // 1. Freeze screen frames
        capturedFrames = CaptureService.shared.captureAllScreens()
        annotationDoc = AnnotationDocument()
        history.reset(with: annotationDoc)
        currentSelection = nil
        isAnnotating = false

        // 2. Open overlay window on each display
        for screen in NSScreen.screens {
            let window = createOverlayWindow(for: screen)
            overlayWindows.append(window)
            window.makeKeyAndOrderFront(nil)
        }

        // 3. Setup event monitor for guaranteed instant keyboard shortcut handling
        setupKeyMonitor()

        NSApp.activate(ignoringOtherApps: true)
    }

    private func setupKeyMonitor() {
        removeKeyMonitor()
        localKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            guard let self = self, !self.overlayWindows.isEmpty else { return event }
            if self.handleKeyEvent(event) {
                return nil // Event consumed
            }
            return event
        }
    }

    private func removeKeyMonitor() {
        if let monitor = localKeyMonitor {
            NSEvent.removeMonitor(monitor)
            localKeyMonitor = nil
        }
    }

    func handleKeyEvent(_ event: NSEvent) -> Bool {
        let isCmd = event.modifierFlags.contains(.command)
        let isShift = event.modifierFlags.contains(.shift)

        switch event.keyCode {
        case 53: // Esc: Cancel selection and close overlay
            closeOverlay()
            return true

        case 35: // P (Pin) or ⌘P
            if let sel = currentSelection, !sel.isEmpty {
                toolbarDidSelectPin()
                return true
            }

        case 8: // C (Copy) or ⌘C
            if let sel = currentSelection, !sel.isEmpty {
                toolbarDidSelectCopy()
                return true
            }

        case 36: // Enter / Return (Copy & Finish)
            if let sel = currentSelection, !sel.isEmpty {
                toolbarDidSelectCopy()
                return true
            }

        case 1: // S (Save) or ⌘S
            if let sel = currentSelection, !sel.isEmpty {
                toolbarDidSelectSave()
                return true
            }

        case 0, 14: // A or E (Annotate)
            if !isCmd && currentSelection != nil && !currentSelection!.isEmpty {
                toolbarDidSelectAnnotate()
                return true
            }

        case 6 where isCmd: // ⌘Z (Undo) or ⌘⇧Z (Redo)
            if isShift {
                toolbarDidRedo()
            } else {
                toolbarDidUndo()
            }
            return true

        default:
            break
        }

        return false
    }

    func closeOverlay() {
        removeKeyMonitor()
        for window in overlayWindows {
            window.orderOut(nil)
        }
        overlayWindows.removeAll()
        hideToolbar()
        capturedFrames.removeAll()
        currentSelection = nil
        NSCursor.arrow.set()
    }

    private func createOverlayWindow(for screen: NSScreen) -> NSWindow {
        let panel = OverlayWindowPanel(
            contentRect: screen.frame,
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        panel.isReleasedWhenClosed = false
        panel.level = .screenSaver
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = false
        panel.ignoresMouseEvents = false

        let canvasView = OverlayCanvasView(
            frame: CGRect(origin: .zero, size: screen.frame.size),
            screen: screen,
            manager: self
        )
        panel.contentView = canvasView

        return panel
    }

    // MARK: - Synchronized Redraw across all screens
    func updateSelection(_ rect: SelectionRect?) {
        self.currentSelection = rect
        for window in overlayWindows {
            window.contentView?.needsDisplay = true
        }

        if let rect = rect, !rect.isEmpty {
            showToolbar(near: rect.standardized)
        } else {
            hideToolbar()
        }
    }

    // MARK: - Floating Toolbar
    func showToolbar(near selectionRect: CGRect) {
        let toolbarView = FloatingToolbarView(
            delegate: self,
            isAnnotating: isAnnotating,
            selectedTool: activeTool,
            selectedColor: activeColor,
            strokeWidth: activeStrokeWidth
        )
        let hostingView = NSHostingView(rootView: toolbarView)
        hostingView.layout()

        let toolbarSize = hostingView.fittingSize

        // Calculate placement: below selection or flipped above if close to bottom
        let margin: CGFloat = 10
        var x = selectionRect.maxX - toolbarSize.width
        var y = selectionRect.minY - toolbarSize.height - margin

        let screenBounds = NSScreen.main?.visibleFrame ?? CGRect(x: 0, y: 0, width: 1920, height: 1080)
        if y < screenBounds.minY {
            y = selectionRect.maxY + margin
        }
        if x < screenBounds.minX {
            x = selectionRect.minX
        }

        let toolbarRect = CGRect(x: x, y: y, width: toolbarSize.width, height: toolbarSize.height)

        if toolbarPanel == nil {
            toolbarPanel = FloatingToolbarPanel(view: hostingView)
        } else {
            toolbarPanel?.contentView = hostingView
        }

        toolbarPanel?.setFrame(toolbarRect, display: true)
        toolbarPanel?.makeKeyAndOrderFront(nil)
    }

    func hideToolbar() {
        toolbarPanel?.orderOut(nil)
        toolbarPanel = nil
    }

    // MARK: - FloatingToolbarDelegate
    func toolbarDidSelectPin() {
        guard let finalImage = createFinalRenderedImage() else { return }
        let origin = currentSelection?.standardized.origin
        closeOverlay()
        PinWindowManager.shared.createPin(image: finalImage, at: origin)
    }

    func toolbarDidSelectAnnotate() {
        isAnnotating.toggle()
        activeTool = isAnnotating ? .rectangle : .select
        if let sel = currentSelection {
            showToolbar(near: sel.standardized)
        }
    }

    func toolbarDidSelectCopy() {
        guard let finalImage = createFinalRenderedImage() else { return }
        PasteboardService.shared.copyImage(finalImage)
        SoundService.shared.playCaptureSound()
        closeOverlay()
    }

    func toolbarDidSelectSave() {
        guard let finalImage = createFinalRenderedImage() else { return }
        closeOverlay()

        let savePanel = NSSavePanel()
        savePanel.allowedContentTypes = [.png]
        savePanel.nameFieldStringValue = "PinShot_\(Int(Date().timeIntervalSince1970)).png"

        savePanel.begin { response in
            guard response == .OK, let url = savePanel.url else { return }
            if let tiff = finalImage.tiffRepresentation,
               let bitmap = NSBitmapImageRep(data: tiff),
               let pngData = bitmap.representation(using: .png, properties: [:]) {
                try? pngData.write(to: url)
            }
        }
    }

    func toolbarDidSelectClose() {
        closeOverlay()
    }

    func toolbarDidChangeTool(_ tool: AnnotationToolType) {
        activeTool = tool
    }

    func toolbarDidChangeColor(_ color: ColorRepresentation) {
        activeColor = color
    }

    func toolbarDidChangeStrokeWidth(_ width: CGFloat) {
        activeStrokeWidth = width
    }

    func toolbarDidUndo() {
        if let prevDoc = history.undo() {
            annotationDoc = prevDoc
            for window in overlayWindows {
                window.contentView?.needsDisplay = true
            }
        }
    }

    func toolbarDidRedo() {
        if let nextDoc = history.redo() {
            annotationDoc = nextDoc
            for window in overlayWindows {
                window.contentView?.needsDisplay = true
            }
        }
    }

    func pushAnnotationHistory() {
        history.push(annotationDoc)
    }

    private func createFinalRenderedImage() -> NSImage? {
        guard let selection = currentSelection?.standardized,
              let baseCropped = CaptureService.shared.cropSelection(rect: selection, from: capturedFrames) else {
            return nil
        }

        // Adjust annotation coordinates relative to selection origin
        var relativeDoc = AnnotationDocument()
        for item in annotationDoc.items {
            var relItem = item
            relItem.startPoint = CGPoint(
                x: item.startPoint.x - selection.origin.x,
                y: item.startPoint.y - selection.origin.y
            )
            relItem.endPoint = CGPoint(
                x: item.endPoint.x - selection.origin.x,
                y: item.endPoint.y - selection.origin.y
            )
            relItem.points = item.points.map {
                CGPoint(x: $0.x - selection.origin.x, y: $0.y - selection.origin.y)
            }
            relativeDoc.items.append(relItem)
        }

        return AnnotationRenderer.render(document: relativeDoc, onto: baseCropped)
    }
}
