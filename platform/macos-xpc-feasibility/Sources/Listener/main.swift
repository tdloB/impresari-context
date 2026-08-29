// SPDX-License-Identifier: Apache-2.0
import Foundation
import Darwin

@main
struct SyntheticLoopbackListener {
    static func main() {
        guard CommandLine.arguments.count == 3 else {
            exit(2)
        }
        let portPath = CommandLine.arguments[1]
        let resultPath = CommandLine.arguments[2]
        let descriptor = socket(AF_INET, SOCK_STREAM, 0)
        guard descriptor >= 0 else {
            exit(3)
        }
        defer { close(descriptor) }

        var address = sockaddr_in()
        address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
        address.sin_family = sa_family_t(AF_INET)
        address.sin_port = 0
        address.sin_addr = in_addr(s_addr: inet_addr("127.0.0.1"))
        let bound = withUnsafePointer(to: &address) { pointer in
            pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) { socketAddress in
                bind(
                    descriptor,
                    socketAddress,
                    socklen_t(MemoryLayout<sockaddr_in>.size)
                )
            }
        }
        guard bound == 0, listen(descriptor, 1) == 0 else {
            exit(4)
        }

        var actual = sockaddr_in()
        var length = socklen_t(MemoryLayout<sockaddr_in>.size)
        let named = withUnsafeMutablePointer(to: &actual) { pointer in
            pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) { socketAddress in
                getsockname(descriptor, socketAddress, &length)
            }
        }
        guard named == 0 else {
            exit(5)
        }
        let port = UInt16(bigEndian: actual.sin_port)
        do {
            try String(port).write(
                toFile: portPath,
                atomically: true,
                encoding: .utf8
            )
        } catch {
            exit(6)
        }

        var pollDescriptor = pollfd(fd: descriptor, events: Int16(POLLIN), revents: 0)
        let pollResult = poll(&pollDescriptor, 1, 2_000)
        let outcome = pollResult == 0 ? "connection_denied" : "connection_observed"
        do {
            try outcome.write(
                toFile: resultPath,
                atomically: true,
                encoding: .utf8
            )
        } catch {
            exit(7)
        }
        exit(pollResult == 0 ? 0 : 8)
    }
}
