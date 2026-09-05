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
            Color(red: 0.35, green: 0.46, blue: 1.00), // Electric Blue
            Color(red: 0.70, green: 0.28, blue: 0.98), // Vivid Violet
            Color(red: 0.98, green: 0.32, blue: 0.68), // Magenta
            Color(red: 1.00, green: 0.52, blue: 0.20), // Sunset Orange
            Color(red: 0.98, green: 0.82, blue: 0.30), // Sunburst Gold
            Color(red: 0.16, green: 0.82, blue: 0.98)  // Cyan Loop
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
                    // Deep ambient chromatic bloom (pure diffuse aura, no hard lines or borders)
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(auraGradient)
                        .blur(radius: isHovering ? 18 : 12)
                        .opacity(isHovering ? 0.85 : 0.60)

                    // Secondary closer vibrant radiance
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(auraGradient)
                        .blur(radius: isHovering ? 8 : 5)
                        .opacity(isHovering ? 0.75 : 0.50)

                    // Soft dark under-shadow for natural elevation
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(0.30))
                        .blur(radius: 8)
                        .offset(y: 3)
                }

            case .classicShadow:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(isHovering ? 0.45 : 0.30))
                    .blur(radius: isHovering ? 14 : 9)
                    .offset(y: 4)

            case .minimalBorder:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(0.15))
                    .blur(radius: 4)

            case .none:
                EmptyView()
            }
        }
    }
}
