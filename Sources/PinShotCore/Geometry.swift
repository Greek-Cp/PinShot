import Foundation
import CoreGraphics

/// Model representing a normalized rectangle with helper utilities for screen space calculations.
public struct SelectionRect: Equatable, Sendable {
    public var origin: CGPoint
    public var size: CGSize

    public init(origin: CGPoint, size: CGSize) {
        self.origin = origin
        self.size = size
    }

    public init(x: CGFloat, y: CGFloat, width: CGFloat, height: CGFloat) {
        self.origin = CGPoint(x: x, y: y)
        self.size = CGSize(width: width, height: height)
    }

    public var minX: CGFloat { min(origin.x, origin.x + size.width) }
    public var maxX: CGFloat { max(origin.x, origin.x + size.width) }
    public var minY: CGFloat { min(origin.y, origin.y + size.height) }
    public var maxY: CGFloat { max(origin.y, origin.y + size.height) }

    public var width: CGFloat { abs(size.width) }
    public var height: CGFloat { abs(size.height) }

    public var standardized: CGRect {
        CGRect(x: minX, y: minY, width: width, height: height)
    }

    public var isEmpty: Bool {
        width < 1 || height < 1
    }

    /// Creates a normalized selection rect from two arbitrary drag points.
    public static func fromDrag(start: CGPoint, current: CGPoint) -> SelectionRect {
        let x = min(start.x, current.x)
        let y = min(start.y, current.y)
        let w = abs(current.x - start.x)
        let h = abs(current.y - start.y)
        return SelectionRect(x: x, y: y, width: w, height: h)
    }

    /// Calculates which handle is hit at a given point with a hit tolerance.
    public func hitHandle(at point: CGPoint, tolerance: CGFloat = 10) -> ResizeHandle? {
        let rect = standardized
        let handles: [(ResizeHandle, CGPoint)] = [
            (.topLeft, CGPoint(x: rect.minX, y: rect.minY)),
            (.top, CGPoint(x: rect.midX, y: rect.minY)),
            (.topRight, CGPoint(x: rect.maxX, y: rect.minY)),
            (.right, CGPoint(x: rect.maxX, y: rect.midY)),
            (.bottomRight, CGPoint(x: rect.maxX, y: rect.maxY)),
            (.bottom, CGPoint(x: rect.midX, y: rect.maxY)),
            (.bottomLeft, CGPoint(x: rect.minX, y: rect.maxY)),
            (.left, CGPoint(x: rect.minX, y: rect.midY))
        ]

        for (handle, handlePoint) in handles {
            let hitRect = CGRect(
                x: handlePoint.x - tolerance,
                y: handlePoint.y - tolerance,
                width: tolerance * 2,
                height: tolerance * 2
            )
            if hitRect.contains(point) {
                return handle
            }
        }
        return nil
    }

    /// Resizes the selection rectangle given active handle and new point.
    public func resized(with handle: ResizeHandle, to point: CGPoint) -> SelectionRect {
        var rect = standardized
        switch handle {
        case .topLeft:
            rect = CGRect(x: point.x, y: point.y, width: rect.maxX - point.x, height: rect.maxY - point.y)
        case .top:
            rect = CGRect(x: rect.minX, y: point.y, width: rect.width, height: rect.maxY - point.y)
        case .topRight:
            rect = CGRect(x: rect.minX, y: point.y, width: point.x - rect.minX, height: rect.maxY - point.y)
        case .right:
            rect = CGRect(x: rect.minX, y: rect.minY, width: point.x - rect.minX, height: rect.height)
        case .bottomRight:
            rect = CGRect(x: rect.minX, y: rect.minY, width: point.x - rect.minX, height: point.y - rect.minY)
        case .bottom:
            rect = CGRect(x: rect.minX, y: rect.minY, width: rect.width, height: point.y - rect.minY)
        case .bottomLeft:
            rect = CGRect(x: point.x, y: rect.minY, width: rect.maxX - point.x, height: point.y - rect.minY)
        case .left:
            rect = CGRect(x: point.x, y: rect.minY, width: rect.maxX - point.x, height: rect.height)
        }
        return SelectionRect(origin: rect.origin, size: rect.size).standardizedRect
    }

    private var standardizedRect: SelectionRect {
        let std = standardized
        return SelectionRect(x: std.origin.x, y: std.origin.y, width: std.width, height: std.height)
    }
}

/// Identifiers for 8-point resize handles.
public enum ResizeHandle: Sendable {
    case topLeft, top, topRight, right, bottomRight, bottom, bottomLeft, left
}

/// Multi-screen virtual coordinate space mapper.
public struct ScreenGeometryMapper: Sendable {
    /// Converts a point from AppKit coordinates (bottom-left origin of primary screen)
    /// to Top-Left unified coordinate system (used by CoreGraphics / ScreenCaptureKit).
    public static func flipToTopLeft(point: CGPoint, primaryScreenHeight: CGFloat) -> CGPoint {
        CGPoint(x: point.x, y: primaryScreenHeight - point.y)
    }

    /// Converts a rect from AppKit coordinates to CoreGraphics top-left coordinates.
    public static func flipToTopLeft(rect: CGRect, primaryScreenHeight: CGFloat) -> CGRect {
        CGRect(
            x: rect.origin.x,
            y: primaryScreenHeight - rect.origin.y - rect.height,
            width: rect.width,
            height: rect.height
        )
    }

    /// Calculates the union bounding box across multiple screen frames.
    public static func unionBoundingBox(of screens: [CGRect]) -> CGRect {
        guard let first = screens.first else { return .zero }
        return screens.dropFirst().reduce(first) { $0.union($1) }
    }

    /// Finds intersection rects between a selection rectangle and individual screen frames.
    public static func intersections(selection: CGRect, screens: [CGRect]) -> [(screenIndex: Int, localRect: CGRect)] {
        var results: [(Int, CGRect)] = []
        for (index, screen) in screens.enumerated() {
            let intersection = selection.intersection(screen)
            if !intersection.isNull && intersection.width > 0 && intersection.height > 0 {
                // Convert to screen local coordinates
                let localRect = CGRect(
                    x: intersection.origin.x - screen.origin.x,
                    y: intersection.origin.y - screen.origin.y,
                    width: intersection.width,
                    height: intersection.height
                )
                results.append((index, localRect))
            }
        }
        return results
    }
}
