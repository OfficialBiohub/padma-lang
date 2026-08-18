# Padma domain libraries: HTTP, AI, and backend automation

Padma domain libraries are designed for small Termux-first programs and project automation. They are not an unrestricted web-server framework, an API-key vault, or a way to bypass operating-system security. Every network request requires a declared project capability, every sensitive value stays outside Padma source, and all structured data crosses the boundary as JSON.

## Network contract

`http.get(url)` remains the text convenience API. New request APIs are bounded wrappers around HTTPS/HTTP requests:

| API | Contract |
|---|---|
| `http.post(url, data)` | Serializes a JSON-compatible Padma value, sends it as JSON, and returns the response text. |
| `http.json(url, data)` | Serializes `data` as JSON, sends a bounded POST request, and decodes a JSON response into Padma data. |

Project mode requires `network = ["http"]`. URLs must be absolute `http://` or `https://` URLs without credentials, whitespace, or common local/private hosts. Callers cannot inject curl flags. Requests use a fixed argument vector, no shell, a 30-second timeout, and a 256 KiB response limit.

## AI service contract

AI credentials must never be written inside a `.pd` file, `padma.toml`, source comment, or shell history. An Android user sets a secret in the current shell session, for example `export PADMA_AI_KEY='...'`, then uses only its **environment variable name** in Padma.

| API | Contract |
|---|---|
| `ai.request(endpoint, secret_env_name, payload)` | Sends caller-supplied JSON payload to an explicit provider endpoint, with `Authorization: Bearer <secret>`, and returns decoded JSON. |

All AI APIs require `network = ["ai"]`. The secret variable name must match an uppercase ASCII environment-name policy, and the value is injected only into a child HTTP request. Padma never prints, returns, or writes the secret. Model output remains data: it is never executed as Padma, Python, JavaScript, shell, or browser code.

## Local backend automation contract

Padma does not open a public HTTP port in this milestone. Exposing a long-running server needs separate lifecycle, authentication, and deployment controls. Instead, `backend.response(status, headers, body)` constructs a validated response map that a reviewed Termux adapter, reverse proxy, or future dedicated server runtime can serialize. `automation.write_json(path, value)` writes one JSON job/output file inside the project root and requires `filesystem = ["write"]`.

This lets a Padma program safely produce a request-processing result for local batch jobs, webhooks handled by a separate verified adapter, or backend queues without silently opening the phone to the network.

## Capability example

```toml
[capabilities]
network = ["http", "ai"]
filesystem = ["write"]
```

Grant only the capability that a project needs. A public HTTP API request does not need `ai`; an AI request does not need `process`; writing a response file does not need network permission.
