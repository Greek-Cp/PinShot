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
                    // 1. Base subtle dark shadow (for depth on any background)
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(isHovering ? 0.30 : 0.20))
                        .blur(radius: isHovering ? 8 : 5)
                        .offset(y: 2)

                    // 2. Apple Intelligence Chromatic Rainbow Aura (Visible ONLY when Highlighted/Hovered)
                    ZStack {
                        // Soft diffuse outer aura bloom
                        RoundedRectangle(cornerRadius: cornerRadius + 2)
                            .stroke(
                                AngularGradient(
                                    gradient: Gradient(colors: auraColors),
                                    center: .center,
                                    angle: .degrees(rotationAngle)
                                ),
                                lineWidth: 8
                            )
                            .blur(radius: 8)
                            .opacity(0.85)

                        // Vibrant closer rim radiance
                        RoundedRectangle(cornerRadius: cornerRadius)
                            .stroke(
                                AngularGradient(
                                    gradient: Gradient(colors: auraColors),
                                    center: .center,
                                    angle: .degrees(rotationAngle)
                                ),
                                lineWidth: 3.5
                            )
                            .blur(radius: 2)
                            .opacity(0.95)
                    }
                    .opacity(isHovering ? 1.0 : 0.0)
                    .animation(.easeInOut(duration: 0.22), value: isHovering)
                }
                .onAppear {
                    withAnimation(.linear(duration: 5.5).repeatForever(autoreverses: false)) {
                        rotationAngle = 360
                    }
                }

            case .classicShadow:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(isHovering ? 0.45 : 0.28))
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
