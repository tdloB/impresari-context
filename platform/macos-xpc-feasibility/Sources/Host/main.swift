// SPDX-License-Identifier: Apache-2.0
import Foundation

@main
struct ProbeHost {
    static func main() {
        let arguments = CommandLine.arguments
        guard arguments.count == 8,
              let probeMode = ProbeMode(rawValue: arguments[1]),
              let unrelatedProcessID = Int32(arguments[6]),
              let loopbackPort = UInt16(arguments[7]),
              loopbackPort > 0 else {
            exit(1)
        }
        let canaries = zip(
            ProbeContract.canaryCategories,
            arguments[2...5]
        ).map { category, path in
            ProbeCanary(category: category, path: path)
        }
        let request = ProbeRequest(
            schemaName: "macos-xpc-sandbox-probe-request",
            schemaVersion: "1.0.0",
            probeMode: probeMode,
            syntheticPayload: Data("synthetic-only".utf8),
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

        let preparationCompletion = DispatchSemaphore(value: 0)
        let preparationLock = NSLock()
        var preparation: Data?
        var preparationFailed = false
        let preparationProxy = connection.remoteObjectProxyWithErrorHandler { _ in
            preparationLock.lock()
            preparationFailed = true
            preparationLock.unlock()
            preparationCompletion.signal()
        }
        guard let preparationService = preparationProxy as? ImpresariSandboxProbeProtocol else {
            connection.invalidate()
            exit(2)
        }
        preparationService.prepareSyntheticProbe(requestBytes) { data in
            preparationLock.lock()
            preparation = data
            preparationLock.unlock()
            preparationCompletion.signal()
        }
        guard preparationCompletion.wait(timeout: .now() + .seconds(5)) == .success else {
            connection.invalidate()
            exit(3)
        }
        preparationLock.lock()
        let preparationOutput = preparation
        let didPreparationFail = preparationFailed
        preparationLock.unlock()
        guard !didPreparationFail,
              let preparationOutput,
              preparationOutput.count <= ProbeContract.maximumReceiptBytes,
              let decodedPreparation = try? JSONDecoder().decode(
                  ProbePreparationReceipt.self,
                  from: preparationOutput
              ),
              decodedPreparation.schemaName == "macos-xpc-sandbox-probe-preparation",
              decodedPreparation.schemaVersion == "1.0.0",
              decodedPreparation.probeMode == probeMode,
              decodedPreparation.serviceProcessID > 1,
              !decodedPreparation.authorityAdded else {
            connection.invalidate()
            exit(4)
        }
        FileHandle.standardError.write(Data("PREPARED ".utf8))
        FileHandle.standardError.write(preparationOutput)
        FileHandle.standardError.write(Data([0x0a]))
        guard decodedPreparation.requestedLimitApplied,
              decodedPreparation.limitErrorNumber == 0 else {
            connection.invalidate()
            exit(4)
        }

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

        guard completion.wait(timeout: .now() + .seconds(15)) == .success else {
            connection.invalidate()
            exit(5)
        }
        connection.invalidate()

        lock.lock()
        let output = receipt
        let didFail = failed
        lock.unlock()
        if didFail && (probeMode == .cpuLimit || probeMode == .supervisorTimeout) {
            exit(20)
        }
        guard !didFail,
              let output,
              output.count <= ProbeContract.maximumReceiptBytes,
              validateReceipt(output, mode: probeMode) else {
            exit(6)
        }
        FileHandle.standardOutput.write(output)
        FileHandle.standardOutput.write(Data([0x0a]))
    }

    private static func validateReceipt(_ output: Data, mode: ProbeMode) -> Bool {
        if mode == .baseline,
           let decoded = try? JSONDecoder().decode(ProbeReceipt.self, from: output) {
            return decoded.schemaName == "macos-xpc-sandbox-probe-receipt" &&
                decoded.schemaVersion == "1.0.0" &&
                decoded.probeMode == .baseline &&
                decoded.serviceProcessID > 1 &&
                decoded.requestAccepted &&
                !decoded.osConfined &&
                !decoded.productionAdmitted &&
                !decoded.sourceRetained &&
                !decoded.authorityAdded
        }
        guard let decoded = try? JSONDecoder().decode(
            ProbeResourceReceipt.self,
            from: output
        ) else {
            return false
        }
        return decoded.schemaName == "macos-xpc-sandbox-resource-probe-receipt" &&
            decoded.schemaVersion == "1.0.0" &&
            decoded.probeMode == mode &&
            decoded.serviceProcessID > 1 &&
            !decoded.sourceRetained &&
            !decoded.authorityAdded
    }
}
