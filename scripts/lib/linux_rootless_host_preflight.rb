# frozen_string_literal: true
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"

module LinuxRootlessHostPreflight
  REQUIRED_CONTROLLERS = %w[cpu memory pids].freeze
  AUTHORITY = {
    "policy_read" => "fixed_bundled_policy",
    "host_metadata_read" => "fixed_linux_platform_files",
    "workspace_source_read" => "denied",
    "source_write" => "denied",
    "process_execution" => "denied",
    "network" => "denied",
    "credential_access" => "denied",
    "privilege_use" => "denied",
    "service_mutation" => "denied",
    "background_monitoring" => "denied",
    "analyzer_execution" => "denied",
  }.freeze
  SAFE_NEXT_STEPS = {
    "ready_for_synthetic_rehearsal" => "Run only the source-free transient-user-unit synthetic rehearsal; do not launch a real analyzer or admit production.",
    "unavailable" => "Report rootless preflight unavailable without starting a user manager, requesting privilege, or installing a service.",
    "unsupported" => "Report the host unsupported for this profile; do not fall back to application-only confinement while claiming IAR-1B.",
    "insufficient_delegation" => "Report the missing delegation prerequisite and keep synthetic execution, production, and real analyzers closed.",
    "invalid_host_state" => "Reject the malformed host observation and perform no repair, service change, or privileged fallback.",
  }.freeze

  module_function

  def receipt(policy_identity, observed, status, reason_code, checks)
    {
      "schema_name" => "linux-rootless-host-preflight",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-topology-v1",
      "policy_identity" => policy_identity,
      "profile" => "rootless_user_manager",
      "status" => status,
      "reason_code" => reason_code,
      "observed" => observed,
      "checks" => checks,
      "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
      "preflight_candidate_active" => status == "ready_for_synthetic_rehearsal",
      "synthetic_child_cycle_executed" => false,
      "os_confined" => false,
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "authority" => AUTHORITY,
    }
  end

  def assess(policy_identity, observed)
    checks = %w[policy_identity_bound authority_closed raw_cgroup_path_suppressed]
    return receipt(policy_identity, observed, "unsupported", "non_linux_platform", checks + ["non_linux_platform"]) unless observed.fetch("platform") == "linux"
    return receipt(policy_identity, observed, "unavailable", "cgroup_interface_unavailable", checks + ["cgroup_interface_unavailable"]) if observed.fetch("cgroup_mode") == "unavailable"
    return receipt(policy_identity, observed, "unsupported", "unified_cgroup_v2_required", checks + ["legacy_or_hybrid_rejected"]) unless observed.fetch("cgroup_mode") == "unified_v2"
    checks << "unified_cgroup_v2"

    unless observed.fetch("current_membership_valid")
      return receipt(policy_identity, observed, "invalid_host_state", "current_cgroup_membership_invalid", checks + ["current_cgroup_membership_invalid"])
    end
    checks << "current_cgroup_membership_valid"

    unless observed.fetch("user_manager_cgroup_present")
      return receipt(policy_identity, observed, "unavailable", "user_manager_cgroup_unavailable", checks + ["user_manager_cgroup_unavailable"])
    end
    unless observed.fetch("user_manager_transport_present")
      return receipt(policy_identity, observed, "unavailable", "user_manager_transport_unavailable", checks + ["user_manager_transport_unavailable"])
    end
    unless observed.fetch("user_manager_process_present")
      return receipt(policy_identity, observed, "unavailable", "user_manager_process_unavailable", checks + ["user_manager_process_unavailable"])
    end
    checks << "existing_user_manager_observed"

    missing = REQUIRED_CONTROLLERS - observed.fetch("controllers")
    unless missing.empty?
      return receipt(policy_identity, observed, "insufficient_delegation", "required_controller_missing", checks + missing.map { |controller| "#{controller}_controller_missing" })
    end
    checks << "required_controllers_available"

    unless observed.fetch("delegation_write_marker")
      return receipt(policy_identity, observed, "insufficient_delegation", "delegation_write_marker_missing", checks + ["delegation_write_marker_missing"])
    end
    checks << "delegation_write_marker_observed"

    receipt(policy_identity, observed, "ready_for_synthetic_rehearsal", "rootless_prerequisites_observed", checks + ["synthetic_child_cycle_pending"])
  end

  def policy_identity(path)
    bytes = File.binread(path, 65_536)
    policy = JSON.parse(bytes)
    unless policy["schema_name"] == "linux-isolation-production-topology-policy" &&
        policy["schema_version"] == "1.0.0" &&
        policy["policy_id"] == "linux-iar-1b-production-topology-v1" &&
        policy.dig("decision", "default_profile") == "rootless_user_manager" &&
        policy.dig("decision", "automatic_sudo") == "denied" &&
        policy.dig("decision", "privileged_daemon") == "denied"
      raise "unsupported Linux production-topology policy"
    end
    Digest::SHA256.hexdigest(bytes)
  rescue JSON::ParserError
    raise "invalid Linux production-topology policy"
  end

  def fixed_read(path, limit)
    return nil unless File.file?(path) && !File.symlink?(path)

    value = File.binread(path, limit + 1)
    return "" if value.nil?

    value.bytesize > limit ? nil : value
  rescue Errno::EACCES, Errno::ENOENT, Errno::ENOTDIR, Errno::EISDIR
    nil
  end

  def controller_list(value)
    return [] unless value

    value.split.select { |controller| REQUIRED_CONTROLLERS.include?(controller) }.uniq.sort
  end

  def membership_valid?(value)
    return false unless value && value.bytesize <= 4096 && !value.include?("\0")

    lines = value.lines(chomp: true)
    return false unless lines.length == 1 && lines.first.start_with?("0::/")

    segments = lines.first.delete_prefix("0::/").split("/", -1)
    segments.none? { |segment| segment == ".." || segment.include?("\0") }
  end

  def process_present?(service_root)
    [File.join(service_root, "cgroup.procs"), File.join(service_root, "init.scope", "cgroup.procs")].any? do |path|
      value = fixed_read(path, 65_536)
      value && value.lines.any? { |line| line.strip.match?(/\A[1-9][0-9]*\z/) }
    end
  end

  def live_observation
    unless RUBY_PLATFORM.include?("linux")
      return base_observation.merge(
        "platform" => "other",
        "kernel_release" => "not_observed",
        "cgroup_mode" => "unavailable",
      )
    end

    uid = Process.uid
    service_root = "/sys/fs/cgroup/user.slice/user-#{uid}.slice/user@#{uid}.service"
    root_controllers = fixed_read("/sys/fs/cgroup/cgroup.controllers", 4096)
    membership = fixed_read("/proc/self/cgroup", 4096)
    kernel_release = fixed_read("/proc/sys/kernel/osrelease", 129)&.strip
    kernel_release = "not_observed" unless kernel_release&.match?(/\A[A-Za-z0-9._+-]{1,128}\z/)
    manager_controllers = fixed_read(File.join(service_root, "cgroup.controllers"), 4096)
    enabled_controllers = fixed_read(File.join(service_root, "cgroup.subtree_control"), 4096)
    cgroup_mode = if root_controllers
      "unified_v2"
    elsif membership
      "legacy_or_hybrid"
    else
      "unavailable"
    end

    base_observation.merge(
      "platform" => "linux",
      "kernel_release" => kernel_release,
      "cgroup_mode" => cgroup_mode,
      "current_membership_valid" => membership_valid?(membership),
      "user_manager_cgroup_present" => File.directory?(service_root) && !File.symlink?(service_root),
      "user_manager_transport_present" => File.socket?("/run/user/#{uid}/systemd/private"),
      "user_manager_process_present" => process_present?(service_root),
      "delegation_write_marker" => File.writable?(File.join(service_root, "cgroup.subtree_control")),
      "controllers" => controller_list(manager_controllers),
      "enabled_controllers" => controller_list(enabled_controllers),
    )
  end

  def base_observation
    {
      "platform" => "other",
      "kernel_release" => "not_observed",
      "cgroup_mode" => "unavailable",
      "current_membership_valid" => false,
      "user_manager_cgroup_present" => false,
      "user_manager_transport_present" => false,
      "user_manager_process_present" => false,
      "delegation_write_marker" => false,
      "controllers" => [],
      "enabled_controllers" => [],
      "raw_cgroup_path_recorded" => false,
    }
  end
end
