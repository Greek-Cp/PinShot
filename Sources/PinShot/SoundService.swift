import Cocoa
import AudioToolbox

@MainActor
final class SoundService {
    static let shared = SoundService()

    func playCaptureSound() {
        NSSound(named: "Tink")?.play()
    }

    func playPinSound() {
        NSSound(named: "Pop")?.play()
    }
}
