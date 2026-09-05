import SwiftUI
import PinShotCore

public struct PinAuraGlowView: View {
    let cornerRadius: CGFloat
    let style: PinShadowStyle
    let isHovering: Bool

    // Apple Intelligence Chromatic Colors
    private let auraGradient = AngularGradient(
        gradient: Gradient(colors: [
            Color(red: 0.16, green: 0.82, blue: 0.98), // Cyan
            Color(red: 0.35, green: 0.46, blue: 1.00), // Blue
            Color(red: 0.70, green: 0.28, blue: 0.98), // Violet
            Color(red: 0.98, green: 0.32, blue: 0.68), // Magenta
            Color(red: 1.00, green: 0.52, blue: 0.20), // Orange
            Color(red: 0.98, green: 0.82, blue: 0.30), // Gold
            Color(red: 0.16, green: 0.82, blue: 0.98)  // Loop
        ]),
        center: .center
    )

    public init(cornerRadius: CGFloat = 10, style: PinShadowStyle, isHovering: Bool = false) {
        self.cornerRadius = cornerRadius
        self.style = style
        self.isHovering = isHovering
    }

    public var body: some View {
        Group {
            switch style {
            case .appleIntelligence:
                ZStack {
                    // Outer diffuse chromatic bloom (Metal GPU accelerated)
                    RoundedRectangle(cornerRadius: cornerRadius + 2)
                        .stroke(auraGradient, lineWidth: isHovering ? 6.0 : 4.0)
                        .blur(radius: isHovering ? 12 : 8)
                        .opacity(isHovering ? 0.90 : 0.70)

                    // Inner radiant glow
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(auraGradient, lineWidth: isHovering ? 2.5 : 1.5)
                        .blur(radius: isHovering ? 3 : 1.5)
                        .opacity(0.85)

                    // Subtle black shadow underneath for depth
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(0.15))
                        .blur(radius: 6)
                }
                .drawingGroup() // Metal-accelerated GPU compositor for 0% CPU impact

            case .classicShadow:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(isHovering ? 0.45 : 0.30))
                    .blur(radius: isHovering ? 12 : 8)
                    .offset(y: 4)

            case .minimalBorder:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .stroke(isHovering ? Color.blue.opacity(0.8) : Color.white.opacity(0.3), lineWidth: 1.2)

            case .none:
                EmptyView()
            }
        }
    }
}
