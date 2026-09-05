import Foundation
import CoreGraphics

/// Application configuration and preferences model.
public struct PinShotSettings: Equatable, Sendable, Codable {
    public var globalShortcutName: String
    public var globalKeyCode: UInt32
    public var globalModifiers: UInt32
    public var pinShortcutName: String
    public var copyShortcutName: String
    public var saveShortcutName: String
    public var saveDirectoryPath: String
    public var imageFormat: ImageOutputFormat
    public var toolbarOrientation: ToolbarOrientation
    public var playSoundEffects: Bool
    public var showMagnifierLoupe: Bool
    public var autoCopyToClipboardOnCapture: Bool

    public init(
        globalShortcutName: String = "⌘⇧A (Command + Shift + A)",
        globalKeyCode: UInt32 = 0, // 'A' key
        globalModifiers: UInt32 = 0x0100 | 0x0200, // cmdKey | shiftKey
        pinShortcutName: String = "⌘P / P",
        copyShortcutName: String = "⌘C / C",
        saveShortcutName: String = "⌘S / S",
        saveDirectoryPath: String = (FileManager.default.urls(for: .picturesDirectory, in: .userDomainMask).first?.appendingPathComponent("PinShots").path) ?? "~/Pictures/PinShots",
        imageFormat: ImageOutputFormat = .png,
        toolbarOrientation: ToolbarOrientation = .horizontal,
        playSoundEffects: Bool = true,
        showMagnifierLoupe: Bool = true,
        autoCopyToClipboardOnCapture: Bool = false
    ) {
        self.globalShortcutName = globalShortcutName
        self.globalKeyCode = globalKeyCode
        self.globalModifiers = globalModifiers
        self.pinShortcutName = pinShortcutName
        self.copyShortcutName = copyShortcutName
        self.saveShortcutName = saveShortcutName
        self.saveDirectoryPath = saveDirectoryPath
        self.imageFormat = imageFormat
        self.toolbarOrientation = toolbarOrientation
        self.playSoundEffects = playSoundEffects
        self.showMagnifierLoupe = showMagnifierLoupe
        self.autoCopyToClipboardOnCapture = autoCopyToClipboardOnCapture
    }
}

public enum ImageOutputFormat: String, CaseIterable, Sendable, Codable {
    case png = "PNG"
    case jpeg = "JPEG"
    case tiff = "TIFF"
}

public enum ToolbarOrientation: String, CaseIterable, Sendable, Codable {
    case horizontal = "Horizontal"
    case vertical = "Vertical"
}
