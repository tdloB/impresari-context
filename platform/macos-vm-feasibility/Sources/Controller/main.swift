// SPDX-License-Identifier: Apache-2.0

import CryptoKit
import Foundation
import Virtualization

private let profileID = "iar-macos-local-vm-synthetic-matrix-v1"
private let profileDigest = "sha256:a411dc8d896b9b516cb535786fe2d12f17c6bfed3b39b2104c040e7556507522"
private let kernelDigest = "8b216f74e7f89def4604adf69e2345437363aff4819101bb1551c9e83cd35cdd"
private let initramfsDigest = "cc87a9a68d06826277dd759befd318272a7876540b4287cfd6fe0ac67552bfbf"
private let kernelBytes: UInt64 = 36_110_336
private let inputBytes = 4_096
private let scratchBytes: UInt64 = 1_048_576
private let maximumInitramfsBytes: UInt64 = 2_097_152
private let maximumSerialBytes = 65_536
private let controllerTimeout: DispatchTimeInterval = .seconds(30)

private enum ControllerFailure: Error {
    case usage
    case unsupported
    case invalidIdentity
    case invalidJob
    case invalidConfiguration
    case start
    case timeout
    case cancelled
    case outputLimit
    case guest
    case cleanup
}

private enum Scenario: String {
    case success
    case malformedResult = "malformed-result"
    case outputFlood = "output-flood"
    case timeout
    case descendantTimeout = "descendant-timeout"
    case earlyExit = "early-exit"
    case cancellation

    var deadline: DispatchTimeInterval {
        switch self {
        case .timeout, .descendantTimeout: return .seconds(2)
        default: return controllerTimeout
        }
    }

    var guestMode: String {
        self == .cancellation ? Scenario.timeout.rawValue : rawValue
    }
}

private struct GuestReceipt: Decodable {
    let schemaName: String
    let schemaVersion: String
    let result: String
    let exactInputVerified: Bool
    let readOnlyInputVerified: Bool
    let scratchInitiallyClean: Bool
    let scratchCapacityVerified: Bool
    let networkDeviceAbsent: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case result
        case exactInputVerified = "exact_input_verified"
        case readOnlyInputVerified = "read_only_input_verified"
        case scratchInitiallyClean = "scratch_initially_clean"
        case scratchCapacityVerified = "scratch_capacity_verified"
        case networkDeviceAbsent = "network_device_absent"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

private struct ControllerReceipt: Encodable {
    let schemaName = "macos-local-vm-matrix-job-receipt"
    let schemaVersion = "1.0.0"
    let profileID: String
    let profileDigest: String
    let jobID: String
    let result: String
    let kernelDigest: String
    let initramfsDigest: String
    let inputDigest: String
    let virtualizationSupported: Bool
    let configurationValidated: Bool
    let cpuCount: String
    let memoryBytes: String
    let serialPorts: String
    let storageDevices: String
    let networkDevices: String
    let directoryShares: String
    let graphicsDevices: String
    let audioDevices: String
    let inputDevices: String
    let exactInputVerified: Bool
    let readOnlyInputVerified: Bool
    let scratchInitiallyClean: Bool
    let scratchCapacityVerified: Bool
    let networkDeviceAbsent: Bool
    let jobRemoved: Bool
    let vmConfined: Bool
    let productionAdmitted: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case profileID = "profile_id"
        case profileDigest = "profile_digest"
        case jobID = "job_id"
        case result
        case kernelDigest = "kernel_digest"
        case initramfsDigest = "initramfs_digest"
        case inputDigest = "input_digest"
        case virtualizationSupported = "virtualization_supported"
        case configurationValidated = "configuration_validated"
        case cpuCount = "cpu_count"
        case memoryBytes = "memory_bytes"
        case serialPorts = "serial_ports"
        case storageDevices = "storage_devices"
        case networkDevices = "network_devices"
        case directoryShares = "directory_shares"
        case graphicsDevices = "graphics_devices"
        case audioDevices = "audio_devices"
        case inputDevices = "input_devices"
        case exactInputVerified = "exact_input_verified"
        case readOnlyInputVerified = "read_only_input_verified"
        case scratchInitiallyClean = "scratch_initially_clean"
        case scratchCapacityVerified = "scratch_capacity_verified"
        case networkDeviceAbsent = "network_device_absent"
        case jobRemoved = "job_removed"
        case vmConfined = "vm_confined"
        case productionAdmitted = "production_admitted"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

private final class VirtualMachineDelegate: NSObject, VZVirtualMachineDelegate {
    let stopped = DispatchSemaphore(value: 0)
    private(set) var error: Error?

    func guestDidStop(_ virtualMachine: VZVirtualMachine) {
        stopped.signal()
    }

    func virtualMachine(_ virtualMachine: VZVirtualMachine, didStopWithError error: Error) {
        self.error = error
        stopped.signal()
    }
}

private final class CancellationState {
    private let lock = NSLock()
    private var value = false

    func cancel() {
        lock.lock()
        value = true
        lock.unlock()
    }

    func isCancelled() -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return value
    }
}

private final class BoundedSerialCapture {
    let writer: FileHandle
    private let reader: FileHandle
    private let finished = DispatchSemaphore(value: 0)
    private let lock = NSLock()
    private var buffer = Data()
    private var overflow = false

    init() {
        let pipe = Pipe()
        reader = pipe.fileHandleForReading
        writer = pipe.fileHandleForWriting
    }

    func start() {
        DispatchQueue.global(qos: .userInitiated).async { [self] in
            while true {
                let chunk = (try? reader.read(upToCount: 4_096)) ?? Data()
                if chunk.isEmpty { break }
                lock.lock()
                let remaining = max(0, maximumSerialBytes - buffer.count)
                if chunk.count > remaining { overflow = true }
                if remaining > 0 { buffer.append(chunk.prefix(remaining)) }
                lock.unlock()
            }
            try? reader.close()
            finished.signal()
        }
    }

    func closeAndCollect() -> (Data, Bool) {
        try? writer.close()
        _ = finished.wait(timeout: .now() + .seconds(2))
        lock.lock()
        defer { lock.unlock() }
        return (buffer, overflow)
    }
}

private func digest(_ url: URL) throws -> String {
    let handle = try FileHandle(forReadingFrom: url)
    defer { try? handle.close() }
    var hasher = SHA256()
    while true {
        let data = try handle.read(upToCount: 65_536) ?? Data()
        if data.isEmpty { break }
        hasher.update(data: data)
    }
    return hasher.finalize().map { String(format: "%02x", $0) }.joined()
}

private func exactFileSize(_ url: URL) throws -> UInt64 {
    let values = try url.resourceValues(forKeys: [.fileSizeKey, .isRegularFileKey, .isSymbolicLinkKey])
    guard values.isRegularFile == true, values.isSymbolicLink != true, let size = values.fileSize else {
        throw ControllerFailure.invalidIdentity
    }
    return UInt64(size)
}

private func validJobID(_ value: String) -> Bool {
    guard !value.isEmpty, value.count <= 32 else { return false }
    return value.unicodeScalars.allSatisfy {
        CharacterSet(charactersIn: "abcdefghijklmnopqrstuvwxyz0123456789-").contains($0)
    }
}

private func makeInput() -> Data {
    var data = Data("IMPRESARI_VM_INPUT_V1\nsynthetic-only\n".utf8)
    data.append(Data(repeating: 0, count: inputBytes - data.count))
    return data
}

private func createScratch(at url: URL) throws {
    guard FileManager.default.createFile(atPath: url.path, contents: nil) else {
        throw ControllerFailure.invalidJob
    }
    let handle = try FileHandle(forWritingTo: url)
    try handle.truncate(atOffset: scratchBytes)
    try handle.synchronize()
    try handle.close()
}

private func guestReceipt(from serialData: Data) throws -> GuestReceipt {
    guard serialData.count <= maximumSerialBytes,
          let text = String(data: serialData, encoding: .utf8),
          let line = text.split(separator: "\n").last(where: { $0.hasPrefix("IMPRESARI_VM_RECEIPT ") })
    else {
        throw ControllerFailure.guest
    }
    let json = line.dropFirst("IMPRESARI_VM_RECEIPT ".count)
    let receipt = try JSONDecoder().decode(GuestReceipt.self, from: Data(json.utf8))
    guard receipt.schemaName == "macos-local-vm-guest-receipt",
          receipt.schemaVersion == "1.0.0",
          receipt.result == "passed",
          receipt.exactInputVerified,
          receipt.readOnlyInputVerified,
          receipt.scratchInitiallyClean,
          receipt.scratchCapacityVerified,
          receipt.networkDeviceAbsent,
          !receipt.sourceRetained,
          !receipt.authorityAdded
    else {
        throw ControllerFailure.guest
    }
    return receipt
}

private func printFailure(_ failure: ControllerFailure) {
    let category: String
    switch failure {
    case .usage: category = "usage"
    case .unsupported: category = "unsupported"
    case .invalidIdentity: category = "invalid_identity"
    case .invalidJob: category = "invalid_job"
    case .invalidConfiguration: category = "invalid_configuration"
    case .start: category = "start_failed"
    case .timeout: category = "timeout"
    case .cancelled: category = "cancelled"
    case .outputLimit: category = "output_limit"
    case .guest: category = "guest_failed"
    case .cleanup: category = "cleanup_failed"
    }
    print("{\"schema_name\":\"macos-local-vm-matrix-failure\",\"schema_version\":\"1.0.0\",\"profile_id\":\"\(profileID)\",\"profile_digest\":\"\(profileDigest)\",\"category\":\"\(category)\",\"vm_confined\":false,\"production_admitted\":false,\"analyzer_execution\":false,\"source_retained\":false,\"authority_added\":false}")
}

private func printDiagnostic(_ error: Error) {
    let value = error as NSError
    let description = value.localizedDescription.replacingOccurrences(of: "\n", with: " ")
    let line = "macOS VM failure domain=\(value.domain) code=\(value.code) description=\(description)\n"
    FileHandle.standardError.write(Data(line.utf8))
}

@available(macOS 13.0, *)
private func run() throws -> ControllerReceipt {
    guard CommandLine.arguments.count == 4 else { throw ControllerFailure.usage }
    guard VZVirtualMachine.isSupported else { throw ControllerFailure.unsupported }

    let assetRoot = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true).standardizedFileURL
    let jobID = CommandLine.arguments[2]
    guard validJobID(jobID), let scenario = Scenario(rawValue: CommandLine.arguments[3])
    else {
        throw ControllerFailure.usage
    }

    let kernelURL = assetRoot.appendingPathComponent("Image", isDirectory: false)
    let initramfsURL = assetRoot.appendingPathComponent("impresari-initramfs.gz", isDirectory: false)
    guard try exactFileSize(kernelURL) == kernelBytes,
          try digest(kernelURL) == kernelDigest,
          try exactFileSize(initramfsURL) <= maximumInitramfsBytes,
          try digest(initramfsURL) == initramfsDigest
    else {
        throw ControllerFailure.invalidIdentity
    }

    let jobsRoot = assetRoot.deletingLastPathComponent().appendingPathComponent("jobs", isDirectory: true)
    try FileManager.default.createDirectory(at: jobsRoot, withIntermediateDirectories: true,
                                            attributes: [.posixPermissions: 0o700])
    let jobRoot = jobsRoot.appendingPathComponent(jobID, isDirectory: true)
    guard !FileManager.default.fileExists(atPath: jobRoot.path) else {
        throw ControllerFailure.invalidJob
    }
    try FileManager.default.createDirectory(at: jobRoot, withIntermediateDirectories: false,
                                            attributes: [.posixPermissions: 0o700])
    var cleanupNeeded = true
    defer {
        if cleanupNeeded {
            try? FileManager.default.removeItem(at: jobRoot)
        }
    }

    let inputURL = jobRoot.appendingPathComponent("input.raw")
    let scratchURL = jobRoot.appendingPathComponent("scratch.raw")
    let controllerReadyURL = jobRoot.appendingPathComponent("controller.ready")
    let input = makeInput()
    try input.write(to: inputURL, options: [.atomic])
    try createScratch(at: scratchURL)
    let inputDigest = try digest(inputURL)

    let bootLoader = VZLinuxBootLoader(kernelURL: kernelURL)
    bootLoader.initialRamdiskURL = initramfsURL
    bootLoader.commandLine = "console=hvc0 rdinit=/init panic=-1 quiet impresari.mode=\(scenario.guestMode)"

    let serialCapture = BoundedSerialCapture()
    serialCapture.start()
    let serialAttachment = VZFileHandleSerialPortAttachment(fileHandleForReading: nil,
                                                             fileHandleForWriting: serialCapture.writer)
    let serialPort = VZVirtioConsoleDeviceSerialPortConfiguration()
    serialPort.attachment = serialAttachment

    let inputAttachment = try VZDiskImageStorageDeviceAttachment(url: inputURL, readOnly: true)
    let scratchAttachment = try VZDiskImageStorageDeviceAttachment(url: scratchURL, readOnly: false)
    let inputDevice = VZVirtioBlockDeviceConfiguration(attachment: inputAttachment)
    let scratchDevice = VZVirtioBlockDeviceConfiguration(attachment: scratchAttachment)

    let configuration = VZVirtualMachineConfiguration()
    configuration.platform = VZGenericPlatformConfiguration()
    configuration.bootLoader = bootLoader
    configuration.cpuCount = 1
    configuration.memorySize = 268_435_456
    configuration.serialPorts = [serialPort]
    configuration.storageDevices = [inputDevice, scratchDevice]
    configuration.networkDevices = []
    configuration.directorySharingDevices = []
    configuration.graphicsDevices = []
    configuration.audioDevices = []
    configuration.keyboards = []
    configuration.pointingDevices = []
    do {
        try configuration.validate()
    } catch {
        throw ControllerFailure.invalidConfiguration
    }

    let queue = DispatchQueue(label: "studio.boldthaus.impresari-context.vm-feasibility")
    let machine = VZVirtualMachine(configuration: configuration, queue: queue)
    let delegate = VirtualMachineDelegate()
    machine.delegate = delegate
    let cancellation = CancellationState()
    let started = DispatchSemaphore(value: 0)
    var startFailure: Error?
    queue.async {
        machine.start { result in
            if case let .failure(error) = result { startFailure = error }
            started.signal()
        }
    }
    guard started.wait(timeout: .now() + .seconds(10)) == .success else {
        throw ControllerFailure.start
    }
    if let startFailure {
        printDiagnostic(startFailure)
        throw ControllerFailure.start
    }
    let cancellationQueue = DispatchQueue(label: "studio.boldthaus.impresari-context.vm-feasibility.cancel")
    let cancellationTimer = DispatchSource.makeTimerSource(queue: cancellationQueue)
    let cancellationDeadline: DispatchTime = scenario == .cancellation ? .now() + .milliseconds(250) : .distantFuture
    cancellationTimer.schedule(deadline: cancellationDeadline)
    cancellationTimer.setEventHandler {
        if scenario == .cancellation && !cancellation.isCancelled() {
            cancellation.cancel()
            queue.async {
                machine.stop { _ in delegate.stopped.signal() }
            }
        }
    }
    cancellationTimer.resume()
    defer { cancellationTimer.cancel() }
    guard FileManager.default.createFile(atPath: controllerReadyURL.path, contents: Data()) else {
        throw ControllerFailure.invalidJob
    }
    if delegate.stopped.wait(timeout: .now() + scenario.deadline) != .success {
        let stopped = DispatchSemaphore(value: 0)
        queue.async {
            machine.stop { _ in stopped.signal() }
        }
        _ = stopped.wait(timeout: .now() + .seconds(5))
        throw ControllerFailure.timeout
    }
    if cancellation.isCancelled() { throw ControllerFailure.cancelled }
    if delegate.error != nil { throw ControllerFailure.guest }

    let (serialData, serialOverflow) = serialCapture.closeAndCollect()
    if serialOverflow { throw ControllerFailure.outputLimit }
    let guest = try guestReceipt(from: serialData)
    guard try exactFileSize(inputURL) == UInt64(inputBytes),
          try digest(inputURL) == inputDigest,
          try exactFileSize(scratchURL) == scratchBytes
    else {
        throw ControllerFailure.guest
    }

    do {
        try FileManager.default.removeItem(at: jobRoot)
    } catch {
        throw ControllerFailure.cleanup
    }
    guard !FileManager.default.fileExists(atPath: jobRoot.path) else {
        throw ControllerFailure.cleanup
    }
    cleanupNeeded = false

    return ControllerReceipt(
        profileID: profileID,
        profileDigest: profileDigest,
        jobID: jobID,
        result: "feasibility_passed",
        kernelDigest: "sha256:\(kernelDigest)",
        initramfsDigest: "sha256:\(initramfsDigest)",
        inputDigest: "sha256:\(inputDigest)",
        virtualizationSupported: true,
        configurationValidated: true,
        cpuCount: "1",
        memoryBytes: "268435456",
        serialPorts: "1",
        storageDevices: "2",
        networkDevices: "0",
        directoryShares: "0",
        graphicsDevices: "0",
        audioDevices: "0",
        inputDevices: "0",
        exactInputVerified: guest.exactInputVerified,
        readOnlyInputVerified: guest.readOnlyInputVerified,
        scratchInitiallyClean: guest.scratchInitiallyClean,
        scratchCapacityVerified: guest.scratchCapacityVerified,
        networkDeviceAbsent: guest.networkDeviceAbsent,
        jobRemoved: true,
        vmConfined: false,
        productionAdmitted: false,
        sourceRetained: false,
        authorityAdded: false
    )
}

do {
    guard #available(macOS 13.0, *) else { throw ControllerFailure.unsupported }
    let receipt = try run()
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys]
    let data = try encoder.encode(receipt)
    guard let text = String(data: data, encoding: .utf8) else { throw ControllerFailure.guest }
    print(text)
} catch let failure as ControllerFailure {
    printFailure(failure)
    exit(1)
} catch {
    printFailure(.guest)
    exit(1)
}
