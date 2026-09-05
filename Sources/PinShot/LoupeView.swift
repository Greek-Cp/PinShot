import Cocoa
import SwiftUI
import PinShotCore

struct LoupeView: View {
    let pixelColor: ColorRepresentation
    let coordinate: CGPoint
    let zoomImage: NSImage?

    var body: some View {
        VStack(spacing: 4) {
            // Magnified Pixel Area
            ZStack {
                if let img = zoomImage {
                    Image(nsImage: img)
                        .interpolation(.none)
                        .resizable()
                        .frame(width: 90, height: 90)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                } else {
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color(red: pixelColor.red, green: pixelColor.green, blue: pixelColor.blue))
                        .frame(width: 90, height: 90)
                }

                // Center Crosshair Indicator
                Rectangle()
                    .stroke(Color.white, lineWidth: 1.5)
                    .frame(width: 10, height: 10)
                    .shadow(color: .black, radius: 1)
            }
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.white.opacity(0.8), lineWidth: 1.5)
            )

            // Color Information Bar
            VStack(spacing: 2) {
                Text(pixelColor.hexString)
                    .font(.system(size: 11, weight: .bold, design: .monospaced))
                    .foregroundColor(.white)

                Text(pixelColor.rgbString)
                    .font(.system(size: 9, weight: .medium, design: .monospaced))
                    .foregroundColor(.white.opacity(0.8))

                Text("X: \(Int(coordinate.x))  Y: \(Int(coordinate.y))")
                    .font(.system(size: 8, weight: .regular, design: .monospaced))
                    .foregroundColor(.white.opacity(0.6))
            }
            .padding(.vertical, 3)
            .padding(.horizontal, 6)
            .background(Color.black.opacity(0.75))
            .cornerRadius(4)
        }
        .padding(6)
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(.ultraThinMaterial)
                .shadow(color: .black.opacity(0.4), radius: 10, x: 0, y: 4)
        )
    }
}
