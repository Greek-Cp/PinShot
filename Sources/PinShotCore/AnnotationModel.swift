import Foundation
import CoreGraphics

/// Types of available annotation tools.
public enum AnnotationToolType: String, CaseIterable, Sendable {
    case select = "Select"
    case rectangle = "Rectangle"
    case ellipse = "Ellipse"
    case arrow = "Arrow"
    case line = "Line"
    case pen = "Pen"
    case highlighter = "Highlighter"
    case text = "Text"
    case step = "Step"
    case mosaic = "Mosaic"
}

/// An individual annotation element on the canvas.
public struct AnnotationItem: Identifiable, Equatable, Sendable {
    public let id: UUID
    public var toolType: AnnotationToolType
    public var startPoint: CGPoint
    public var endPoint: CGPoint
    public var points: [CGPoint] // For freehand pen / highlighter
    public var text: String
    public var stepNumber: Int
    public var strokeColor: ColorRepresentation
    public var fillColor: ColorRepresentation?
    public var strokeWidth: CGFloat
    public var fontSize: CGFloat
    public var isHighlighted: Bool

    public init(
        id: UUID = UUID(),
        toolType: AnnotationToolType,
        startPoint: CGPoint = .zero,
        endPoint: CGPoint = .zero,
        points: [CGPoint] = [],
        text: String = "",
        stepNumber: Int = 1,
        strokeColor: ColorRepresentation = .red,
        fillColor: ColorRepresentation? = nil,
        strokeWidth: CGFloat = 3.0,
        fontSize: CGFloat = 16.0,
        isHighlighted: Bool = false
    ) {
        self.id = id
        self.toolType = toolType
        self.startPoint = startPoint
        self.endPoint = endPoint
        self.points = points.isEmpty ? [startPoint, endPoint] : points
        self.text = text
        self.stepNumber = stepNumber
        self.strokeColor = strokeColor
        self.fillColor = fillColor
        self.strokeWidth = strokeWidth
        self.fontSize = fontSize
        self.isHighlighted = isHighlighted
    }

    /// Returns the bounding box of this annotation element.
    public var bounds: CGRect {
        switch toolType {
        case .select:
            return .zero
        case .rectangle, .ellipse, .mosaic:
            let minX = min(startPoint.x, endPoint.x)
            let minY = min(startPoint.y, endPoint.y)
            let w = abs(endPoint.x - startPoint.x)
            let h = abs(endPoint.y - startPoint.y)
            return CGRect(x: minX, y: minY, width: w, height: h).insetBy(dx: -strokeWidth, dy: -strokeWidth)
        case .arrow, .line:
            let minX = min(startPoint.x, endPoint.x)
            let minY = min(startPoint.y, endPoint.y)
            let w = abs(endPoint.x - startPoint.x)
            let h = abs(endPoint.y - startPoint.y)
            return CGRect(x: minX, y: minY, width: w, height: h).insetBy(dx: -strokeWidth * 3, dy: -strokeWidth * 3)
        case .pen, .highlighter:
            guard let first = points.first else { return .zero }
            var minX = first.x
            var maxX = first.x
            var minY = first.y
            var maxY = first.y
            for pt in points {
                minX = min(minX, pt.x)
                maxX = max(maxX, pt.x)
                minY = min(minY, pt.y)
                maxY = max(maxY, pt.y)
            }
            return CGRect(x: minX, y: minY, width: maxX - minX, height: maxY - minY)
                .insetBy(dx: -strokeWidth, dy: -strokeWidth)
        case .step:
            let radius = max(strokeWidth * 4, 16.0)
            return CGRect(x: startPoint.x - radius, y: startPoint.y - radius, width: radius * 2, height: radius * 2)
        case .text:
            let estimatedWidth = max(CGFloat(text.count) * (fontSize * 0.65), 60)
            let estimatedHeight = fontSize * 1.5
            return CGRect(x: startPoint.x, y: startPoint.y, width: estimatedWidth, height: estimatedHeight)
        }
    }
}

/// The document containing all annotations for a screenshot session.
public struct AnnotationDocument: Equatable, Sendable {
    public var items: [AnnotationItem]
    public var nextStepIndex: Int

    public init(items: [AnnotationItem] = [], nextStepIndex: Int = 1) {
        self.items = items
        self.nextStepIndex = nextStepIndex
    }

    /// Adds a new annotation item and automatically increments step index if it's a step badge.
    public mutating func addItem(_ item: AnnotationItem) {
        var newItem = item
        if item.toolType == .step {
            newItem.stepNumber = nextStepIndex
            nextStepIndex += 1
        }
        items.append(newItem)
    }

    /// Removes item by ID and recalculates step badges if necessary.
    public mutating func removeItem(id: UUID) {
        items.removeAll(where: { $0.id == id })
        reindexSteps()
    }

    /// Re-indexes all step badges sequentially (1, 2, 3...).
    public mutating func reindexSteps() {
        var currentStep = 1
        for i in 0..<items.count {
            if items[i].toolType == .step {
                items[i].stepNumber = currentStep
                currentStep += 1
            }
        }
        nextStepIndex = currentStep
    }

    public var isEmpty: Bool {
        items.isEmpty
    }
}
