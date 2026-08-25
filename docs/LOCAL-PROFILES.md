# Local Profile Toolkit v1

Padma Local Profile Toolkit v1 ছোট JSON-derived configuration/profile map validate করে। এটি student note preference, family budget setting, small-business report preference, freelancer document preset, developer task preset, creator metadata setting, privacy-safe local setting, এবং **user-owned game project** fixture/accessibility setting-এর জন্য ব্যবহারযোগ্য।

> এটি profile value ব্যবহার করে কোনো game, account, browser, network, device, process, payment, upload, download, or cloud action চালায় না।

## APIs

| API | Result | Capability |
|---|---|---|
| `profile.validate(profile, schema)` | Validated value plus explicitly declared defaultsসহ map return করে | কোনো capability নয় |
| `profile.summary(profile, schema)` | Redacted validation summary return করে; profile values return করে না | কোনো capability নয় |

Both APIs in-memory map নিয়ে কাজ করে। Local JSON file ব্যবহার করতে existing safe composition ব্যবহার করুন:

```padma
ধরি profile = json.parse(file.read("data/profile.json"))
```

That composition needs `filesystem = ["read"]`; profile validation itself does not require file, network, account, process, or device authority.

## Schema format

Schema হলো bounded map। Every top-level key is a profile field. Each field rule has `type` and optional `required` or `default`:

```padma
ধরি schema = {"displayName": {"type": "text", "required": true}, "soundEnabled": {"type": "boolean", "default": true}, "theme": {"type": "text", "default": "light"}, "attempts": {"type": "number"}}
```

| Field | Rule |
|---|---|
| Profile/schema key | 1–64 byte, no whitespace/control character, maximum 32 declared fields |
| `type` | Exactly `text`, `number`, `boolean`, or `null` |
| `required` | Optional boolean; a required field cannot also use a default |
| `default` | Optional scalar of the declared type; used only when the profile omits that field |
| Profile value | Must be a declared scalar of its declared type; text is bounded to 1,024 bytes and cannot contain control characters |

Unknown profile field, missing required field, nested list/map value, unsupported schema key, mismatched value/default, oversized/unsafe key, or invalid scalar type gives `P1072`. Raw profile values are not included in the error or `profile.summary` output.

## Redacted summary

`profile.summary` returns only validation metadata: `valid`, field count, explicit/defaulted/optional-missing counts, declared field names, and fixed disabled-action markers. It never returns values, credentials, account state, browser state, device state, or any runtime action result.

## User-owned game project boundary

For a game project you own or are authorized to test, this can validate offline settings such as difficulty, sound, color mode, test-fixture score limits, or accessibility preferences. It cannot crack a game, cheat, bypass anti-cheat, modify another user’s account/item, manipulate a running game process, or create a multiplayer unfair advantage.

## Termux example

```bash
cd ~/padma-lang
cargo build --release
export PATH="$HOME/padma-lang/target/release:$PATH"
cd examples/local-profile-validator
padma .
```

The example reads a project-local JSON preference file, applies a safe explicit `theme` default, and prints a redacted summary. It does not write files or start an external action.
