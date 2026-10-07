// Removes the top `rows` pixel rows of a PNG in place: `swift crop-top.swift <file> <rows>`.
import AppKit

let file = URL(fileURLWithPath: CommandLine.arguments[1])
let rows = Int(CommandLine.arguments[2])!
guard let source = CGImageSourceCreateWithURL(file as CFURL, nil),
      let image = CGImageSourceCreateImageAtIndex(source, 0, nil),
      let cropped = image.cropping(to: CGRect(x: 0, y: rows, width: image.width, height: image.height - rows)),
      let destination = CGImageDestinationCreateWithURL(file as CFURL, "public.png" as CFString, 1, nil)
else { fatalError("cannot crop \(file.path)") }
CGImageDestinationAddImage(destination, cropped, nil)
guard CGImageDestinationFinalize(destination) else { fatalError("cannot write \(file.path)") }
