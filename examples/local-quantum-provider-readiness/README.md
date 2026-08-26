# Local Quantum Provider Readiness Assessment

This Termux-first project validates a provider-neutral review request and returns the controls required before a future remote adapter could be designed. It needs no capability grant because it does not contact IBM Quantum, Amazon Braket, another provider, or a QPU.

```sh
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-quantum-provider-readiness
padma .
```

Expected terminal output:

```text
aws-braket
128
accepted-not-returned
assessment-only
dedicated-capability-design-required
not-defined
not-queried
disabled
disabled
disabled
```

The request identifies a reviewed local artifact by format, SHA-256 fingerprint, and byte count. Its short public policy note is checked but deliberately not returned. The output makes clear that a future adapter must add explicit capability, credential handling, visible confirmation, current cost/quota review, job/cancellation/provenance, and bounded polling controls.

This is **not** an AWS or IBM login, token reader, saved-account lookup, endpoint/backend selector, price checker, task/job submitter, poller, canceller, result downloader, S3 client, provider SDK, or QPU executor. Do not place an API key, token, account ID, job/task ID, endpoint, URL, ARN, CRN, or circuit source in this request.
