# Local Optimisation Building Blocks v1

## Purpose and execution boundary

`optimize.*` provides small deterministic **classical numerical building blocks** for inspecting an explicitly described local objective. It is intended as a prerequisite for future local hybrid experiments, not an automatic optimisation service or a quantum algorithm implementation.

> Numerical finite-difference tools approximate derivatives of real scalar functions at a coordinate vector. [1] [2]

Every v1 call runs in memory. It does not read or write files, invoke a callback, run a loop, mutate variables, start a process, access a network, contact a provider/QPU, read credentials, estimate hardware energy, or submit a job.

## Supported objective

The only v1 objective is a bounded separable weighted quadratic:

\[
f(\mathbf{x}) = \sum_{i=0}^{n-1} w_i(x_i-t_i)^2.
\]

```padma
ধরি objective = {
  "parameters": [2, -1],
  "targets": [1, 3],
  "weights": [2, 0.5],
  "lowerBounds": [-5, -5],
  "upperBounds": [5, 5]
}
```

The explicit schema intentionally avoids arbitrary Padma callbacks, source text, scripts, providers, URLs, files, symbolic values, Hamiltonians, or quantum circuit handles.

## APIs

| API | Result | Capability |
|---|---|---|
| `optimize.quadratic_value(objective)` | One deterministic scalar objective value. | None |
| `optimize.finite_difference_gradient(objective, epsilon)` | Centered finite-difference gradient at the declared parameter vector. | None |
| `optimize.projected_gradient_step(objective, settings)` | A single non-mutating projected gradient-descent proposal. | None |

## Strict objective contract

| Field | Rule |
|---|---|
| Fields | Exactly `parameters`, `targets`, `weights`, `lowerBounds`, `upperBounds`. |
| Vector length | Every vector has the same length from `1` through `16`. |
| Parameters/targets | Finite real values in `[-1_000_000, 1_000_000]`. Every current parameter lies within its corresponding closed bound. |
| Weights | Finite positive real values up to `1_000_000`. |
| Bounds | Finite values within the local range and `lowerBounds[i] < upperBounds[i]`. |

The objective value, all internally evaluated points, gradient entries, and output values must remain finite and within the v1 local numeric policy.

## Gradient and one-step proposal

`finite_difference_gradient` requires a finite `epsilon` in `[0.000001, 1]`, and each parameter must remain strictly interior by at least `epsilon`; this makes the centered formula deterministic without hidden one-sided boundary behaviour:

\[
g_i \approx \frac{f(\mathbf{x}+\epsilon\mathbf{e}_i)-f(\mathbf{x}-\epsilon\mathbf{e}_i)}{2\epsilon}.
\]

`projected_gradient_step` accepts exactly `{ "learningRate": ..., "epsilon": ... }`. `learningRate` must be finite in `(0, 1]`; `epsilon` follows the same rule above. It calculates one proposal

\[
x'_i = \operatorname{clamp}(x_i-\eta g_i, l_i, u_i),
\]

then returns `objectiveBefore`, `gradient`, `proposedParameters`, and `objectiveAfter`. It **does not** replace the caller’s parameter vector, iterate, choose a stopping criterion, accept/reject a proposal, or optimise until convergence. Bound-constrained numerical optimisers use the same broad concept of lower and upper bounds, but Padma v1 deliberately exposes only one audited calculation step. [3]

## Result markers and exclusions

Results include versioned method labels and fixed markers: `iteration = "not-run"`, `mutation = "disabled"`, `callback = "disabled"`, `provider = "not-configured"`, `qpu = "disabled"`, `network = "disabled"`, and `childProcess = "disabled"`.

This feature is **not** a VQE/QAOA/QML/Grover framework, a gradient-based training loop, an automatic hyperparameter search, a provider adapter, or a deployment system. Combining a local circuit/Hamiltonian output with these primitives remains an explicit user-written calculation; no feature automatically connects them or claims a quantum advantage.

## Safety and diagnostics

`P1088` reports objective/settings schema, vector shape, finite numeric, bounds, interior epsilon, or local invariant errors. Every API is pure/in-memory and has no sensitive capability grant.

## References

[1] [SciPy: Finite Difference Differentiation](https://docs.scipy.org/doc/scipy/reference/differentiate.html)  
[2] [SciPy: `approx_fprime`](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.approx_fprime.html)  
[3] [SciPy: Optimisation tutorial and bound constraints](https://docs.scipy.org/doc/scipy/tutorial/optimize.html)
