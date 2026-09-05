import SwiftUI
import PinShotCore

public struct PinAuraGlowView: View {
    let cornerRadius: CGFloat
    let style: PinShadowStyle
    let isHovering: Bool

    @State private var rotationAngle: Double = 0

    // Apple Intelligence Vibrant Chromatic Colors
    private let auraColors: [Color] = [
        Color(red: 0.15, green: 0.85, blue: 1.00), // Neon Cyan
        Color(red: 0.35, green: 0.45, blue: 1.00), // Electric Blue
        Color(red: 0.72, green: 0.25, blue: 1.00), // Vivid Violet / Purple
        Color(red: 1.00, green: 0.28, blue: 0.65), // Magenta / Hot Pink
        Color(red: 1.00, green: 0.50, blue: 0.15), // Sunset Orange
        Color(red: 1.00, green: 0.82, blue: 0.25), // Sunburst Gold
        Color(red: 0.15, green: 0.85, blue: 1.00)  // Neon Cyan Loop
    ]

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
                    // 1. Soft outer ambient chromatic bloom (stays safely within 20pt spread)
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(
                            AngularGradient(
                                gradient: Gradient(colors: auraColors),
                                center: .center,
                                angle: .degrees(rotationAngle)
                            ),
                            lineWidth: isHovering ? 12 : 8
                        )
                        .blur(radius: isHovering ? 10 : 7)
                        .opacity(isHovering ? 0.90 : 0.75)

                    // 2. Crisp radiant rim glow around image perimeter
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(
                            AngularGradient(
                                gradient: Gradient(colors: auraColors),
                                center: .center,
                                angle: .degrees(rotationAngle)
                            ),
                            lineWidth: isHovering ? 4 : 2.5
                        )
                        .blur(radius: isHovering ? 3 : 1.5)
                        .opacity(isHovering ? 1.0 : 0.90)

                    // 3. Natural soft dark under-shadow for contrast
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(0.30))
                        .blur(radius: 6)
                        .offset(y: 2)
                }
                .onAppear {
                    // Smooth, continuous rotation around the perimeter
                    withAnimation(.linear(duration: 6.0).repeatForever(autoreverses: false)) {
                        rotationAngle = 360
                    }
                }

            case .classicShadow:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(isHovering ? 0.45 : 0.30))
                    .blur(radius: isHovering ? 10 : 7)
                    .offset(y: 3)

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
