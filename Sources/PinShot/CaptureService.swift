import Cocoa
import CoreGraphics
import PinShotCore

public struct CapturedScreenFrame: Sendable {
    public let screenFrame: CGRect
    public let image: CGImage
    public let scaleFactor: CGFloat
}

@MainActor
final class CaptureService {
    static let shared = CaptureService()

    /// Captures freeze frames of all connected screens simultaneously.
    func captureAllScreens() -> [CapturedScreenFrame] {
        var frames: [CapturedScreenFrame] = []

        for screen in NSScreen.screens {
            guard let screenNumber = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? CGDirectDisplayID else {
                continue
            }

            if let cgImage = CGDisplayCreateImage(screenNumber) {
                let scale = screen.backingScaleFactor
                frames.append(
                    CapturedScreenFrame(
                        screenFrame: screen.frame,
                        image: cgImage,
                        scaleFactor: scale
                    )
                )
            }
        }

        return frames
    }

    /// Crops a combined region from captured screen frames spanning across displays.
    func cropSelection(rect: CGRect, from capturedFrames: [CapturedScreenFrame]) -> NSImage? {
        guard rect.width > 1, rect.height > 1, !capturedFrames.isEmpty else { return nil }

        let resultImage = NSImage(size: rect.size)
        resultImage.lockFocus()

        guard NSGraphicsContext.current?.cgContext != nil else {
            resultImage.unlockFocus()
            return nil
        }

        for frame in capturedFrames {
            // Check intersection between global selection rect and this screen
            let intersection = rect.intersection(frame.screenFrame)
            if !intersection.isNull && intersection.width > 0 && intersection.height > 0 {
                // Calculate local screen coordinates (AppKit Y is bottom-up)
                let localX = (intersection.origin.x - frame.screenFrame.origin.x) * frame.scaleFactor
                let localY = (frame.screenFrame.height - (intersection.origin.y - frame.screenFrame.origin.y) - intersection.height) * frame.scaleFactor
                let localW = intersection.width * frame.scaleFactor
                let localH = intersection.height * frame.scaleFactor

                let cropRect = CGRect(x: localX, y: localY, width: localW, height: localH)

                if let croppedCG = frame.image.cropping(to: cropRect) {
                    let drawOriginX = intersection.origin.x - rect.origin.x
                    let drawOriginY = intersection.origin.y - rect.origin.y
                    let drawRect = CGRect(x: drawOriginX, y: drawOriginY, width: intersection.width, height: intersection.height)

                    let subImage = NSImage(cgImage: croppedCG, size: intersection.size)
                    subImage.draw(in: drawRect)
                }
            }
        }

        resultImage.unlockFocus()
        return resultImage
    }
}
