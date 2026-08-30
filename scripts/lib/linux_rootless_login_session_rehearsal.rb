# frozen_string_literal: true
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"

module LinuxRootlessLoginSessionRehearsal
  SESSION_KEYS = %w[
    schema_name schema_version expected_source_commit host ordinal login_kind
    session_identity user_manager_invocation_identity lingering_enabled
    preflight synthetic_rehearsal package ended_cleanly user_manager_terminated
  ].freeze
  CLEANUP_KEYS = %w[
    temporary_user_absent home_absent isolated_ssh_process_absent
    system_ssh_service_unchanged lingering_absent user_manager_absent
    product_service_absent authorization_policy_absent delegated_cgroups_empty
    staged_source_absent
  ].freeze
  SAFE_NEXT_STEPS = {
    "login_session_candidate" => "Retain this exact-host rootless reentry candidate and compose it only with the remaining exact lifecycle and release gates; production and real analyzers remain closed.",
    "unsupported" => "Report profile A unsupported on this host; do not enable lingering, install a service, or substitute a process restart.",
    "session_failed" => "Withdraw rootless reentry evidence and reproduce both genuine PAM/logind sessions on a fresh supported ephemeral host.",
    "identity_mismatch" => "Reject the composition and reproduce the package and both sessions from one exact source on one fresh ephemeral host.",
    "cleanup_failed" => "Withdraw rootless reentry evidence until the temporary user, manager, login transport, cgroups, and staged state are independently absent.",
  }.freeze
  AUTHORITY = {
    "ephemeral_host_administrator_setup" => "temporary_user_and_isolated_loopback_ssh_only",
    "session_user_privilege" => "denied",
    "workspace_source_read" => "denied",
    "network" => "loopback_login_only",
    "credential_access" => "ephemeral_test_key_only",
    "persistent_service" => "denied",
    "analyzer_execution" => "denied",
    "product_authority_added" => false,
  }.freeze

  class ContractError < StandardError; end

  module_function

  def exact_keys!(value, expected, label)
    raise ContractError, "#{label} must be an object" unless value.is_a?(Hash)
    actual = value.keys.sort
    wanted = expected.sort
    raise ContractError, "#{label} keys drifted" unless actual == wanted
  end

  def digest?(value)
    value.is_a?(String) && value.match?(/\A[0-9a-f]{64}\z/)
  end

  def source_commit?(value)
    value.is_a?(String) && value.match?(/\A[0-9a-f]{40}\z/)
  end

  def validate_package!(package, expected_source)
    exact_keys!(package, %w[schema_name schema_version policy_id profile host baseline candidate package_scope phases reentry_evidence excluded_lifecycle_evidence clean_state status safe_next_step claims], "package receipt")
    raise ContractError, "package receipt is not the rootless partial lifecycle" unless
      package["schema_name"] == "linux-isolation-package-lifecycle-rehearsal" &&
      package["schema_version"] == "1.0.0" &&
      package["profile"] == "rootless_user_manager" &&
      package["status"] == "package_lifecycle_partial" &&
      package["reentry_evidence"] == "real_login_session_required"
    raise ContractError, "package source identity mismatch" unless package.dig("candidate", "source_commit") == expected_source
    claims = package["claims"]
    raise ContractError, "package receipt overclaimed" unless claims.is_a?(Hash) && claims.values.none?
    candidate = package.fetch("candidate")
    exact_keys!(candidate, %w[archive_sha256 manifest_sha256 project_version source_commit], "candidate package identity")
    %w[archive_sha256 manifest_sha256].each do |key|
      raise ContractError, "package #{key} is malformed" unless digest?(candidate[key])
    end
  end

  def validate_session!(session, expected_source, ordinal)
    exact_keys!(session, SESSION_KEYS, "session observation")
    exact_keys!(session["host"], %w[operating_system kernel_release architecture environment], "session host")
    exact_keys!(session["preflight"], %w[status receipt_identity], "session preflight")
    exact_keys!(session["synthetic_rehearsal"], %w[status receipt_identity real_analyzer_used], "session synthetic rehearsal")
    exact_keys!(session["package"], %w[candidate_archive_sha256 candidate_manifest_sha256 source_commit binary_set_identity], "session package")
    raise ContractError, "session observation contract drifted" unless
      session["schema_name"] == "linux-rootless-login-session-observation" &&
      session["schema_version"] == "1.0.0" &&
      session["expected_source_commit"] == expected_source &&
      session["ordinal"] == ordinal &&
      session["login_kind"] == "pam_logind" &&
      session["lingering_enabled"] == false &&
      [true, false].include?(session["ended_cleanly"]) &&
      [true, false].include?(session["user_manager_terminated"])
    raise ContractError, "raw or malformed session identity" unless digest?(session["session_identity"])
    raise ContractError, "raw or malformed user-manager identity" unless digest?(session["user_manager_invocation_identity"])
    raise ContractError, "session package source mismatch" unless session.dig("package", "source_commit") == expected_source
    %w[candidate_archive_sha256 candidate_manifest_sha256 binary_set_identity].each do |key|
      raise ContractError, "session package identity malformed" unless digest?(session.dig("package", key))
    end
  end

  def validate_cleanup!(cleanup)
    exact_keys!(cleanup, CLEANUP_KEYS, "cleanup observation")
    raise ContractError, "cleanup observation contains a non-boolean" unless cleanup.values.all? { |value| value == true || value == false }
  end

  def session_ready?(session)
    session.dig("preflight", "status") == "ready_for_synthetic_rehearsal" &&
      digest?(session.dig("preflight", "receipt_identity")) &&
      session.dig("synthetic_rehearsal", "status") == "candidate_passed" &&
      digest?(session.dig("synthetic_rehearsal", "receipt_identity")) &&
      session.dig("synthetic_rehearsal", "real_analyzer_used") == false &&
      session["ended_cleanly"] == true
  end

  def host_supported?(host)
    host == {
      "operating_system" => "linux",
      "kernel_release" => host["kernel_release"],
      "architecture" => host["architecture"],
      "environment" => "github-hosted-ephemeral",
    } && host["kernel_release"].is_a?(String) && !host["kernel_release"].empty? &&
      %w[x86_64 aarch64 other].include?(host["architecture"])
  end

  def build(expected_source:, package_bytes:, package:, first:, second:, cleanup:)
    raise ContractError, "expected source commit is malformed" unless source_commit?(expected_source)
    validate_package!(package, expected_source)
    validate_session!(first, expected_source, 1)
    validate_session!(second, expected_source, 2)
    validate_cleanup!(cleanup)

    same_host = first["host"] == second["host"]
    package_identity = {
      "candidate_archive_sha256" => package.dig("candidate", "archive_sha256"),
      "candidate_manifest_sha256" => package.dig("candidate", "manifest_sha256"),
      "source_commit" => package.dig("candidate", "source_commit"),
    }
    session_packages = [first, second].map do |session|
      session.fetch("package").slice("candidate_archive_sha256", "candidate_manifest_sha256", "source_commit")
    end
    same_package = session_packages.all? { |identity| identity == package_identity }
    same_binary_set = first.dig("package", "binary_set_identity") == second.dig("package", "binary_set_identity")
    distinct_session = first["session_identity"] != second["session_identity"]
    distinct_manager = first["user_manager_invocation_identity"] != second["user_manager_invocation_identity"]
    transition = {
      "first_session_closed" => first["ended_cleanly"] == true,
      "first_user_manager_terminated" => first["user_manager_terminated"] == true,
      "distinct_session_identity" => distinct_session,
      "distinct_user_manager_identity" => distinct_manager,
      "same_package_identity" => same_package && same_binary_set,
      "process_restart_substitution_used" => false,
    }

    if !same_host || !host_supported?(first["host"])
      status = "unsupported"
      reason = "host_or_session_transport_unsupported"
    elsif !session_ready?(first) || !session_ready?(second)
      status = "session_failed"
      reason = "pam_logind_session_or_synthetic_rehearsal_failed"
    elsif !transition.values_at("first_session_closed", "first_user_manager_terminated", "distinct_session_identity", "distinct_user_manager_identity", "same_package_identity").all?
      status = "identity_mismatch"
      reason = "session_manager_or_package_identity_mismatch"
    elsif !cleanup.values.all?
      status = "cleanup_failed"
      reason = "ephemeral_host_cleanup_incomplete"
    else
      status = "login_session_candidate"
      reason = "genuine_rootless_login_reentry_verified"
    end

    {
      "schema_name" => "linux-rootless-login-session-rehearsal",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-lifecycle-v1",
      "expected_source_commit" => expected_source,
      "host" => first["host"],
      "package" => {
        "lifecycle_receipt_identity" => Digest::SHA256.hexdigest(package_bytes),
        "candidate_archive_sha256" => package.dig("candidate", "archive_sha256"),
        "candidate_manifest_sha256" => package.dig("candidate", "manifest_sha256"),
        "project_version" => package.dig("candidate", "project_version"),
        "source_commit" => package.dig("candidate", "source_commit"),
        "binary_set_identity" => first.dig("package", "binary_set_identity"),
        "identity_preserved_across_sessions" => same_package && same_binary_set,
      },
      "temporary_user" => {"non_privileged" => true, "lingering_enabled" => false, "sudo_available" => false, "persistent" => false, "name_recorded" => false},
      "login_transport" => {"kind" => "isolated_loopback_ssh_pam_logind", "network_scope" => "loopback_only", "system_ssh_service_modified" => false, "persistent_service_created" => false},
      "sessions" => [first, second].map do |session|
        session.slice("ordinal", "login_kind", "session_identity", "user_manager_invocation_identity", "lingering_enabled", "preflight", "synthetic_rehearsal", "ended_cleanly").merge(
          "package_binary_set_identity" => session.dig("package", "binary_set_identity")
        )
      end,
      "transition" => transition,
      "cleanup" => cleanup,
      "status" => status,
      "reason_code" => reason,
      "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
      "rootless_reentry_candidate_active" => status == "login_session_candidate",
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "persistent_service_authorized" => false,
      "authority" => AUTHORITY,
    }
  end
end
