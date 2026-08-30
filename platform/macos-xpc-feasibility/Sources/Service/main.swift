// SPDX-License-Identifier: Apache-2.0
import Foundation
import Darwin

final class ProbeService: NSObject, ImpresariSandboxProbeProtocol {
    func prepareSyntheticProbe(
        _ request: Data,
        withReply reply: @escaping (Data) -> Void
    ) {
        guard let decoded = decodeRequest(request) else {
            reply(Data())
            return
        }

        let limitResult: (applied: Bool, errorNumber: Int32)
        switch decoded.probeMode {
        case .baseline, .supervisorTimeout:
            limitResult = (true, 0)
        case .cpuLimit:
            limitResult = applyLimit(resource: RLIMIT_CPU, value: 1)
        case .memoryLimit:
            let currentVirtualSize = impresari_current_virtual_size()
            limitResult = applyLimit(
                resource: RLIMIT_AS,
                value: currentVirtualSize + (128 * 1_024 * 1_024)
            )
        case .descendantLimit:
            limitResult = applyLimit(resource: RLIMIT_NPROC, value: 0)
        }

        let receipt = ProbePreparationReceipt(
            schemaName: "macos-xpc-sandbox-probe-preparation",
            schemaVersion: "1.0.0",
            probeMode: decoded.probeMode,
            serviceProcessID: getpid(),
            requestedLimitApplied: limitResult.applied,
            limitErrorNumber: limitResult.errorNumber,
            authorityAdded: false
        )
        reply(canonicalJSON(receipt) ?? Data())
    }

    func runSyntheticProbe(
        _ request: Data,
        withReply reply: @escaping (Data) -> Void
    ) {
        guard let decoded = decodeRequest(request) else {
            reply(Data())
            return
        }

        switch decoded.probeMode {
        case .baseline:
            runBaselineProbe(decoded, reply: reply)
        case .memoryLimit:
            runMemoryProbe(reply: reply)
        case .descendantLimit:
            runDescendantProbe(reply: reply)
        case .cpuLimit:
            runCPUExhaustion()
        case .supervisorTimeout:
            while true {
                sleep(1)
            }
        }
    }

    private func runBaselineProbe(
        _ decoded: ProbeRequest,
        reply: @escaping (Data) -> Void
    ) {

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
        let containerProbe = {
            do {
                try decoded.syntheticPayload.write(
                    to: temporary,
                    options: .withoutOverwriting
                )
                let verified = try Data(contentsOf: temporary) == decoded.syntheticPayload
                try FileManager.default.removeItem(at: temporary)
                return (
                    verified,
                    FileManager.default.fileExists(atPath: temporary.path)
                )
            } catch {
                try? FileManager.default.removeItem(at: temporary)
                return (
                    false,
                    FileManager.default.fileExists(atPath: temporary.path)
                )
            }
        }()

        let receipt = ProbeReceipt(
            schemaName: "macos-xpc-sandbox-probe-receipt",
            schemaVersion: "1.0.0",
            probeMode: .baseline,
            serviceProcessID: getpid(),
            requestAccepted: true,
            appContainerReadWriteVerified: containerProbe.0,
            canaryDenials: canaryDenials,
            deviceAccessDenied: false,
            unrelatedProcessAccessDenied: unrelatedProcessAccessDenied,
            networkDenialVerified: networkDenialVerified,
            resourceLimitsVerified: false,
            descendantContainmentVerified: false,
            osConfined: false,
            productionAdmitted: false,
            sourceRetained: containerProbe.1,
            authorityAdded: false
        )
        reply(canonicalJSON(receipt) ?? Data())
    }

    private func runMemoryProbe(reply: @escaping (Data) -> Void) {
        let requestedBytes = 1_024 * 1_024 * 1_024
        let mapping = mmap(
            nil,
            requestedBytes,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON,
            -1,
            0
        )
        let denied = mapping == MAP_FAILED
        if !denied {
            munmap(mapping, requestedBytes)
        }
        replyResource(
            mode: .memoryLimit,
            memoryAllocationDenied: denied,
            forkDenied: false,
            spawnDenied: false,
            reply: reply
        )
    }

    private func runDescendantProbe(reply: @escaping (Data) -> Void) {
        let descendantProbe = impresari_probe_descendants()
        replyResource(
            mode: .descendantLimit,
            memoryAllocationDenied: false,
            forkDenied: descendantProbe.fork_denied,
            spawnDenied: descendantProbe.spawn_denied,
            reply: reply
        )
    }

    private func runCPUExhaustion() -> Never {
        var accumulator: UInt64 = 0
        while true {
            accumulator &+= 1
            if accumulator == UInt64.max {
                accumulator = 0
            }
        }
    }

    private func replyResource(
        mode: ProbeMode,
        memoryAllocationDenied: Bool,
        forkDenied: Bool,
        spawnDenied: Bool,
        reply: @escaping (Data) -> Void
    ) {
        let receipt = ProbeResourceReceipt(
            schemaName: "macos-xpc-sandbox-resource-probe-receipt",
            schemaVersion: "1.0.0",
            probeMode: mode,
            serviceProcessID: getpid(),
            memoryAllocationDenied: memoryAllocationDenied,
            forkDenied: forkDenied,
            spawnDenied: spawnDenied,
            sourceRetained: false,
            authorityAdded: false
        )
        reply(canonicalJSON(receipt) ?? Data())
    }

    private func decodeRequest(_ request: Data) -> ProbeRequest? {
        guard request.count <= ProbeContract.maximumRequestBytes,
              let decoded = try? JSONDecoder().decode(ProbeRequest.self, from: request),
              decoded.schemaName == "macos-xpc-sandbox-probe-request",
              decoded.schemaVersion == "1.0.0",
              !decoded.authorityAdded,
              decoded.unrelatedProcessID > 1,
              decoded.loopbackPort > 0,
              decoded.syntheticPayload == Data("synthetic-only".utf8),
              decoded.canaries.map(\.category) == ProbeContract.canaryCategories,
              Set(decoded.canaries.map(\.path)).count == decoded.canaries.count else {
            return nil
        }
        return decoded
    }

    private func applyLimit(
        resource: Int32,
        value: UInt64
    ) -> (applied: Bool, errorNumber: Int32) {
        var limit = rlimit(rlim_cur: value, rlim_max: value)
        errno = 0
        let result = setrlimit(resource, &limit)
        return (result == 0, result == 0 ? 0 : errno)
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
