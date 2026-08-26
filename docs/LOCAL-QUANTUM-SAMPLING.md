# Reproducible Local Quantum Sampling v1

## Purpose and execution boundary

`quantum.sample_counts` turns the existing deterministic local probability map into a **reproducible finite-shot count map**. It runs entirely inside the Padma process and has no provider, QPU, credential, network, process, filesystem, browser, account, payment, or device authority.

> A state-vector sampler uses a circuit’s state-vector probabilities to produce finite-shot outcomes; sampler result data may be represented as bitstrings or counts. [1] [2]

## API

```padma
ধরি request = {"shots": 256, "seed": 20260826}
ধরি counts = quantum.sample_counts(circuit, request)
দেখাও counts["shots"]
দেখাও counts["counts"]
```

| API | Result | Capability |
|---|---|---|
| `quantum.sample_counts(circuit, request)` | A deterministic ordered sparse bitstring-to-count map and redacted local-only metadata. | None |

## Strict request contract

| Field | Rule |
|---|---|
| `shots` | Required whole number from `1` through `100_000`. |
| `seed` | Required whole integer from `0` through `9_007_199_254_740_991` (`2^53 - 1`), preserving exact Padma numeric representation. |
| Other fields | Rejected, including `provider`, `backend`, `noiseModel`, `url`, `credential`, `token`, `seedMode`, `random`, and `shotsDefault`. |

Every call uses a versioned **SplitMix64** pseudo-random sequence initialized solely from the supplied integer seed. Each value selects one outcome from the existing fixed-order probability cumulative distribution. The same circuit, request, and Padma version therefore return identical counts; the output is a reproducibility/debugging tool, not a cryptographic random source or a hardware-randomness claim.

## Result shape and invariants

```text
{
  "shots": 256,
  "seed": 20260826,
  "counts": {"00": 131, "11": 125},
  "distinctOutcomeCount": 2,
  "method": "local-seeded-cdf-sampler-v1",
  "provider": "not-configured",
  "qpu": "disabled",
  "network": "disabled",
  "childProcess": "disabled"
}
```

The count map is lexicographically ordered, sparse (zero-count outcomes are omitted), and its values always sum exactly to `shots`. Bitstring labels reuse the circuit’s declared classical measurement mapping. A different seed intentionally may return a different finite sample; it does not change the underlying local probabilities.

## Semantic limits

The sampler runs after the whole circuit has been evolved. It does not expose a collapsed state, reuse state after sampling, support mid-circuit measurement, implement a noise model, claim hardware counts, execute a provider job, or use a default hidden seed. IBM’s state-vector sampler also distinguishes pure state-vector simulation from mid-circuit measurement behaviour and accepts an explicit seed configuration. [1]

`P1086` reports malformed requests, invalid integer/range fields, unavailable probability data, or a count-total invariant failure. Existing `P1083` validates circuits and `P1084` protects the 12-qubit state-vector resource bound.

## References

[1] [IBM Quantum: StatevectorSampler](https://quantum.cloud.ibm.com/docs/api/qiskit/qiskit.primitives.StatevectorSampler)  
[2] [IBM Quantum: Sampler examples](https://quantum.cloud.ibm.com/docs/en/guides/sampler-examples)
