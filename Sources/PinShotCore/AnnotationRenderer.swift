import Foundation
import CoreGraphics
import CoreImage

#if canImport(AppKit)
import AppKit

public struct AnnotationRenderer: Sendable {
    /// Renders an AnnotationDocument on top of a base NSImage and returns the composited NSImage safely.
    public static func render(
        document: AnnotationDocument,
        onto baseImage: NSImage,
        scale: CGFloat = 1.0
    ) -> NSImage {
        let size = baseImage.size
        guard size.width > 0, size.height > 0 else { return baseImage }
        guard !document.isEmpty else { return baseImage }

        let composited = NSImage(size: size, flipped: false) { dstRect in
            baseImage.draw(in: dstRect)

            if let context = NSGraphicsContext.current?.cgContext {
                for item in document.items {
                    drawItem(item, in: context, imageSize: size)
                }
            }
            return true
        }

        return composited
    }

    private static func drawItem(_ item: AnnotationItem, in context: CGContext, imageSize: CGSize) {
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
            if let fill = item.fillColor {
                context.setFillColor(CGColor(red: fill.red, green: fill.green, blue: fill.blue, alpha: fill.alpha))
                context.fill(rect)
            }
            context.stroke(rect)

        case .ellipse:
            let rect = CGRect(
                x: min(item.startPoint.x, item.endPoint.x),
                y: min(item.startPoint.y, item.endPoint.y),
                width: abs(item.endPoint.x - item.startPoint.x),
                height: abs(item.endPoint.y - item.startPoint.y)
            )
            if let fill = item.fillColor {
                context.setFillColor(CGColor(red: fill.red, green: fill.green, blue: fill.blue, alpha: fill.alpha))
                context.fillEllipse(in: rect)
            }
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

        case .text:
            let attributes: [NSAttributedString.Key: Any] = [
                .font: NSFont.systemFont(ofSize: item.fontSize, weight: .semibold),
                .foregroundColor: NSColor(
                    red: item.strokeColor.red,
                    green: item.strokeColor.green,
                    blue: item.strokeColor.blue,
                    alpha: item.strokeColor.alpha
                )
            ]
            let attributedString = NSAttributedString(string: item.text, attributes: attributes)
            attributedString.draw(at: item.startPoint)

        case .step:
            let radius = max(item.strokeWidth * 3.5, 14.0)
            let center = item.startPoint
            let circleRect = CGRect(x: center.x - radius, y: center.y - radius, width: radius * 2, height: radius * 2)

            // Fill background circle
            context.setFillColor(strokeCGColor)
            context.fillEllipse(in: circleRect)

            // Draw white step number
            let numStr = "\(item.stepNumber)"
            let textAttributes: [NSAttributedString.Key: Any] = [
                .font: NSFont.boldSystemFont(ofSize: radius * 1.1),
                .foregroundColor: NSColor.white
            ]
            let numAttr = NSAttributedString(string: numStr, attributes: textAttributes)
            let textSize = numAttr.size()
            let textOrigin = CGPoint(
                x: center.x - (textSize.width / 2),
                y: center.y - (textSize.height / 2)
            )
            numAttr.draw(at: textOrigin)

        case .mosaic:
            let rect = CGRect(
                x: min(item.startPoint.x, item.endPoint.x),
                y: min(item.startPoint.y, item.endPoint.y),
                width: abs(item.endPoint.x - item.startPoint.x),
                height: abs(item.endPoint.y - item.startPoint.y)
            )
            context.setLineDash(phase: 0, lengths: [4, 4])
            context.stroke(rect)
        }

        context.restoreGState()
    }

    private static func drawArrow(from start: CGPoint, to end: CGPoint, strokeWidth: CGFloat, in context: CGContext) {
        context.move(to: start)
        context.addLine(to: end)
        context.strokePath()

        let angle = atan2(end.y - start.y, end.x - start.x)
        let arrowLength = max(strokeWidth * 4.5, 14.0)
        let arrowAngle: CGFloat = .pi / 6

        let p1 = CGPoint(
            x: end.x - arrowLength * cos(angle - arrowAngle),
            y: end.y - arrowLength * sin(angle - arrowAngle)
        )
        let p2 = CGPoint(
            x: end.x - arrowLength * cos(angle + arrowAngle),
            y: end.y - arrowLength * sin(angle + arrowAngle)
        )

        context.move(to: end)
        context.addLine(to: p1)
        context.addLine(to: p2)
        context.closePath()
        context.fillPath()
    }
}
#endif
