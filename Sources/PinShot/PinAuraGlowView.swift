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
                    // 1. Apple Intelligence Chromatic Rainbow Aura (Visible ONLY when Highlighted/Hovered)
                    ZStack {
                        // Wide outer ambient chromatic bloom
                        RoundedRectangle(cornerRadius: cornerRadius)
                            .stroke(
                                AngularGradient(
                                    gradient: Gradient(colors: auraColors),
                                    center: .center,
                                    angle: .degrees(rotationAngle)
                                ),
                                lineWidth: 14
                            )
                            .blur(radius: 10)
                            .opacity(0.95)

                        // Closer radiant rim glow around image perimeter
                        RoundedRectangle(cornerRadius: cornerRadius)
                            .stroke(
                                AngularGradient(
                                    gradient: Gradient(colors: auraColors),
                                    center: .center,
                                    angle: .degrees(rotationAngle)
                                ),
                                lineWidth: 4.5
                            )
                            .blur(radius: 2.5)
                            .opacity(1.0)
                    }
                    .opacity(isHovering ? 1.0 : 0.0) // Sembunyi jika tidak di-highlight!
                    .animation(.easeInOut(duration: 0.22), value: isHovering)

                    // 2. Base clean minimal elevation shadow (always present for subtle depth)
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(isHovering ? 0.35 : 0.22))
                        .blur(radius: isHovering ? 8 : 5)
                        .offset(y: 2)
                }
                .onAppear {
                    // Smooth continuous chromatic rotation
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
