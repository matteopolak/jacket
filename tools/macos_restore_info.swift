// Query trusted Apple restore metadata. This creates no guest and executes no Jai code.
import Foundation
import Virtualization

struct RestoreInfo: Encodable {
    let url: URL
    let build: String
    let version: String
    let minimumCPUCount: Int
    let minimumMemoryBytes: UInt64
    let hardwareModelSupported: Bool
}

enum InspectionError: Error { case unsupportedHost, noSupportedConfiguration }

@main
struct RestoreInspector {
    static func main() async {
        do {
            guard VZVirtualMachine.isSupported else { throw InspectionError.unsupportedHost }
            let image = try await VZMacOSRestoreImage.latestSupported
            guard let requirements = image.mostFeaturefulSupportedConfiguration else {
                throw InspectionError.noSupportedConfiguration
            }
            let version = image.operatingSystemVersion
            let info = RestoreInfo(
                url: image.url, build: image.buildVersion,
                version: "\(version.majorVersion).\(version.minorVersion).\(version.patchVersion)",
                minimumCPUCount: requirements.minimumSupportedCPUCount,
                minimumMemoryBytes: requirements.minimumSupportedMemorySize,
                hardwareModelSupported: requirements.hardwareModel.isSupported
            )
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            FileHandle.standardOutput.write(try encoder.encode(info))
            FileHandle.standardOutput.write(Data([10]))
        } catch {
            FileHandle.standardError.write(Data("Restore metadata lookup failed: \(error)\n".utf8))
            exit(1)
        }
    }
}
