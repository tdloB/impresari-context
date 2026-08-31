// SPDX-License-Identifier: Apache-2.0

import AppKit
import CryptoKit
import Foundation
import Virtualization

private let profileID = "iar-macos-local-vm-synthetic-matrix-v2"
private let profileDigest = "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888"
private let kernelDigest = "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5"
private let initramfsDigest = "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b"
private let resourceProfileID = "iar-macos-local-vm-resource-canary-v2"
private let resourceProfileDigest = "sha256:82d3cbf4b68866b92794a06e86948ccaf2492b3b4cb38e7e70503562c61d1de0"
private let resourceInitramfsDigest = "1a4029b781020260e4cb8c18271e3a01e1920f1448d87a71678e12cc617a1ec3"
private let kernelBytes: UInt64 = 36_175_872
private let inputBytes = 4_096
private let scratchBytes: UInt64 = 1_048_576
private let maximumInitramfsBytes: UInt64 = 2_097_152
private let maximumSerialBytes = 65_536
private let controllerTimeout: DispatchTimeInterval = .seconds(30)
private let hostCanaryMarkers = [
    "IMPRESARI_HOST_HOME_CANARY_V1",
    "IMPRESARI_HOST_REPOSITORY_CANARY_V1",
    "IMPRESARI_HOST_CACHE_CANARY_V1",
    "IMPRESARI_HOST_CREDENTIAL_CANARY_V1",
    "IMPRESARI_HOST_DEVICE_CANARY_V1",
    "IMPRESARI_HOST_PROCESS_CANARY_V1",
]
private var activeProfileID = profileID
private var activeProfileDigest = profileDigest

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
    case resourceCanary = "resource-canary"
    case hostInterruption = "host-interruption"

    var deadline: DispatchTimeInterval {
        switch self {
        case .timeout, .descendantTimeout: return .seconds(2)
        default: return controllerTimeout
        }
    }

    var guestMode: String {
        self == .cancellation || self == .hostInterruption ? Scenario.timeout.rawValue : rawValue
    }
}

private enum StopReason: String {
    case cancellation
    case syntheticHostInterruption = "synthetic-job-private-trigger"
    case operatingSystemSleep = "os-will-sleep"
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

private struct ResourceGuestReceipt: Decodable {
    let schemaName: String
    let schemaVersion: String
    let result: String
    let attachedDeviceSetExact: Bool
    let hostCanaryBytesAbsent: Bool
    let hostPathsAbsent: Bool
    let hostProcessInvisible: Bool
    let memoryPressureContained: Bool
    let memoryOOMKills: String
    let cpuPressureBounded: Bool
    let cpuUsageMicroseconds: String
    let cpuThrottledPeriods: String
    let pidsPeak: String
    let jobCgroupRemoved: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case result
        case attachedDeviceSetExact = "attached_device_set_exact"
        case hostCanaryBytesAbsent = "host_canary_bytes_absent"
        case hostPathsAbsent = "host_paths_absent"
        case hostProcessInvisible = "host_process_invisible"
        case memoryPressureContained = "memory_pressure_contained"
        case memoryOOMKills = "memory_oom_kills"
        case cpuPressureBounded = "cpu_pressure_bounded"
        case cpuUsageMicroseconds = "cpu_usage_usec"
        case cpuThrottledPeriods = "cpu_throttled_periods"
        case pidsPeak = "pids_peak"
        case jobCgroupRemoved = "job_cgroup_removed"
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

private struct ResourceControllerReceipt: Encodable {
    let schemaName = "macos-local-vm-resource-canary-receipt"
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
    let storageDevices: String
    let networkDevices: String
    let directoryShares: String
    let hostCanaryCorpusCreated: Bool
    let hostCanaryCorpusUnchanged: Bool
    let attachedDeviceSetExact: Bool
    let hostCanaryBytesAbsent: Bool
    let hostPathsAbsent: Bool
    let hostProcessInvisible: Bool
    let memoryPressureContained: Bool
    let memoryOOMKills: String
    let cpuPressureBounded: Bool
    let cpuUsageMicroseconds: String
    let cpuThrottledPeriods: String
    let pidsPeak: String
    let jobCgroupRemoved: Bool
    let jobRemoved: Bool
    let vmConfined: Bool
    let productionAdmitted: Bool
    let analyzerExecution: Bool
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
        case storageDevices = "storage_devices"
        case networkDevices = "network_devices"
        case directoryShares = "directory_shares"
        case hostCanaryCorpusCreated = "host_canary_corpus_created"
        case hostCanaryCorpusUnchanged = "host_canary_corpus_unchanged"
        case attachedDeviceSetExact = "attached_device_set_exact"
        case hostCanaryBytesAbsent = "host_canary_bytes_absent"
        case hostPathsAbsent = "host_paths_absent"
        case hostProcessInvisible = "host_process_invisible"
        case memoryPressureContained = "memory_pressure_contained"
        case memoryOOMKills = "memory_oom_kills"
        case cpuPressureBounded = "cpu_pressure_bounded"
        case cpuUsageMicroseconds = "cpu_usage_usec"
        case cpuThrottledPeriods = "cpu_throttled_periods"
        case pidsPeak = "pids_peak"
        case jobCgroupRemoved = "job_cgroup_removed"
        case jobRemoved = "job_removed"
        case vmConfined = "vm_confined"
        case productionAdmitted = "production_admitted"
        case analyzerExecution = "analyzer_execution"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

private struct InterruptionControllerReceipt: Encodable {
    let schemaName = "macos-local-vm-interruption-controller-receipt"
    let schemaVersion = "1.0.0"
    let profileID: String
    let profileDigest: String
    let jobID: String
    let result = "synthetic_interruption_handled"
    let interruptionSource: String
    let sleepObserverInstalled = true
    let sharedStopHandlerUsed = true
    let virtualizationSupported = true
    let configurationValidated = true
    let virtualMachineStopped = true
    let jobRemoved = true
    let realHostSleepObserved: Bool
    let vmConfined = false
    let productionAdmitted = false
    let analyzerExecution = false
    let sourceRetained = false
    let authorityAdded = false

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case profileID = "profile_id"
        case profileDigest = "profile_digest"
        case jobID = "job_id"
        case result
        case interruptionSource = "interruption_source"
        case sleepObserverInstalled = "sleep_observer_installed"
        case sharedStopHandlerUsed = "shared_stop_handler_used"
        case virtualizationSupported = "virtualization_supported"
        case configurationValidated = "configuration_validated"
        case virtualMachineStopped = "virtual_machine_stopped"
        case jobRemoved = "job_removed"
        case realHostSleepObserved = "real_host_sleep_observed"
        case vmConfined = "vm_confined"
        case productionAdmitted = "production_admitted"
        case analyzerExecution = "analyzer_execution"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

private enum ControllerOutput: Encodable {
    case matrix(ControllerReceipt)
    case resource(ResourceControllerReceipt)
    case interruption(InterruptionControllerReceipt)

    func encode(to encoder: Encoder) throws {
        switch self {
        case let .matrix(receipt): try receipt.encode(to: encoder)
        case let .resource(receipt): try receipt.encode(to: encoder)
        case let .interruption(receipt): try receipt.encode(to: encoder)
        }
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

private final class StopState {
    private let lock = NSLock()
    private var reason: StopReason?

    func request(_ requested: StopReason) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard reason == nil else { return false }
        reason = requested
        return true
    }

    func current() -> StopReason? {
        lock.lock()
        defer { lock.unlock() }
        return reason
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

private func resourceGuestReceipt(from serialData: Data) throws -> ResourceGuestReceipt {
    guard serialData.count <= maximumSerialBytes,
          let text = String(data: serialData, encoding: .utf8),
          let line = text.split(separator: "\n").last(where: { $0.hasPrefix("IMPRESARI_VM_RECEIPT ") })
    else {
        throw ControllerFailure.guest
    }
    for marker in hostCanaryMarkers where serialData.range(of: Data(marker.utf8)) != nil {
        throw ControllerFailure.guest
    }
    let json = line.dropFirst("IMPRESARI_VM_RECEIPT ".count)
    let receipt = try JSONDecoder().decode(ResourceGuestReceipt.self, from: Data(json.utf8))
    guard let oomKills = UInt64(receipt.memoryOOMKills), oomKills >= 1,
          let cpuUsage = UInt64(receipt.cpuUsageMicroseconds),
          (50_000 ... 400_000).contains(cpuUsage),
          let throttled = UInt64(receipt.cpuThrottledPeriods), throttled >= 1,
          let pidsPeak = UInt64(receipt.pidsPeak), pidsPeak <= 8,
          receipt.schemaName == "macos-local-vm-resource-canary-guest-receipt",
          receipt.schemaVersion == "1.0.0",
          receipt.result == "passed",
          receipt.attachedDeviceSetExact,
          receipt.hostCanaryBytesAbsent,
          receipt.hostPathsAbsent,
          receipt.hostProcessInvisible,
          receipt.memoryPressureContained,
          receipt.cpuPressureBounded,
          receipt.jobCgroupRemoved,
          !receipt.sourceRetained,
          !receipt.authorityAdded
    else {
        throw ControllerFailure.guest
    }
    return receipt
}

private func createHostCanaryCorpus(at root: URL) throws {
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: false,
                                            attributes: [.posixPermissions: 0o700])
    for (index, marker) in hostCanaryMarkers.enumerated() {
        let url = root.appendingPathComponent("canary-\(index).txt", isDirectory: false)
        try Data("\(marker)\n".utf8).write(to: url, options: [.atomic])
    }
}

private func hostCanaryCorpusUnchanged(at root: URL) -> Bool {
    for (index, marker) in hostCanaryMarkers.enumerated() {
        let url = root.appendingPathComponent("canary-\(index).txt", isDirectory: false)
        guard let bytes = try? Data(contentsOf: url), bytes == Data("\(marker)\n".utf8) else {
            return false
        }
    }
    let entries = try? FileManager.default.contentsOfDirectory(atPath: root.path)
    return entries?.count == hostCanaryMarkers.count
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
    let schemaName = activeProfileID == resourceProfileID
        ? "macos-local-vm-resource-canary-failure"
        : "macos-local-vm-matrix-failure"
    print("{\"schema_name\":\"\(schemaName)\",\"schema_version\":\"1.0.0\",\"profile_id\":\"\(activeProfileID)\",\"profile_digest\":\"\(activeProfileDigest)\",\"category\":\"\(category)\",\"vm_confined\":false,\"production_admitted\":false,\"analyzer_execution\":false,\"source_retained\":false,\"authority_added\":false}")
}

private func printDiagnostic(_ error: Error) {
    let value = error as NSError
    let description = value.localizedDescription.replacingOccurrences(of: "\n", with: " ")
    let line = "macOS VM failure domain=\(value.domain) code=\(value.code) description=\(description)\n"
    FileHandle.standardError.write(Data(line.utf8))
}

@available(macOS 13.0, *)
private func run() throws -> ControllerOutput {
    guard CommandLine.arguments.count == 4 else { throw ControllerFailure.usage }
    guard VZVirtualMachine.isSupported else { throw ControllerFailure.unsupported }

    let assetRoot = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true).standardizedFileURL
    let jobID = CommandLine.arguments[2]
    guard validJobID(jobID), let scenario = Scenario(rawValue: CommandLine.arguments[3])
    else {
        throw ControllerFailure.usage
    }

    let resourceScenario = scenario == .resourceCanary
    activeProfileID = resourceScenario ? resourceProfileID : profileID
    activeProfileDigest = resourceScenario ? resourceProfileDigest : profileDigest

    let kernelURL = assetRoot.appendingPathComponent("Image", isDirectory: false)
    let initramfsName = resourceScenario
        ? "impresari-resource-initramfs.gz"
        : "impresari-initramfs.gz"
    let expectedInitramfsDigest = resourceScenario ? resourceInitramfsDigest : initramfsDigest
    let initramfsURL = assetRoot.appendingPathComponent(initramfsName, isDirectory: false)
    guard try exactFileSize(kernelURL) == kernelBytes,
          try digest(kernelURL) == kernelDigest,
          try exactFileSize(initramfsURL) <= maximumInitramfsBytes,
          try digest(initramfsURL) == expectedInitramfsDigest
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
    let externalCancellationURL = jobRoot.appendingPathComponent("cancel.request")
    let hostInterruptionURL = jobRoot.appendingPathComponent("host-interruption.request")
    let hostCanaryRoot = jobRoot.appendingPathComponent("host-canaries", isDirectory: true)
    let input = makeInput()
    try input.write(to: inputURL, options: [.atomic])
    try createScratch(at: scratchURL)
    if resourceScenario {
        try createHostCanaryCorpus(at: hostCanaryRoot)
    }
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
    let stopState = StopState()
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
    let requestStop: (StopReason) -> Void = { reason in
        guard stopState.request(reason) else { return }
        queue.async {
            machine.stop { _ in delegate.stopped.signal() }
        }
    }
    let sleepNotificationQueue = OperationQueue()
    sleepNotificationQueue.maxConcurrentOperationCount = 1
    let sleepObserver = NSWorkspace.shared.notificationCenter.addObserver(
        forName: NSWorkspace.willSleepNotification,
        object: nil,
        queue: sleepNotificationQueue
    ) { _ in
        requestStop(.operatingSystemSleep)
    }
    defer {
        NSWorkspace.shared.notificationCenter.removeObserver(sleepObserver)
        sleepNotificationQueue.cancelAllOperations()
    }
    let cancellationQueue = DispatchQueue(label: "studio.boldthaus.impresari-context.vm-feasibility.cancel")
    let cancellationTimer = DispatchSource.makeTimerSource(queue: cancellationQueue)
    let cancellationStarted = DispatchTime.now().uptimeNanoseconds
    cancellationTimer.schedule(deadline: .now() + .milliseconds(25), repeating: .milliseconds(25))
    cancellationTimer.setEventHandler {
        let elapsed = DispatchTime.now().uptimeNanoseconds - cancellationStarted
        let internalCancellation = scenario == .cancellation && elapsed >= 250_000_000
        let externalCancellation = FileManager.default.fileExists(atPath: externalCancellationURL.path)
        if internalCancellation || externalCancellation {
            requestStop(.cancellation)
        }
        if FileManager.default.fileExists(atPath: hostInterruptionURL.path) {
            requestStop(.syntheticHostInterruption)
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
    if stopState.current() == .cancellation { throw ControllerFailure.cancelled }
    if let reason = stopState.current(), reason != .cancellation {
        _ = serialCapture.closeAndCollect()
        do {
            try FileManager.default.removeItem(at: jobRoot)
        } catch {
            throw ControllerFailure.cleanup
        }
        guard !FileManager.default.fileExists(atPath: jobRoot.path) else {
            throw ControllerFailure.cleanup
        }
        cleanupNeeded = false
        return .interruption(InterruptionControllerReceipt(
            profileID: profileID,
            profileDigest: profileDigest,
            jobID: jobID,
            interruptionSource: reason.rawValue,
            realHostSleepObserved: reason == .operatingSystemSleep
        ))
    }
    if delegate.error != nil { throw ControllerFailure.guest }

    let (serialData, serialOverflow) = serialCapture.closeAndCollect()
    if serialOverflow { throw ControllerFailure.outputLimit }
    let guest = resourceScenario ? nil : try guestReceipt(from: serialData)
    let resourceGuest = resourceScenario ? try resourceGuestReceipt(from: serialData) : nil
    let canariesUnchanged = resourceScenario && hostCanaryCorpusUnchanged(at: hostCanaryRoot)
    guard try exactFileSize(inputURL) == UInt64(inputBytes),
          try digest(inputURL) == inputDigest,
          try exactFileSize(scratchURL) == scratchBytes,
          !resourceScenario || canariesUnchanged
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

    if let resourceGuest {
        return .resource(ResourceControllerReceipt(
            profileID: resourceProfileID,
            profileDigest: resourceProfileDigest,
            jobID: jobID,
            result: "partial_resource_canary_passed",
            kernelDigest: "sha256:\(kernelDigest)",
            initramfsDigest: "sha256:\(resourceInitramfsDigest)",
            inputDigest: "sha256:\(inputDigest)",
            virtualizationSupported: true,
            configurationValidated: true,
            cpuCount: "1",
            memoryBytes: "268435456",
            storageDevices: "2",
            networkDevices: "0",
            directoryShares: "0",
            hostCanaryCorpusCreated: true,
            hostCanaryCorpusUnchanged: canariesUnchanged,
            attachedDeviceSetExact: resourceGuest.attachedDeviceSetExact,
            hostCanaryBytesAbsent: resourceGuest.hostCanaryBytesAbsent,
            hostPathsAbsent: resourceGuest.hostPathsAbsent,
            hostProcessInvisible: resourceGuest.hostProcessInvisible,
            memoryPressureContained: resourceGuest.memoryPressureContained,
            memoryOOMKills: resourceGuest.memoryOOMKills,
            cpuPressureBounded: resourceGuest.cpuPressureBounded,
            cpuUsageMicroseconds: resourceGuest.cpuUsageMicroseconds,
            cpuThrottledPeriods: resourceGuest.cpuThrottledPeriods,
            pidsPeak: resourceGuest.pidsPeak,
            jobCgroupRemoved: resourceGuest.jobCgroupRemoved,
            jobRemoved: true,
            vmConfined: false,
            productionAdmitted: false,
            analyzerExecution: false,
            sourceRetained: false,
            authorityAdded: false
        ))
    }
    guard let guest else { throw ControllerFailure.guest }
    return .matrix(ControllerReceipt(
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
    ))
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
