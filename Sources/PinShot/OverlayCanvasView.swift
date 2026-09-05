import Cocoa
import SwiftUI
import PinShotCore

final class OverlayCanvasView: NSView {
    let screen: NSScreen
    unowned let manager: OverlayWindowManager

    private var isDraggingSelection: Bool = false
    private var isResizingSelection: Bool = false
    private var activeResizeHandle: ResizeHandle?
    private var dragStartPoint: CGPoint = .zero

    private var isDrawingAnnotation: Bool = false
    private var currentAnnotationItem: AnnotationItem?

    private var mouseLocation: CGPoint = .zero
    private var loupeHostingView: NSHostingView<LoupeView>?

    init(frame: CGRect, screen: NSScreen, manager: OverlayWindowManager) {
        self.screen = screen
        self.manager = manager
        super.init(frame: frame)
        self.wantsLayer = true

        let trackingArea = NSTrackingArea(
            rect: bounds,
            options: [.activeAlways, .mouseMoved, .mouseEnteredAndExited, .inVisibleRect],
            owner: self,
            userInfo: nil
        )
        addTrackingArea(trackingArea)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var acceptsFirstResponder: Bool {
        true
    }

    // MARK: - Drawing
    override func draw(_ dirtyRect: NSRect) {
        guard let context = NSGraphicsContext.current?.cgContext else { return }

        // 1. Semi-transparent backdrop
        context.setFillColor(CGColor(red: 0, green: 0, blue: 0, alpha: 0.35))
        context.fill(bounds)

        // 2. Clear out selection rectangle
        if let selection = manager.currentSelection {
            let stdGlobal = selection.standardized
            // Convert global rect to local view rect
            let localRect = CGRect(
                x: stdGlobal.origin.x - screen.frame.origin.x,
                y: stdGlobal.origin.y - screen.frame.origin.y,
                width: stdGlobal.width,
                height: stdGlobal.height
            )

            // Clear cutout
            context.clear(localRect)

            // Selection border
            context.setStrokeColor(CGColor(red: 0.2, green: 0.5, blue: 1.0, alpha: 1.0))
            context.setLineWidth(1.5)
            context.stroke(localRect)

            // Draw 8 resize handles
            drawHandles(in: localRect, context: context)

            // Draw annotations within global space
            drawAnnotations(context: context)
        }
    }

    private func drawHandles(in rect: CGRect, context: CGContext) {
        let handleSize: CGFloat = 8.0
        let handles: [CGPoint] = [
            CGPoint(x: rect.minX, y: rect.minY),
            CGPoint(x: rect.midX, y: rect.minY),
            CGPoint(x: rect.maxX, y: rect.minY),
            CGPoint(x: rect.maxX, y: rect.midY),
            CGPoint(x: rect.maxX, y: rect.maxY),
            CGPoint(x: rect.midX, y: rect.maxY),
            CGPoint(x: rect.minX, y: rect.maxY),
            CGPoint(x: rect.minX, y: rect.midY)
        ]

        context.setFillColor(CGColor(red: 1, green: 1, blue: 1, alpha: 1))
        context.setStrokeColor(CGColor(red: 0.2, green: 0.5, blue: 1.0, alpha: 1))
        context.setLineWidth(1.5)

        for pt in handles {
            let handleRect = CGRect(
                x: pt.x - (handleSize / 2),
                y: pt.y - (handleSize / 2),
                width: handleSize,
                height: handleSize
            )
            context.fill(handleRect)
            context.stroke(handleRect)
        }
    }

    private func drawAnnotations(context: CGContext) {
        var allItems = manager.annotationDoc.items
        if let current = currentAnnotationItem {
            allItems.append(current)
        }

        for item in allItems {
            // Localize coordinates
            var localItem = item
            localItem.startPoint = CGPoint(
                x: item.startPoint.x - screen.frame.origin.x,
                y: item.startPoint.y - screen.frame.origin.y
            )
            localItem.endPoint = CGPoint(
                x: item.endPoint.x - screen.frame.origin.x,
                y: item.endPoint.y - screen.frame.origin.y
            )
            localItem.points = item.points.map {
                CGPoint(x: $0.x - screen.frame.origin.x, y: $0.y - screen.frame.origin.y)
            }

            drawAnnotationItem(localItem, context: context)
        }
    }

    private func drawAnnotationItem(_ item: AnnotationItem, context: CGContext) {
        context.saveGState()
        let strokeCGColor = CGColor(
            red: item.strokeColor.red,
            green: item.strokeColor.green,
            blue: item.strokeColor.blue,
            alpha: item.strokeColor.alpha
        )

        context.setStrokeColor(strokeCGColor)
        context.setLineWidth(item.strokeWidth)
        context.setLineCap(.round)
        context.setLineJoin(.round)

        switch item.toolType {
        case .select:
            break
        case .rectangle:
            let rect = CGRect(
                x: min(item.startPoint.x, item.endPoint.x),
                y: min(item.startPoint.y, item.endPoint.y),
                width: abs(item.endPoint.x - item.startPoint.x),
                height: abs(item.endPoint.y - item.startPoint.y)
            )
            context.stroke(rect)
        case .ellipse:
            let rect = CGRect(
                x: min(item.startPoint.x, item.endPoint.x),
                y: min(item.startPoint.y, item.endPoint.y),
                width: abs(item.endPoint.x - item.startPoint.x),
                height: abs(item.endPoint.y - item.startPoint.y)
            )
            context.strokeEllipse(in: rect)
        case .line:
            context.move(to: item.startPoint)
            context.addLine(to: item.endPoint)
            context.strokePath()
        case .arrow:
            drawArrow(from: item.startPoint, to: item.endPoint, strokeWidth: item.strokeWidth, in: context)
        case .pen:
            guard item.points.count > 1 else { break }
            context.move(to: item.points[0])
            for pt in item.points.dropFirst() {
                context.addLine(to: pt)
            }
            context.strokePath()
        case .highlighter:
            guard item.points.count > 1 else { break }
            context.setAlpha(0.35)
            context.setLineWidth(item.strokeWidth * 2.5)
            context.move(to: item.points[0])
            for pt in item.points.dropFirst() {
                context.addLine(to: pt)
            }
            context.strokePath()
        case .step:
            let radius = max(item.strokeWidth * 3.5, 14.0)
            let center = item.startPoint
            let circleRect = CGRect(x: center.x - radius, y: center.y - radius, width: radius * 2, height: radius * 2)
            context.setFillColor(strokeCGColor)
            context.fillEllipse(in: circleRect)

            let numStr = "\(item.stepNumber)"
            let textAttributes: [NSAttributedString.Key: Any] = [
                .font: NSFont.boldSystemFont(ofSize: radius * 1.1),
                .foregroundColor: NSColor.white
            ]
            let numAttr = NSAttributedString(string: numStr, attributes: textAttributes)
            let textSize = numAttr.size()
            let textOrigin = CGPoint(x: center.x - (textSize.width / 2), y: center.y - (textSize.height / 2))
            numAttr.draw(at: textOrigin)
        case .text, .mosaic:
            break
        }

        context.restoreGState()
    }

    private func drawArrow(from start: CGPoint, to end: CGPoint, strokeWidth: CGFloat, in context: CGContext) {
        context.move(to: start)
        context.addLine(to: end)
        context.strokePath()

        let angle = atan2(end.y - start.y, end.x - start.x)
        let arrowLength = max(strokeWidth * 4.5, 14.0)
        let arrowAngle: CGFloat = .pi / 6

        let p1 = CGPoint(x: end.x - arrowLength * cos(angle - arrowAngle), y: end.y - arrowLength * sin(angle - arrowAngle))
        let p2 = CGPoint(x: end.x - arrowLength * cos(angle + arrowAngle), y: end.y - arrowLength * sin(angle + arrowAngle))

        context.move(to: end)
        context.addLine(to: p1)
        context.addLine(to: p2)
        context.closePath()
        context.fillPath()
    }

    // MARK: - Mouse Events
    private func globalPoint(from localPoint: CGPoint) -> CGPoint {
        CGPoint(x: screen.frame.origin.x + localPoint.x, y: screen.frame.origin.y + localPoint.y)
    }

    override func mouseDown(with event: NSEvent) {
        let localPoint = convert(event.locationInWindow, from: nil)
        let globalPt = globalPoint(from: localPoint)

        if manager.isAnnotating && manager.activeTool != .select {
            // Start drawing annotation
            isDrawingAnnotation = true
            currentAnnotationItem = AnnotationItem(
                toolType: manager.activeTool,
                startPoint: globalPt,
                endPoint: globalPt,
                points: [globalPt],
                strokeColor: manager.activeColor,
                strokeWidth: manager.activeStrokeWidth
            )
            return
        }

        // Check if hitting a resize handle
        if let selection = manager.currentSelection,
           let handle = selection.hitHandle(at: globalPt) {
            isResizingSelection = true
            activeResizeHandle = handle
            return
        }

        // Start new selection drag
        isDraggingSelection = true
        dragStartPoint = globalPt
        manager.updateSelection(SelectionRect(origin: globalPt, size: .zero))
    }

    override func mouseDragged(with event: NSEvent) {
        let localPoint = convert(event.locationInWindow, from: nil)
        let globalPt = globalPoint(from: localPoint)

        if isDrawingAnnotation, var item = currentAnnotationItem {
            item.endPoint = globalPt
            item.points.append(globalPt)
            currentAnnotationItem = item
            needsDisplay = true
            return
        }

        if isResizingSelection, let handle = activeResizeHandle, let selection = manager.currentSelection {
            let resized = selection.resized(with: handle, to: globalPt)
            manager.updateSelection(resized)
            return
        }

        if isDraggingSelection {
            let newRect = SelectionRect.fromDrag(start: dragStartPoint, current: globalPt)
            manager.updateSelection(newRect)
        }
    }

    override func mouseUp(with event: NSEvent) {
        if isDrawingAnnotation, let item = currentAnnotationItem {
            manager.annotationDoc.addItem(item)
            manager.pushAnnotationHistory()
            currentAnnotationItem = nil
            isDrawingAnnotation = false
            needsDisplay = true
            return
        }

        isDraggingSelection = false
        isResizingSelection = false
        activeResizeHandle = nil
    }

    override func mouseMoved(with event: NSEvent) {
        let localPoint = convert(event.locationInWindow, from: nil)
        mouseLocation = localPoint
    }

    // MARK: - Keyboard shortcuts
    override func keyDown(with event: NSEvent) {
        switch event.keyCode {
        case 53: // Esc
            manager.closeOverlay()
        case 35: // P (Pin)
            manager.toolbarDidSelectPin()
        case 0:  // A (Annotate)
            manager.toolbarDidSelectAnnotate()
        case 8:  // C (Copy)
            manager.toolbarDidSelectCopy()
        case 1:  // S (Save)
            manager.toolbarDidSelectSave()
        case 6 where event.modifierFlags.contains(.command): // ⌘Z (Undo)
            if event.modifierFlags.contains(.shift) {
                manager.toolbarDidRedo()
            } else {
                manager.toolbarDidUndo()
            }
        default:
            super.keyDown(with: event)
        }
    }
}
