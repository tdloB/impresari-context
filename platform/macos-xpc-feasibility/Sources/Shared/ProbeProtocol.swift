// SPDX-License-Identifier: Apache-2.0
import Foundation

@objc protocol ImpresariSandboxProbeProtocol {
    func prepareSyntheticProbe(
        _ request: Data,
        withReply reply: @escaping (Data) -> Void
    )

    func runSyntheticProbe(
        _ request: Data,
        withReply reply: @escaping (Data) -> Void
    )
}

enum ProbeContract {
    static let serviceName = "studio.boldthaus.impresari-context.SandboxProbe"
    static let maximumRequestBytes = 16_384
    static let maximumReceiptBytes = 16_384
    static let canaryCategories = [
        "cache", "credential", "home", "repository"
    ]
}

enum ProbeMode: String, Codable {
    case baseline
    case cpuLimit = "cpu_limit"
    case memoryLimit = "memory_limit"
    case descendantLimit = "descendant_limit"
    case productionProfile = "production_profile"
    case supervisorTimeout = "supervisor_timeout"
}

struct ProbeCanary: Codable, Equatable {
    let category: String
    let path: String
}

struct ProbeRequest: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let probeMode: ProbeMode
    let syntheticPayload: Data
    let canaries: [ProbeCanary]
    let syntheticDevicePath: String
    let unrelatedProcessID: Int32
    let loopbackPort: UInt16
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case probeMode = "probe_mode"
        case syntheticPayload = "synthetic_payload"
        case canaries
        case syntheticDevicePath = "synthetic_device_path"
        case unrelatedProcessID = "unrelated_process_id"
        case loopbackPort = "loopback_port"
        case authorityAdded = "authority_added"
    }
}

struct ProbePreparationReceipt: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let probeMode: ProbeMode
    let serviceProcessID: Int32
    let requestedLimitApplied: Bool
    let limitErrorNumber: Int32
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case probeMode = "probe_mode"
        case serviceProcessID = "service_process_id"
        case requestedLimitApplied = "requested_limit_applied"
        case limitErrorNumber = "limit_error_number"
        case authorityAdded = "authority_added"
    }
}

struct ProbeReceipt: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let probeMode: ProbeMode
    let serviceProcessID: Int32
    let requestAccepted: Bool
    let appContainerReadWriteVerified: Bool
    let canaryDenials: [String: Bool]
    let deviceAccessDenied: Bool
    let unrelatedProcessAccessDenied: Bool
    let networkDenialVerified: Bool
    let resourceLimitsVerified: Bool
    let descendantContainmentVerified: Bool
    let osConfined: Bool
    let productionAdmitted: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case probeMode = "probe_mode"
        case serviceProcessID = "service_process_id"
        case requestAccepted = "request_accepted"
        case appContainerReadWriteVerified = "app_container_read_write_verified"
        case canaryDenials = "canary_denials"
        case deviceAccessDenied = "device_access_denied"
        case unrelatedProcessAccessDenied = "unrelated_process_access_denied"
        case networkDenialVerified = "network_denial_verified"
        case resourceLimitsVerified = "resource_limits_verified"
        case descendantContainmentVerified = "descendant_containment_verified"
        case osConfined = "os_confined"
        case productionAdmitted = "production_admitted"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

struct ProbeResourceReceipt: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let probeMode: ProbeMode
    let serviceProcessID: Int32
    let memoryAllocationDenied: Bool
    let forkDenied: Bool
    let spawnDenied: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case probeMode = "probe_mode"
        case serviceProcessID = "service_process_id"
        case memoryAllocationDenied = "memory_allocation_denied"
        case forkDenied = "fork_denied"
        case spawnDenied = "spawn_denied"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

struct ProbeProductionProfileReceipt: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let probeMode: ProbeMode
    let serviceProcessID: Int32
    let profileID: String
    let profileDigest: String
    let cpuSeconds: UInt64
    let addressSpaceGrowthBytes: UInt64
    let processDescendants: UInt64
    let fileDescriptors: UInt64
    let temporaryFileBytes: UInt64
    let effectiveProfileVerified: Bool
    let osConfined: Bool
    let productionAdmitted: Bool
    let sourceRetained: Bool
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case probeMode = "probe_mode"
        case serviceProcessID = "service_process_id"
        case profileID = "profile_id"
        case profileDigest = "profile_digest"
        case cpuSeconds = "cpu_seconds"
        case addressSpaceGrowthBytes = "address_space_growth_bytes"
        case processDescendants = "process_descendants"
        case fileDescriptors = "file_descriptors"
        case temporaryFileBytes = "temporary_file_bytes"
        case effectiveProfileVerified = "effective_profile_verified"
        case osConfined = "os_confined"
        case productionAdmitted = "production_admitted"
        case sourceRetained = "source_retained"
        case authorityAdded = "authority_added"
    }
}

func canonicalJSON<T: Encodable>(_ value: T) -> Data? {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
    return try? encoder.encode(value)
}
