// SPDX-License-Identifier: Apache-2.0
import Foundation
import Darwin

final class ProbeService: NSObject, ImpresariSandboxProbeProtocol {
    func runSyntheticProbe(
        _ request: Data,
        withReply reply: @escaping (Data) -> Void
    ) {
        guard request.count <= ProbeContract.maximumRequestBytes,
              let decoded = try? JSONDecoder().decode(ProbeRequest.self, from: request),
              decoded.schemaName == "macos-xpc-sandbox-probe-request",
              decoded.schemaVersion == "1.0.0",
              !decoded.authorityAdded,
              decoded.unrelatedProcessID > 1,
              decoded.loopbackPort > 0,
              decoded.canaries.map(\.category) == ProbeContract.canaryCategories,
              Set(decoded.canaries.map(\.path)).count == decoded.canaries.count else {
            reply(Data())
            return
        }

        let canaryDenials = Dictionary(
            uniqueKeysWithValues: decoded.canaries.map { canary in
                let denied = (try? Data(
                    contentsOf: URL(fileURLWithPath: canary.path)
                )) == nil
                return (canary.category, denied)
            }
        )
        errno = 0
        let processProbe = kill(decoded.unrelatedProcessID, 0)
        let unrelatedProcessAccessDenied = processProbe == -1 && errno == EPERM
        let networkDenialVerified = loopbackConnectionIsDenied(decoded.loopbackPort)

        let temporary = FileManager.default.temporaryDirectory
            .appendingPathComponent("synthetic-probe", isDirectory: false)
        let synthetic = Data("synthetic-only".utf8)
        let appContainerReadWriteVerified = {
            do {
                try synthetic.write(to: temporary, options: .withoutOverwriting)
                let verified = try Data(contentsOf: temporary) == synthetic
                try FileManager.default.removeItem(at: temporary)
                return verified
            } catch {
                try? FileManager.default.removeItem(at: temporary)
                return false
            }
        }()

        let receipt = ProbeReceipt(
            schemaName: "macos-xpc-sandbox-probe-receipt",
            schemaVersion: "1.0.0",
            requestAccepted: true,
            appContainerReadWriteVerified: appContainerReadWriteVerified,
            canaryDenials: canaryDenials,
            deviceAccessDenied: false,
            unrelatedProcessAccessDenied: unrelatedProcessAccessDenied,
            networkDenialVerified: networkDenialVerified,
            resourceLimitsVerified: false,
            descendantContainmentVerified: false,
            osConfined: false,
            productionAdmitted: false,
            sourceRetained: false,
            authorityAdded: false
        )
        reply(canonicalJSON(receipt) ?? Data())
    }
}

private func loopbackConnectionIsDenied(_ port: UInt16) -> Bool {
    let descriptor = socket(AF_INET, SOCK_STREAM, 0)
    guard descriptor >= 0 else {
        return false
    }
    defer { close(descriptor) }

    var address = sockaddr_in()
    address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
    address.sin_family = sa_family_t(AF_INET)
    address.sin_port = port.bigEndian
    address.sin_addr = in_addr(s_addr: inet_addr("127.0.0.1"))
    errno = 0
    let result = withUnsafePointer(to: &address) { pointer in
        pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) { socketAddress in
            connect(
                descriptor,
                socketAddress,
                socklen_t(MemoryLayout<sockaddr_in>.size)
            )
        }
    }
    return result == -1 && (errno == EPERM || errno == EACCES)
}

final class ProbeServiceDelegate: NSObject, NSXPCListenerDelegate {
    func listener(
        _ listener: NSXPCListener,
        shouldAcceptNewConnection connection: NSXPCConnection
    ) -> Bool {
        connection.exportedInterface = NSXPCInterface(
            with: ImpresariSandboxProbeProtocol.self
        )
        connection.exportedObject = ProbeService()
        connection.resume()
        return true
    }
}

@main
struct ProbeServiceMain {
    static func main() {
        let listener = NSXPCListener.service()
        let delegate = ProbeServiceDelegate()
        listener.delegate = delegate
        listener.resume()
        RunLoop.current.run()
    }
}
