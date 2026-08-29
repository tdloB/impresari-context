// SPDX-License-Identifier: Apache-2.0
import Foundation

@main
struct ProbeHost {
    static func main() {
        let arguments = CommandLine.arguments
        guard arguments.count == 7,
              let unrelatedProcessID = Int32(arguments[5]),
              let loopbackPort = UInt16(arguments[6]),
              loopbackPort > 0 else {
            exit(1)
        }
        let canaries = zip(
            ProbeContract.canaryCategories,
            arguments[1...4]
        ).map { category, path in
            ProbeCanary(category: category, path: path)
        }
        let request = ProbeRequest(
            schemaName: "macos-xpc-sandbox-probe-request",
            schemaVersion: "1.0.0",
            canaries: canaries,
            unrelatedProcessID: unrelatedProcessID,
            loopbackPort: loopbackPort,
            authorityAdded: false
        )
        guard let requestBytes = canonicalJSON(request),
              requestBytes.count <= ProbeContract.maximumRequestBytes else {
            exit(1)
        }
        let connection = NSXPCConnection(serviceName: ProbeContract.serviceName)
        connection.remoteObjectInterface = NSXPCInterface(
            with: ImpresariSandboxProbeProtocol.self
        )
        connection.resume()

        let completion = DispatchSemaphore(value: 0)
        let lock = NSLock()
        var receipt: Data?
        var failed = false

        let proxy = connection.remoteObjectProxyWithErrorHandler { _ in
            lock.lock()
            failed = true
            lock.unlock()
            completion.signal()
        }
        guard let service = proxy as? ImpresariSandboxProbeProtocol else {
            connection.invalidate()
            exit(2)
        }
        service.runSyntheticProbe(requestBytes) { data in
            lock.lock()
            receipt = data
            lock.unlock()
            completion.signal()
        }

        guard completion.wait(timeout: .now() + .seconds(5)) == .success else {
            connection.invalidate()
            exit(3)
        }
        connection.invalidate()

        lock.lock()
        let output = receipt
        let didFail = failed
        lock.unlock()
        guard !didFail,
              let output,
              output.count <= ProbeContract.maximumReceiptBytes,
              let decoded = try? JSONDecoder().decode(ProbeReceipt.self, from: output),
              decoded.schemaName == "macos-xpc-sandbox-probe-receipt",
              decoded.schemaVersion == "1.0.0",
              decoded.requestAccepted,
              !decoded.osConfined,
              !decoded.productionAdmitted,
              !decoded.sourceRetained,
              !decoded.authorityAdded else {
            exit(4)
        }
        FileHandle.standardOutput.write(output)
        FileHandle.standardOutput.write(Data([0x0a]))
    }
}
