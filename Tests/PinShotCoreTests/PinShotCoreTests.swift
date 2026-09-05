import Testing
import CoreGraphics
import Foundation
#if canImport(AppKit)
import AppKit
#endif
@testable import PinShotCore

@Suite("Selection & Geometry Tests")
struct GeometryTests {
    @Test("SelectionRect normalization and bounds")
    func testSelectionRectNormalization() {
        let rect = SelectionRect.fromDrag(
            start: CGPoint(x: 300, y: 400),
            current: CGPoint(x: 100, y: 150)
        )
        #expect(rect.minX == 100)
        #expect(rect.minY == 150)
        #expect(rect.maxX == 300)
        #expect(rect.maxY == 400)
        #expect(rect.width == 200)
        #expect(rect.height == 250)
        #expect(!rect.isEmpty)

        let emptyRect = SelectionRect(x: 10, y: 10, width: 0, height: 0)
        #expect(emptyRect.isEmpty)
    }

    @Test("Hit handle detection on all 8 resize anchors")
    func testHitHandleDetection() {
        let rect = SelectionRect(x: 100, y: 100, width: 200, height: 200)
        
        #expect(rect.hitHandle(at: CGPoint(x: 100, y: 100), tolerance: 10) == .topLeft)
        #expect(rect.hitHandle(at: CGPoint(x: 200, y: 100), tolerance: 10) == .top)
        #expect(rect.hitHandle(at: CGPoint(x: 300, y: 100), tolerance: 10) == .topRight)
        #expect(rect.hitHandle(at: CGPoint(x: 300, y: 200), tolerance: 10) == .right)
        #expect(rect.hitHandle(at: CGPoint(x: 300, y: 300), tolerance: 10) == .bottomRight)
        #expect(rect.hitHandle(at: CGPoint(x: 200, y: 300), tolerance: 10) == .bottom)
        #expect(rect.hitHandle(at: CGPoint(x: 100, y: 300), tolerance: 10) == .bottomLeft)
        #expect(rect.hitHandle(at: CGPoint(x: 100, y: 200), tolerance: 10) == .left)

        // Center hit should be nil
        #expect(rect.hitHandle(at: CGPoint(x: 200, y: 200), tolerance: 10) == nil)
    }

    @Test("Resizing selection rectangle with handles")
    func testResizeOperations() {
        let rect = SelectionRect(x: 100, y: 100, width: 200, height: 200)
        
        // Drag bottom-right corner outwards to (350, 350)
        let resizedBR = rect.resized(with: .bottomRight, to: CGPoint(x: 350, y: 350))
        #expect(resizedBR.width == 250)
        #expect(resizedBR.height == 250)
        #expect(resizedBR.minX == 100)
        #expect(resizedBR.minY == 100)

        // Drag top-left corner to (50, 50)
        let resizedTL = rect.resized(with: .topLeft, to: CGPoint(x: 50, y: 50))
        #expect(resizedTL.minX == 50)
        #expect(resizedTL.minY == 50)
        #expect(resizedTL.width == 250)
        #expect(resizedTL.height == 250)
    }

    @Test("Multi-Screen Intersection & Coordinate Mapping")
    func testMultiScreenIntersection() {
        let screens = [
            CGRect(x: 0, y: 0, width: 1920, height: 1080),
            CGRect(x: 1920, y: 0, width: 2560, height: 1440)
        ]

        let selectionSpanningScreens = CGRect(x: 1800, y: 200, width: 400, height: 300)
        let intersections = ScreenGeometryMapper.intersections(selection: selectionSpanningScreens, screens: screens)

        #expect(intersections.count == 2)
        // Screen 0 part: 1800..1920 (width 120)
        #expect(intersections[0].screenIndex == 0)
        #expect(intersections[0].localRect.width == 120)
        #expect(intersections[0].localRect.origin.x == 1800)

        // Screen 1 part: 1920..2200 (width 280)
        #expect(intersections[1].screenIndex == 1)
        #expect(intersections[1].localRect.width == 280)
        #expect(intersections[1].localRect.origin.x == 0)

        // Union bounding box
        let union = ScreenGeometryMapper.unionBoundingBox(of: screens)
        #expect(union.width == CGFloat(4480))
        #expect(union.height == CGFloat(1440))
    }

    @Test("Flip coordinates to Top-Left system")
    func testCoordinateFlip() {
        let primaryHeight: CGFloat = 1080
        let appKitPoint = CGPoint(x: 100, y: 200)
        let flippedPoint = ScreenGeometryMapper.flipToTopLeft(point: appKitPoint, primaryScreenHeight: primaryHeight)
        #expect(flippedPoint.x == CGFloat(100))
        #expect(flippedPoint.y == CGFloat(880))

        let appKitRect = CGRect(x: 50, y: 100, width: 200, height: 300)
        let flippedRect = ScreenGeometryMapper.flipToTopLeft(rect: appKitRect, primaryScreenHeight: primaryHeight)
        #expect(flippedRect.origin.x == CGFloat(50))
        #expect(flippedRect.origin.y == CGFloat(680))
        #expect(flippedRect.width == CGFloat(200))
        #expect(flippedRect.height == CGFloat(300))
    }
}

@Suite("Color Representation & Conversion Tests")
struct ColorTests {
    @Test("Hex Parsing and Formatting")
    func testHexColor() {
        let color = ColorRepresentation.fromHex("#FF5733")
        #expect(color != nil)
        #expect(color?.hexString == "#FF5733")

        let white = ColorRepresentation.fromHex("FFFFFF")
        #expect(white?.hexString == "#FFFFFF")
        #expect(white?.rgbString == "rgb(255, 255, 255)")
        #expect(white?.isDark == false)

        let black = ColorRepresentation.black
        #expect(black.isDark == true)
        #expect(black.hexString == "#000000")

        let invalidHex = ColorRepresentation.fromHex("invalid")
        #expect(invalidHex == nil)
    }

    @Test("Color palette defaults")
    func testColorPaletteDefaults() {
        #expect(ColorRepresentation.defaultPresets.count == 9)
        #expect(ColorRepresentation.red.hexString == "#FF3B30")
        #expect(ColorRepresentation.green.hexString == "#34C759")
        #expect(ColorRepresentation.blue.hexString == "#007AFF")
    }
}

@Suite("History Stack (Undo/Redo) Tests")
struct HistoryStackTests {
    @Test("Undo and Redo mutations with branch clear")
    func testHistoryOperations() {
        var history = HistoryStack<Int>(initialState: 0)
        #expect(!history.canUndo)
        #expect(!history.canRedo)

        history.push(1)
        history.push(2)
        #expect(history.currentState == 2)
        #expect(history.canUndo)

        // Undo
        let state1 = history.undo()
        #expect(state1 == 1)
        #expect(history.currentState == 1)
        #expect(history.canRedo)

        // Redo
        let state2 = history.redo()
        #expect(state2 == 2)
        #expect(history.currentState == 2)

        // Undo then push new state -> Redo branch must clear
        history.undo()
        history.push(99)
        #expect(history.currentState == 99)
        #expect(!history.canRedo)
    }

    @Test("Capacity limits in HistoryStack")
    func testHistoryCapacity() {
        var history = HistoryStack<Int>(initialState: 0, maxCapacity: 3)
        history.push(1)
        history.push(2)
        history.push(3)
        // With capacity 3, undo stack contains [1, 2, 3]
        #expect(history.currentState == 3)
        #expect(history.undo() == 2)
        #expect(history.undo() == 1)
        #expect(history.undo() == nil)
        #expect(!history.canUndo)
    }
}

@Suite("Annotation Document & Rendering Tests")
struct AnnotationDocumentTests {
    @Test("Auto-incrementing and reindexing step badges")
    func testStepNumbering() {
        var doc = AnnotationDocument()

        let step1 = AnnotationItem(toolType: .step, startPoint: CGPoint(x: 10, y: 10))
        let step2 = AnnotationItem(toolType: .step, startPoint: CGPoint(x: 50, y: 50))
        let step3 = AnnotationItem(toolType: .step, startPoint: CGPoint(x: 100, y: 100))

        doc.addItem(step1)
        doc.addItem(step2)
        doc.addItem(step3)

        #expect(doc.items.count == 3)
        #expect(doc.items[0].stepNumber == 1)
        #expect(doc.items[1].stepNumber == 2)
        #expect(doc.items[2].stepNumber == 3)

        // Remove the middle step and verify reindexing
        doc.removeItem(id: doc.items[1].id)
        #expect(doc.items.count == 2)
        #expect(doc.items[0].stepNumber == 1)
        #expect(doc.items[1].stepNumber == 2)
    }

    @Test("Annotation bounds calculation for different tools")
    func testAnnotationBounds() {
        let rectItem = AnnotationItem(
            toolType: .rectangle,
            startPoint: CGPoint(x: 10, y: 20),
            endPoint: CGPoint(x: 110, y: 120),
            strokeWidth: 2
        )
        #expect(rectItem.bounds.width >= 100)
        #expect(rectItem.bounds.height >= 100)

        let penItem = AnnotationItem(
            toolType: .pen,
            points: [
                CGPoint(x: 10, y: 10),
                CGPoint(x: 20, y: 30),
                CGPoint(x: 50, y: 40)
            ],
            strokeWidth: 2
        )
        #expect(penItem.bounds.width >= 40)
        #expect(penItem.bounds.height >= 30)
    }

    #if canImport(AppKit)
    @Test("Compositing annotations on base NSImage")
    func testAnnotationCompositing() {
        let size = CGSize(width: 200, height: 200)
        let baseImage = NSImage(size: size)
        baseImage.lockFocus()
        NSColor.white.setFill()
        CGRect(origin: .zero, size: size).fill()
        baseImage.unlockFocus()

        var doc = AnnotationDocument()
        doc.addItem(
            AnnotationItem(
                toolType: .rectangle,
                startPoint: CGPoint(x: 20, y: 20),
                endPoint: CGPoint(x: 180, y: 180),
                strokeColor: .red,
                strokeWidth: 4
            )
        )
        doc.addItem(
            AnnotationItem(
                toolType: .step,
                startPoint: CGPoint(x: 50, y: 50),
                strokeColor: .blue
            )
        )

        let composited = AnnotationRenderer.render(document: doc, onto: baseImage)
        #expect(composited.size == size)
    }
    #endif
}

@Suite("Settings Tests")
struct SettingsTests {
    @Test("Default settings values")
    func testDefaultSettings() {
        let settings = PinShotSettings()
        #expect(settings.globalShortcut == "Command + Shift + A")
        #expect(settings.imageFormat == .png)
        #expect(settings.toolbarOrientation == .horizontal)
        #expect(settings.playSoundEffects == true)
        #expect(settings.showMagnifierLoupe == true)
    }
}
