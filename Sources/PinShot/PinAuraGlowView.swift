import SwiftUI
import PinShotCore

public struct PinAuraGlowView: View {
    let cornerRadius: CGFloat
    let style: PinShadowStyle
    let isHovering: Bool

    public init(cornerRadius: CGFloat = 10, style: PinShadowStyle = .nativeOutline, isHovering: Bool = false) {
        self.cornerRadius = cornerRadius
        self.style = style
        self.isHovering = isHovering
    }

    public var body: some View {
        Group {
            switch style {
            case .nativeOutline:
                ZStack {
                    // Soft natural macOS drop shadow
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(isHovering ? 0.35 : 0.22))
                        .blur(radius: isHovering ? 10 : 6)
                        .offset(y: 3)

                    // Clean crisp native outline (Visible on highlight/hover)
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(
                            isHovering ? Color(NSColor.controlAccentColor).opacity(0.9) : Color.white.opacity(0.15),
                            lineWidth: isHovering ? 2.0 : 1.0
                        )
                }

            case .classicShadow:
                RoundedRectangle(cornerRadius: cornerRadius)
                    .fill(Color.black.opacity(isHovering ? 0.45 : 0.28))
                    .blur(radius: isHovering ? 12 : 7)
                    .offset(y: 4)

            case .minimalBorder:
                ZStack {
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .fill(Color.black.opacity(0.20))
                        .blur(radius: 5)

                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(isHovering ? Color.white.opacity(0.8) : Color.white.opacity(0.25), lineWidth: 1.0)
                }

            case .none:
                EmptyView()
            }
        }
    }
}
