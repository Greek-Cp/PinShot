import Foundation
import CoreGraphics

/// Application configuration and preferences.
public struct PinShotSettings: Equatable, Sendable {
    public var globalShortcut: String
    public var saveDirectoryPath: String
    public var imageFormat: ImageOutputFormat
    public var toolbarOrientation: ToolbarOrientation
    public var playSoundEffects: Bool
    public var showMagnifierLoupe: Bool
    public var autoCopyToClipboardOnCapture: Bool

    public init(
        globalShortcut: String = "Command + Shift + A",
        saveDirectoryPath: String = (FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first?.appendingPathComponent("PinShots").path) ?? "~/Pictures/PinShots",
        imageFormat: ImageOutputFormat = .png,
        toolbarOrientation: ToolbarOrientation = .horizontal,
        playSoundEffects: Bool = true,
        showMagnifierLoupe: Bool = true,
        autoCopyToClipboardOnCapture: Bool = false
    ) {
        self.globalShortcut = globalShortcut
        self.saveDirectoryPath = saveDirectoryPath
        self.imageFormat = imageFormat
        self.toolbarOrientation = toolbarOrientation
        self.playSoundEffects = playSoundEffects
        self.showMagnifierLoupe = showMagnifierLoupe
        self.autoCopyToClipboardOnCapture = autoCopyToClipboardOnCapture
    }
}

public enum ImageOutputFormat: String, CaseIterable, Sendable {
    case png = "PNG"
    case jpeg = "JPEG"
    case tiff = "TIFF"
}

public enum ToolbarOrientation: String, CaseIterable, Sendable {
    case horizontal = "Horizontal"
    case vertical = "Vertical"
}
