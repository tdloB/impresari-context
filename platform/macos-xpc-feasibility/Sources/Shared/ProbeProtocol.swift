// SPDX-License-Identifier: Apache-2.0
import Foundation

@objc protocol ImpresariSandboxProbeProtocol {
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

struct ProbeCanary: Codable, Equatable {
    let category: String
    let path: String
}

struct ProbeRequest: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
    let canaries: [ProbeCanary]
    let unrelatedProcessID: Int32
    let loopbackPort: UInt16
    let authorityAdded: Bool

    enum CodingKeys: String, CodingKey {
        case schemaName = "schema_name"
        case schemaVersion = "schema_version"
        case canaries
        case unrelatedProcessID = "unrelated_process_id"
        case loopbackPort = "loopback_port"
        case authorityAdded = "authority_added"
    }
}

struct ProbeReceipt: Codable, Equatable {
    let schemaName: String
    let schemaVersion: String
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

func canonicalJSON<T: Encodable>(_ value: T) -> Data? {
    let encoder = JSONEncoder()
    encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
    return try? encoder.encode(value)
}
