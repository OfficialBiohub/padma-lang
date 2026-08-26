# Quantum Provider Readiness Assessment v1

## Purpose and execution boundary

`quantum.provider_readiness(request)` is a **deterministic local assessment artifact** for recording the controls that must exist before a later provider adapter could be considered. It validates only a provider label, reviewed OpenQASM artifact fingerprint, and a short public policy note. It does not authenticate, read saved accounts, resolve an endpoint, select a backend, retrieve current cost/quota, submit a job, poll, cancel, store provenance, or communicate with a QPU.

> A successful Padma assessment does not mean that IBM Quantum, Amazon Braket, or another provider accepts the artifact, supports its gates, has an available backend, or will execute it.

This separation is deliberate. IBM’s non-local runtime service requires a token and can load account material from a configuration file; its job workflow has identifiers, usage tracking, and cancellation semantics. [1] [2] Amazon Braket task submission creates a quantum task, has asynchronous polling, and can write results to S3; its SDK also exposes task cancellation and metadata. [3] [4] Those are credentialed and potentially cost-relevant external actions, not local language inspection.

## API and strict request

| API | Result | Capability |
|---|---|---|
| `quantum.provider_readiness(request)` | Redacted deterministic controls/readiness map. | None |

```padma
ধরি request = {"provider": "aws-braket", "artifact": {"format": "openqasm-3.0-padma-renderer-subset", "sourceSha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "sourceBytes": 128}, "policyNote": "Manual review of current cost and cancellation controls"}
দেখাও quantum.provider_readiness(request)["reviewState"]
```

| Field | Rule |
|---|---|
| `provider` | Exactly `ibm-quantum`, `aws-braket`, or `other-reviewed`. Labels do not perform provider selection or compatibility validation. |
| `artifact` | Exactly `format`, lowercase `sourceSha256` in `sha256:<64-hex>` form, and whole `sourceBytes` from `1` through `1,048,576`. No source text, file path, URL, bucket, ARN, CRN, endpoint, or task/job identifier is accepted. |
| `policyNote` | Required single-line public text up to 256 bytes. It rejects raw markup, URLs, contact delimiters, and secret/account/job/endpoint/credential terms. It is validated but not returned. |

The only accepted artifact format is `openqasm-3.0-padma-renderer-subset`, the fingerprint form emitted by the M36 local assessment. This is a recorded review input, not a parser/import or provider compatibility operation.

## Deterministic result and required controls

The result returns artifact fingerprint/count, the selected label, a redacted `policyNote = "accepted-not-returned"`, and `reviewState = "assessment-only"`. It includes required-controls markers for: a dedicated capability design; credential reference without secret storage; fresh visible confirmation before every remote job; current cost/quota disclosure; job identifier/cancellation/provenance design; and bounded polling/result-retention policy. `other-reviewed` additionally requires provider-specific adapter security review.

| Fixed output marker | v1 value |
|---|---|
| `capability` | `not-defined` |
| `authentication`, `backendSelection`, `submission`, `polling`, `cancellation`, `providerSdk` | `disabled` |
| `credential`, `account` | `not-read` |
| `endpoint` | `not-configured` |
| `costQuota` | `not-queried` |
| `job`, `provenance` | `not-created` |
| `qpu`, `network`, `childProcess` | `disabled` |

## Safety and future adapter threshold

`P1090` rejects malformed/unknown request or artifact fields, unsupported provider labels, invalid fingerprint/byte bounds, policy notes that could contain secret/account/job/endpoint material, and any action-oriented field such as `credential`, `endpoint`, `source`, or `submitNow`. No raw policy note is returned in output or diagnostics.

Before any future remote adapter can be proposed, it must separately provide: an explicit capability grant, secret reference/handling policy, fresh user confirmation for every billable job, provider-specific current cost/quota presentation, user-visible destination/device selection, bounded polling, cancellation execution, redacted job provenance, retention/deletion policy, failure recovery, and provider-specific security regressions. This v1 assessment implements none of those actions.

## Explicit exclusions

No IBM/AWS SDK, REST/gRPC client, HTTP call, provider account/session, saved-account lookup, token/environment read, backend listing/selection, price/usage lookup, S3 access, task/job creation, task result retrieval, cancellation, poll loop, source upload, file I/O, or process runs. It does not claim QASM portability, hardware correctness, availability, performance, quantum advantage, or provider acceptance.

## References

[1] [IBM Quantum Compute: `QiskitRuntimeService`](https://quantum.cloud.ibm.com/docs/api/qiskit-ibm-runtime/qiskit-runtime-service)  
[2] [IBM Quantum: Monitor or cancel a job](https://quantum.cloud.ibm.com/docs/guides/monitor-job)  
[3] [Amazon Braket: Submit quantum tasks](https://docs.aws.amazon.com/braket/latest/developerguide/braket-submit-tasks-to-braket.html)  
[4] [Amazon Braket: Track and cancel quantum tasks](https://docs.aws.amazon.com/braket/latest/developerguide/braket-monitor-tasks-sdk.html)
