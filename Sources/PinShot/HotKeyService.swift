import Cocoa
import Carbon

@MainActor
final class HotKeyService {
    static let shared = HotKeyService()
    
    private var hotKeyRef: EventHotKeyRef?
    private var eventHandlerRef: EventHandlerRef?
    private var action: (@MainActor () -> Void)?

    private init() {}

    /// Registers or updates the global hotkey according to user settings.
    func registerDefaultHotKey(action: @escaping @MainActor () -> Void) {
        self.action = action
        reloadHotKey()
    }

    func reloadHotKey() {
        unregisterHotKey()

        let settings = SettingsManager.shared.settings
        let keyCode: UInt32 = settings.globalKeyCode
        let modifiers: UInt32 = settings.globalModifiers

        let hotKeyID = EventHotKeyID(
            signature: OSType(0x50534854), // 'PSHT' in 4-byte hex
            id: 1
        )

        var eventType = EventTypeSpec(
            eventClass: OSType(kEventClassKeyboard),
            eventKind: UInt32(kEventHotKeyPressed)
        )

        let handlerBlock: EventHandlerUPP = { _, _, userData -> OSStatus in
            guard let userData = userData else { return noErr }
            let service = Unmanaged<HotKeyService>.fromOpaque(userData).takeUnretainedValue()
            Task { @MainActor in
                service.action?()
            }
            return noErr
        }

        let selfPtr = Unmanaged.passUnretained(self).toOpaque()
        InstallEventHandler(
            GetApplicationEventTarget(),
            handlerBlock,
            1,
            &eventType,
            selfPtr,
            &eventHandlerRef
        )

        let status = RegisterEventHotKey(
            keyCode,
            modifiers,
            hotKeyID,
            GetApplicationEventTarget(),
            0,
            &hotKeyRef
        )

        if status != noErr {
            print("Failed to register global hotkey with status: \(status)")
        }
    }

    func unregisterHotKey() {
        if let hotKey = hotKeyRef {
            UnregisterEventHotKey(hotKey)
            hotKeyRef = nil
        }
        if let handler = eventHandlerRef {
            RemoveEventHandler(handler)
            eventHandlerRef = nil
        }
    }
}
