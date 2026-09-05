import Foundation
import CoreGraphics

/// Helper for color representation, parsing, and formatting.
public struct ColorRepresentation: Equatable, Sendable {
    public var red: CGFloat
    public var green: CGFloat
    public var blue: CGFloat
    public var alpha: CGFloat

    public init(red: CGFloat, green: CGFloat, blue: CGFloat, alpha: CGFloat = 1.0) {
        self.red = min(max(red, 0), 1)
        self.green = min(max(green, 0), 1)
        self.blue = min(max(blue, 0), 1)
        self.alpha = min(max(alpha, 0), 1)
    }

    public init(r255: Int, g255: Int, b255: Int, a255: Int = 255) {
        self.init(
            red: CGFloat(r255) / 255.0,
            green: CGFloat(g255) / 255.0,
            blue: CGFloat(b255) / 255.0,
            alpha: CGFloat(a255) / 255.0
        )
    }

    /// Returns standard 6-character HEX string e.g. "#FF007A"
    public var hexString: String {
        let r = Int(round(red * 255))
        let g = Int(round(green * 255))
        let b = Int(round(blue * 255))
        return String(format: "#%02X%02X%02X", r, g, b)
    }

    /// Returns standard RGB format e.g. "rgb(255, 0, 122)"
    public var rgbString: String {
        let r = Int(round(red * 255))
        let g = Int(round(green * 255))
        let b = Int(round(blue * 255))
        return "rgb(\(r), \(g), \(b))"
    }

    /// Computes perceived luminance according to ITU-R BT.709
    public var luminance: CGFloat {
        0.2126 * red + 0.7152 * green + 0.0722 * blue
    }

    /// Returns true if white text has better contrast against this color.
    public var isDark: Bool {
        luminance < 0.5
    }

    /// Parse a Hex string (supports "#RRGGBB" or "RRGGBB")
    public static func fromHex(_ hex: String) -> ColorRepresentation? {
        var cleanHex = hex.trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
        if cleanHex.hasPrefix("#") {
            cleanHex.removeFirst()
        }

        guard cleanHex.count == 6, let hexValue = UInt32(cleanHex, radix: 16) else {
            return nil
        }

        let r = Int((hexValue >> 16) & 0xFF)
        let g = Int((hexValue >> 8) & 0xFF)
        let b = Int(hexValue & 0xFF)
        return ColorRepresentation(r255: r, g255: g, b255: b)
    }

    // Default palette presets
    public static let red = ColorRepresentation(r255: 255, g255: 59, b255: 48)
    public static let orange = ColorRepresentation(r255: 255, g255: 149, b255: 0)
    public static let yellow = ColorRepresentation(r255: 255, g255: 204, b255: 0)
    public static let green = ColorRepresentation(r255: 52, g255: 199, b255: 89)
    public static let cyan = ColorRepresentation(r255: 50, g255: 173, b255: 230)
    public static let blue = ColorRepresentation(r255: 0, g255: 122, b255: 255)
    public static let purple = ColorRepresentation(r255: 175, g255: 82, b255: 222)
    public static let white = ColorRepresentation(r255: 255, g255: 255, b255: 255)
    public static let black = ColorRepresentation(r255: 0, g255: 0, b255: 0)

    public static let defaultPresets: [ColorRepresentation] = [
        .red, .orange, .yellow, .green, .cyan, .blue, .purple, .white, .black
    ]
}
