import Foundation
import Cocoa
import PinShotCore

@MainActor
final class SettingsManager: ObservableObject {
    static let shared = SettingsManager()

    private let userDefaultsKey = "com.fable.PinShot.settings"

    @Published var settings: PinShotSettings {
        didSet {
            saveSettings()
            onSettingsChanged?()
        }
    }

    var onSettingsChanged: (() -> Void)?

    private init() {
        if let data = UserDefaults.standard.data(forKey: userDefaultsKey),
           let decoded = try? JSONDecoder().decode(PinShotSettings.self, from: data) {
            self.settings = decoded
        } else {
            self.settings = PinShotSettings()
        }
    }

    private func saveSettings() {
        if let encoded = try? JSONEncoder().encode(settings) {
            UserDefaults.standard.set(encoded, forKey: userDefaultsKey)
        }
    }

    func resetToDefaults() {
        settings = PinShotSettings()
    }
}
