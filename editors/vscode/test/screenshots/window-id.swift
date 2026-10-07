// Prints the CGWindowID of the largest on-screen window owned by one of the given process IDs,
// for `screencapture -l<id>`. Used by the screenshot harness only (macOS).
import CoreGraphics
import Foundation

let pids = Set(CommandLine.arguments.dropFirst().compactMap { Int32($0) })
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
var best: (id: Int, area: Double)? = nil
for window in windows {
    guard let pid = window[kCGWindowOwnerPID as String] as? Int32, pids.contains(pid),
          let layer = window[kCGWindowLayer as String] as? Int, layer == 0,
          let id = window[kCGWindowNumber as String] as? Int,
          let bounds = window[kCGWindowBounds as String] as? [String: Double] else { continue }
    let area = (bounds["Width"] ?? 0) * (bounds["Height"] ?? 0)
    if best == nil || area > best!.area { best = (id, area) }
}
if let best { print(best.id) } else { exit(1) }
