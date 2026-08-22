# Browser drafts and user takeover: source record

**Research date:** 2026-08-22

This record supports M11’s local-only browser interaction draft foundation. It does not authorize a browser control runner, credential access, CAPTCHA bypass, JavaScript injection, form submission, posting, uploads/downloads, payment, account changes, or autonomous agent actions.

| Source | Relevant finding | Padma design consequence |
|---|---|---|
| [OWASP Transaction Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Transaction_Authorization_Cheat_Sheet.html) | A person must be able to identify and acknowledge significant action data; each sensitive transaction should have unique, time-limited authorization and the final execution gate must validate the specific action. | Padma drafts remain inert data. Any later sensitive browser action requires a user-visible display of the exact destination/action payload, a fresh per-action approval controlled by the destination service, and user takeover rather than Padma execution. |
| [OWASP AI Agent Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html) | Agents need least-privilege tools, human approval for high-impact or irreversible actions, action previews, explicit approval binding, and output validation. | A `browser:draft` capability must create only strict local descriptors. Generated data cannot cause an external action; login, CAPTCHA, form, post, upload/download, account, payment, or deletion states always return `user-takeover-required`. |
| [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html) | Authorization is distinct from authentication; systems should use least privilege, deny by default, and validate permission for each request. | Padma does not infer browser authorization from a plan, user login, or previously approved action. No draft receives implicit browser, credential, profile, process, network, or external-action authority. |

## M11 implementation boundary

The implemented M11 increment parses a project-local `padma-browser-draft.toml` manifest and renders a deterministic inspection-only draft descriptor. It validates bounded review text and project-relative attachment metadata; it does not read any attachment, inspect a page, call a browser, perform DNS/network activity, submit content, or retain approval state.
