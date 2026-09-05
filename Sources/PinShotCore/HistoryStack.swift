import Foundation

/// A generic, thread-safe Undo/Redo stack for maintaining mutation history.
public struct HistoryStack<T: Equatable & Sendable>: Sendable {
    private var undoStack: [T]
    private var redoStack: [T]
    public let maxCapacity: Int

    public init(initialState: T, maxCapacity: Int = 50) {
        self.undoStack = [initialState]
        self.redoStack = []
        self.maxCapacity = maxCapacity
    }

    /// The current active state.
    public var currentState: T {
        undoStack.last!
    }

    public var canUndo: Bool {
        undoStack.count > 1
    }

    public var canRedo: Bool {
        !redoStack.isEmpty
    }

    /// Pushes a new state to history and clears the redo branch.
    public mutating func push(_ newState: T) {
        guard newState != currentState else { return }
        undoStack.append(newState)
        if undoStack.count > maxCapacity {
            undoStack.removeFirst()
        }
        redoStack.removeAll()
    }

    /// Undoes to the previous state.
    @discardableResult
    public mutating func undo() -> T? {
        guard canUndo else { return nil }
        let current = undoStack.removeLast()
        redoStack.append(current)
        return currentState
    }

    /// Redoes the previously undone state.
    @discardableResult
    public mutating func redo() -> T? {
        guard canRedo else { return nil }
        let next = redoStack.removeLast()
        undoStack.append(next)
        return next
    }

    /// Resets stack with a new initial state.
    public mutating func reset(with state: T) {
        undoStack = [state]
        redoStack.removeAll()
    }
}
