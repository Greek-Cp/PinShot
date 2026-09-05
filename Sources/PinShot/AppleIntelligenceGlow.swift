import SwiftUI

public struct AppleIntelligencePalette {
    public static let colors: [Color] = [
        Color(red: 0.18, green: 0.82, blue: 0.98), // Neon Cyan
        Color(red: 0.35, green: 0.45, blue: 1.00), // Electric Blue
        Color(red: 0.70, green: 0.28, blue: 0.98), // Vivid Violet
        Color(red: 0.98, green: 0.32, blue: 0.68), // Magenta Pink
        Color(red: 1.00, green: 0.52, blue: 0.20), // Warm Coral / Sunset Orange
        Color(red: 0.98, green: 0.82, blue: 0.30), // Sunburst Gold
        Color(red: 0.18, green: 0.82, blue: 0.98)  // Neon Cyan wrap-around
    ]
}

/// Dynamic Apple Intelligence-style rotating iridescent chromatic glow border.
public struct AppleIntelligenceGlowBorder: View {
    let cornerRadius: CGFloat
    let lineWidth: CGFloat
    let isHovered: Bool
    
    @State private var rotationAngle: Double = 0
    @State private var pulseIntensity: Double = 0.85

    public init(cornerRadius: CGFloat = 12, lineWidth: CGFloat = 2.5, isHovered: Bool = false) {
        self.cornerRadius = cornerRadius
        self.lineWidth = lineWidth
        self.isHovered = isHovered
    }

    public var body: some View {
        ZStack {
            // 1. Deep diffused chromatic bloom layer
            RoundedRectangle(cornerRadius: cornerRadius)
                .stroke(
                    AngularGradient(
                        gradient: Gradient(colors: AppleIntelligencePalette.colors),
                        center: .center,
                        angle: .degrees(rotationAngle)
                    ),
                    lineWidth: lineWidth * 3.0
                )
                .blur(radius: isHovered ? 12 : 7)
                .opacity(isHovered ? 0.95 : 0.65)

            // 2. Medium radiant neon glow
            RoundedRectangle(cornerRadius: cornerRadius)
                .stroke(
                    AngularGradient(
                        gradient: Gradient(colors: AppleIntelligencePalette.colors),
                        center: .center,
                        angle: .degrees(rotationAngle)
                    ),
                    lineWidth: lineWidth * 1.6
                )
                .blur(radius: isHovered ? 4 : 2.5)
                .opacity(isHovered ? 1.0 : 0.85)

            // 3. Crisp sharp iridescent edge line
            RoundedRectangle(cornerRadius: cornerRadius)
                .stroke(
                    AngularGradient(
                        gradient: Gradient(colors: AppleIntelligencePalette.colors),
                        center: .center,
                        angle: .degrees(rotationAngle)
                    ),
                    lineWidth: lineWidth
                )
                .opacity(0.95)

            // 4. White specular rim glint
            RoundedRectangle(cornerRadius: cornerRadius)
                .stroke(
                    LinearGradient(
                        colors: [Color.white.opacity(0.8), Color.white.opacity(0.0), Color.white.opacity(0.4)],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    lineWidth: 0.75
                )
        }
        .onAppear {
            withAnimation(.linear(duration: 4.5).repeatForever(autoreverses: false)) {
                rotationAngle = 360
            }
            withAnimation(.easeInOut(duration: 2.0).repeatForever(autoreverses: true)) {
                pulseIntensity = 1.0
            }
        }
    }
}
