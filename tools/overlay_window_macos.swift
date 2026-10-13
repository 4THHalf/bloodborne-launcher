// Test geometry only. Reads window bounds and focus metadata; never captures pixels.
import AppKit
import CoreGraphics
import Foundation
let pid = CommandLine.arguments.count > 1 ? Int(CommandLine.arguments[1]) ?? 0 : 0
let front = Int(NSWorkspace.shared.frontmostApplication?.processIdentifier ?? 0)
var rect = CGDisplayBounds(CGMainDisplayID())
var found = pid == 0
if pid != 0, let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] {
    for window in windows {
        guard window[kCGWindowOwnerPID as String] as? Int == pid,
              window[kCGWindowLayer as String] as? Int == 0,
              let bounds = window[kCGWindowBounds as String] as? [String: Any],
              let candidate = CGRect(dictionaryRepresentation: bounds as CFDictionary), candidate.width > 200, candidate.height > 200 else { continue }
        rect = candidate; found = true; break
    }
}
let result: [String: Any] = ["found":found, "front_pid":front, "x":rect.minX, "y":rect.minY, "width":rect.width, "height":rect.height]
let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
print(String(decoding:data,as:UTF8.self))
