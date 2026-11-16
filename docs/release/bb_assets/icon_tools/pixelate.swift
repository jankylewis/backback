// pixelate.swift <in.png> <out.png> [grid=80]
// Pixel-art version of a transparent icon: crop to the subject, fit it into a grid x grid canvas,
// make the alpha hard (a pixel is fully opaque or fully clear), then scale to 1024 with
// nearest-neighbour. Hard alpha also removes the colour fringe of the old background.
import AppKit

/// Stop with a clear message when a step that cannot fail on a good input fails.
func must<T>(_ value: T?, _ what: String) -> T {
    guard let value else { fatalError(what) }
    return value
}

func context(_ side: Int) -> CGContext {
    let space = must(CGColorSpace(name: CGColorSpace.sRGB), "no sRGB color space")
    return must(CGContext(data: nil, width: side, height: side, bitsPerComponent: 8, bytesPerRow: side * 4,
                          space: space, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue), "no bitmap context")
}

/// The smallest rectangle that holds every pixel that is not clear.
func subjectBounds(_ image: NSBitmapImageRep) -> CGRect {
    var minX = image.pixelsWide, minY = image.pixelsHigh, maxX = 0, maxY = 0
    for y in 0..<image.pixelsHigh {
        for x in 0..<image.pixelsWide where (image.colorAt(x: x, y: y)?.alphaComponent ?? 0) > 0.05 {
            minX = min(minX, x); maxX = max(maxX, x); minY = min(minY, y); maxY = max(maxY, y)
        }
    }
    return CGRect(x: minX, y: minY, width: maxX - minX + 1, height: maxY - minY + 1)
}

/// Opaque enough stays, fully opaque; the rest becomes clear (this drops the background fringe).
func hardenAlpha(_ small: CGContext, grid: Int) {
    let pixels = must(small.data, "no pixel data").bindMemory(to: UInt8.self, capacity: grid * grid * 4)
    for index in 0..<(grid * grid) {
        let alpha = Double(pixels[index * 4 + 3])
        if alpha < 215 {
            for channel in 0..<4 { pixels[index * 4 + channel] = 0 }
            continue
        }
        for channel in 0..<3 {
            pixels[index * 4 + channel] = UInt8(min(255, Double(pixels[index * 4 + channel]) * 255 / alpha))
        }
        pixels[index * 4 + 3] = 255
    }
}

let args = CommandLine.arguments
guard args.count >= 3 else { fatalError("usage: pixelate <in.png> <out.png> [grid]") }
let grid = args.count > 3 ? must(Int(args[3]), "grid must be a number") : 80
let source = must(NSBitmapImageRep(data: try Data(contentsOf: URL(fileURLWithPath: args[1]))), "not an image")
let crop = must(source.cgImage?.cropping(to: subjectBounds(source)), "cannot crop")

let small = context(grid)
small.interpolationQuality = .high
let scale = CGFloat(grid) / CGFloat(max(crop.width, crop.height))
let width = CGFloat(crop.width) * scale, height = CGFloat(crop.height) * scale
small.draw(crop, in: CGRect(x: (CGFloat(grid) - width) / 2, y: (CGFloat(grid) - height) / 2,
                            width: width, height: height))
hardenAlpha(small, grid: grid)

let big = context(1024)
big.interpolationQuality = .none
big.draw(must(small.makeImage(), "no small image"), in: CGRect(x: 0, y: 0, width: 1024, height: 1024))
let output = NSBitmapImageRep(cgImage: must(big.makeImage(), "no big image"))
try must(output.representation(using: .png, properties: [:]), "cannot encode PNG")
    .write(to: URL(fileURLWithPath: args[2]))
