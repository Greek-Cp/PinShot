import Cocoa
import SwiftUI
import PinShotCore

@MainActor
public protocol FloatingToolbarDelegate: AnyObject {
    func toolbarDidSelectPin()
    func toolbarDidSelectAnnotate()
    func toolbarDidSelectCopy()
    func toolbarDidSelectSave()
    func toolbarDidSelectClose()
    func toolbarDidChangeTool(_ tool: AnnotationToolType)
    func toolbarDidChangeColor(_ color: ColorRepresentation)
    func toolbarDidChangeStrokeWidth(_ width: CGFloat)
    func toolbarDidUndo()
    func toolbarDidRedo()
}

struct FloatingToolbarView: View {
    weak var delegate: FloatingToolbarDelegate?
    @State var isAnnotating: Bool = false
    @State var selectedTool: AnnotationToolType = .rectangle
    @State var selectedColor: ColorRepresentation = .red
    @State var strokeWidth: CGFloat = 3.0
    var isVertical: Bool = false

    var body: some View {
        VStack(spacing: 8) {
            // Main Action Bar
            HStack(spacing: 6) {
                // Pin Button
                ToolbarButton(
                    icon: "pin.fill",
                    title: "Pin (P)",
                    tint: .orange
                ) {
                    delegate?.toolbarDidSelectPin()
                }

                // Annotate Toggle Button
                ToolbarButton(
                    icon: isAnnotating ? "pencil.circle.fill" : "pencil.circle",
                    title: "Annotate (A)",
                    tint: isAnnotating ? .blue : .primary
                ) {
                    isAnnotating.toggle()
                    delegate?.toolbarDidSelectAnnotate()
                }

                Divider()
                    .frame(height: 18)

                // Copy Button
                ToolbarButton(
                    icon: "doc.on.doc.fill",
                    title: "Copy (⌘C)",
                    tint: .green
                ) {
                    delegate?.toolbarDidSelectCopy()
                }

                // Save Button
                ToolbarButton(
                    icon: "arrow.down.circle.fill",
                    title: "Save (⌘S)",
                    tint: .blue
                ) {
                    delegate?.toolbarDidSelectSave()
                }

                Divider()
                    .frame(height: 18)

                // Close Button
                ToolbarButton(
                    icon: "xmark.circle.fill",
                    title: "Cancel (Esc)",
                    tint: .red
                ) {
                    delegate?.toolbarDidSelectClose()
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)

            // Extended Annotation Sub-Toolbar (Conditional)
            if isAnnotating {
                Divider()

                VStack(spacing: 8) {
                    // Tool Selection
                    HStack(spacing: 8) {
                        ToolOptionButton(icon: "rectangle", isSelected: selectedTool == .rectangle) {
                            selectedTool = .rectangle
                            delegate?.toolbarDidChangeTool(.rectangle)
                        }
                        ToolOptionButton(icon: "oval", isSelected: selectedTool == .ellipse) {
                            selectedTool = .ellipse
                            delegate?.toolbarDidChangeTool(.ellipse)
                        }
                        ToolOptionButton(icon: "arrow.up.right", isSelected: selectedTool == .arrow) {
                            selectedTool = .arrow
                            delegate?.toolbarDidChangeTool(.arrow)
                        }
                        ToolOptionButton(icon: "pencil.tip", isSelected: selectedTool == .pen) {
                            selectedTool = .pen
                            delegate?.toolbarDidChangeTool(.pen)
                        }
                        ToolOptionButton(icon: "highlighter", isSelected: selectedTool == .highlighter) {
                            selectedTool = .highlighter
                            delegate?.toolbarDidChangeTool(.highlighter)
                        }
                        ToolOptionButton(icon: "textformat", isSelected: selectedTool == .text) {
                            selectedTool = .text
                            delegate?.toolbarDidChangeTool(.text)
                        }
                        ToolOptionButton(icon: "1.circle.fill", isSelected: selectedTool == .step) {
                            selectedTool = .step
                            delegate?.toolbarDidChangeTool(.step)
                        }
                        ToolOptionButton(icon: "squareshape.split.3x3", isSelected: selectedTool == .mosaic) {
                            selectedTool = .mosaic
                            delegate?.toolbarDidChangeTool(.mosaic)
                        }

                        Spacer()

                        // Undo / Redo
                        Button(action: { delegate?.toolbarDidUndo() }) {
                            Image(systemName: "arrow.uturn.backward")
                                .font(.system(size: 12, weight: .semibold))
                        }
                        .buttonStyle(.plain)

                        Button(action: { delegate?.toolbarDidRedo() }) {
                            Image(systemName: "arrow.uturn.forward")
                                .font(.system(size: 12, weight: .semibold))
                        }
                        .buttonStyle(.plain)
                    }
                    .padding(.horizontal, 10)

                    // Color Swatches & Stroke Width
                    HStack(spacing: 8) {
                        ForEach(ColorRepresentation.defaultPresets, id: \.hexString) { preset in
                            Circle()
                                .fill(Color(red: preset.red, green: preset.green, blue: preset.blue))
                                .frame(width: 16, height: 16)
                                .overlay(
                                    Circle()
                                        .stroke(Color.white, lineWidth: selectedColor == preset ? 2 : 0)
                                )
                                .shadow(radius: 1)
                                .onTapGesture {
                                    selectedColor = preset
                                    delegate?.toolbarDidChangeColor(preset)
                                }
                        }

                        Spacer()

                        // Stroke size picker
                        Picker("", selection: $strokeWidth) {
                            Text("S").tag(CGFloat(2.0))
                            Text("M").tag(CGFloat(4.0))
                            Text("L").tag(CGFloat(7.0))
                        }
                        .pickerStyle(.segmented)
                        .frame(width: 80)
                        .onChange(of: strokeWidth) { _, newWidth in
                            delegate?.toolbarDidChangeStrokeWidth(newWidth)
                        }
                    }
                    .padding(.horizontal, 10)
                    .padding(.bottom, 6)
                }
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 12)
                .fill(.ultraThinMaterial)
                .shadow(color: .black.opacity(0.25), radius: 10, x: 0, y: 5)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .stroke(Color(NSColor.separatorColor).opacity(0.4), lineWidth: 0.8)
        )
        .frame(minWidth: 320)
    }
}

private struct ToolbarButton: View {
    let icon: String
    let title: String
    let tint: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(tint)
                Text(title)
                    .font(.system(size: 11, weight: .medium))
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 5)
            .background(Color(NSColor.controlBackgroundColor).opacity(0.4))
            .cornerRadius(6)
        }
        .buttonStyle(.plain)
    }
}

private struct ToolOptionButton: View {
    let icon: String
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: icon)
                .font(.system(size: 12, weight: isSelected ? .bold : .regular))
                .foregroundColor(isSelected ? .blue : .secondary)
                .frame(width: 24, height: 24)
                .background(isSelected ? Color.blue.opacity(0.15) : Color.clear)
                .cornerRadius(4)
        }
        .buttonStyle(.plain)
    }
}

final class FloatingToolbarPanel: NSPanel {
    init(view: NSView) {
        super.init(
            contentRect: .zero,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        self.isOpaque = false
        self.backgroundColor = .clear
        self.level = .screenSaver + 1
        self.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]
        self.contentView = view
        self.hasShadow = true
    }
}
